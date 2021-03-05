#![deny(unused_must_use)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate json;

#[cfg(feature = "default")]
#[macro_use]
extern crate pi_core;
#[cfg(feature = "profiling_heap")]
#[macro_use]
extern crate profiling_pi_core;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use std::thread;
use std::time::{Duration, Instant};
use std::{env, fs::read_to_string};

use clap::{App, Arg, ArgMatches, SubCommand};
use env_logger;
use json::{array, parse, stringify};
use num_cpus::get_physical;
use parking_lot::{Condvar, Mutex, RwLock, WaitTimeoutResult};

use hash::XHashMap;
use r#async::rt::{
    multi_thread::{MultiTaskPool, MultiTaskRuntime},
    single_thread::{SingleTaskRunner, SingleTaskRuntime},
    spawn_worker_thread, AsyncRuntime,
};
use tcp::{
    buffer_pool::WriteBufferPool,
    connect::TcpSocket,
    driver::SocketConfig,
    server::{AsyncPortsFactory, SocketListener},
};
use vm_builtin::{
    utils::set_used_rust_heap_size_getter, ContextHandle, VmEvent, VmEventHandler, VmEventValue,
    VmStartupSnapshot,
};
use vm_core::{debug, init_v8, vm};
use ws::server::WebsocketListenerFactory;

#[cfg(feature = "default")]
use pi_core::{
    allocator::alloced_size,
    console::{set_console_shell_ctrlc_handler, ConsoleShell, ConsoleShellBuilder},
    create_snapshot_vm, finish_snapshot, init_snapshot, init_v8_env, init_work_vm,
    terminal::VmTerminal,
};
use pi_core_builtin::set_external_async_runtime;
use pi_core_lib::set_file_async_runtime;
use pi_serv_ext::register_ext_functions;
use pi_serv_lib::{
    js_db::global_db_mgr,
    js_gray::{GRAY_MGR, VID_CONTEXTS},
};
use pi_serv_lib::{
    set_pi_serv_lib_file_runtime, set_pi_serv_lib_main_async_runtime, set_store_runtime,
};
#[cfg(feature = "profiling_heap")]
use profiling_pi_core::{
    console::{set_console_shell_ctrlc_handler, ConsoleShell, ConsoleShellBuilder},
    create_snapshot_vm, finish_snapshot, init_snapshot, init_v8_env, init_work_vm,
    set_default_ctrlc_handler,
    terminal::VmTerminal,
};

mod hotfix;
mod init;
mod js_net;

use crate::js_net::create_listener_pid;
use hotfix::{hotfix_listen_backend, hotfix_listen_frontend, HOTFIX_FILES};
use init::{init_js, read_init_source};
use js_net::{create_http_pid, reg_pi_serv_handle, start_network_services};
use pi_db::db_collect::collect_db;

#[cfg(feature = "default")]
init_global_counter_alloc! {}
#[cfg(feature = "profiling_heap")]
init_global_profiling_alloc! {}

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
        let pool = MultiTaskPool::new("PI-SERV-FILE".to_string(), get_physical() * 5, 2 * 1024 * 1024, 10, Some(10));
        pool.startup(false)
    };
    //Mqtt端口代理映射表
    static ref MQTT_PORTS: Arc<Mutex<HashSet<(u16, String)>>> = Arc::new(Mutex::new(HashSet::new()));
    //Http端口代理映射表
    static ref HTTP_PORTS: Arc<Mutex<Vec<(u16, String)>>> = Arc::new(Mutex::new(vec![]));

    //控制台
    static ref CONSOLE_SHELL: RwLock<Option<ConsoleShell>> = RwLock::new(None);

    //自动GC
    static ref AUTO_GC: RwLock<Vec<AtomicUsize>> = RwLock::new(Vec::new());
}

/*
* 同步执行入口，退出时会中止主线程
*/
fn main() {
    //初始化分析堆内存的堆分配器
    #[cfg(feature = "profiling_heap")]
    init_profiling_alloctor();

    set_used_rust_heap_size_getter(alloced_size);

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
                .help("Sets init vm heap size")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("MAX_HEAP_SIZE") //虚拟机最大堆大小
                .short("H")
                .long("MAX_HEAP_SIZE")
                .value_name("Mbytes")
                .help("Sets max vm heap size")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("WORK_VM_MULTIPLE") //工作虚拟机倍数
                .short("W")
                .long("WORK_VM_MULTIPLE")
                .value_name("Multiple")
                .help("Sets multiple of work vm amount")
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
            Arg::with_name("CONSOLE") //工作虚拟机控制台模式
                .short("C")
                .long("CONSOLE")
                .help("Enable with console for work vm"),
        )
        .arg(
            Arg::with_name("PROFILING") //虚拟机剖析模式
                .short("P")
                .long("PROFILING")
                .help("Enable profiling vm"),
        )
        .arg(
            Arg::with_name("LITE") //打开虚拟机Lite模式
                .short("L")
                .long("LITE")
                .help("Enable vm lite mode"),
        )
        .arg(
            Arg::with_name("JITLESS") //禁止虚拟机jit
                .short("J")
                .long("JITLESS")
                .help("Disable vm jit"),
        )
        .arg(
            Arg::with_name("trace-gc") //打开gc跟踪
                .long("trace-gc")
                .help("Output in stdout following each garbage collection"),
        )
        .arg(
            Arg::with_name("trace-gc-detail") //打开gc详细跟踪
                .long("trace-gc-detail")
                .help("Output detail info in stdout following each garbage collection"),
        )
        .arg(
            Arg::with_name("trace-alloc") //打开分配跟踪
                .long("trace-alloc")
                .help("Output in stdout following each allocate"),
        )
        .arg(
            Arg::with_name("promise-statistics") //打开Promise统计
                .long("promise-statistics")
                .help("Enable promise statistics"),
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

    let mut is_profiling = false;
    if let Some(_) = matches.index_of("PROFILING") {
        //允许剖析虚拟机
        is_profiling = true;
    }

    let mut is_lite = false;
    if let Some(_) = matches.index_of("LITE") {
        //打开虚拟机lite模式
        is_lite = true;
    }

    let mut is_jitless = false;
    if let Some(_) = matches.index_of("JITLESS") {
        //允许剖析虚拟机
        is_jitless = true;
    }

    let mut is_trace_gc = false;
    if let Some(_) = matches.index_of("trace-gc") {
        //允许gc跟踪
        is_trace_gc = true;
    }

    let mut is_trace_gc_detail = false;
    if let Some(_) = matches.index_of("trace-gc-detail") {
        //允许gc跟踪
        is_trace_gc_detail = true;
    }

    let mut is_trace_alloc = false;
    if let Some(_) = matches.index_of("trace-alloc") {
        //允许分配跟踪
        is_trace_alloc = true;
    }
    if let Some(_) = matches.index_of("promise-statistics") {
        //允许Promise统计
        vm::enable_global_promise_count();
    }

    let (init_heap_size, max_heap_size, debug_port) = init_v8_env(
        init_heap_size,
        max_heap_size,
        debug_port,
        is_profiling,
        is_lite,
        is_jitless,
        is_trace_gc,
        is_trace_gc_detail,
        is_trace_alloc,
    );
    let mut init_vm = create_snapshot_vm(
        init_heap_size,
        max_heap_size,
        debug_port,
        MAIN_RUN_STATUS.clone(),
        MAIN_CONDVAR.clone(),
    );
    let init_vm_runner = init_vm.take_runner().unwrap();
    let queue_len_getter = init_vm_runner.get_inner_handler();

    //启动初始虚拟机线程，并运行初始虚拟机
    let init_vm_handle = spawn_worker_thread(
        "Init-Vm",
        2 * 1024 * 1024,
        MAIN_RUN_STATUS.clone(),
        MAIN_CONDVAR.clone(),
        10,
        Some(10),
        move || {
            let run_time = init_vm_runner.run();
            (init_vm_runner.queue_len() == 0, run_time)
        },
        move || {
            if let Some(len) = queue_len_getter.len() {
                len
            } else {
                0
            }
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

    set_store_runtime(FILES_ASYNC_RUNTIME.clone()).await;
    // 注册文件异步运行时
    set_file_async_runtime(FILES_ASYNC_RUNTIME.clone());
    set_pi_serv_lib_file_runtime(FILES_ASYNC_RUNTIME.clone());
    set_pi_serv_lib_main_async_runtime(MAIN_ASYNC_RUNTIME.clone());
    // 注册pi_serv方法
    reg_pi_serv_handle();
    // 注册pi_serv_builtin运行时
    set_external_async_runtime(AsyncRuntime::Local(MAIN_ASYNC_RUNTIME.clone()));
    // 设置env
    set_current_env();

    let snapshot_context = init_snapshot(&init_vm).await;

    init_js(
        debug_port.is_some(),
        init_vm.clone(),
        snapshot_context.clone(),
        matches.clone(),
    )
    .await;
    finish_snapshot(&init_vm, snapshot_context).await;

    let mut work_vm_count: usize = 2 * get_physical(); //默认工作虚拟机数量为本地cpu物理核数的2倍
    if let Some(value) = matches.value_of("WORK_VM_MULTIPLE") {
        match value.parse::<usize>() {
            Err(e) => {
                panic!("Init work vm failed, reason: {:?}", e);
            }
            Ok(0) => {
                work_vm_count = 1;
            }
            Ok(count) => {
                work_vm_count = get_physical() * count;
            }
        }
    }
    let workers = init_work_vm(
        &init_vm,
        init_heap_size,
        max_heap_size,
        debug_port,
        work_vm_count,
        "PI-SERV",
        2,
        10,
        None,
        move |work_vm_runner: vm::VmRunner| {
            move || {
                let run_time = work_vm_runner.run();
                (work_vm_runner.queue_len() == 0, run_time)
            }
        },
    );

    let vms: Vec<vm::Vm> = workers.iter().map(|(_, vm)| vm.clone()).collect();
    reigster_vms_events(&vms, debug_port.is_some());
    init_default_gray(vms.clone());

    //所有虚拟机启动完成之后创建listener pid
    init_listener_pid();
    init_http_listener_pid();

    // 最后启动网络服务
    let _ = start_network_services(16384, 16384, 16384, 100000, 2048, 2097152, 10);

    enable_hotfix();

    init_mfa(workers[0].clone().1).await;

    pid_close_count_init(workers.len());

    if let Some((worker, worker_context)) = init_console(matches.clone(), &workers).await {
        if let Some(console_shell) = CONSOLE_SHELL.read().as_ref() {
            //启动控制台
            let vid = worker.get_vid();
            let cid = worker_context.0;
            let console_shell = console_shell.clone();

            //设置控制台Ctrl+C的处理器
            let prompt_prefix = "Pid{".to_string()
                + vid.to_string().as_str()
                + ", "
                + cid.to_string().as_str()
                + "}";
            set_console_shell_ctrlc_handler(console_shell.clone(), prompt_prefix.clone());

            console_shell.set_prompt_prefix(prompt_prefix.as_str());
            console_shell.show_welcome_info(
                "PiServ",
                "0.2.0",
                Some("Workers: ".to_string() + workers.len().to_string().as_str()),
            );
            console_shell.wait_input(prompt_prefix);
        }
    } else {
        //不启动控制台，且需要分析堆内存，则设置Ctrl+C的默认处理器
        #[cfg(feature = "profiling_heap")]
        set_default_ctrlc_handler();
    }

    if let Some(start_at) = get_db_collect_time() {
        collect_db(FILES_ASYNC_RUNTIME.clone(), start_at);
    } else {
        warn!("no database collect time specified");
    }
}

// 执行mfa
pub async fn init_mfa(worker: vm::Vm) {
    let file = std::fs::read_to_string("../dst_server/ptconfig.json").unwrap();
    let json = parse(&file).unwrap();
    let mfas = &json["MFA"];
    let mut execs = vec![];
    for m in mfas.members() {
        debug!(
            "mfa exec module = {:?}, func = {:?}, args = {:?}",
            m["module"], m["function"], m["args"]
        );
        execs.push(format!(
            r#" 
            var envMod = Module.require('pi_utils/lang/env');
            self.env = new envMod.Env();
            env.other = new Map();
            var mgrMod = Module.require('pi_pt/db/mgr');
            if (!env.dbMgr) env.dbMgr = new mgrMod.Mgr(mgr);

            var f = Module.modules["{}"].exports.{}.apply(Module.modules["{}"].exports.{}, {});
            if (f instanceof Promise) f.then(() => console.log("mfa init finished")).catch(e => console.log("mfa init failed", e.toString()));"#,
            m["module"], m["function"], m["module"], m["function"], m["args"]
        ));
    }

    let cid = worker.alloc_context_id();
    let context = worker.new_context(None, cid, None).await.unwrap();

    for exec in execs {
        let _ = worker.execute(context, "", &exec).await;
    }
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

//初始化HTTP端口的监听Pid
fn init_http_listener_pid() {
    for (port, host) in HTTP_PORTS.lock().iter() {
        create_http_pid(host, port.clone());
    }
}

// 注册虚拟机关心处理的事件
fn reigster_vms_events(workers: &[vm::Vm], is_debug_mode: bool) {
    // 设置虚拟机的事件回调
    for worker in workers {
        let vm = worker.clone();
        let event_handler = VmEventHandler::new(
            AsyncRuntime::Local(MAIN_ASYNC_RUNTIME.clone()),
            move |event, vid| match event {
                VmEventValue::CreatedContext(context) => {
                    debug!(
                        "Vm event handler: VmEventValue::CreatedContext, vid = {:?}, cid = {:?}",
                        vid, context.0
                    );

                    // 非调试模式下才需要重新require热更过的文件
                    if !is_debug_mode {
                        // 获取已经热更新过的文件
                        let mut hotfixed_files = HOTFIX_FILES
                            .lock()
                            .iter()
                            .map(|(key, val)| (key.clone(), val.clone()))
                            .collect::<Vec<(String, usize)>>();
                        // 按版本号从小到大排序
                        hotfixed_files.sort_by(|a, b| a.1.cmp(&b.1));

                        for (path, _) in hotfixed_files {
                            let vm = vm.clone();
                            let _ = MAIN_ASYNC_RUNTIME.spawn(MAIN_ASYNC_RUNTIME.alloc(), async move {
                                if let Ok(Some(func)) = vm
                                    .get_property(context.clone(), "self.Module.require")
                                    .await
                                {
                                    let p = vm
                                        .to_js_value(context.clone(), stringify(path.clone()))
                                        .await
                                        .unwrap()
                                        .unwrap();
                                    let dir = vm
                                        .to_js_value(context.clone(), stringify(""))
                                        .await
                                        .unwrap()
                                        .unwrap();
                                    let force = vm
                                        .to_js_value(context.clone(), stringify(true))
                                        .await
                                        .unwrap()
                                        .unwrap();
                                    if let Err(_e) = vm
                                        .call(context.clone(), &func, vec![p.clone(), dir, force])
                                        .await
                                    {
                                        warn!("hotfix call require error");
                                    } else {
                                        debug!("new context update hotfix file, vid = {:?}, cid = {:?}, path = {:?}", vid, context.0, path);
                                    }
                                } else {
                                    warn!("get Module.require error");
                                }
                            });
                        }
                    }

                    VID_CONTEXTS
                        .lock()
                        .entry(vid)
                        .and_modify(|v| {
                            v.push(context);
                        })
                        .or_insert(vec![context]);

                    if is_debug_mode {
                        let vm = GRAY_MGR.read().vm_instance(0, vid).unwrap();
                        let vm_copy = vm.clone();
                        vm.spawn_task(async move {
                            let source =
                                read_init_source("../dst_server/pi_pt/init.js".to_string()).await;
                            if let Err(e) =
                                vm_copy.execute(context, "init.js", source.as_ref()).await
                            {
                                panic!(e);
                            }
                        });
                    }
                }

                VmEventValue::RemovedContext(context) => {
                    debug!(
                        "Vm event handler: VmEventValue::RemovedContext, vid = {:?}, cid = {:?}",
                        vid, context.0
                    );
                    pid_close_count(vid, context);
                    VID_CONTEXTS.lock().entry(vid).and_modify(|v| {
                        v.retain(|ctx| *ctx != context);
                    });
                }
            },
        );
        worker.register_event_handler(VmEvent::CreatedContext, event_handler.clone());
        worker.register_event_handler(VmEvent::RemovedContext, event_handler);
    }
}

// 通过环境变量控制是否启动热更新
fn enable_hotfix() {
    if env::var("ENABLE_HOTFIX").is_ok() {
        info!("Start listen hotfix...");
        hotfix_listen_backend(String::from("../dst_server"));
        hotfix_listen_frontend();
    }
}

//初始化控制台
async fn init_console(
    matches: ArgMatches<'static>,
    workers: &Vec<(Arc<AtomicBool>, vm::Vm)>,
) -> Option<(vm::Vm, ContextHandle)> {
    if let None = matches.index_of("CONSOLE") {
        //不打开控制台，则忽略
        return None;
    }

    let (_, worker) = &workers[0];
    let context = worker
        .new_context(None, worker.alloc_context_id(), None)
        .await
        .unwrap();

    //初始化控制台运行时
    let console_runner = SingleTaskRunner::new();
    let console_runtime = console_runner.startup().unwrap();
    thread::Builder::new()
        .name("Pi-Exec-Console".to_string())
        .stack_size(2 * 1024 * 1024)
        .spawn(move || {
            loop {
                //推动控制台异步运行时
                let start_time = Instant::now();
                if let Err(e) = console_runner.run() {
                    panic!("Main loop failed, reason: {:?}", e);
                }
                let run_time = Instant::now() - start_time;

                if let Some(remaining_interval) = Duration::from_millis(10).checked_sub(run_time) {
                    //本次运行少于循环间隔，则休眠剩余的循环间隔，并继续执行任务
                    thread::sleep(remaining_interval);
                }
            }
        })
        .unwrap();

    //构建控制台，并注册
    let console_shell: ConsoleShell = ConsoleShellBuilder::new(
        AsyncRuntime::Local(console_runtime),
        worker.clone(),
        context,
    )
    .buffer_stdout()
    .enable_color_stdout()
    .enable_color_stderr()
    .set_title("PiServ Console Shell")
    .build();
    *CONSOLE_SHELL.write() = Some(console_shell);

    Some((worker.clone(), context))
}

// 环境变量设置
fn set_current_env() {
    if env::var("CURRENT_LIMIT").is_ok() {
        env::set_var("current", "true");
    } else {
        env::set_var("current", "false");
    }
}

// 初始化pid关闭计数
fn pid_close_count_init(vm_count: usize) {
    let len = vm_count + 1;
    let start = AUTO_GC.read().len();
    for _ in start..len {
        (*AUTO_GC.write()).push(AtomicUsize::new(1));
    }
}

/// pid关闭计数
/// vid 虚拟机ID
/// context 虚拟机上下文
fn pid_close_count(vid: usize, _context: ContextHandle) {
    let atom = &(*AUTO_GC.read())[vid];
    let old = atom.fetch_add(1, Ordering::SeqCst);
    let mut count = 10;
    if let Ok(var) = env::var("AUTO_GC_COUNT") {
        count = var.parse().unwrap_or(10);
    }
    // 是否需要GC
    if old >= count {
        // 重置计数
        atom.store(1, Ordering::Relaxed);
        // 获取虚拟机实例
        let vm = GRAY_MGR.read().vm_instance(0, vid).unwrap();
        // gc
        let vm_copy = vm.clone();
        vm.spawn_task(async move {
            let now = Instant::now();
            let context = vm_copy
                .new_context(None, vm_copy.alloc_context_id(), None)
                .await
                .unwrap();
            let _ = vm_copy.gc(context).await;
            let _ = vm_copy.remove_context(context).await;
            info!("auto gc ok vid:{:?}, time:{:?}", vid, Instant::now() - now);
        });
    }
}

// 获取数据库表整理时间 0~23
fn get_db_collect_time() -> Option<u32> {
    let file = std::fs::read_to_string("../dst_server/ptconfig.json").unwrap();
    let json = parse(&file).unwrap();
    let time = &json["collect_time"];
    time.as_u32()
}