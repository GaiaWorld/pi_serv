#![feature(fs_read_write)]
#![feature(splice)]
#![feature(exclusive_range_pattern)]
#![feature(unboxed_closures)]
#![feature(vec_remove_item)]
#![feature(nll)]
#[warn(dead_code)]
extern crate clap;
extern crate core;
extern crate fnv;
extern crate json;
extern crate magnetic;
extern crate mqtt_tmp;
extern crate net;
extern crate nodec;
extern crate pi_crypto;
extern crate pi_db;
extern crate pi_vm;
extern crate rand;
extern crate rpc_tmp;
extern crate toml;
// extern crate pi_p2p;
extern crate apm;
extern crate atom;
extern crate bon;
extern crate file;
extern crate gray;
extern crate guid;
extern crate guid64;
extern crate handler;
extern crate hash_value;
extern crate httpc;
extern crate http;
extern crate https;
extern crate https_external;
extern crate hash;
extern crate libc;
extern crate mqtt;
extern crate mqtt_proxy;
extern crate mqtt3;
extern crate ordmap;
extern crate pi_store;
extern crate sinfo;
extern crate tcp;
extern crate time;
extern crate timer;
extern crate util as lib_util;
extern crate worker;
extern crate ws;
extern crate parking_lot;
extern crate futures;
extern crate crossbeam_channel;

extern crate hex;
extern crate regex;
extern crate serde_json;
extern crate dashmap;
extern crate wheel;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

#[macro_use]
extern crate env_logger;
extern crate chrono;

#[cfg(any(unix))]
extern crate glob;

pub mod hotfix;
pub mod init_js;
pub mod js_async;
pub mod js_base;
pub mod js_db;
pub mod js_env;
pub mod js_file;
pub mod js_httpc;
pub mod js_lib;
pub mod js_net;
pub mod js_net_rpc_client;
pub mod js_vm;
pub mod util;
pub mod webshell;
pub mod ptmgr;
pub mod binary;
pub mod timer_task;

mod def_build;
mod js_util;
mod pi_crypto_build;
mod pi_db_build;
mod pi_lib_gray_build;
mod pi_lib_guid_build;
mod pi_lib_guid_build64;
mod pi_lib_sinfo_build;
mod pi_math_hash_build;
mod pi_net_net_build;
mod pi_serv_build;
mod pi_vm_build;
// mod pi_p2p_build;
mod pi_net_httpc_build;
mod pi_net_https_build;
mod pi_store_build;
mod license_client;

use std::env;
use std::io::{self, Result as IOResult, Write, BufReader, Read};
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crossbeam_channel::unbounded;
use clap::{App, Arg, ArgMatches};
#[cfg(not(unix))]
use pi_vm::adapter::load_lib_backtrace;
use pi_vm::adapter::{
    register_global_vm_heap_collect_timer, register_native_object, set_vm_timeout,
};
use pi_vm::adapter::{JSType, JS};
use pi_vm::bonmgr::{jstype_ptr, ptr_jstype, BonMgr, CallResult, FnMeta, BON_MGR};
use pi_vm::pi_vm_impl::push_callback;
use pi_vm::shell::SHELL_MANAGER;

use atom::Atom;
use time::now_millisecond;
use timer::TIMER;
use worker::impls::{
    JS_TASK_POOL, JS_WORKER_WALKER, NET_TASK_POOL, NET_WORKER_WALKER, STORE_TASK_POOL,
    STORE_WORKER_WALKER, TASK_POOL_TIMER,
};
use worker::worker::WorkerType;
use worker::worker_pool::WorkerPool;

use init_js::exec_js;
use webshell::init_shell;
use js_env::{current_dir, env_var, set_current_dir, set_env_var};

use apm::allocator::{get_max_alloced_limit, set_max_alloced_limit, CounterSystemAllocator};
use apm::common::SysStat;
use ptmgr::PLAT_MGR;

use js_net::InsecureHttpRpcRequstHandler;
use http::{server::HttpListenerFactory,
    virtual_host::{VirtualHostTab, VirtualHost},
    route::HttpRoute,
    middleware::MiddlewareChain,
    default_parser::DefaultParser,
    port::HttpPort};
use http::virtual_host::VirtualHostPool;
use tcp::server::{AsyncPortsFactory, SocketListener};
use tcp::connect::TcpSocket;
use tcp::driver::{SocketConfig};
use tcp::buffer_pool::WriteBufferPool;
use tcp::util::{TlsConfig};
use std::fs::File;
use license_client::License;
use binary::Binary;
use timer_task::tick;
use chrono::Local;
use hotfix::{NOTIFY_CHAN, GRAY_TABLE};

#[global_allocator]
static ALLOCATOR: CounterSystemAllocator = CounterSystemAllocator;

fn args() -> clap::ArgMatches<'static> {
    let matches = App::new("pi_server")
        .version("1.0")
        .author("test. <test@gmail.com>")
        .about("aboutXXXX")
        .arg(
            Arg::with_name("shell")
                .short("s")
                .long("shell")
                .value_name("BOOL")
                .takes_value(true)
                .help("Open the console at startup"),
        )
        .arg(
            Arg::with_name("max_heap")
                .short("H")
                .long("max_heap")
                .value_name("GByte")
                .takes_value(true)
                .help("Max heap limit on runtime"),
        )
        .arg(
            Arg::with_name("exec_file")
                .short("e")
                .long("init")
                .value_name("INIT_FILE")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("projects")
                .short("p")
                .long("projects")
                .multiple(true)
                .value_name("PROJECTS")
                .takes_value(true),
        )
        .get_matches();
    matches
}

// 设置pi_pt需要用到的环境变量
fn set_piserv_env_var(matches: &ArgMatches) {
    let init_exec_path = matches.value_of("exec_file").unwrap();
    let projs = match matches.values_of("projects") {
        Some(p) => p
            .map(|s| s.to_string().replace("\\", "/"))
            .collect::<Vec<String>>(),
        None => vec![],
    };
    let current_dir = env::current_dir().unwrap();
    let current_dir_parent = current_dir.parent().unwrap().to_str().unwrap();

    let path = Path::new(init_exec_path)
        .iter()
        .filter_map(|x| if x == "." || x == ".." { None } else { Some(x) })
        .map(|x| x.to_str().unwrap())
        .collect::<Vec<&str>>();

    let root: PathBuf = [vec![current_dir_parent], path].concat().iter().collect();
    let project_root = root
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_str()
        .unwrap()
        .replace("\\", "/");

    let cur_dir = current_dir.to_str().unwrap();

    set_env_var("PROJECTS", &projs.as_slice().join(" "));

    let cur_dir = env::current_dir();

    // 如果没有出现 -p 参数
    if matches.occurrences_of("projects") == 0 {
        set_env_var("PROJECT_ROOT", cur_dir.unwrap().to_str().unwrap());
    } else {
        set_env_var("PROJECT_ROOT", &project_root);
    }

    set_env_var("PATH_SEPERATOR", &MAIN_SEPARATOR.to_string());
}

// 启动存储任务工作线程
fn start_store_worker(processor: usize) {
    let worker_pool1 = Box::new(WorkerPool::new(
        "Store Worker".to_string(),
        WorkerType::Store,
        processor,
        1024 * 1024,
        10000,
        STORE_WORKER_WALKER.clone(),
    ));
    worker_pool1.run(STORE_TASK_POOL.clone());
}

// 启动js虚拟机工作线程
fn start_js_worker(processor: usize) {
    let worker_pool0 = Box::new(WorkerPool::new(
        "JS Worker".to_string(),
        WorkerType::Js,
        processor * 2,
        1024 * 1024,
        10000,
        JS_WORKER_WALKER.clone(),
    ));
    worker_pool0.run(JS_TASK_POOL.clone());
}

// 启动网络io工作线程
fn start_network_worker(processor: usize) {
    let worker_pool = Box::new(WorkerPool::new(
        "Network Worker".to_string(),
        WorkerType::Net,
        processor,
        1024 * 1024,
        30000,
        NET_WORKER_WALKER.clone(),
    ));
    worker_pool.run(NET_TASK_POOL.clone());
}

// 注册rust暴露的函数
fn register_bon_mgr() {
    pi_crypto_build::register(&BON_MGR);
    pi_math_hash_build::register(&BON_MGR);
    pi_db_build::register(&BON_MGR);
    pi_lib_guid_build::register(&BON_MGR);
    pi_lib_guid_build64::register(&BON_MGR);
    pi_lib_gray_build::register(&BON_MGR);
    pi_lib_sinfo_build::register(&BON_MGR);
    pi_db_build::register(&BON_MGR);
    def_build::register(&BON_MGR);
    pi_net_net_build::register(&BON_MGR);
    pi_serv_build::register(&BON_MGR);
    pi_vm_build::register(&BON_MGR);
    js_async::register(&BON_MGR);
    // pi_p2p_build::register(&BON_MGR);
    pi_net_httpc_build::register(&BON_MGR);
    pi_net_https_build::register(&BON_MGR);
    pi_store_build::register(&BON_MGR);
    register(&BON_MGR);
}

// 处理虚拟机最大堆设置参数
fn set_vm_max_heap(matches: &ArgMatches, sys: &SysStat) {
    if let Some(max_heap) = matches.value_of("max_heap") {
        match max_heap.parse::<f64>() {
            Err(e) => {
                panic!("set max heap limit failed, reason: {:?}", e);
            }
            Ok(val) => {
                set_max_alloced_limit((val * 1024.0 * 1024.0 * 1024.0).floor() as usize);
                info!(
                    "===> Set Max Heap Limit Ok, size: {} B",
                    get_max_alloced_limit()
                );
            }
        }
    } else {
        //没有设置最大堆限制
        #[cfg(any(windows))]
        {
            let (total_memory, _, _, _, _, _) = sys.memory_usage();
            set_max_alloced_limit(((total_memory * 1024) as f64 * 0.75).floor() as usize);
            info!(
                "===> Set Max Heap Limit Ok, size: {} B",
                get_max_alloced_limit()
            );
        }
        #[cfg(any(unix))]
        {
            let sys = sys.special_platform().unwrap();
            match sys.sys_virtual_memory_detal() {
                None => {
                    //获取内存占用信息失败，则使用默认最大堆限制
                    warn!(
                        "!!!> Set Max Heap Limit Failed, used default max heap limit, size: {}B",
                        get_max_alloced_limit()
                    );
                }
                Some(info) => {
                    //获取内存占用信息成功
                    let total_memory = info.0;
                    set_max_alloced_limit((total_memory as f64 * 0.75).floor() as usize);
                    info!(
                        "===> Set Max Heap Limit Ok, size: {} B",
                        get_max_alloced_limit()
                    );
                }
            }
        }
    }
}

// 初始化流程
fn init_js(matches: &ArgMatches) {
    let init_exec_path = matches.value_of("exec_file").unwrap();
    exec_js(init_exec_path.to_string());
}

// 根据命令行参数决定是否启动shell
fn enable_shell(matches: &ArgMatches) {
    if let Some("true") = matches.value_of("shell") {
        let (req_sender, req_receiver) = channel();
        let (resp_sender, resp_receiver) = channel();

        let req_sender_copy = req_sender.clone();
        let resp = Arc::new(
            move |result: IOResult<Arc<Vec<u8>>>, req: Option<Box<FnOnce(Arc<Vec<u8>>)>>| {
                resp_sender.send(result);
                req_sender.send(req);
            },
        );

        let s = SHELL_MANAGER.write().unwrap().open();
        if let Some(shell) = s {
            let req = SHELL_MANAGER.write().unwrap().connect(shell, resp.clone());
            if req.is_none() {
                eprintln!("Connect Error");
            }
            req_sender_copy.send(req);

            println!("Shell v0.1");

            let mut req: Option<Box<dyn FnOnce(Arc<Vec<u8>>)>> = None;
            loop {
                print!(">");
                io::stdout().flush();

                let mut buffer = String::new();
                while let Err(e) = io::stdin().read_line(&mut buffer) {
                    eprintln!("Input Error, {:?}", e);
                    print!(">");
                    io::stdout().flush();
                }

                if buffer.trim().as_bytes() == b"exit" {
                    println!("Shell closed");
                    return;
                }

                if let None = req {
                    //当前没有请求回调，则接收请求回调
                    match req_receiver.recv() {
                        Err(e) => {
                            eprintln!("Shell Suspend, {:?}", e);
                            return;
                        }
                        Ok(new) => {
                            if new.is_none() {
                                println!("Shell closed");
                                return;
                            }
                            req = new; //更新请求回调
                        }
                    }
                }

                if let Some(r) = req.take() {
                    r(Arc::new(buffer.into_bytes()));
                }

                //接收请求响应
                match resp_receiver.recv() {
                    Err(e) => eprintln!("Output Error, {:?}", e),
                    Ok(result) => match result {
                        Err(e) => eprintln!("{:?}", e),
                        Ok(r) => println!(
                            "{output}",
                            output = String::from_utf8_lossy(&r[..]).as_ref()
                        ),
                    },
                }
            }
        }
    }
}

fn main() {
    // 启动license服务
    license_handle();
    // 启动日志系统
    env_logger::builder().format(|buf, record| {
        writeln!(
            buf,
            "{} {} [{}] {}",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            record.level(),
            record.module_path().unwrap_or("<unnamed>"),
            &record.args()
        )
    }).init();

    // 加载堆栈跟踪库
    #[cfg(not(unix))]
    load_lib_backtrace();

    // 启动定时器
    TIMER.run();
    TASK_POOL_TIMER.run();

    // 注册本地对象
    register_native_object();

    let sys = SysStat::new();
    let processor = sys.processor_count();

    // 启动存储任务工作线程
    start_store_worker(processor);

    // 启动js虚拟机工作线程
    start_js_worker(processor);

    // 启动网络io工作线程
    start_network_worker(processor);

    // 注册rust暴露的函数
    register_bon_mgr();

    let matches = args();

    set_vm_max_heap(&matches, &sys);
    set_piserv_env_var(&matches);
    init_js(&matches);

    // 初始化虚拟机执行完毕没有通知机制, 如果不延迟一段时间启动shell, 项目的字节码还未完全准备好，导致只能加载部分字节码
    thread::sleep(Duration::from_secs(5));
    init_shell();

    // 启动全局虚拟机堆整理
    set_vm_timeout(60000);
    register_global_vm_heap_collect_timer(3000);

    // 根据命令行参数决定是否启动shell
    enable_shell(&matches);

    println!("\n\n################# pi_serv initialized successfully #################\n\n");

    // 单独一个线程处理定时任务
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(10));
            tick();
        }
    });

    let mut counter: u64 = 0;


    // 主线程处理热更时垃圾回收
    loop {
        counter = counter.wrapping_add(1);
        let mut to_be_removed = vec![];
        {
            let gray_tab = &GRAY_TABLE.read().jsgrays;
            let vmfs: Vec<_> = NOTIFY_CHAN.1.try_iter().collect();

            for (version, vmf_name) in vmfs {
                if let Some(gray) = gray_tab.get(version) {
                    if let Some(jsgray) = gray.get(&vmf_name) {
                        if jsgray.factory.size() == jsgray.factory.free_buf_size() + jsgray.factory.free_pool_size() {
                            debug!("hotfix remove vmfactory name =  {:?}, version = {:?}", vmf_name, version);
                            to_be_removed.push((version, vmf_name));
                            // gray.remove(&vmf_name);
                        } else {
                            // 没有回收成功的再次放回队列中
                            let _ = NOTIFY_CHAN.0.try_send((version, vmf_name));
                        }
                    }
                }
            }
        }

        for (version, vmf_name) in to_be_removed {
            if let Some(gray) = GRAY_TABLE.write().jsgrays.get_mut(version) {
                gray.remove(&vmf_name);
            }
        }

        if counter % 20 == 0 {
            println!("###############loop, {}", now_millisecond());
        }
        thread::sleep(Duration::from_millis(500));
    }
}

/**
* 同步的设置定时异步回调
* @param ms 间隔的时长，单位毫秒
* @param cb 异步回调
* @returns 返回定时任务的编号
*/
fn call_3344344275_async(js: Arc<JS>, v: Vec<JSType>) -> Option<CallResult> {
    let param_error = "param error in set_timeout";

    // timeout
    let timeout = &v[0];
    if !timeout.is_number() {
        return Some(CallResult::Err(String::from(param_error)));
    }
    let timeout = timeout.get_u32();

    // call_index
    let call_index = &v[1];
    if !call_index.is_number() {
        return Some(CallResult::Err(String::from(param_error)));
    }
    let call_index = call_index.get_u32();

    let jscopy = js.clone();

    // println!("index: {}, timeout: {}", call_index, timeout);
    match push_callback(
        jscopy.clone(),
        call_index,
        Box::new(move |js: Arc<JS>| {
            let ptr = Box::into_raw(Box::new(js.clone())) as usize;
            ptr_jstype(js.get_objs(), js.clone(), ptr, 2884638791);
            1
        }),
        Some(timeout),
        Atom::from("call_3344344275_async1"),
    ) {
        Some(r) => js.new_i32(r as i32),
        None => js.new_undefined(),
    };
    //     let result = js_base::set_timeout(jst0,jst1,jst2,jst3,Box::new(call_back));let mut result = match result{
    //         Some(v) => { let mut v = js.new_i32(v as i32);
    //  v}
    //         None => js.new_null()
    //     };

    Some(CallResult::Ok)
}

fn register(mgr: &BonMgr) {
    mgr.regist_fun_meta(FnMeta::CallArg(call_3344344275_async), 3344344275);
}

// 执行license服务
fn license_handle() {
    let file = File::open("license");
    let license_str = match file {
        Ok(result) => {
            let mut buf_reader = BufReader::new(result);
            let mut contents = String::new();
            match buf_reader.read_to_string(&mut contents) {
                Ok(_) => contents,
                _ => "".to_string(),
            }
        },
        _ => "".to_string(),
    };
    let mut license = License::new(license_str);
    License::set_timer(&mut license, 1 * 60 * 60 * 1000);
}
