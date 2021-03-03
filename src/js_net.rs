use std::cell::RefCell;
use std::env;
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use std::sync::RwLock as StdRwlock;
use std::sync::{Arc, Mutex};

use fnv::FnvHashMap;
use fnv::FnvHashSet;
use futures::future::BoxFuture;
use mqtt3;
use parking_lot::RwLock;
use ws::server::WebsocketListenerFactory;

use atom::Atom;
use gray::{GrayTab, GrayVersion};
use handler::{Args, Handler, SGenType};
use https_external::header::HeaderMap;
use mqtt::server::{
    add_topic, publish_topic, register_listener, register_service, WsMqttBrokerFactory,
    WssMqttBrokerFactory,
};
use mqtt::util::AsyncResult;
use mqtt_proxy::service::{MqttConnectHandle, MqttEvent, MqttProxyListener, MqttProxyService};
use mqtt_tmp::data::Server;
use mqtt_tmp::server::{ClientStub, ServerNode};
use mqtt_tmp::session::Session;
use net::api::{NetManager, TlsManager};
use net::api::{Socket, Stream};
use std::io::Result as IOResult;
use tcp::buffer_pool::WriteBufferPool;
use tcp::connect::TcpSocket;
use tcp::driver::{
    AsyncIOWait, AsyncServiceFactory, Socket as SocketTrait, SocketConfig, Stream as StreamTrait,
};
use tcp::server::{AsyncPortsFactory, AsyncWaitsHandle, SocketListener};
use tcp::tls_connect::TlsSocket as FTlsSocket;
use tcp::util::{close_socket, TlsConfig};

use rusty_v8 as v8;
use vm_builtin::buffer::NativeArrayBuffer;
use vm_builtin::process::{process_close, process_send, process_spawn, Pid, ProcessMsg};
use vm_builtin::ContextHandle;
use vm_core::vm::{send_to_process, JSValue, Vm};

use crate::FILES_ASYNC_RUNTIME;
use crate::HTTP_PORTS;
use crate::MQTT_PORTS;
use hash::XHashMap;
use http::batch_load::BatchLoad;
use http::cors_handler::CORSHandler;
use http::default_parser::DefaultParser;
use http::file_load::FileLoad;
use http::files_load::FilesLoad;
use http::middleware::{Middleware, MiddlewareChain, MiddlewareResult};
use http::multi_parts::MutilParts;
use http::port::HttpPort;
use http::range_load::RangeLoad;
use http::request::HttpRequest;
use http::response::{HttpResponse, ResponseHandler};
use http::route::HttpRoute;
use http::server::HttpListenerFactory;
use http::static_cache::StaticCache;
use http::upload::UploadFile;
use http::virtual_host::VirtualHostPool;
use http::virtual_host::{VirtualHost, VirtualHostTab};
use pi_core::create_snapshot_vm;
use pi_serv_lib::js_gray::GRAY_MGR;
use pi_serv_lib::js_net::{HttpConnect, HttpHeaders, MqttCon};
use pi_serv_lib::{set_pi_serv_handle, PiServNetHandle};

lazy_static! {
    // http Rpc
    static ref HTTP_ENDPOINT: Arc<RwLock<FnvHashMap<String, String>>> = Arc::new(RwLock::new(FnvHashMap::default()));
    // https
    static ref SECURE_SERVICES: Arc<RwLock<Vec<SecureServices>>> = Arc::new(RwLock::new(vec![]));
    // http
    static ref INSECURE_SERVICES: Arc<RwLock<Vec<InsecureServices>>> = Arc::new(RwLock::new(vec![]));
    // 每个端口的证书配置 (port, (cert_path, priv_key_path))
    static ref CERTIFICATES: Arc<RwLock<FnvHashMap<u16, (String, String)>>> = Arc::new(RwLock::new(FnvHashMap::default()));
    // https配置
    static ref SECURE_HTTP_CONFIGS: Arc<RwLock<FnvHashMap<u16, Vec<HttpConfig>>>> = Arc::new(RwLock::new(FnvHashMap::default()));
    // http配置
    static ref INSECURE_HTTP_CONFIGS: Arc<RwLock<FnvHashMap<u16, Vec<HttpConfig>>>> = Arc::new(RwLock::new(FnvHashMap::default()));
    // 记录mqtt mroker中包含的topic
    static ref BROKER_TOPICS: Arc<RwLock<FnvHashMap<String, FnvHashSet<String>>>> = Arc::new(RwLock::new(FnvHashMap::default()));
    // mqtt名称绑定listenerPID
    static ref BUILD_LISTENER_TAB: Arc<RwLock<FnvHashMap<String, Pid>>> = Arc::new(RwLock::new(FnvHashMap::default()));
    // http端口绑定listenerPID
    static ref BUILD_HTTP_LISTENER_TAB: Arc<RwLock<FnvHashMap<String, (Pid, Vm)>>> = Arc::new(RwLock::new(FnvHashMap::default()));
    // 记录所有的静态资源缓存
    pub static ref HTTP_STATIC_CACHES: Arc<RwLock<Vec<Arc<StaticCache>>>> = Arc::new(RwLock::new(vec![]));
}

// 注册pi_ser方法
pub fn reg_pi_serv_handle() {
    let pi_serv_handle = PiServNetHandle {
        bind_mqtt_tcp_port: bind_mqtt_tcp_port,
        start_network_services: start_network_services,
        parse_http_config: parse_http_config,
        config_certificate: config_certificate,
        broker_has_topic: broker_has_topic,
        register_broker_topic: register_broker_topic,
        register_http_endpoint: register_http_endpoint,
        get_http_endpoint: get_http_endpoint,
    };
    // 注入pi_ser_net方法到pi_serv_lib
    set_pi_serv_handle(pi_serv_handle);
}

struct InsecureServices(
    (
        u16,
        Box<
            dyn AsyncServiceFactory<
                Connect = TcpSocket,
                Waits = AsyncWaitsHandle,
                Out = (),
                Future = BoxFuture<'static, ()>,
            >,
        >,
    ),
);
struct SecureServices(
    (
        u16,
        Box<
            dyn AsyncServiceFactory<
                Connect = FTlsSocket,
                Waits = AsyncWaitsHandle,
                Out = (),
                Future = BoxFuture<'static, ()>,
            >,
        >,
    ),
);

unsafe impl Send for InsecureServices {}
unsafe impl Sync for InsecureServices {}

unsafe impl Send for SecureServices {}
unsafe impl Sync for SecureServices {}

fn get_pid(broker_name: &String) -> Pid {
    BUILD_LISTENER_TAB.read().get(broker_name).cloned().unwrap()
}

// 获取http对应的pid
fn get_http_pid(host: &String) -> (Pid, Vm) {
    debug!("get_http_pid host:{:?}", host);
    BUILD_HTTP_LISTENER_TAB.read().get(host).cloned().unwrap()
}

// Mqtt连接和关闭连接处理
#[derive(Clone)]
struct MqttConnectHandler {}

impl MqttConnectHandler {
    pub fn new() -> Self {
        MqttConnectHandler {}
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

    fn handle(
        &self,
        env: Arc<dyn GrayVersion>,
        _topic: Atom,
        args: Args<Self::A, Self::B, Self::C, Self::D, Self::E, Self::F, Self::G, Self::H>,
    ) -> Self::HandleResult {
        let connect = unsafe { Arc::from_raw(Arc::into_raw(env) as *const MqttConnectHandle) };
        let current = match env::var("current").unwrap().as_str() {
            "true" => true,
            _ => false,
        };
        // let connect_id = connect.get_id();
        // let port = connect.get_local_port().unwrap();
        match args {
            Args::OneArgs(MqttEvent::Connect(
                socket_id,
                broker_name,
                client_id,
                keep_alive,
                is_clean_session,
                user,
                pwd,
                result,
            )) => {
                let pid = get_pid(&broker_name);
                //处理Mqtt连接
                let mqtt_connection = MqttCon::new(
                    connect,
                    Some(result),
                    socket_id,
                    client_id,
                    Some(keep_alive),
                    Some(is_clean_session),
                    user,
                    pwd,
                );
                let mqtt_connection_ptr = Arc::into_raw(Arc::new(mqtt_connection)) as usize;
                let mut msgs = Vec::with_capacity(4);
                msgs.push(ProcessMsg::String("connect".to_string()));
                msgs.push(ProcessMsg::Number(socket_id as f64));
                msgs.push(ProcessMsg::Number(mqtt_connection_ptr as f64));
                msgs.push(ProcessMsg::String(broker_name));
                msgs.push(ProcessMsg::Boolean(current));
                // listenerPID发送消息
                send_to_process(None, pid, ProcessMsg::Array(msgs));
            }
            Args::OneArgs(MqttEvent::Disconnect(socket_id, broker_name, _client_id, _reason)) => {
                let pid = get_pid(&broker_name);
                //处理Mqtt连接关闭
                let mut msgs = Vec::with_capacity(2);
                msgs.push(ProcessMsg::String("disconnect".to_string()));
                msgs.push(ProcessMsg::Number(socket_id as f64));
                // listenerPID发送消息
                send_to_process(None, pid, ProcessMsg::Array(msgs));
            }
            _ => panic!("invalid MqttConnectHandler handler args"),
        }
    }
}

// 处理mqtt 订阅，退订和消息发布
#[derive(Clone)]
struct MqttRequestHandler {}

impl MqttRequestHandler {
    pub fn new() -> Self {
        MqttRequestHandler {}
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

    fn handle(
        &self,
        env: Arc<dyn GrayVersion>,
        _topic: Atom,
        args: Args<Self::A, Self::B, Self::C, Self::D, Self::E, Self::F, Self::G, Self::H>,
    ) -> Self::HandleResult {
        let connect = unsafe { Arc::from_raw(Arc::into_raw(env) as *const MqttConnectHandle) };
        let session = connect.get_session().unwrap();
        let context = session.as_ref().get_context();

        let mut pid_option = None; // pid不存在则发送到listenerPID处理

        // 获取会话中的pid
        if let Some(data) = context.get::<(Pid, bool)>() {
            let (pid, current) = data.as_ref().clone();
            debug!("!!!!!!!!!!!!!!!!!js_net pid:{:?}", pid);
            // 限流(达到软上限，就限制rpc并发)
            if current {
                // 获取虚拟机队列长度
                let vm = GRAY_MGR.read().vm_instance(0, pid.0).unwrap();
                let vm_queue_len = vm.queue_len();
                if vm_queue_len > 0 && !connect.is_passive() {
                    connect.set_passive(true);
                } else if connect.is_passive() {
                    connect.set_passive(false);
                }
            }
            pid_option = Some(pid);
        }

        match args {
            Args::OneArgs(MqttEvent::Sub(socket_id, broker_name, _client_id, topics, _result)) => {
                //处理Mqtt订阅主题
                let mut msgs = Vec::new();
                let mut topics_msg = Vec::new();

                for (sub_topic, _) in topics {
                    topics_msg.push(ProcessMsg::String(sub_topic));
                }
                msgs.push(ProcessMsg::String("sub".to_string()));
                msgs.push(ProcessMsg::Number(socket_id as f64));
                msgs.push(ProcessMsg::Array(topics_msg));

                let pid = if let Some(pid) = pid_option {
                    pid
                } else {
                    get_pid(&broker_name)
                };

                // PID发送消息
                send_to_process(None, pid, ProcessMsg::Array(msgs));
            }
            Args::OneArgs(MqttEvent::Unsub(socket_id, broker_name, _client_id, topics)) => {
                //处理Mqtt退订主题
                let mut msgs = Vec::new();
                let mut topics_msg = Vec::new();
                for sub_topic in topics {
                    topics_msg.push(ProcessMsg::String(sub_topic));
                }
                msgs.push(ProcessMsg::String("unsub".to_string()));
                msgs.push(ProcessMsg::Number(socket_id as f64));
                msgs.push(ProcessMsg::Array(topics_msg));

                let pid = if let Some(pid) = pid_option {
                    pid
                } else {
                    get_pid(&broker_name)
                };

                // PID发送消息
                send_to_process(None, pid, ProcessMsg::Array(msgs));
            }
            Args::OneArgs(MqttEvent::Publish(
                socket_id,
                broker_name,
                _client_id,
                _address,
                topic,
                payload,
            )) => {
                //处理Mqtt发布主题
                let payload_copy = payload.to_vec();
                let buf = NativeArrayBuffer::with_shared(payload_copy.into_boxed_slice());

                let mut msgs = Vec::new();
                msgs.push(ProcessMsg::String("publish".to_string()));
                msgs.push(ProcessMsg::Number(socket_id as f64));
                msgs.push(ProcessMsg::String(topic));
                msgs.push(ProcessMsg::SharedArrayBuffer(buf));

                let pid = if let Some(pid) = pid_option {
                    pid
                } else {
                    get_pid(&broker_name)
                };

                // PID发送消息
                send_to_process(None, pid, ProcessMsg::Array(msgs));
            }
            _ => panic!("invalid MqttRequestHandler handler args"),
        }
    }
}

// 判断broker中是否包含rpc topic
pub fn broker_has_topic(broker_name: String, topic: String) -> bool {
    if let Some(set) = BROKER_TOPICS.read().get(&broker_name) {
        if let Some(_) = set.get(&topic) {
            return true;
        }
    }

    // 是否在全局broker name
    if let Some(set) = BROKER_TOPICS.read().get("*") {
        if let Some(_) = set.get(&topic) {
            return true;
        }
    }

    false
}

// 注册topic
pub fn register_broker_topic(broker_name: String, topic: String) -> bool {
    let mut brokers = BROKER_TOPICS.write();

    if let Some(topics) = brokers.get_mut(&broker_name) {
        topics.insert(topic);
    } else {
        let mut set = FnvHashSet::default();
        set.insert(topic);
        brokers.insert(broker_name, set);
    }
    true
}

// 创建listenerPID
pub fn create_listener_pid(port: u16, broker_name: &String) {
    // 判断pid是否存在
    if BUILD_LISTENER_TAB.read().get(broker_name).is_none() {
        // 获取基础灰度对应的vm列表
        let vids = GRAY_MGR.read().gray_vids(0).unwrap();
        // 更加port取余分配vm
        let id = (port as usize) % vids.len();
        let vm = GRAY_MGR.read().vm_instance(0, vids[id]).unwrap();
        let vm_copy = vm.clone();
        let cid = vm.alloc_context_id();
        debug!("!!!!!!!!!!!!!!create_listener_pid cid:{:?}", cid);
        vm.spawn_task(async move {
            let context = vm_copy.new_context(None, cid, None).await.unwrap();
            if let Err(e) = vm_copy
                .execute(
                    context,
                    "start_listener_pid.js",
                    r#"_$listener_set_receive();"#,
                )
                .await
            {
                panic!(e);
            }
        });
        BUILD_LISTENER_TAB
            .write()
            .insert(broker_name.clone(), Pid(vm.get_vid(), cid));
    }
}

// 创建httpPID（每host一个）
pub fn create_http_pid(host: &String, port: u16) {
    debug!(
        "!!!!!!!!!!!!!!create_http_pid host:{:?}, port:{:?}",
        host, port
    );
    let hostPort = host.to_owned() + ":" + port.to_string().as_str();
    // 判断pid是否存在
    if BUILD_HTTP_LISTENER_TAB.read().get(&hostPort).is_none() {
        debug!("!!!!!!!!!!!!!!create_http_pid 1111111111");
        // 获取基础灰度对应的vm列表
        let vids = GRAY_MGR.read().gray_vids(0).unwrap();
        // 更加port取余分配vm
        let id = (port as usize) % vids.len();
        let vm = GRAY_MGR.read().vm_instance(0, vids[id]).unwrap();
        let vm_copy = vm.clone();
        let cid = vm.alloc_context_id();
        debug!(
            "!!!!!!!!!!!!!!create_http_pid 222222222 vid:{:?}",
            vm.get_vid()
        );
        vm.spawn_task(async move {
            let context = vm_copy.new_context(None, cid, None).await.unwrap();
            if let Err(e) = vm_copy
                .execute(
                    context,
                    "http_session_pid.js",
                    r#"_$http_rpc_set_receive();"#,
                )
                .await
            {
                panic!(e);
            }
        });
        debug!("!!!!!!!!!!!!!!create_http_pid 333333333333333");
        BUILD_HTTP_LISTENER_TAB
            .write()
            .insert(hostPort, (Pid(vm.get_vid(), cid), vm));
    }
}

// 绑定mqtt监听器
pub fn bind_mqtt_tcp_port(port: u16, use_tls: bool, protocol: String, broker_name: String) {
    let inserted = MQTT_PORTS.lock().insert((port, broker_name.clone()));
    if inserted {
        let event_handler = Arc::new(MqttConnectHandler::new());
        let rpc_handler = Arc::new(MqttRequestHandler::new());
        let listener = Arc::new(MqttProxyListener::with_handler(Some(event_handler)));
        let service = Arc::new(MqttProxyService::with_handler(Some(rpc_handler)));

        if use_tls {
            let broker_factory = Arc::new(WssMqttBrokerFactory::new(&protocol, &broker_name, port));

            SECURE_SERVICES.write().push(SecureServices((
                port.clone(),
                Box::new(
                    WebsocketListenerFactory::<FTlsSocket>::with_protocol_factory(broker_factory),
                ),
            )));
        } else {
            let broker_factory = Arc::new(WsMqttBrokerFactory::new(&protocol, &broker_name, port));

            INSECURE_SERVICES.write().push(InsecureServices((
                port.clone(),
                Box::new(
                    WebsocketListenerFactory::<TcpSocket>::with_protocol_factory(broker_factory),
                ),
            )));
        }
        register_listener(&broker_name, listener);
        register_service(&broker_name, service);
    }
}

// httpMsg包装
#[derive(Clone)]
struct HttpMsg(pub Arc<RefCell<XHashMap<String, SGenType>>>);

unsafe impl Send for HttpMsg {}

#[derive(Clone)]
pub struct SecureHttpRpcRequestHandler {}

#[derive(Clone)]
pub struct InsecureHttpRpcRequstHandler {}

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
    fn handle(
        &self,
        _env: Arc<dyn GrayVersion>,
        topic: Atom,
        args: Args<Self::A, Self::B, Self::C, Self::D, Self::E, Self::F, Self::G, Self::H>,
    ) -> Self::HandleResult {
        match args {
            Args::FourArgs(addr, headers, msg, handler) => {
                for (key, value) in headers.iter() {
                    debug!("!!!!!!!!!!!!!!headers:{:?}: {:?}", key, value);
                }
                let host_c = headers.get("host").unwrap().to_str().unwrap();
                let host = if let None = host_c.find(":") {
                    host_c.to_owned() + ":80"
                } else {
                    host_c.to_string()
                };
                let (pid, _vm) = get_http_pid(&host);
                //  http连接
                let mut http_connect = HttpConnect::new(addr);
                http_connect.set_insecure_resp_handle(handler);
                let con_ptr = Box::into_raw(Box::new(http_connect)) as usize;
                // http请求头
                let http_header = HttpHeaders::new(headers);
                let handlers_ptr = Box::into_raw(Box::new(http_header)) as usize;
                let msg = HttpMsg(msg);
                let topic = (*topic).clone();
                let data = set_data(msg);
                let mut msgs = Vec::with_capacity(4);
                msgs.push(ProcessMsg::Number(con_ptr as f64));
                msgs.push(ProcessMsg::Number(handlers_ptr as f64));
                msgs.push(ProcessMsg::String(topic));
                msgs.push(ProcessMsg::Array(data));
                // PID发送消息
                send_to_process(None, pid, ProcessMsg::Array(msgs));
            }
            _ => panic!("invalid HttpRpcRequestHandler handler args"),
        }
    }
}

impl InsecureHttpRpcRequstHandler {
    pub fn new() -> Self {
        InsecureHttpRpcRequstHandler {}
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
    fn handle(
        &self,
        _env: Arc<dyn GrayVersion>,
        topic: Atom,
        args: Args<Self::A, Self::B, Self::C, Self::D, Self::E, Self::F, Self::G, Self::H>,
    ) -> Self::HandleResult {
        match args {
            Args::FourArgs(addr, headers, msg, handler) => {
                for (key, value) in headers.iter() {
                    debug!("!!!!!!!!!!!!!!headers:{:?}: {:?}", key, value);
                }
                let host_c = headers.get("host").unwrap().to_str().unwrap();
                let host = if let None = host_c.find(":") {
                    host_c.to_owned() + ":443"
                } else {
                    host_c.to_string()
                };

                let (pid, _vm) = get_http_pid(&host);
                //  http连接
                let mut http_connect = HttpConnect::new(addr);
                http_connect.set_secure_resp_handle(handler);
                let con_ptr = Box::into_raw(Box::new(http_connect)) as usize;
                // http请求头
                let http_header = HttpHeaders::new(headers);
                let handlers_ptr = Box::into_raw(Box::new(http_header)) as usize;
                let msg = HttpMsg(msg);
                let topic = (*topic).clone();
                let data = set_data(msg);
                let mut msgs = Vec::with_capacity(4);
                msgs.push(ProcessMsg::Number(con_ptr as f64));
                msgs.push(ProcessMsg::Number(handlers_ptr as f64));
                msgs.push(ProcessMsg::String(topic));
                msgs.push(ProcessMsg::Array(data));
                // PID发送消息
                send_to_process(None, pid, ProcessMsg::Array(msgs));
            }
            _ => panic!("invalid HttpRpcRequestHandler handler args"),
        }
    }
}

// 设置http请求参数
fn set_data(msg: HttpMsg) -> Vec<ProcessMsg> {
    let mut data = Vec::new();
    let values: Vec<(String, SGenType)> = msg
        .0
        .borrow()
        .iter()
        .map(|(key, val)| (key.clone(), val.clone()))
        .collect();
    for (key, val) in values {
        data.push(ProcessMsg::String(key));
        match val {
            SGenType::Str(s) => {
                data.push(ProcessMsg::String(s));
            }
            SGenType::Bin(bin) => {
                let buf = NativeArrayBuffer::with_shared(bin.into_boxed_slice());
                data.push(ProcessMsg::SharedArrayBuffer(buf));
            }
            _ => {
                unimplemented!();
            }
        }
    }
    data
}

// 注册http rpc
pub fn register_http_endpoint(key: String, val: String) {
    HTTP_ENDPOINT.write().insert(key, val);
}

// 获取http Rpc
pub fn get_http_endpoint(key: &str) -> Option<String> {
    HTTP_ENDPOINT.read().get(key).cloned()
}

impl SecureHttpRpcRequestHandler {
    pub fn new() -> Self {
        SecureHttpRpcRequestHandler {}
    }
}

fn build_secure_service() -> Result<(), String> {
    for (port, http_configs) in SECURE_HTTP_CONFIGS.read().iter() {
        let handler = Arc::new(SecureHttpRpcRequestHandler::new());
        let http_port = Arc::new(HttpPort::with_handler(None, handler));

        let r = build_service::<FTlsSocket>(port.clone(), http_configs, http_port);
        SECURE_SERVICES.write().push(SecureServices(r));
    }
    Ok(())
}

fn build_insecure_service() -> Result<(), String> {
    for (port, http_configs) in INSECURE_HTTP_CONFIGS.read().iter() {
        let handler = Arc::new(InsecureHttpRpcRequstHandler::new());
        let http_port = Arc::new(HttpPort::with_handler(None, handler));

        let r = build_service::<TcpSocket>(port.clone(), http_configs, http_port);
        INSECURE_SERVICES.write().push(InsecureServices(r));
    }

    Ok(())
}

// 启动网络服务
pub fn start_network_services(
    recv_buf_size: usize,
    send_buf_size: usize,
    read_buf_cap: usize,
    write_buf_cap: usize,
    pool_size: usize,
    stack_size: usize,
    timeout: usize,
) -> Result<(), String> {
    // 准备安全服务配置
    build_secure_service()?;
    // 准备非安全服务配置
    build_insecure_service()?;

    let mut secure_services: Vec<(
        u16,
        TlsConfig,
        Box<
            dyn AsyncServiceFactory<
                Connect = FTlsSocket,
                Waits = AsyncWaitsHandle,
                Out = (),
                Future = BoxFuture<'static, ()>,
            >,
        >,
    )> = vec![];
    for SecureServices((port, service)) in SECURE_SERVICES.write().drain(..).into_iter() {
        debug!("start_network_services secure service port = {:?}", port);
        match CERTIFICATES.read().get(&port) {
            Some((cert_path, priv_key_path)) => {
                let tls_config = TlsConfig::new_server(
                    "",
                    false,
                    cert_path,
                    priv_key_path,
                    "",
                    "",
                    "",
                    512,
                    false,
                    "",
                )
                .unwrap();

                secure_services.push((port, tls_config, service));
            }
            None => panic!(
                "port {:?} configured use TLS, but no certificate specified",
                port
            ),
        }
    }

    let mut insecure_services: Vec<(
        u16,
        Box<
            dyn AsyncServiceFactory<
                Connect = TcpSocket,
                Waits = AsyncWaitsHandle,
                Out = (),
                Future = BoxFuture<'static, ()>,
            >,
        >,
    )> = vec![];
    for InsecureServices((port, service)) in INSECURE_SERVICES.write().drain(..).into_iter() {
        debug!("start_network_services insecure service port = {:?}", port);
        insecure_services.push((port, service));
    }

    if insecure_services.len() > 0 {
        global_bind_tcp_ports(
            "0.0.0.0".to_string(),
            insecure_services,
            recv_buf_size,
            send_buf_size,
            read_buf_cap,
            write_buf_cap,
            pool_size,
            stack_size,
            timeout,
        );
    }

    if secure_services.len() > 0 {
        global_bind_tls_ports(
            "0.0.0.0".to_string(),
            secure_services,
            recv_buf_size,
            send_buf_size,
            read_buf_cap,
            write_buf_cap,
            pool_size,
            stack_size,
            timeout,
        );
    }

    Ok(())
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
    pub fn new(
        scheme: String,
        host: String,
        port: u16,
        methods: Vec<String>,
        max_age: Option<usize>,
    ) -> Self {
        CorsAllow {
            scheme,
            host,
            port,
            methods,
            max_age,
        }
    }
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
    pub fn add_route_for_hosts(
        &mut self,
        virtual_hosts: Vec<String>,
        endpoint: String,
        methods: Vec<String>,
        handler_name: String,
    ) {
        let route = HttpRouteTable::new(endpoint, methods, handler_name);
        self.virtual_hosts
            .borrow_mut()
            .entry(virtual_hosts)
            .and_modify(|routes| {
                routes.push(route.clone());
            })
            .or_insert_with(|| vec![route]);
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
            handler_name,
        }
    }
}

/**
* 为指定地址的指定端口，设置指定的网络服务工厂，并绑定对应的Tls端口
*/
pub fn global_bind_tls_ports<S: SocketTrait + StreamTrait>(
    ip: String, //绑定的本地ip地址
    binds: Vec<(
        u16,
        TlsConfig,
        Box<
            dyn AsyncServiceFactory<
                Connect = S,
                Waits = AsyncWaitsHandle,
                Out = (),
                Future = BoxFuture<'static, ()>,
            >,
        >,
    )>,
    recv_buffer_size: usize,      //连接的接收缓冲区，单位B
    send_buffer_size: usize,      //连接的发送缓冲区，单位B
    read_buffer_capacity: usize,  //连接的读缓冲区，单位B
    write_buffer_capacity: usize, //连接的写缓冲区，单位次
    pool_size: usize,             //连接池的初始容量
    stack_size: usize,            //连接线程的栈大小
    timeout: usize,               //连接轮询的间隔时长，单位毫秒
) {
    let mut ports = Vec::with_capacity(binds.len());
    let mut factory = AsyncPortsFactory::<S>::new();
    for (port, tls_cfg, service) in binds {
        ports.push((port, tls_cfg));
        factory.bind(port, service);
    }

    let mut config = SocketConfig::with_tls(&ip, ports.as_slice());
    config.set_option(
        recv_buffer_size,
        send_buffer_size,
        read_buffer_capacity,
        write_buffer_capacity,
    );
    let buffer = WriteBufferPool::new(10000, 10, 3).ok().unwrap();
    match SocketListener::<S, _>::bind(
        factory,
        buffer,
        config,
        pool_size,
        stack_size,
        1024,
        Some(timeout),
    ) {
        Err(e) => {
            panic!("Bind tcp port Error, reason: {:?}", e);
        }
        Ok(_) => {
            info!(
                "===> Bind tcp port ok, ports: {:?}",
                ports
                    .iter()
                    .cloned()
                    .unzip::<_, _, Vec<u16>, Vec<TlsConfig>>()
                    .0
            );
        }
    }
}

/**
* 为指定地址的指定端口，设置指定的网络服务工厂，并绑定对应的Tcp端口
*/
pub fn global_bind_tcp_ports<S: SocketTrait + StreamTrait>(
    ip: String, //绑定的本地ip地址
    binds: Vec<(
        u16,
        Box<
            dyn AsyncServiceFactory<
                Connect = S,
                Waits = AsyncWaitsHandle,
                Out = (),
                Future = BoxFuture<'static, ()>,
            >,
        >,
    )>,
    recv_buffer_size: usize,      //连接的接收缓冲区，单位B
    send_buffer_size: usize,      //连接的发送缓冲区，单位B
    read_buffer_capacity: usize,  //连接的读缓冲区，单位B
    write_buffer_capacity: usize, //连接的写缓冲区，单位次
    pool_size: usize,             //连接池的初始容量
    stack_size: usize,            //连接线程的栈大小
    timeout: usize,               //连接轮询的间隔时长，单位毫秒
) {
    let mut ports = Vec::with_capacity(binds.len());
    let mut factory = AsyncPortsFactory::<S>::new();
    for (port, service) in binds {
        ports.push(port);
        factory.bind(port, service);
    }

    let mut config = SocketConfig::new(&ip, factory.bind_ports().as_slice());
    config.set_option(
        recv_buffer_size,
        send_buffer_size,
        read_buffer_capacity,
        write_buffer_capacity,
    );
    let buffer = WriteBufferPool::new(10000, 10, 3).ok().unwrap();
    match SocketListener::<S, _>::bind(
        factory,
        buffer,
        config,
        pool_size,
        stack_size,
        1024,
        Some(timeout),
    ) {
        Err(e) => {
            panic!("Bind tcp port Error, reason: {:?}", e);
        }
        Ok(_) => {
            info!("===> Bind tcp port ok, ports: {:?}", ports);
        }
    }
}

fn build_service<S: SocketTrait + StreamTrait>(
    port: u16,
    http_configs: &Vec<HttpConfig>,
    http_port: Arc<HttpPort<S>>,
) -> (
    u16,
    Box<
        dyn AsyncServiceFactory<
            Connect = S,
            Waits = AsyncWaitsHandle,
            Out = (),
            Future = BoxFuture<'static, ()>,
        >,
    >,
) {
    let mut hosts = VirtualHostTab::new();
    let mut keep_alive = 60000;
    for http_config in http_configs {
        if http_config.keep_alive_timeout > 0 {
            keep_alive = http_config.keep_alive_timeout;
        }
        let enable_cache = http_config.static_cache_max_len > 0
            && http_config.static_cache_max_size > 0
            && http_config.static_cache_collect_time > 0;
        let cors_handler = if http_config.cors {
            Arc::new(CORSHandler::new(
                "OPTIONS, GET, POST".to_string(),
                Some(365 * 24 * 60 * 60),
            ))
        } else {
            Arc::new(CORSHandler::new("OPTIONS, GET, POST".to_string(), None))
        };

        if !http_config.cors {
            for config in http_config.cors_allows.borrow().iter() {
                if let Err(e) = cors_handler.allow_origin(
                    config.scheme.clone(),
                    config.host.clone(),
                    config.port,
                    &config.methods,
                    &[],
                    config.max_age,
                ) {
                    panic!(
                        "failed to add origin, error = {:?}, config= {:?}",
                        e, config
                    );
                }
            }
        }

        let parser = Arc::new(DefaultParser::with(
            http_config.parser_min_plain_text_size,
            http_config.parse_compress_level,
        ));
        let multi_parts = Arc::new(MutilParts::with(http_config.multi_parts_block_size));
        let range_load = Arc::new(RangeLoad::new());

        let file_load;
        let files_load;
        let batch_load;

        if enable_cache {
            let cache = Arc::new(StaticCache::new(
                http_config.static_cache_max_size,
                http_config.static_cache_max_len,
            ));
            // 记录静态缓存对象，前端资源热更时删除所有缓存
            HTTP_STATIC_CACHES.write().push(cache.clone());
            StaticCache::run_collect(
                cache.clone(),
                "http cache".to_string(),
                http_config.static_cache_collect_time,
            );
            file_load = Arc::new(FileLoad::new(
                FILES_ASYNC_RUNTIME.clone(),
                http_config.file_load_location.clone(),
                Some(cache.clone()),
                http_config.file_load_need_cache,
                true,
                true,
                false,
                http_config.file_load_max_age,
            ));
            files_load = Arc::new(FilesLoad::new(
                FILES_ASYNC_RUNTIME.clone(),
                http_config.files_load_location.clone(),
                Some(cache.clone()),
                http_config.files_load_need_cache,
                true,
                true,
                false,
                http_config.files_load_max_age,
            ));
            batch_load = Arc::new(BatchLoad::new(
                FILES_ASYNC_RUNTIME.clone(),
                http_config.batch_load_location.clone(),
                Some(cache.clone()),
                http_config.batch_load_need_cache,
                true,
                true,
                false,
                http_config.batch_load_max_age,
            ));
        } else {
            file_load = Arc::new(FileLoad::new(
                FILES_ASYNC_RUNTIME.clone(),
                http_config.file_load_location.clone(),
                None,
                http_config.file_load_need_cache,
                true,
                true,
                false,
                http_config.file_load_max_age,
            ));
            files_load = Arc::new(FilesLoad::new(
                FILES_ASYNC_RUNTIME.clone(),
                http_config.files_load_location.clone(),
                None,
                http_config.files_load_need_cache,
                true,
                true,
                false,
                http_config.files_load_max_age,
            ));
            batch_load = Arc::new(BatchLoad::new(
                FILES_ASYNC_RUNTIME.clone(),
                http_config.batch_load_location.clone(),
                None,
                http_config.batch_load_need_cache,
                true,
                true,
                false,
                http_config.batch_load_max_age,
            ));
        }

        let upload = Arc::new(UploadFile::new(
            FILES_ASYNC_RUNTIME.clone(),
            http_config.upload_file_location.clone(),
        ));

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
                .at("/")
                .options(cors_middleware.clone())
                .at("/**")
                .options(cors_middleware.clone());

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
                debug!(
                    "add insecure host = {:?}, port = {:?}",
                    vh, http_config.port
                );
                let _ = hosts.add(vh, host.clone());
            }
        }
    }

    (
        port.clone(),
        Box::new(HttpListenerFactory::with_hosts(hosts.clone(), keep_alive)),
    )
}

// 解析http配置
pub fn parse_http_config(jstr: String) {
    debug!("!!!!!!!!!!!parse_http_config");
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
                            config["virtualHost"]
                                .members()
                                .map(|s| s.to_string())
                                .collect::<Vec<String>>()
                        } else {
                            ip.split(";")
                                .map(|s| s.to_string())
                                .collect::<Vec<String>>()
                        }
                    }
                    None => config["virtualHost"]
                        .members()
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>(),
                };

                let mut static_cache_collect_time: u64 = 0;
                let mut static_cache_max_size: usize = 0;
                let mut static_cache_max_len: usize = 0;
                for (key, val) in config["staticCache"].entries() {
                    match key {
                        "maxSize" => static_cache_max_size = val.as_usize().unwrap(),
                        "maxLen" => static_cache_max_len = val.as_usize().unwrap(),
                        "collectTime" => static_cache_collect_time = val.as_u64().unwrap(),
                        _ => warn!("unknown field"),
                    }
                }
                http_config.config_static_cache(
                    static_cache_max_size,
                    static_cache_max_len,
                    static_cache_collect_time,
                );

                http_config.config_cors(config["CORS"].as_bool().unwrap());

                for cors_allow in config["CORSAllows"].members() {
                    let scheme = cors_allow["scheme"].as_str().unwrap().to_string();
                    let host = cors_allow["host"].as_str().unwrap().to_string();
                    let port = cors_allow["port"].as_u16().unwrap();
                    let methods = cors_allow["methods"]
                        .members()
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>();
                    let max_age = cors_allow["maxAge"].as_usize();

                    // 如果配置了环境变量PTCONFIG_IP，则需要新增跨域规则
                    if let Some(ips) = replace_ip.clone() {
                        // 非https跨域配置
                        if !http_port {
                            for ip in ips.split(";") {
                                let c = CorsAllow::new(
                                    scheme.clone(),
                                    ip.to_string(),
                                    port,
                                    methods.clone(),
                                    max_age,
                                );
                                http_config.add_cors_allow(c);
                            }
                        }
                    }
                    let c = CorsAllow::new(scheme, host, port, methods, max_age);
                    http_config.add_cors_allow(c);
                }

                let port = config["port"].as_u16().unwrap();
                debug!("!!!!!!!!!!!!!!parse_http_config 1111111111");
                // 注册http rpc
                for host in &virtual_host {
                    HTTP_PORTS.lock().push((port, host.clone()));
                }
                debug!("!!!!!!!!!!!!!!parse_http_config 222222222");
                http_config.bind_http_port(port);
                http_config
                    .config_set_keep_alive_timeout(config["keepAliveTimeout"].as_usize().unwrap());

                let mut parser_min_plain_text_size: usize = 0;
                let mut parse_compress_level: Option<u32> = None;
                for (key, val) in config["parser"].entries() {
                    match key {
                        "minPlainTextSize" => parser_min_plain_text_size = val.as_usize().unwrap(),
                        "compressLevel" => parse_compress_level = val.as_u32(),
                        _ => warn!("unknown field"),
                    }
                }
                http_config.config_parser(parser_min_plain_text_size, parse_compress_level);

                let mut multi_parts_block_size: usize = 0;
                for (key, val) in config["mutilParts"].entries() {
                    match key {
                        "blockSize" => multi_parts_block_size = val.as_usize().unwrap(),
                        _ => warn!("unknown field"),
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
                        _ => warn!("unknown field {:?}", key),
                    }
                }
                http_config.config_file_load(
                    file_load_location,
                    file_load_need_cache,
                    file_load_max_age,
                );

                let mut files_load_location: String = "".to_string();
                let mut files_load_need_cache: bool = false;
                let mut files_load_max_age: usize = 0;
                for (key, val) in config["filesLoad"].entries() {
                    match key {
                        "location" => files_load_location = val.as_str().unwrap().to_string(),
                        "needCache" => files_load_need_cache = val.as_bool().unwrap(),
                        "maxAge" => files_load_max_age = val.as_usize().unwrap(),
                        _ => warn!("unknown field {:?}", key),
                    }
                }
                http_config.config_files_load(
                    files_load_location,
                    files_load_need_cache,
                    files_load_max_age,
                );

                let mut batch_load_location: String = "".to_string();
                let mut batch_load_need_cache: bool = false;
                let mut batch_load_max_age: usize = 0;
                for (key, val) in config["batchLoad"].entries() {
                    match key {
                        "location" => batch_load_location = val.as_str().unwrap().to_string(),
                        "needCache" => batch_load_need_cache = val.as_bool().unwrap(),
                        "maxAge" => batch_load_max_age = val.as_usize().unwrap(),
                        _ => warn!("unknown field {:?}", key),
                    }
                }
                http_config.config_batch_load(
                    batch_load_location,
                    batch_load_need_cache,
                    batch_load_max_age,
                );

                for (key, val) in config["uploadFile"].entries() {
                    match key {
                        "location" => {
                            http_config.config_upload_file(val.as_str().unwrap().to_string())
                        }
                        _ => warn!("unknown field {:?}", key),
                    }
                }

                for route in config["routeTable"].members() {
                    let endpoint = route["endpoint"].as_str().unwrap().to_string();
                    let methods = route["methods"]
                        .members()
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>();
                    let handler_name = route["handlerName"].as_str().unwrap().to_string();
                    http_config.add_route_for_hosts(
                        virtual_host.clone(),
                        endpoint,
                        methods,
                        handler_name,
                    );
                }
                debug!("parsed http config ----- {:?}", http_config);
                if http_port {
                    SECURE_HTTP_CONFIGS
                        .write()
                        .entry(port)
                        .and_modify(|configs| configs.push(http_config.clone()))
                        .or_insert(vec![http_config]);
                } else {
                    INSECURE_HTTP_CONFIGS
                        .write()
                        .entry(port)
                        .and_modify(|configs| configs.push(http_config.clone()))
                        .or_insert(vec![http_config]);
                }
            }
        }

        Err(e) => {
            panic!(
                "JSON parse error, please make sure it is a json string: {:?}, error: {:?}",
                jstr, e
            );
        }
    }
}

// 配置证书
pub fn config_certificate(port: u16, cert_path: String, priv_key_path: String) {
    CERTIFICATES
        .write()
        .insert(port, (cert_path, priv_key_path));
}
