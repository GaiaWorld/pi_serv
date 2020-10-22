use std::sync::{Arc, Mutex };
use std::sync::{ RwLock as StdRwlock };
use std::net::SocketAddr;
use std::io::{Error, ErrorKind};
use std::env;
use std::cell::RefCell;

use fnv::FnvHashMap;
use fnv::FnvHashSet;
use mqtt3;
use parking_lot::RwLock;
use futures::future::BoxFuture;

use pi_vm::adapter::{JS};
use pi_vm::pi_vm_impl::{new_queue, remove_queue};
use pi_vm::bonmgr::{ptr_jstype, NObject};
use handler::{Args, Handler, SGenType};
use gray::{GrayVersion, GrayTab};
use atom::Atom;
// use pi_p2p::manage::P2PManage;
use net::api::{Socket, Stream};
use net::api::{NetManager, TlsManager};
use mqtt_tmp::server::{ServerNode, ClientStub};
use std::io::{Result as IOResult};
use mqtt_tmp::data::Server;
use mqtt_tmp::session::Session;
use js_lib::JSGray;
use worker::task::TaskType;
use worker::impls::{unlock_js_task_queue, cast_js_task, cast_net_task};
use tcp::connect::TcpSocket;
use tcp::tls_connect::TlsSocket as FTlsSocket;
use tcp::server::{AsyncWaitsHandle, AsyncPortsFactory, SocketListener};
use tcp::driver::{Socket as SocketTrait, Stream as StreamTrait, SocketConfig, AsyncIOWait, AsyncServiceFactory};
use tcp::buffer_pool::WriteBufferPool;
use tcp::util::{close_socket, TlsConfig};
use ws::server::WebsocketListenerFactory;
use mqtt::server::{WsMqttBrokerFactory, WssMqttBrokerFactory, register_listener, register_service, add_topic, publish_topic};
use mqtt::util::AsyncResult;
use mqtt_proxy::service::{MqttEvent, MqttConnectHandle, MqttProxyListener, MqttProxyService};
use ptmgr::{PLAT_MGR, PlatMgrTrait};
use https_external::header::HeaderMap;
use hash::XHashMap;

use http::virtual_host::VirtualHostPool;
use http::server::HttpListenerFactory;
use http::virtual_host::{VirtualHostTab, VirtualHost};
use http::route::HttpRoute;
use http::middleware::{MiddlewareResult, Middleware, MiddlewareChain};
use http::cors_handler::CORSHandler;
use http::default_parser::DefaultParser;
use http::multi_parts::MutilParts;
use http::file_load::FileLoad;
use http::files_load::FilesLoad;
use http::batch_load::BatchLoad;
use http::upload::UploadFile;
use http::port::HttpPort;
use http::static_cache::StaticCache;
use http::request::HttpRequest;
use http::response::{ResponseHandler, HttpResponse};
use http::range_load::RangeLoad;
use crate::js_httpc::{HttpClientOptions, create_http_client, HttpClientBody, post};
use httpc::{SharedHttpc, HttpClient};

use binary::Binary;

use hotfix::get_gray_table;


lazy_static! {
    static ref HTTP_ENDPOINT: Arc<RwLock<FnvHashMap<String, String>>> = Arc::new(RwLock::new(FnvHashMap::default()));
    static ref SECURE_SERVICES: Arc<RwLock<Vec<SecureServices>>> = Arc::new(RwLock::new(vec![]));
    static ref INSECURE_SERVICES: Arc<RwLock<Vec<InsecureServices>>> = Arc::new(RwLock::new(vec![]));
    // 每个端口的证书配置 (port, (cert_path, priv_key_path))
    static ref CERTIFICATES: Arc<RwLock<FnvHashMap<u16, (String, String)>>> = Arc::new(RwLock::new(FnvHashMap::default()));
    static ref SECURE_HTTP_CONFIGS: Arc<RwLock<FnvHashMap<u16, Vec<HttpConfig>>>> = Arc::new(RwLock::new(FnvHashMap::default()));
    static ref INSECURE_HTTP_CONFIGS: Arc<RwLock<FnvHashMap<u16, Vec<HttpConfig>>>> = Arc::new(RwLock::new(FnvHashMap::default()));
    static ref BROKER_TOPICS: Arc<RwLock<FnvHashMap<String, FnvHashSet<String>>>> = Arc::new(RwLock::new(FnvHashMap::default()));
}

struct InsecureServices((u16, Box<dyn AsyncServiceFactory<Connect = TcpSocket, Waits = AsyncWaitsHandle, Out = (), Future = BoxFuture<'static, ()>>>));
struct SecureServices((u16, Box<dyn AsyncServiceFactory<Connect = FTlsSocket, Waits = AsyncWaitsHandle, Out = (), Future = BoxFuture<'static, ()>>>));

unsafe impl Send for InsecureServices {}
unsafe impl Sync for InsecureServices {}

unsafe impl Send for SecureServices {}
unsafe impl Sync for SecureServices {}

/**
* 为指定的Mqtt服务器设置topic
* @param server_node Mqtt服务器
* @param topic Topic
* @param can_publish 是否可发布
* @param can_subscribe 是否可订阅
* @returns 返回是否设置成功
* @throws 失败抛出原因描述
*/
pub fn set_mqtt_topic(server_node: &ServerNode, topic: String, can_publish: bool, can_subscribe: bool) -> Result<bool, String> {
    let topic = Atom::from(topic);
    let server_node1 = server_node.clone();
    match server_node.set_topic_meta(topic.clone(), can_publish,can_subscribe, Box::new(move |_c:ClientStub, r:IOResult<Arc<Vec<u8>>>| {
        match r {
            Ok(v) => {
                match server_node1.publish(false, mqtt3::QoS::AtMostOnce, topic.clone(),Vec::from(v.as_slice())) {
                    Ok(_) => (),
                    Err(s) => {println!("{}, topic:{}", s.to_string(), topic.as_str());},
                }
            },
            Err(s) => {
                println!("{}, topic:{}", s.to_string(), topic.as_str());
            },
        }
    })) {
        Ok(_) => Ok(true),
        Err(s) => Err(s.to_string()),
    } 
}

/**
* 为指定的Mqtt服务器取消topic
* @param server_node Mqtt服务器
* @param topic Topic
* @returns 返回设置结果，成功返回空
* @throws 失败抛出原因描述
*/
pub fn unset_mqtt_topic(server_node: &ServerNode, topic: String) -> Result<(), String> {
    match server_node.unset_topic_meta(Atom::from(topic)) {
        Ok(r) => Ok(r),
        Err(r) => Err(r.to_string()),
    }
}

pub enum QoS{
    AtMostOnce = 0,
    AtLeastOnce = 1,
    ExactlyOnce = 2,
}

/**
* 发布指定topic的消息
* @param server Mqtt服务器
* @param retain 是否保留Mqtt会话
* @param qos Qos
* @param topic Topic
* @param payload 消息体
* @returns 返回发布结果，成功返回空
& @throws 失败抛出原因描述
*/
pub fn mqtt_publish(server: &ServerNode, retain: bool, qos: QoS, topic: String, payload: &[u8]) -> Result<(), Error>{
    let qos = match qos {
        QoS::AtMostOnce => mqtt3::QoS::AtMostOnce,
        QoS::AtLeastOnce => mqtt3::QoS::AtLeastOnce,
        QoS::ExactlyOnce => mqtt3::QoS::ExactlyOnce,
    };
    server.publish(retain, qos, Atom::from(topic), Vec::from(payload))
}

/**
* 回应指定指定topic发布的消息
* @param session Mqtt会话
* @param topic Topic
* @param data 回应的数据
*/
pub fn mqtt_respond(session: &Arc<Session>, topic: String, data: &[u8]) {
    session.respond(Atom::from(topic), Vec::from(data));
}

//为pi_p2p封装一个P2PManage::new方法
// pub fn p2p_manage_new(addr: &str, arr1: Vec<String>, arr2: Vec<u32>) -> P2PManage {

//     let mut map: FnvHashMap<SocketAddr, u64> = FnvHashMap::default();
//     let mut i = 0;
//     for time in arr2 {
//         map.insert(arr1.get(i).unwrap().parse().unwrap(), time as u64);
//         i += 1;
//     }
//     P2PManage::new(addr.parse().unwrap(), map)
// }

// /**
// * 创建一个公共Socket的引用计数
// */
// pub fn creat_arc_sokect(socket: Socket ) -> Arc<Socket>{
//     Arc::new(socket)
// }

use hotfix::GrayTable;

// 设置http请求参数
fn set_data(vm: Arc<JS>, msg: Arc<RefCell<XHashMap<String, SGenType>>>) {
    let data = vm.new_object();
    for (key, val) in msg.borrow_mut().drain() {
        match val {
            SGenType::Str(s) => {
                vm.set_field(&data, String::from(key), &mut vm.new_str(s.to_string()).unwrap());
            }
            SGenType::Bin(bin) => {
                let ptr = Box::into_raw(Box::new(Binary::new(bin))) as usize;
                let nobj = NObject{meta_hash: 3610954401};
                vm.get_objs().borrow_mut().insert(ptr, nobj);
                let mut n = vm.new_native_object(ptr);
                vm.set_field(&data, String::from(key), &mut n);
            }
            _ => {
                unimplemented!();
            }
        }
    }
}

#[derive(Clone)]
pub struct SecureHttpRpcRequestHandler {
    gray_tab: 	Arc<RwLock<GrayTable>>,
}

#[derive(Clone)]
pub struct InsecureHttpRpcRequstHandler {
    gray_tab: 	Arc<RwLock<GrayTable>>,
}

unsafe impl Send for SecureHttpRpcRequestHandler {}
unsafe impl Sync for SecureHttpRpcRequestHandler {}

unsafe impl Send for InsecureHttpRpcRequstHandler {}
unsafe impl Sync for InsecureHttpRpcRequstHandler {}

impl Handler for InsecureHttpRpcRequstHandler {
    type A = SocketAddr;
    type B = Arc<HeaderMap>;
    type C = Arc<RefCell<XHashMap<String, SGenType>>>;
    type D = ResponseHandler<TcpSocket>;
    type E = ();
    type F = ();
    type G = ();
    type H = ();
    type HandleResult = ();

    //处理方法
    fn handle(&self, env: Arc<dyn GrayVersion>, topic: Atom, args: Args<Self::A, Self::B, Self::C, Self::D, Self::E, Self::F, Self::G, Self::H>) -> Self::HandleResult {
        let topic_handler = self.clone();
        let topic_name = topic.clone();

        let id = env.get_id();
        let queue = new_queue(id); //创建指定socket的同步静态队列
        let func = Box::new(move |lock: Option<isize>| {
            let gray_tab = topic_handler.gray_tab.read();
            let gray = match gray_tab.jsgrays.last() {
                Some(g) => {
                    g.values().last()
                }
                None => panic!("gray is not exist"),
            };

            if let Some(gray) = gray {
                let mgr = gray.mgr.clone();
                let topic_name = topic.clone();
                let real_args = Box::new(move |vm: Arc<JS>| -> usize {
                    let ptr = Box::into_raw(Box::new(mgr.clone())) as usize;
                    ptr_jstype(vm.get_objs(), vm.clone(), ptr, 2976191628); // mgr 参数
                    vm.new_str((*topic_name).to_string()); // topic 参数

                    match args {
                        Args::FourArgs(addr, headers, msg, handler) => {
                            let mut http_connect = HttpConnect::new(addr);
                            
                            http_connect.set_insecure_resp_handle(handler);

                            let ptr = Box::into_raw(Box::new(http_connect)) as usize;
                            ptr_jstype(vm.get_objs(), vm.clone(), ptr, 63358028); // HttpConnect 参数
                            let http_header = HttpHeaders::new(headers);
                            let ptr = Box::into_raw(Box::new(http_header)) as usize;
                            ptr_jstype(vm.get_objs(), vm.clone(), ptr, 1654202482); // HttpHeaders 参数

                            set_data(vm, msg);
                        }
                        _ => panic!("invalid HttpRpcRequestHandler handler args"),
                    }
                    5 // _$http_rpc 总共5个参数
                });
                gray.factory.call(Some(id), Atom::from("_$http_rpc"), real_args, Atom::from((*topic).to_string() + " http_rpc task"));

                //解锁当前同步静态队列，保证虚拟机执行
                if !unlock_js_task_queue(queue) {
                    warn!("!!!> Topic Handle Error, unlock task queue failed, queue: {:?}", queue);
                }
            } else {
                error!("can't found handler for topic: {:?}", topic);
            }
        });
        cast_js_task(TaskType::Sync(true), 0, Some(queue), func, Atom::from("topic ".to_string() + &topic_name + " handle http_rpc task"));
    }
}

impl InsecureHttpRpcRequstHandler {
    pub fn new(gray: &Arc<RwLock<GrayTable>>) -> Self {
        InsecureHttpRpcRequstHandler {
            gray_tab: gray.clone()
        }
    }
}

impl Handler for SecureHttpRpcRequestHandler {
    type A = SocketAddr;
    type B = Arc<HeaderMap>;
    type C = Arc<RefCell<XHashMap<String, SGenType>>>;
    type D = ResponseHandler<FTlsSocket>;
    type E = ();
    type F = ();
    type G = ();
    type H = ();
    type HandleResult = ();

    //处理方法
    fn handle(&self, env: Arc<dyn GrayVersion>, topic: Atom, args: Args<Self::A, Self::B, Self::C, Self::D, Self::E, Self::F, Self::G, Self::H>) -> Self::HandleResult {
        let topic_handler = self.clone();
        let topic_name = topic.clone();

        let id = env.get_id();
        let queue = new_queue(id); //创建指定socket的同步静态队列
        let func = Box::new(move |lock: Option<isize>| {
            let gray_tab = topic_handler.gray_tab.read();
            let gray = match gray_tab.jsgrays.last() {
                Some(g) => {
                    g.values().last()
                }
                None => panic!("gray is not exist"),
            };

            if let Some(gray) = gray {
                let mgr = gray.mgr.clone();
                let topic_name = topic.clone();
                let real_args = Box::new(move |vm: Arc<JS>| -> usize {
                    let ptr = Box::into_raw(Box::new(mgr.clone())) as usize;
                    ptr_jstype(vm.get_objs(), vm.clone(), ptr, 2976191628);
                    vm.new_str((*topic_name).to_string());

                    match args {
                        Args::FourArgs(addr, headers, msg, handler) => {
                            let mut http_connect = HttpConnect::new(addr);
                            
                            http_connect.set_secure_resp_handle(handler);

                            let ptr = Box::into_raw(Box::new(http_connect)) as usize;
                            ptr_jstype(vm.get_objs(), vm.clone(), ptr, 63358028);
                            let http_header = HttpHeaders::new(headers);
                            let ptr = Box::into_raw(Box::new(http_header)) as usize;
                            ptr_jstype(vm.get_objs(), vm.clone(), ptr, 1654202482);

                            set_data(vm, msg);
                        }
                        _ => panic!("invalid HttpRpcRequestHandler handler args"),
                    }
                    5 // _$http_rpc 总共5个参数
                });
                gray.factory.call(Some(id), Atom::from("_$http_rpc"), real_args, Atom::from((*topic).to_string() + " http_rpc task"));

                //解锁当前同步静态队列，保证虚拟机执行
                if !unlock_js_task_queue(queue) {
                    warn!("!!!> Topic Handle Error, unlock task queue failed, queue: {:?}", queue);
                }
            } else {
                error!("can't found handler for topic: {:?}", topic);
            }
        });
        cast_js_task(TaskType::Sync(true), 0, Some(queue), func, Atom::from("topic ".to_string() + &topic_name + " handle http_rpc task"));
    }
}

impl SecureHttpRpcRequestHandler {
    pub fn new(gray: &Arc<RwLock<GrayTable>>) -> Self {
        SecureHttpRpcRequestHandler {
            gray_tab: gray.clone()
        }
    }
}

pub struct HttpConnect {
    peer_addr: SocketAddr,
    conn_type: ConnectType,
}

impl  HttpConnect {
    pub fn new(peer_addr: SocketAddr) -> Self {
        Self {
            peer_addr,
            conn_type: ConnectType::Unknow
        }
    }

    pub fn set_secure_resp_handle(&mut self, handle: ResponseHandler<FTlsSocket>) {
        self.conn_type = ConnectType::Secure(handle);
    }

    pub fn set_insecure_resp_handle(&mut self, handle: ResponseHandler<TcpSocket>) {
        self.conn_type = ConnectType::InSecure(handle);
    }

    // 返回 string 比较好，如果返回 [string, u16]的元组，js层没有解析方法
    pub fn peer_addr(&self) -> String {
        return self.peer_addr.to_string();
    }

    pub fn set(&self, key: &str, val: &str) {
        match self.conn_type.clone() {
            ConnectType::InSecure(handle) => {
                handle.header(key, val);
            }
            ConnectType::Secure(handle) => {
                handle.header(key, val)
            }
            ConnectType::Unknow => {
                panic!("unknow connect type")
            }
        }
    }

    pub fn reply_http_rpc(&self, data: &[u8]) -> Result<bool, std::io::Error> {
        match self.conn_type.clone() {
            ConnectType::InSecure(insecure_handle) => {
                if data.len() == 0 {
                    insecure_handle.finish()?;
                    return Ok(true)
                }
                insecure_handle.write(Vec::from(data))?;
                insecure_handle.finish()?;
                Ok(true)
            }
            ConnectType::Secure(secure_handle) => {
                if data.len() == 0 {
                    secure_handle.finish()?;
                    return Ok(true)
                }
                secure_handle.write(Vec::from(data))?;
                secure_handle.finish()?;
                Ok(true)
            }
            ConnectType::Unknow => {
                Err(Error::new(ErrorKind::Other, "Unknow connect type"))
            }
        }
    }

    pub fn set_status_code(&self, code: u16) {
        match self.conn_type.clone() {
            ConnectType::InSecure(insecure_handle) => {
                insecure_handle.status(code);
            }
            ConnectType::Secure(secure_handle) => {
                secure_handle.status(code);
            }
            ConnectType::Unknow => {
                panic!("unknow connect type");
            }
        }
    }

    pub fn is_secure(&self) -> bool {
        match self.conn_type {
            ConnectType::InSecure(_) => false,
            ConnectType::Secure(_) => true,
            ConnectType::Unknow => {
                panic!("Unknow connect type")
            }
        }
    }
}

pub struct HttpHeaders {
    headers: Arc<HeaderMap>
}

impl HttpHeaders {
    pub fn new(headers: Arc<HeaderMap>) -> Self {
        Self {
            headers
        }
    }

    // 获取指定头
    pub fn get(&self, key: &str) -> Option<&str> {
        match self.headers.get(key) {
            Some(val) => {
                match val.to_str() {
                    Ok(v) => {
                        Some(v)
                    }
                    Err(_) => None
                }
            }
            None => None
        }
    }
}


// 处理连接和关闭连接
#[derive(Clone)]
struct MqttConnectHandler {
    gray_tab: Arc<RwLock<GrayTable>>,
}

impl MqttConnectHandler {
    pub fn new(gray: &Arc<RwLock<GrayTable>>) -> Self {
        MqttConnectHandler {
            gray_tab: gray.clone()
        }
    }
}

impl Handler for MqttConnectHandler {
    type A = MqttEvent;
    type B = ();
    type C = ();
    type D = ();
    type E = ();
    type F = ();
    type G = ();
    type H = ();
    type HandleResult = ();

    fn handle(&self, env: Arc<dyn GrayVersion>, topic: Atom, args: Args<Self::A, Self::B, Self::C, Self::D, Self::E, Self::F, Self::G, Self::H>) -> Self::HandleResult {
        let topic_handler = self.clone();
        let id = env.get_id();
        let queue = new_queue(id);
        let func = Box::new(move |lock: Option<isize>| {
            let gray_tab = topic_handler.gray_tab.read();
            let gray = match gray_tab.jsgrays.last() {
                Some(g) => {
                    g.values().last()
                }
                None => panic!("gray is not exist"),
            };

            if let Some(gray) = gray {
                let mgr = gray.mgr.clone();
                let mgr_ptr = Box::into_raw(Box::new(mgr.clone())) as usize; // 数据库管理器
                let connect = unsafe { Arc::from_raw(Arc::into_raw(env) as *const MqttConnectHandle) };
                match args {
                    Args::OneArgs(MqttEvent::Connect(socket_id, broker_name, client_id, keep_alive, is_clean_session, user, pwd, result)) => {
                        //处理Mqtt连接
                        let client_id_clone = client_id.clone();
                        let real_args = Box::new(move |vm: Arc<JS>| -> usize {
                            ptr_jstype(vm.get_objs(), vm.clone(), mgr_ptr, 2976191628);
                            let mqtt_connection = MqttConnection::new(connect, Some(result), socket_id, client_id, Some(keep_alive), Some(is_clean_session), user, pwd);
                            let mqtt_connection_ptr = Box::into_raw(Box::new(mqtt_connection)) as usize;
                            ptr_jstype(vm.get_objs(), vm.clone(), mqtt_connection_ptr, 1629990554);
                            let _ = vm.new_str(broker_name);
                            3
                        });

                        gray.factory.call(Some(id), Atom::from("_$mqttConnect"), real_args, Atom::from(format!("mqtt connect, client_id = {:?}, socket_id = {:?}, keep_alive = {:?}", client_id_clone, socket_id, keep_alive)));
                    }
                    Args::OneArgs(MqttEvent::Disconnect(socket_id, broker_name, client_id, reason)) => {
                        //处理Mqtt连接关闭
                        let client_id_clone = client_id.clone();
                        let real_args = Box::new(move |vm: Arc<JS>| -> usize {
                            ptr_jstype(vm.get_objs(), vm.clone(), mgr_ptr, 2976191628);
                            let mqtt_connection = MqttConnection::new(connect, None, socket_id, client_id, None, None, None, None);
                            let mqtt_connection_ptr = Box::into_raw(Box::new(mqtt_connection)) as usize;
                            ptr_jstype(vm.get_objs(), vm.clone(), mqtt_connection_ptr, 1629990554);
    
                            match reason {
                                Ok(_) => {
                                    let _ = vm.new_str("".to_string());
                                }
                                Err(e) => {
                                    let _ = vm.new_str(e.to_string());
                                }
                            }
                            let _ = vm.new_str(broker_name);
                            4
                        });

                        gray.factory.call(Some(id), Atom::from("_$mqttDisconnect"), real_args, Atom::from(format!("mqtt disconnect, client_id = {:?}, socket_id = {:?}", client_id_clone, socket_id)));
                    },
                    _ => panic!("invalid MqttConnectHandler handler args"),
                }

                if !unlock_js_task_queue(queue) {
                    warn!("!!!> MqttConnectHandler Error, unlock task queue failed, queue: {:?}", queue);
                }
            } else {
                error!("can't found handler for topic: {:?}", topic);
            }
        });
        cast_js_task(TaskType::Sync(true), 0, Some(queue), func, Atom::from("MqttConnectHandler"));
    }
}

// 处理mqtt 订阅，退订和消息发布
#[derive(Clone)]
struct MqttRequestHandler {
    gray_tab: Arc<RwLock<GrayTable>>,
}

impl MqttRequestHandler {
    pub fn new(gray: &Arc<RwLock<GrayTable>>) -> Self {
        MqttRequestHandler {
            gray_tab: gray.clone()
        }
    }
}

impl Handler for MqttRequestHandler {
    type A = MqttEvent;
    type B = ();
    type C = ();
    type D = ();
    type E = ();
    type F = ();
    type G = ();
    type H = ();
    type HandleResult = ();

    fn handle(&self, env: Arc<dyn GrayVersion>, topic: Atom, args: Args<Self::A, Self::B, Self::C, Self::D, Self::E, Self::F, Self::G, Self::H>) -> Self::HandleResult {
        let topic_handler = self.clone();
        let id = env.get_id();
        let queue = new_queue(id);

        let func = Box::new(move |lock: Option<isize>| {
            let gray_tab = topic_handler.gray_tab.read();
            let gray = match gray_tab.jsgrays.last() {
                Some(g) => {
                    g.values().last()
                }
                None => panic!("gray is not exist"),
            };

            if let Some(gray) = gray {
                let mgr = gray.mgr.clone();
                let mgr_ptr = Box::into_raw(Box::new(mgr.clone())) as usize; // 数据库管理器
                let connect = unsafe { Arc::from_raw(Arc::into_raw(env) as *const MqttConnectHandle) };

                match args {
                    Args::OneArgs(MqttEvent::Sub(socket_id, broker_name, client_id, topics, result)) => {
                        //处理Mqtt订阅主题
                        let real_args = Box::new(move |vm: Arc<JS>| -> usize {
                            ptr_jstype(vm.get_objs(), vm.clone(), mgr_ptr, 2976191628);
                            let mqtt_connection = MqttConnection::new(connect, Some(result), socket_id, client_id, None, None, None, None);
                            let mqtt_connection_ptr = Box::into_raw(Box::new(mqtt_connection)) as usize;
                            ptr_jstype(vm.get_objs(), vm.clone(), mqtt_connection_ptr, 1629990554);

                            let arr = vm.new_array();
                            for (index, topic) in topics.into_iter().enumerate() {
                                let mut value = vm.new_str(topic.0).unwrap();
                                vm.set_index(&arr, index as u32, &mut value);
                            }
                            let _ = vm.new_str(broker_name);
                            4
                        });
                        gray.factory.call(Some(id), Atom::from("_$mqttSub"), real_args, Atom::from("MqttRequestHandler _$mqttSub"));
                    },
                    Args::OneArgs(MqttEvent::Unsub(socket_id, broker_name, client_id, topics)) => {
                        //处理Mqtt退订主题
                        let real_args = Box::new(move |vm: Arc<JS>| -> usize {
                            ptr_jstype(vm.get_objs(), vm.clone(), mgr_ptr, 2976191628);
                            let mqtt_connection = MqttConnection::new(connect, None, socket_id, client_id, None, None, None, None);
                            let mqtt_connection_ptr = Box::into_raw(Box::new(mqtt_connection)) as usize;
                            ptr_jstype(vm.get_objs(), vm.clone(), mqtt_connection_ptr, 1629990554);
    
                            let arr = vm.new_array();
                            for (index, topic) in topics.into_iter().enumerate() {
                                let mut value = vm.new_str(topic).unwrap();
                                vm.set_index(&arr, index as u32, &mut value);
                            }
                            let _ = vm.new_str(broker_name);
                            4
                        });
                        gray.factory.call(Some(id), Atom::from("_$mqttUnSub"), real_args, Atom::from("MqttRequestHandler _$mqttUnSub"));
                    },
                    Args::OneArgs(MqttEvent::Publish(socket_id, broker_name, client_id, address, topic, payload)) => {
                        //处理Mqtt发布主题
                        let real_args = Box::new(move |vm: Arc<JS>| -> usize {
                            ptr_jstype(vm.get_objs(), vm.clone(), mgr_ptr, 2976191628);
                            let mqtt_connection = MqttConnection::new(connect, None, socket_id, client_id, None, None, None, None);
                            let mqtt_connection_ptr = Box::into_raw(Box::new(mqtt_connection)) as usize;
                            ptr_jstype(vm.get_objs(), vm.clone(), mqtt_connection_ptr, 1629990554);
    
                            if let Some(addr) = address {
                                let _ = vm.new_str(addr.to_string());
                            } else {
                                let _ = vm.new_str("".to_string());
                            }
                            let _ = vm.new_str(topic);
                            let buffer = vm.new_uint8_array(payload.len() as u32);
                            buffer.from_bytes(payload.as_ref());
                            let _ = vm.new_str(broker_name);

                            6
                        });
                        gray.factory.call(Some(id), Atom::from("_$mqttSend"), real_args, Atom::from("MqttRequestHandler _$mqttSend"));
                    },
                    _ => panic!("invalid MqttRequestHandler handler args"),
                }

                if !unlock_js_task_queue(queue) {
                    warn!("!!!> MqttRequestHandler Error, unlock task queue failed, queue: {:?}", queue);
                }
            } else {
                error!("can't found handler for topic: {:?}", topic);
            }
        });
        cast_js_task(TaskType::Sync(true), 0, Some(queue), func, Atom::from("MqttRequestHandler"));
    }
}

pub fn register_broker_topic(broker_name: String, topic: String) {
    let mut brokers = BROKER_TOPICS.write();

    if let Some(topics) = brokers.get_mut(&broker_name) {
        topics.insert(topic);
    } else {
        let mut set = FnvHashSet::default();
        set.insert(topic);
        brokers.insert(broker_name, set);
    }
}

pub fn broker_has_topic(broker_name: String, topic: String) -> bool {
    if let Some(set) = BROKER_TOPICS.read().get(&broker_name) {
        if let Some(_) = set.get(&topic) {
            return true
        }
    }

    // 是否在全局broker name
    if let Some(set) = BROKER_TOPICS.read().get("*") {
        if let Some(_) = set.get(&topic) {
            return true
        }
    }

    false
}

pub struct MqttConnection {
    handle: Arc<MqttConnectHandle>,
    connection_result: Option<AsyncResult>,
    socket_id: usize,
    client_id: String,
    keep_alive: Option<u16>,
    is_clean_session: Option<bool>,
    user: Option<String>,
    pwd: Option<String>,
}

impl MqttConnection {
    pub fn new(handle: Arc<MqttConnectHandle>, connection_result: Option<AsyncResult>, socket_id: usize, client_id: String, keep_alive: Option<u16>, is_clean_session: Option<bool>, user: Option<String>, pwd: Option<String>) -> Self {
        Self {
            handle,
            connection_result,
            socket_id,
            client_id,
            keep_alive,
            is_clean_session,
            user,
            pwd,
        }
    }

    pub fn socket_id(&self) -> usize {
        self.socket_id
    }

    pub fn client_id(&self) -> String {
        self.client_id.clone()
    }

    pub fn keep_alive(&self) -> Option<u16> {
        self.keep_alive
    }

    pub fn is_clean_session(&self) -> Option<bool> {
        self.is_clean_session
    }

    pub fn user(&self) -> Option<String> {
        self.user.clone()
    }

    pub fn pwd(&self) -> Option<String> {
        self.pwd.clone()
    }

    pub fn get_token(&self) -> Option<usize> {
        self.handle.get_token()
    }

    pub fn get_local_addr(&self) -> Option<String> {
        self.handle.get_local_addr().map(|addr| addr.to_string())
    }

    pub fn get_remote_addr(&self) -> Option<String> {
        self.handle.get_remote_addr().map(|addr| addr.to_string())
    }

    pub fn is_security(&self) -> bool {
        self.handle.is_security()
    }

    pub fn set_connection_result(&self, result: bool) {
        if result {
            match &self.connection_result {
                Some(res) => res.set(Ok(())),
                None => {}
            }
        } else {
            match &self.connection_result {
                Some(res) => res.set(Err(Error::new(ErrorKind::Other, "connection refused by user"))),
                None => {}
            }
        }
    }

    pub fn wakeup(&self) {
        let _= self.handle.wakeup();
    }

    pub fn sub(&self, topic: String) {
        self.handle.sub(topic)
    }

    pub fn unsub(&self, topic: String) {
        self.handle.unsub(topic)
    }

    pub fn send(&self, topic: String, bin: &[u8]) {
        self.handle.send(&topic, bin.to_vec());
    }

    pub fn reply(&self, bin: &[u8]) {
        self.handle.reply(bin.to_vec());
    }

    pub fn close(&self, reason: String) {
        debug!("close, reason = {:?}", reason);
        let _ = self.handle.close(Ok(()));
    }
 }

pub fn register_http_endpoint(key: String, val: String) {
    HTTP_ENDPOINT.write().insert(key, val);
}

pub fn get_http_endpoint(key: &str) -> Option<String> {
    HTTP_ENDPOINT.read().get(key).cloned()
}

pub fn get_all_http_endpoint() -> Vec<String> {
    HTTP_ENDPOINT.read().keys().map(|s|s.to_string()).collect::<Vec<String>>()
}

pub fn get_all_http_rpc_mods() -> Vec<String> {
    HTTP_ENDPOINT.read().values().map(|s|s.to_string()).collect::<Vec<String>>()
}

pub fn bind_mqtt_tcp_port(port: u16, use_tls: bool, protocol: String, broker_name: String) {
    let event_handler = Arc::new(MqttConnectHandler::new(&get_gray_table()));
    let rpc_handler = Arc::new(MqttRequestHandler::new(&get_gray_table()));
    let listener = Arc::new(MqttProxyListener::with_handler(Some(event_handler)));
    let service = Arc::new(MqttProxyService::with_handler(Some(rpc_handler)));

    if use_tls {
        let broker_factory = Arc::new(WssMqttBrokerFactory::new(&protocol, &broker_name, port));

        SECURE_SERVICES.write().push(SecureServices((port.clone(), Box::new(WebsocketListenerFactory::<FTlsSocket>::with_protocol_factory(
            broker_factory)))));
    } else {
        let broker_factory = Arc::new(WsMqttBrokerFactory::new(&protocol, &broker_name, port));

        INSECURE_SERVICES.write().push(InsecureServices((port.clone(), Box::new(WebsocketListenerFactory::<TcpSocket>::with_protocol_factory(
            broker_factory)))));

    }
    register_listener(&broker_name, listener);
    register_service(&broker_name, service);
}

fn build_secure_service() -> Result<(), String> {
    for (port, http_configs) in SECURE_HTTP_CONFIGS.read().iter() {
        let handler = Arc::new(SecureHttpRpcRequestHandler::new(&get_gray_table()));
        let http_port = Arc::new(HttpPort::with_handler(None, handler));

        let r = build_service::<FTlsSocket>(port.clone(), http_configs, http_port);
        SECURE_SERVICES.write().push(SecureServices(r));
    }
    Ok(())
}

fn build_insecure_service() -> Result<(), String> {
    for (port, http_configs) in INSECURE_HTTP_CONFIGS.read().iter() {
        let handler = Arc::new(InsecureHttpRpcRequstHandler::new(&get_gray_table()));
        let http_port = Arc::new(HttpPort::with_handler(None, handler));

        let r = build_service::<TcpSocket>(port.clone(), http_configs, http_port);
        INSECURE_SERVICES.write().push(InsecureServices(r));
    }

    Ok(())
}

fn build_service<S: SocketTrait + StreamTrait>(port: u16, http_configs: &Vec<HttpConfig>, http_port: Arc<HttpPort<S>>) -> (u16, Box<dyn AsyncServiceFactory<Connect = S, Waits = AsyncWaitsHandle, Out = (), Future = BoxFuture<'static, ()>>>) {
    let mut hosts = VirtualHostTab::new();
    let mut keep_alive = 60000;
    for http_config in http_configs {
        if http_config.keep_alive_timeout > 0 {
            keep_alive = http_config.keep_alive_timeout;
        }
        let enable_cache = http_config.static_cache_max_len > 0 && http_config.static_cache_max_size > 0 && http_config.static_cache_collect_time > 0;
        let cors_handler = if http_config.cors {
            Arc::new(CORSHandler::new("OPTIONS, GET, POST".to_string(), Some(365 * 24 * 60 * 60)))
        } else {
            Arc::new(CORSHandler::new("OPTIONS, GET, POST".to_string(), None))
        };

        if !http_config.cors {
            for config in http_config.cors_allows.borrow().iter() {
                if let Err(e) = cors_handler.allow_origin(config.scheme.clone(), config.host.clone(), config.port, &config.methods, &[], config.max_age) {
                    panic!("failed to add origin, error = {:?}, config= {:?}", e, config);
                }
            }
        }

        let parser = Arc::new(DefaultParser::with(http_config.parser_min_plain_text_size, http_config.parse_compress_level));
        let multi_parts = Arc::new(MutilParts::with(http_config.multi_parts_block_size));
        let range_load = Arc::new(RangeLoad::new());

        let file_load;
        let files_load;
        let batch_load;

        if enable_cache {
            let cache = Arc::new(StaticCache::new(http_config.static_cache_max_size, http_config.static_cache_max_len));
            StaticCache::run_collect(cache.clone(), "http cache".to_string(), http_config.static_cache_collect_time);
            file_load = Arc::new(FileLoad::new(http_config.file_load_location.clone(), Some(cache.clone()), http_config.file_load_need_cache, true, true, false, http_config.file_load_max_age));
            files_load = Arc::new(FilesLoad::new(http_config.files_load_location.clone(), Some(cache.clone()), http_config.files_load_need_cache, true, true, false, http_config.files_load_max_age));
            batch_load = Arc::new(BatchLoad::new(http_config.batch_load_location.clone(), Some(cache.clone()), http_config.batch_load_need_cache, true, true, false, http_config.batch_load_max_age));
        } else {
            file_load = Arc::new(FileLoad::new(http_config.file_load_location.clone(), None, http_config.file_load_need_cache, true, true, false, http_config.file_load_max_age));
            files_load = Arc::new(FilesLoad::new(http_config.files_load_location.clone(), None, http_config.files_load_need_cache, true, true, false, http_config.files_load_max_age));
            batch_load = Arc::new(BatchLoad::new(http_config.batch_load_location.clone(), None, http_config.batch_load_need_cache, true, true, false, http_config.batch_load_max_age));
        }

        let upload = Arc::new(UploadFile::new(http_config.upload_file_location.clone()));

        //构建处理CORS的Options方法的请求的中间件链
        let mut chain = MiddlewareChain::new();
        chain.push_back(cors_handler.clone());
        chain.finish();
        let cors_middleware = Arc::new(chain);

        //构建处理文件加载的中间件链
        let mut chain = MiddlewareChain::new();
        chain.push_back(cors_handler.clone());
        chain.push_back(parser.clone());
        chain.push_back(range_load.clone());
        chain.push_back(file_load);
        chain.finish();
        let file_load_middleware = Arc::new(chain);

        //构建处理文件批量加载的中间件链
        let mut chain = MiddlewareChain::new();
        chain.push_back(cors_handler.clone());
        chain.push_back(parser.clone());
        chain.push_back(range_load.clone());
        chain.push_back(files_load);
        chain.finish();
        let files_load_middleware = Arc::new(chain);

        //构建改进的处理文件批量加载的中间件链
        let mut chain = MiddlewareChain::new();
        chain.push_back(cors_handler.clone());
        chain.push_back(parser.clone());
        chain.push_back(range_load.clone());
        chain.push_back(batch_load);
        chain.finish();
        let batch_load_middleware = Arc::new(chain);

        //构建处理文件上传的中间件链
        let mut chain = MiddlewareChain::new();
        chain.push_back(cors_handler.clone());
        chain.push_back(parser.clone());
        chain.push_back(multi_parts.clone());
        chain.push_back(upload);
        chain.finish();
        let upload_middleware = Arc::new(chain);

        //构建处理动态资源访问的中间件链
        let mut chain = MiddlewareChain::new();
        chain.push_back(cors_handler.clone());
        chain.push_back(parser);
        chain.push_back(multi_parts.clone());
        chain.push_back(http_port.clone());
        chain.finish();
        let port_middleware = Arc::new(chain);

        for (vhs, routes) in http_config.virtual_hosts.borrow().iter() {
            //构建路由
            let mut route = HttpRoute::new();
            route
                .at("/").options(cors_middleware.clone())
                .at("/**").options(cors_middleware.clone());

            for r in routes {
                match r.handler_name.as_str() {
                    "fileLoad" => {
                        if r.methods.contains(&"GET".to_string()) {
                            route.at(&r.endpoint).get(file_load_middleware.clone());
                        }
                        if r.methods.contains(&"POST".to_string()) {
                            route.at(&r.endpoint).post(file_load_middleware.clone());
                        }
                        if r.methods.contains(&"OPTIONS".to_string()) {
                            route.at(&r.endpoint).options(file_load_middleware.clone());
                        }
                    }

                    "filesLoad" => {
                        if r.methods.contains(&"GET".to_string()) {
                            route.at(&r.endpoint).get(files_load_middleware.clone());
                        }
                        if r.methods.contains(&"POST".to_string()) {
                            route.at(&r.endpoint).post(files_load_middleware.clone());
                        }
                        if r.methods.contains(&"OPTIONS".to_string()) {
                            route.at(&r.endpoint).options(files_load_middleware.clone());
                        }
                    }

                    "batchLoad" => {
                        if r.methods.contains(&"GET".to_string()) {
                            route.at(&r.endpoint).get(batch_load_middleware.clone());
                        }
                        if r.methods.contains(&"POST".to_string()) {
                            route.at(&r.endpoint).post(batch_load_middleware.clone());
                        }
                        if r.methods.contains(&"OPTIONS".to_string()) {
                            route.at(&r.endpoint).options(batch_load_middleware.clone());
                        }
                    }

                    "upload" => {
                        if r.methods.contains(&"GET".to_string()) {
                            route.at(&r.endpoint).get(upload_middleware.clone());
                        }
                        if r.methods.contains(&"POST".to_string()) {
                            route.at(&r.endpoint).post(upload_middleware.clone());
                        }
                        if r.methods.contains(&"OPTIONS".to_string()) {
                            route.at(&r.endpoint).options(upload_middleware.clone());
                        }
                    }

                    "port" => {
                        if r.methods.contains(&"GET".to_string()) {
                            route.at(&r.endpoint).get(port_middleware.clone());
                        }
                        if r.methods.contains(&"POST".to_string()) {
                            route.at(&r.endpoint).post(port_middleware.clone());
                        }
                        if r.methods.contains(&"OPTIONS".to_string()) {
                            route.at(&r.endpoint).options(port_middleware.clone());
                        }
                    }

                    _ => {
                        panic!("unsupported secure middleware");
                    }
                }
            }

            //构建虚拟主机
            let host = VirtualHost::with(route);

            // 多个主机共享一个路由表
            for vh in vhs {
                println!("add insecure host = {:?}, port = {:?}", vh, http_config.port);
                let _ = hosts.add(vh, host.clone());
            }
        }
    }

    (port.clone(), Box::new(HttpListenerFactory::with_hosts(hosts.clone(), keep_alive)))
}

pub fn config_certificate(port: u16, cert_path: String, priv_key_path: String) {
    CERTIFICATES.write().insert(port, (cert_path, priv_key_path));
}

pub fn start_network_services(net_kernel_options: NetKernelOptions) -> Result<(), String> {
    // 准备安全服务配置
    build_secure_service()?;
    // 准备非安全服务配置
    build_insecure_service()?;

    let mut secure_services: Vec<(u16, TlsConfig, Box<dyn AsyncServiceFactory<Connect = FTlsSocket, Waits = AsyncWaitsHandle, Out = (), Future = BoxFuture<'static, ()>>>)> = vec![];
    for SecureServices((port, service)) in  SECURE_SERVICES.write().drain(..).into_iter() {
        println!("start_network_services secure service port = {:?}", port);
        match CERTIFICATES.read().get(&port) {
            Some((cert_path, priv_key_path)) => {
                let tls_config = TlsConfig::new_server("",
                                                        false,
                                                        cert_path,
                                                        priv_key_path,
                                                        "",
                                                        "",
                                                        "",
                                                        512,
                                                        false,
                                                        "").unwrap();

                secure_services.push((port, tls_config, service));
            }
            None => panic!("port {:?} configured use TLS, but no certificate specified", port),
        }
    }

    let mut insecure_services: Vec<(u16, Box<dyn AsyncServiceFactory<Connect = TcpSocket, Waits = AsyncWaitsHandle, Out = (), Future = BoxFuture<'static, ()>>>)> = vec![];
    for InsecureServices((port, service)) in INSECURE_SERVICES.write().drain(..).into_iter() {
        debug!("start_network_services insecure service port = {:?}", port);
        insecure_services.push((port, service));
    }

    if insecure_services.len() > 0 {
        global_bind_tcp_ports("0.0.0.0".to_string(), insecure_services, net_kernel_options.recv_buf_size, net_kernel_options.send_buf_size, net_kernel_options.read_buf_cap, net_kernel_options.write_buf_cap, net_kernel_options.pool_size, net_kernel_options.stack_size, net_kernel_options.timeout);
    }

    if secure_services.len() > 0 {
        global_bind_tls_ports("0.0.0.0".to_string(), secure_services, net_kernel_options.recv_buf_size, net_kernel_options.send_buf_size, net_kernel_options.read_buf_cap, net_kernel_options.write_buf_cap, net_kernel_options.pool_size, net_kernel_options.stack_size, net_kernel_options.timeout);
    }

    Ok(())
}

#[derive(Debug, Default, Clone)]
pub struct HttpConfig {
    // 多个主机可以共享一个路由表， 也可以使用不同的路由表
    virtual_hosts: RefCell<FnvHashMap<Vec<String>, Vec<HttpRouteTable>>>,
    port: u16,

    keep_alive_timeout: usize,

    static_cache_max_size: usize,
    static_cache_max_len: usize,
    static_cache_collect_time: u64,

    // 是否允许任意跨域
    cors: bool,
    // 跨域白名单
    cors_allows: RefCell<Vec<CorsAllow>>,

    parser_min_plain_text_size: usize,
    parse_compress_level: Option<u32>,

    multi_parts_block_size: usize,

    file_load_location: String,
    file_load_need_cache: bool,
    file_load_max_age: usize,

    files_load_location: String,
    files_load_need_cache: bool,
    files_load_max_age: usize,

    batch_load_location: String,
    batch_load_need_cache: bool,
    batch_load_max_age: usize,

    upload_file_location: String,

    // 安全连接还是非安全连接
    http_port: bool,
}

unsafe impl Send for HttpConfig {}
unsafe impl Sync for HttpConfig {}

impl HttpConfig {
    pub fn new() -> Self {
        HttpConfig::default()
    }

    pub fn bind_http_port(&mut self, port: u16) {
        self.port = port;
    }

    pub fn config_static_cache(&mut self, max_size: usize, max_len: usize, collect_time: u64) {
        self.static_cache_max_size = max_size;
        self.static_cache_max_len = max_len;
        self.static_cache_collect_time = collect_time;
    }

    pub fn config_set_keep_alive_timeout(&mut self, timeout: usize) {
        self.keep_alive_timeout = timeout;
    }

    pub fn config_cors(&mut self, enable: bool) {
        self.cors = enable;
    }

    pub fn add_cors_allow(&mut self, allow: CorsAllow) {
        self.cors_allows.borrow_mut().push(allow);
    }

    pub fn config_parser(&mut self, min_plain_text_size: usize, compress_level: Option<u32>) {
        self.parser_min_plain_text_size = min_plain_text_size;
        self.parse_compress_level = compress_level;
    }

    pub fn config_multi_parts(&mut self, block_size: usize) {
        self.multi_parts_block_size = block_size;
    }

    pub fn config_file_load(&mut self, location: String, need_cache: bool, max_age: usize) {
        self.file_load_location = location;
        self.file_load_need_cache = need_cache;
        self.file_load_max_age = max_age;
    }

    pub fn config_files_load(&mut self, location: String, need_cache: bool, max_age: usize) {
        self.files_load_location = location;
        self.files_load_need_cache = need_cache;
        self.files_load_max_age = max_age;
    }

    pub fn config_batch_load(&mut self, location: String, need_cache: bool, max_age: usize) {
        self.batch_load_location = location;
        self.batch_load_need_cache = need_cache;
        self.batch_load_max_age = max_age;
    }

    pub fn config_upload_file(&mut self, location: String) {
        self.upload_file_location = location;
    }

    pub fn config_http_port(&mut self, secure: bool) {
        self.http_port = secure;
    }

    // 给虚拟主机添加路由表
    pub fn add_route_for_hosts(&mut self, virtual_hosts: Vec<String>, endpoint: String, methods: Vec<String>, handler_name: String) {
        let route = HttpRouteTable::new(endpoint, methods, handler_name);
        self.virtual_hosts.borrow_mut().entry(virtual_hosts).and_modify(|routes| {
            routes.push(route.clone());
        }).or_insert_with(|| {
            vec![route]
        });
    }
}

#[derive(Debug, Clone)]
pub struct HttpRouteTable {
    endpoint: String,
    methods: Vec<String>,
    handler_name: String,
}

impl HttpRouteTable {
    pub fn new(endpoint: String, methods: Vec<String>, handler_name: String) -> Self {
        HttpRouteTable {
            endpoint,
            methods,
            handler_name
        }
    }
}

#[derive(Debug, Clone)]
pub struct CorsAllow {
    scheme: String,
    host: String,
    port: u16,
    methods: Vec<String>,
    max_age: Option<usize>,
}

impl CorsAllow {
    pub fn new(scheme: String, host: String, port: u16, methods: Vec<String>, max_age: Option<usize>) -> Self {
        CorsAllow {
            scheme,
            host,
            port,
            methods,
            max_age
        }
    }
}

pub struct NetKernelOptions {
    recv_buf_size: usize,
    send_buf_size: usize,
    read_buf_cap: usize,
    write_buf_cap: usize,
    pool_size: usize,
    stack_size: usize,
    timeout: usize
}

impl NetKernelOptions {
    pub fn new(recv_buf_size: usize, send_buf_size: usize, read_buf_cap: usize, write_buf_cap: usize, pool_size: usize, stack_size: usize, timeout: usize) -> Self {
        Self {
            recv_buf_size,
            send_buf_size,
            read_buf_cap,
            write_buf_cap,
            pool_size,
            stack_size,
            timeout
        }
    }
}
/**
* 可以在运行时线程安全的，为全局Mqtt服务器增加指定的主题
*/
pub fn add_global_mqtt_topic(broker_name: String, is_public: bool,   //是否为公共主题，指定用户的主题不是公共主题
                             topic: String) {
    PLAT_MGR.register_mqtt_topic("_$global_mqtt".to_string(), topic.clone()); // 注册全局mqtt topic到平台管理器中
    add_topic(&broker_name, is_public, topic, 0, None);
}

/**
* 可以在运行时线程安全的，在全局Mqtt服务器上发布指定主题的消息
*/
pub fn publish_global_mqtt_topic(broker_name: String, is_public: bool,   //是否为公共主题，指定用户的主题不是公共主题
                                 topic: String, msg: &[u8]) {
    let r = publish_topic(broker_name, is_public, topic, 0, None, Arc::new(msg.to_vec()));
}

/**
* 为指定地址的指定端口，设置指定的网络服务工厂，并绑定对应的Tcp端口
*/
pub fn global_bind_tcp_ports<S: SocketTrait + StreamTrait>(ip: String,                       //绑定的本地ip地址
                                                           binds: Vec<(u16, Box<dyn AsyncServiceFactory<Connect = S, Waits = AsyncWaitsHandle, Out = (), Future = BoxFuture<'static, ()>>>)>,
                                                           recv_buffer_size: usize,          //连接的接收缓冲区，单位B
                                                           send_buffer_size: usize,          //连接的发送缓冲区，单位B
                                                           read_buffer_capacity: usize,      //连接的读缓冲区，单位B
                                                           write_buffer_capacity: usize,     //连接的写缓冲区，单位次
                                                           pool_size: usize,                 //连接池的初始容量
                                                           stack_size: usize,                //连接线程的栈大小
                                                           timeout: usize,                   //连接轮询的间隔时长，单位毫秒
                                                           ) {
    let mut ports = Vec::with_capacity(binds.len());
    let mut factory = AsyncPortsFactory::<S>::new();
    for (port, service) in binds {
        ports.push(port);
        factory.bind(port, service);
    }

    let mut config = SocketConfig::new(&ip, factory.bind_ports().as_slice());
    config.set_option(recv_buffer_size, send_buffer_size, read_buffer_capacity, write_buffer_capacity);
    let buffer = WriteBufferPool::new(10000, 10, 3).ok().unwrap();
    match SocketListener::<S, _>::bind(factory, buffer, config, pool_size, stack_size, 1024, Some(timeout)) {
        Err(e) => {
            panic!("Bind tcp port Error, reason: {:?}", e);
        },
        Ok(_) => {
            info!("===> Bind tcp port ok, ports: {:?}", ports);
        }
    }
}

/**
* 为指定地址的指定端口，设置指定的网络服务工厂，并绑定对应的Tls端口
*/
pub fn global_bind_tls_ports<S: SocketTrait + StreamTrait>(ip: String,                       //绑定的本地ip地址
                                                           binds: Vec<(u16, TlsConfig, Box<dyn AsyncServiceFactory<Connect = S, Waits = AsyncWaitsHandle, Out = (), Future = BoxFuture<'static, ()>>>)>,
                                                           recv_buffer_size: usize,          //连接的接收缓冲区，单位B
                                                           send_buffer_size: usize,          //连接的发送缓冲区，单位B
                                                           read_buffer_capacity: usize,      //连接的读缓冲区，单位B
                                                           write_buffer_capacity: usize,     //连接的写缓冲区，单位次
                                                           pool_size: usize,                 //连接池的初始容量
                                                           stack_size: usize,                //连接线程的栈大小
                                                           timeout: usize,                   //连接轮询的间隔时长，单位毫秒
) {
    let mut ports = Vec::with_capacity(binds.len());
    let mut factory = AsyncPortsFactory::<S>::new();
    for (port, tls_cfg, service) in binds {
        ports.push((port, tls_cfg));
        factory.bind(port, service);
    }

    let mut config = SocketConfig::with_tls(&ip, ports.as_slice());
    config.set_option(recv_buffer_size, send_buffer_size, read_buffer_capacity, write_buffer_capacity);
    let buffer = WriteBufferPool::new(10000, 10, 3).ok().unwrap();
    match SocketListener::<S, _>::bind(factory, buffer, config, pool_size, stack_size, 1024, Some(timeout)) {
        Err(e) => {
            panic!("Bind tcp port Error, reason: {:?}", e);
        },
        Ok(_) => {
            info!("===> Bind tcp port ok, ports: {:?}", ports.iter().cloned().unzip::<_, _, Vec<u16>, Vec<TlsConfig>>().0);
        }
    }
}

pub fn parse_http_config(jstr: String) {
    // 环境变量的ip是以分号分隔的字符串
    let replace_ip = match env::var("PTCONFIG_IP") {
        Ok(ip) => Some(ip),
        Err(_) => None,
    };

    match json::parse(&jstr) {
        Ok(jobj) => {
            for config in jobj["httpConfig"].members() {
                let mut http_config = HttpConfig::new();

                let http_port = config["httpPort"].as_bool().unwrap();
                http_config.config_http_port(config["httpPort"].as_bool().unwrap());

                // 如果配置了环境变量PTCONFIG_IP，则新增虚拟主机
                let virtual_host = match replace_ip.clone() {
                    Some(ip) => {
                        // 如果是 https 配置, 不要用 ip 替换
                        if http_port {
                            config["virtualHost"].members().map(|s|s.to_string()).collect::<Vec<String>>()
                        } else {
                            ip.split(";").map(|s| s.to_string()).collect::<Vec<String>>()
                        }
                    }
                    None => config["virtualHost"].members().map(|s|s.to_string()).collect::<Vec<String>>()
                };

                let mut static_cache_collect_time: u64 = 0;
                let mut static_cache_max_size: usize = 0;
                let mut static_cache_max_len: usize = 0;
                for (key, val) in config["staticCache"].entries() {
                    match key {
                        "maxSize" => static_cache_max_size = val.as_usize().unwrap(),
                        "maxLen" => static_cache_max_len = val.as_usize().unwrap(),
                        "collectTime" => static_cache_collect_time = val.as_u64().unwrap(),
                        _ => warn!("unknown field")
                    }
                }
                http_config.config_static_cache(static_cache_max_size, static_cache_max_len, static_cache_collect_time);

                http_config.config_cors(config["CORS"].as_bool().unwrap());

                for cors_allow in config["CORSAllows"].members() {
                    let scheme = cors_allow["scheme"].as_str().unwrap().to_string();
                    let host = cors_allow["host"].as_str().unwrap().to_string();
                    let port = cors_allow["port"].as_u16().unwrap();
                    let methods = cors_allow["methods"].members().map(|s|s.to_string()).collect::<Vec<String>>();
                    let max_age = cors_allow["maxAge"].as_usize();

                    // 如果配置了环境变量PTCONFIG_IP，则需要新增跨域规则
                    if let Some(ips) = replace_ip.clone() {
                        // 非https跨域配置
                        if !http_port {
                            for ip in ips.split(";") {
                                let c = CorsAllow::new(scheme.clone(), ip.to_string(), port, methods.clone(), max_age);
                                http_config.add_cors_allow(c);
                            }
                        }
                    }
                    let c = CorsAllow::new(scheme, host, port, methods, max_age);
                    http_config.add_cors_allow(c);
                }
                
                let port = config["port"].as_u16().unwrap();
                http_config.bind_http_port(port);
                http_config.config_set_keep_alive_timeout(config["keepAliveTimeout"].as_usize().unwrap());

                let mut parser_min_plain_text_size: usize = 0;
                let mut parse_compress_level: Option<u32> = None;
                for (key, val) in config["parser"].entries() {
                    match key {
                        "minPlainTextSize" => parser_min_plain_text_size = val.as_usize().unwrap(),
                        "compressLevel" => parse_compress_level = val.as_u32(),
                        _ => warn!("unknown field")
                    }
                }
                http_config.config_parser(parser_min_plain_text_size, parse_compress_level);

                let mut multi_parts_block_size: usize = 0;
                for (key, val) in config["mutilParts"].entries() {
                    match key {
                        "blockSize" => multi_parts_block_size = val.as_usize().unwrap(),
                        _ => warn!("unknown field")
                    }
                }
                http_config.config_multi_parts(multi_parts_block_size);

                let mut file_load_location: String = "".to_string();
                let mut file_load_need_cache: bool = false;
                let mut file_load_max_age: usize = 0;
                for (key, val) in config["fileLoad"].entries() {
                    match key {
                        "location" => file_load_location = val.as_str().unwrap().to_string(),
                        "needCache" => file_load_need_cache = val.as_bool().unwrap(),
                        "maxAge" => file_load_max_age = val.as_usize().unwrap(),
                        _ => warn!("unknown field {:?}", key)
                    }
                }
                http_config.config_file_load(file_load_location, file_load_need_cache, file_load_max_age);

                let mut files_load_location: String = "".to_string();
                let mut files_load_need_cache: bool = false;
                let mut files_load_max_age: usize = 0;
                for (key, val) in config["filesLoad"].entries() {
                    match key {
                        "location" => files_load_location = val.as_str().unwrap().to_string(),
                        "needCache" => files_load_need_cache = val.as_bool().unwrap(),
                        "maxAge" => files_load_max_age = val.as_usize().unwrap(),
                        _ => warn!("unknown field {:?}", key)
                    }
                }
                http_config.config_files_load(files_load_location, files_load_need_cache, files_load_max_age);

                let mut batch_load_location: String = "".to_string();
                let mut batch_load_need_cache: bool = false;
                let mut batch_load_max_age: usize = 0;
                for (key, val) in config["batchLoad"].entries() {
                    match key {
                        "location" => batch_load_location = val.as_str().unwrap().to_string(),
                        "needCache" => batch_load_need_cache = val.as_bool().unwrap(),
                        "maxAge" => batch_load_max_age = val.as_usize().unwrap(),
                        _ => warn!("unknown field {:?}", key)
                    }
                }
                http_config.config_batch_load(batch_load_location, batch_load_need_cache, batch_load_max_age);

                for (key, val) in config["uploadFile"].entries() {
                    match key {
                        "location" => http_config.config_upload_file(val.as_str().unwrap().to_string()),
                        _ => warn!("unknown field {:?}", key)
                    }
                }

                for route in config["routeTable"].members() {
                    let endpoint = route["endpoint"].as_str().unwrap().to_string();
                    let methods = route["methods"].members().map(|s|s.to_string()).collect::<Vec<String>>();
                    let handler_name = route["handlerName"].as_str().unwrap().to_string();
                    http_config.add_route_for_hosts(virtual_host.clone(), endpoint, methods, handler_name);
                }
                debug!("parsed http config ----- {:?}", http_config);
                if http_port {
                    SECURE_HTTP_CONFIGS.write().entry(port).and_modify(|configs| configs.push(http_config.clone())).or_insert(vec![http_config]);
                } else {
                    INSECURE_HTTP_CONFIGS.write().entry(port).and_modify(|configs| configs.push(http_config.clone())).or_insert(vec![http_config]);
                }

            }
        }

        Err(e) => {
            panic!("JSON parse error, please make sure it is a json string: {:?}, error: {:?}", jstr, e);
        }
    }
}

/*
* 可以运行时线程安全的关闭指定唯一id的Tcp连接
*/
pub fn close_tcp_socket(uid: usize, reason: String) -> bool {
    close_socket(uid, Err(Error::new(ErrorKind::Other, reason)))
}

pub fn post_gi_data(data: String, url: String) {
    if let Ok(mut client) = create_http_client("test".to_string(), HttpClientOptions::default()) {
        let func = Box::new(move |_r| {
            <HttpClient as SharedHttpc>::add_header(&mut client, Atom::from("content-type"), Atom::from("application/json"));
            let body = HttpClientBody::<String>::body(data);
            post(&client, Atom::from(url), body, Box::new(|r| {
                match r {
                    Ok((a, mut b)) => {
                        debug!("gi response ============= {:?}", b.text());
                    }
                    Err(e) => {
                        warn!("post gi errro ============ {:?}", e);
                    }
                }
            }));
        });
    
        cast_net_task(TaskType::Async(false), 100, None, func, Atom::from("post gi data"));
    }
}

#[derive(Clone)]
enum ConnectType {
    InSecure(ResponseHandler<TcpSocket>),
    Secure(ResponseHandler<FTlsSocket>),
    Unknow
}