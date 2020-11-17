#![allow(unused_imports)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate json;

use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::{Duration, Instant};
use std::{env, fs::read_to_string};

use clap::{App, Arg, ArgMatches, SubCommand};
use env_logger;
use num_cpus::get_physical;
use parking_lot::{Condvar, Mutex, RwLock, WaitTimeoutResult};

use hash::XHashMap;
use r#async::rt::{
    multi_thread::{MultiTaskPool, MultiTaskRuntime},
    single_thread::{SingleTaskRunner, SingleTaskRuntime},
    AsyncRuntime,
};
use tcp::{
    buffer_pool::WriteBufferPool,
    connect::TcpSocket,
    driver::SocketConfig,
    server::{AsyncPortsFactory, SocketListener},
};
use vm_builtin::{ContextHandle, VmStartupSnapshot};
use vm_builtin::{VmEvent, VmEventHandler, VmEventValue};
use vm_core::{debug, init_v8, vm, worker};
use ws::server::WebsocketListenerFactory;

use pi_serv_ext::register_ext_functions;
use pi_serv_lib::set_pi_serv_lib_file_runtime;
use pi_serv_lib::{js_db::global_db_mgr, js_gray::GRAY_MGR};

mod hotfix;
mod init;
mod js_net;

use crate::js_net::create_listener_pid;
use hotfix::{hotfix_listen_backend, hotfix_listen_frontend};
use init::init_js;
use js_net::reg_pi_serv_handle;

lazy_static! {
    //主线程运行状态和线程无条件休眠超时时长
    static ref MAIN_RUN_STATUS: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    static ref MAIN_UNCONDITION_SLEEP_TIMEOUT: u64 = 10;

    //主线程条件变量和线程条件休眠超时时长
    static ref MAIN_CONDVAR: Arc<(AtomicBool, Mutex<()>, Condvar)> = Arc::new((AtomicBool::new(false), Mutex::new(()), Condvar::new()));
    static ref MAIN_CONDITION_SLEEP_TIMEOUT: u64 = 10000;

    //初始化主线程异步运行时
    static ref MAIN_ASYNC_RUNNER: SingleTaskRunner<()> = SingleTaskRunner::new();
    static ref MAIN_ASYNC_RUNTIME: SingleTaskRuntime<()> = MAIN_ASYNC_RUNNER.startup().unwrap();

    //初始化文件异步运行时
    static ref FILES_ASYNC_RUNTIME: MultiTaskRuntime<()> = {
        let pool = MultiTaskPool::new("PI-SERV-FILE".to_string(), get_physical(), 2 * 1024 * 1024, 10, Some(10));
        pool.startup(false)
    };
    //Mqtt端口代理映射表
    static ref MQTT_PORTS: Arc<Mutex<Vec<(u16, String)>>> = Arc::new(Mutex::new(vec![]));
    static ref VID_CONTEXTS: Arc<Mutex<XHashMap<usize, Vec<ContextHandle>>>> = Arc::new(Mutex::new(XHashMap::default()));
}

/*
* 同步执行入口，退出时会中止主线程
*/
fn main() {
    //初始化日志服务器
    env_logger::init();

    //匹配启动时的选项和参数
    let matches = App::new("Pi Serv Main")
        .version("0.2.0")
        .author("YiNeng <yineng@foxmail.com>")
        .arg(
            Arg::with_name("INIT_HEAP_SIZE") //虚拟机初始堆大小
                .short("I")
                .long("INIT_HEAP_SIZE")
                .value_name("Mbytes")
                .help("Set init vm heap size")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("MAX_HEAP_SIZE") //虚拟机最大堆大小
                .short("H")
                .long("MAX_HEAP_SIZE")
                .value_name("Mbytes")
                .help("Set max vm heap size")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("WORK_VM_MULTIPLE") //工作虚拟机倍数
                .short("W")
                .long("WORK_VM_MULTIPLE")
                .value_name("Multiple")
                .help("Set multiple of work vm amount")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("DEBUG") //工作虚拟机调试模式
                .short("D")
                .long("DEBUG")
                .value_name("Port")
                .help("Enable debug work vm on port")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("init-file") // pi_pt入口文件
                .short("i")
                .long("init-file")
                .value_name("init-file")
                .help("pi_pt entry file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("projects") // 要启动的项目
                .short("p")
                .long("projects")
                .value_name("projects")
                .help("projectw to launch")
                .multiple(true)
                .takes_value(true),
        )
        .get_matches();

    //初始化V8环境，并启动初始虚拟机
    let (init_heap_size, max_heap_size, debug_port) = init_v8_env(&matches);
    let mut init_vm = create_init_vm(init_heap_size, max_heap_size, debug_port);
    let init_vm_runner = init_vm.take_runner().unwrap();

    //启动初始虚拟机线程，并运行初始虚拟机
    let init_vm_handle = worker::spawn_worker_thread(
        "Init-Vm",
        2 * 1024 * 1024,
        MAIN_CONDVAR.clone(),
        1000,
        Some(10),
        move || {
            let run_time = init_vm_runner.run();
            (init_vm_runner.queue_len() == 0, run_time)
        },
    );

    let init_vm = init_vm.init().unwrap();
    let matches_copy = matches.clone();
    if let Err(e) = MAIN_ASYNC_RUNTIME.spawn(MAIN_ASYNC_RUNTIME.alloc(), async move {
        async_main(
            matches_copy,
            init_vm_handle,
            init_vm,
            init_heap_size,
            max_heap_size,
            debug_port,
        )
        .await;
    }) {
        panic!("Spawn async main task failed, reason: {:?}", e);
    }

    //主线程循环
    while MAIN_RUN_STATUS.load(Ordering::Relaxed) {
        //推动主线程异步运行时
        let start_time = Instant::now();
        if let Err(e) = MAIN_ASYNC_RUNNER.run() {
            panic!("Main loop failed, reason: {:?}", e);
        }
        let run_time = Instant::now() - start_time;

        if let Some(remaining_interval) =
            Duration::from_millis(*MAIN_UNCONDITION_SLEEP_TIMEOUT).checked_sub(run_time)
        {
            //本次运行少于循环间隔，则休眠剩余的循环间隔，并继续执行任务
            thread::sleep(remaining_interval);
        }
    }
}

//初始化V8环境，如果是调试模式则返回调试端口
fn init_v8_env(matches: &ArgMatches) -> (usize, usize, Option<u16>) {
    let mut init_heap_size = 16 * 1024 * 1024; //默认虚拟机初始堆大小为16MB
    if let Some(size) = matches.value_of("INIT_HEAP_SIZE") {
        match size.parse::<usize>() {
            Err(e) => panic!("Init v8 env failed, reason: {:?}", e),
            Ok(num) => {
                if num.is_power_of_two() {
                    init_heap_size = num * 1024 * 1024;
                }
            }
        }
    }

    let mut max_heap_size = 8096 * 1024 * 1024; //默认虚拟机最大堆大小为8GB
    if let Some(size) = matches.value_of("MAX_HEAP_SIZE") {
        match size.parse::<usize>() {
            Err(e) => panic!("Init v8 env failed, reason: {:?}", e),
            Ok(num) => {
                if num.is_power_of_two() {
                    max_heap_size = num * 1024 * 1024;
                }
            }
        }
    }

    let mut debug_port: u16 = 0;
    if let Some(value) = matches.value_of("DEBUG") {
        match value.parse::<u16>() {
            Err(e) => {
                panic!("Bind debug listene port failed, reason: {:?}", e);
            }
            Ok(port) => {
                if port > 1024 {
                    debug_port = port;
                } else {
                    panic!(
                        "Bind debug listene port failed, port: {}, reason: invalid port",
                        port
                    );
                }
            }
        }
    }

    init_v8(Some(vec![
        "".to_string(),
        "--no-wasm-async-compilation".to_string(),
        "--harmony-top-level-await".to_string(),
        "--expose-gc".to_string(),
    ]));

    if debug_port > 0 {
        //启动虚拟机调试用Websocket服务
        let ws_server_factory = Arc::new(debug::InspectorWebsocketServerFactory::new::<PathBuf>(
            "", None,
        )); // TODO: 浏览器的调试不能指定路径，但是vscode的调试需要指定路径
        let mut factory = AsyncPortsFactory::<TcpSocket>::new();
        factory.bind(
            debug_port,
            Box::new(
                WebsocketListenerFactory::<TcpSocket>::with_protocol_factory(ws_server_factory),
            ),
        );
        let mut config = SocketConfig::new("0.0.0.0", factory.bind_ports().as_slice());
        config.set_option(16384, 16384, 16384, 16);
        let buffer = WriteBufferPool::new(1000, 10, 3).ok().unwrap();
        match SocketListener::bind_with_processor(
            factory,
            buffer,
            config,
            1,
            1024,
            2 * 1024 * 1024,
            1024,
            Some(10),
        ) {
            Err(e) => {
                panic!(
                    "Bind debug listene port failed, port: {}, reason: {:?}",
                    debug_port, e
                );
            }
            Ok(_) => {
                info!("Bind debug listene port ok, port: {}", debug_port);
            }
        }
    }

    info!("Init v8 env ok");
    (init_heap_size, max_heap_size, Some(debug_port))
}

//创建初始虚拟机
fn create_init_vm(
    init_heap_size: usize,
    max_heap_size: usize,
    debug_port: Option<u16>,
) -> vm::UninitedVm {
    let mut builder = vm::VmBuilder::new().snapshot_template();
    builder = builder.heap_limit(init_heap_size, max_heap_size);

    if debug_port.is_some() {
        //允许调试
        builder = builder.enable_inspect();
    }

    builder.bind_condvar_waker(MAIN_CONDVAR.clone()).build()
}

// 注册虚拟机关心处理的事件
fn reigster_vms_events(workers: &[vm::Vm]) {
    // 设置虚拟机的事件回调
    for worker in workers {
        let event_handler = VmEventHandler::new(
            AsyncRuntime::Local(MAIN_ASYNC_RUNTIME.clone()),
            move |event, vid| match event {
                VmEventValue::CreatedContext(context) => {
                    debug!(
                        "vm event handler: VmEventValue::CreatedContext, vid = {:?}",
                        vid
                    );
                    VID_CONTEXTS
                        .lock()
                        .entry(vid)
                        .and_modify(|v| {
                            v.push(context);
                        })
                        .or_insert(vec![]);
                }

                VmEventValue::RemovedContext(context) => {
                    debug!("vm event handler: VmEventValue::RemovedContext");
                    VID_CONTEXTS
                        .lock()
                        .entry(vid)
                        .and_modify(|v| {
                            v.retain(|ctx| *ctx != context);
                        })
                        .or_insert(vec![]);
                }
            },
        );
        worker.register_event_handler(VmEvent::CreatedContext, event_handler.clone());
        worker.register_event_handler(VmEvent::RemovedContext, event_handler);
    }
}

/*
* 异步执行入口，退出时不会中止主线程
*/
async fn async_main(
    matches: ArgMatches<'static>,
    init_vm_handle: Arc<AtomicBool>,
    init_vm: vm::Vm,
    init_heap_size: usize,
    max_heap_size: usize,
    debug_port: Option<u16>,
) {
    // 加载native funtion
    register_ext_functions();

    // 注册文件异步运行时
    set_pi_serv_lib_file_runtime(FILES_ASYNC_RUNTIME.clone());
    // 注册pi_serv方法
    reg_pi_serv_handle();

    let snapshot_context = init_snapshot(&init_vm).await;

    init_js(init_vm.clone(), snapshot_context, matches.clone()).await;

    //TODO 加载项目的入口模块文件, 并加载其静态依赖树中的所有js模块文件

    finish_snapshot(&init_vm, snapshot_context).await;

    let workers = init_work_vm(
        &matches,
        &init_vm,
        init_heap_size,
        max_heap_size,
        debug_port,
    );

    let vms: Vec<vm::Vm> = workers.iter().map(|(_, vm)| vm.clone()).collect();
    init_default_gray(vms.clone());

    //所有虚拟机启动完成之后创建listener pid
    init_listener_pid();
    reigster_vms_events(&vms);
    enable_hotfix();
}

//初始化快照
async fn init_snapshot(init_vm: &vm::Vm) -> ContextHandle {
    let snapshot_context = init_vm
        .new_context(None, init_vm.alloc_context_id(), None)
        .await
        .unwrap();
    if let Err(e) = init_vm
        .execute(
            snapshot_context,
            "Init_Vm_Init_module.js",
            r#"
                    onerror = function(e) {
                        if(e.stack) {
                            print("catch global error, e:", e.stack);
                        } else {
                            print("catch global error, e:", e.message);     
                        }
                    };
                "#,
        )
        .await
    {
        panic!("!!!!!!Init snapshot failed, reason: {:?}", e);
    }
    info!("Init snapshot ok");

    snapshot_context
}

//完成快照
async fn finish_snapshot(init_vm: &vm::Vm, snapshot_context: ContextHandle) {
    if let Err(e) = init_vm.snapshot(snapshot_context).await {
        panic!("!!!!!!Finish snapshot failed, reason: {:?}", e);
    }
    info!("Snapshot finish");
}

//初始化工作虚拟机，返回工作虚拟机
fn init_work_vm(
    matches: &ArgMatches,
    init_vm: &vm::Vm,
    init_heap_size: usize,
    max_heap_size: usize,
    debug_port: Option<u16>,
) -> Vec<(Arc<AtomicBool>, vm::Vm)> {
    let mut work_vm_count: usize = 2 * get_physical(); //默认工作虚拟机数量为本地cpu物理核数的2倍
    if let Some(value) = matches.value_of("WORK_VM_MULTIPLE") {
        match value.parse::<usize>() {
            Err(e) => {
                panic!("Init work vm failed, reason: {:?}", e);
            }
            Ok(count) => {
                work_vm_count = get_physical() * count;
            }
        }
    }

    let mut vec = Vec::with_capacity(work_vm_count);
    let snapshot = VmStartupSnapshot::Snapshot(init_vm.get_snapshot().unwrap());
    let snapshot_bytes = snapshot.bytes();
    for index in 0..work_vm_count {
        //使用指定快照，创建工作虚拟机
        let work_vm_snapshot = VmStartupSnapshot::Boxed(snapshot_bytes.to_vec().into_boxed_slice());
        let mut builder = vm::VmBuilder::new().startup_snapshot(work_vm_snapshot);
        builder = builder.heap_limit(init_heap_size, max_heap_size);

        if debug_port.is_some() {
            //允许调试
            builder = builder.enable_inspect();
        }
        let worker_condvar = Arc::new((AtomicBool::new(false), Mutex::new(()), Condvar::new()));
        let mut work_vm = builder.bind_condvar_waker(worker_condvar.clone()).build();
        let work_vm_runner = work_vm.take_runner().unwrap();

        //启动工作线程，并运行工作虚拟机
        let worker_name = "PI-SERV-WORKER".to_string() + index.to_string().as_str();
        info!(
            "Worker ready, thread: {}, worker: {}",
            worker_name,
            "Vm-".to_string() + work_vm_runner.get_vid().to_string().as_str()
        );

        let worker_handle = worker::spawn_worker_thread(
            worker_name.as_str(),
            2 * 1024 * 1024,
            worker_condvar,
            1000,
            None,
            move || {
                let run_time = work_vm_runner.run();
                (work_vm_runner.queue_len() == 0, run_time)
            },
        );

        let work_vm = work_vm.init().unwrap();
        vec.push((worker_handle, work_vm));
    }

    vec
}

//初始化默认灰度
fn init_default_gray(workers: Vec<vm::Vm>) {
    if let Err(e) = GRAY_MGR.write().add_new_gray(0, workers, global_db_mgr()) {
        panic!("Create default gray failed, reason: {:?}", e);
    }
}

//初始化端口的监听Pid
fn init_listener_pid() {
    for (port, broker_name) in MQTT_PORTS.lock().iter() {
        create_listener_pid(port.clone(), broker_name);
    }
}

// 通过环境变量控制是否启动热更新
fn enable_hotfix() {
    if env::var("ENABLE_HOTFIX").is_ok() {
        info!("start listen hotfix...");
        hotfix_listen_backend(String::from("../dst_server"));
        hotfix_listen_frontend();
    }
}
