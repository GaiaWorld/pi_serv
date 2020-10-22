use std::sync::Arc;
use std::mem::forget;
use std::env;
use std::path::{ Path, PathBuf };
use std::path::Component::Normal;
use std::sync::atomic::{ AtomicUsize, Ordering };

use fnv::FnvHashMap;
use parking_lot::RwLock;
use crossbeam_channel::{Sender, Receiver, unbounded};

use pi_vm::pi_vm_impl::{ VMFactory };
use pi_vm::adapter::{ JS };
use pi_vm::bonmgr::{NativeObjsAuth};
use pi_vm::shell::SHELL_MANAGER;
use atom::Atom;

use file::fs_monitor::{FSMonitorOptions, FSListener, FSMonitor, FSChangeEvent};

use js_lib::JSGray;
use js_env::{ env_var };
use js_file::read_file_string_sync;
use init_js::{read_code, load_core_env};
use js_net::get_all_http_rpc_mods;


lazy_static! {
    // 灰度表
    pub static ref GRAY_TABLE: Arc<RwLock<GrayTable>> = Arc::new(RwLock::new(GrayTable::new()));
    // 每个灰度版本对应的字节码列表
    // { 灰度版本 => { 项目名 => { 模块 id => 字节码 }}}
    pub static ref BYTE_CODE_CACHE: Arc<RwLock<FnvHashMap<usize, FnvHashMap<String, FnvHashMap<String, Arc<Vec<u8>>>>>>> = {
        let mut map = FnvHashMap::default();
        map.insert(0, FnvHashMap::default());
        Arc::new(RwLock::new(map))
    };
    pub static ref GRAY_VERSION: AtomicUsize = AtomicUsize::new(0);
    pub static ref STRUCT_FILES: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::new(vec![]));
    pub static ref NOTIFY_CHAN: (Sender<(usize, Atom)>, Receiver<(usize, Atom)>) = unbounded();
}

pub fn set_struct_files(files: Vec<String>) {
    STRUCT_FILES.write().extend(files.into_iter());
    println!("struct file length = {:?}", STRUCT_FILES.read().len());
}

pub fn get_gray_table() -> Arc<RwLock<GrayTable>> {
    GRAY_TABLE.clone()
}

pub fn register_jsgray(gray_tab: Arc<RwLock<GrayTable>>, version: Option<usize>, jsgray: JSGray) {
    let mut gray_tab = gray_tab.write();
    let name = jsgray.name.clone();
    match version {
        Some(ver) => {
            match gray_tab.jsgrays.get_mut(ver) {
                Some(gray) => {
                    gray.insert(name, Arc::new(jsgray));
                }
                None => {
                    error!("version not found {:?}", version);
                }
            }
        }
        None => {
            match gray_tab.jsgrays.last_mut() {
                Some(gray) => {
                    gray.insert(name, Arc::new(jsgray));
                }
                None => {
                    let mut map = FnvHashMap::default();
                    map.insert(name, Arc::new(jsgray));
                    gray_tab.jsgrays.push(map);
                }
            }
        }
    }
}

// 提升灰度版本号，相应的克隆字节码和jsgray
fn bump_gray_version() {
    let last_version = GRAY_VERSION.load(Ordering::SeqCst);
    let mut map: FnvHashMap<String, FnvHashMap<String, Arc<Vec<u8>>>> = FnvHashMap::default();
    match BYTE_CODE_CACHE.read().get(&last_version) {
        Some(projs) => {
            for proj in projs {
                let mut m = FnvHashMap::default();
                for (k, v) in proj.1.iter() {
                    m.insert(k.clone(), v.clone());
                }
                map.insert(proj.0.clone(), m);
            }
            
        }
        None => {}
    }

    BYTE_CODE_CACHE.write().insert(last_version + 1, map);

    let mut gray_tab = GRAY_TABLE.write();
    let mut map2 = FnvHashMap::default();
    match gray_tab.jsgrays.last() {
        Some(jsgray) => {
            for (k, v) in jsgray.iter() {
                map2.insert(k.clone(), v.clone());
            }
            gray_tab.jsgrays.push(map2);
        }

        None => {}
    }

    // 提升版本号
    GRAY_VERSION.fetch_add(1, Ordering::SeqCst);
}

pub fn get_byte_code(mod_id: String) -> Option<Arc<Vec<u8>>> {
    let last_version = GRAY_VERSION.load(Ordering::SeqCst);
    let proj_name = mod_id.split("/").collect::<Vec<&str>>()[0];

    match BYTE_CODE_CACHE.read().get(&last_version) {
        Some(proj) => {
            match proj.get(proj_name) {
                Some(module) => {
                    module.get(&mod_id).cloned()
                }
                None => None
            }
        }
        None => None
    }
}

pub fn remove_byte_code(mod_id: String) {
    let last_version = GRAY_VERSION.load(Ordering::SeqCst);
    let proj_name = mod_id.split("/").collect::<Vec<&str>>()[0];

    BYTE_CODE_CACHE.write().entry(last_version)
        .and_modify(|version| {
            version.entry(proj_name.to_string()).and_modify(|code|{
                code.remove(&mod_id);
            });
        });
}

pub fn compile_byte_code(mod_id: String, source_code: String) -> Option<Arc< Vec<u8>>> {
    let last_version = GRAY_VERSION.load(Ordering::SeqCst);
    let proj_name = mod_id.split("/").collect::<Vec<&str>>()[0];
    let opts = JS::new(1, Atom::from("hot"), Arc::new(NativeObjsAuth::new(None, None)), None).unwrap();

	match opts.compile(mod_id.clone(), source_code) {
		Some(r) => {
            match BYTE_CODE_CACHE.write().get_mut(&last_version) {
                Some(proj) => {
                    match proj.get_mut(proj_name) {
                        Some(module) => {
                            module.insert(mod_id, Arc::new(r.clone()));
                        }
                        None => {
                            proj.insert(proj_name.to_string(), FnvHashMap::default());
                        }
                    }
                }
                None => {
                    error!("last version should be found");
                }
            }
            Some(Arc::new(r))
		}
		None => None,
	}
}

// 每个项目有自己的虚拟机工厂和字节码缓存
pub struct GrayTable {
    // 每个灰度版本的所有 jsgray
    pub jsgrays: Vec<FnvHashMap<Atom, Arc<JSGray>>>,
}

impl GrayTable {
    pub fn new() -> Self {
        println!("launched projects {:?}", launched_projects());
        GrayTable {
            jsgrays: vec![FnvHashMap::default()],
        }
    }
}

pub fn gray_table_to_arc(gray_tab: GrayTable) -> Arc<RwLock<GrayTable>> {
    Arc::new(RwLock::new(gray_tab))
}


pub fn hotfix_listen(path: String) {
    let listener = FSListener(Arc::new(move |event: FSChangeEvent| {
        match event {
            FSChangeEvent::Create(_path) => {
                // 不处理这个事件
            },
            // 每次文件改变都会增加一个灰度版本号
            FSChangeEvent::Write(path) => {
                let mod_id = normalize_module_id(path.to_str().unwrap());
                if mod_id.ends_with(".js") {
                    debug!("modified path: {:?}", path);
                    let proj_name = get_proj_name_from_path(&path);
                    debug!(" path: {:?}, proj_name: {:?}", path, proj_name);
                    bump_gray_version();
                    module_changed(path);
                }
            },
            FSChangeEvent::Remove(path) => {
                let mod_id = normalize_module_id(path.to_str().unwrap());
                if mod_id.ends_with(".js") {
                    debug!("removed path: {:?}", path);
                    bump_gray_version();
                    if mod_id.ends_with(".event.js") {
                        // 如果删除的是 .event.js 结尾的模块，那么删除对应的虚拟机工厂
                        match GRAY_TABLE.write().jsgrays.last_mut().unwrap().remove(&Atom::from(mod_id.clone())) {
                            Some(_) => debug!("remove factory success : {:?}", mod_id),
                            None => debug!("{:?} factory not exist", mod_id)
                        }
                    }
                    module_changed(path);
                }
            },
            FSChangeEvent::Rename(old, new) => {
                // 名字的变更会引起引入名字的变化，不需要处理
            },
        };
    }));
    let mut monitor = FSMonitor::new(FSMonitorOptions::Dir(Atom::from(path), true, 1000), listener);
    monitor.run().expect("watch dir failed");
    forget(monitor);
}

fn module_changed(path: PathBuf) {
    let path = match path.to_str() {
        Some(path) => path,
        None => {
            error!("module change path is None");
            return
        }
    };
    let mod_id = normalize_module_id(path);

    let mut gray_tab = GRAY_TABLE.write();
    let current_version = gray_tab.jsgrays.len() - 1;
    if let Some(jsgrays) = gray_tab.jsgrays.last_mut() {
        for (k, v) in jsgrays.iter_mut() {
            let auth = Arc::new(NativeObjsAuth::new(None, None));
            let js = match JS::new(1, Atom::from("hotfix compile"), auth.clone(), None) {
                Some(js) => js,
                None => {
                    error!("new hotfix compile vm failed, change path: {:?}", path);
                    return
                }
            };

            load_core_env(&js);

            let mut cur_exe = match env::current_exe() {
                Ok(cur_exe) => cur_exe,
                Err(e) => {
                    error!("get current exe failed, change path: {:?}, error: {:?}", path, e);
                    return
                }
            };
            cur_exe.pop();

            let env_code = read_code(&cur_exe.join("env.js"));
            let core_code = read_code(&cur_exe.join("core.js"));

            let env_code = match js.compile("env.js".to_string(), env_code) {
                Some(env_code) => env_code,
                None => {
                    error!("compile env.js code failed, change path: {:?}", path);
                    return
                }
            };
            let core_code = match js.compile("core.js".to_string(), core_code) {
                Some(env_code) => env_code,
                None => {
                    error!("compile core.js code failed, change path: {:?}", path);
                    return
                }
            };

            let mgr = v.mgr.clone();
            let auth = Arc::new(NativeObjsAuth::new(None, None));
            let mut vmf = VMFactory::new(k.as_str(), 128, 2, 33554432, 33554432, auth);

            // env.js / core.js 代码
            vmf = vmf.append(Arc::new(env_code));
            vmf = vmf.append(Arc::new(core_code));

            let mut files = vec!["pi_pt/net/rpc_entrance.js", "pi_pt/net/mqtt_broker.js", "pi_pt/util/migration.struct.js"
                    , "pi_pt/util/platmgr.struct.js", "pi_pt/rust/pi_serv/webshell.js", "pi_pt/util/migration.event.js"
                    , "pi_pt/util/hotback.struct.js", "pi_pt/db/dblistener.js"];

            let struct_files = STRUCT_FILES.read();
            files.extend(struct_files.iter().map(|s| s.as_str()));

            // http rpc 的热更新
            let http_code = get_all_http_rpc_mods().into_iter().fold("".to_string(), |acc, x| {
                let mut mod_name = x.split(".").nth(0).unwrap().to_string();
                mod_name += ".event";
                acc + format!("Module.require(\'{}\', '');", mod_name).as_str()
            });
            debug!("http_code: {:?}", http_code);

            remove_byte_code(mod_id.clone());

            let mut extra_code = String::from("");
            for file in files.iter() {
                extra_code += format!("Module.require(\'{}\', '');", file).as_str();
            }

            let extra_code = extra_code + format!("Module.require(\'{}\', '');", k.clone().to_string()).as_str();
            let extra_code = extra_code + http_code.as_str();
            let extra_code = match js.compile("rpc_entrance".to_string(), extra_code) {
                Some(extra_code) => extra_code,
                None => {
                    error!("compile extra code failed, change path: {:?}", path);
                    return
                }
            };

            // rpc 功能依赖的代码，和实际处理rpc需要的代码
            vmf = vmf.append(Arc::new(extra_code));
            if let Err(e) = vmf.produce(2) {
                error!("vm factory produce failed, change path: {:?}, error: {:?}", path, e);
                return
            }

            if v.factory.is_depend(&mod_id) {
                debug!("{:?} is a depend for {:?}", mod_id, k);
                let arc_vmf = Arc::new(vmf);

                let jsgray = JSGray::new(&mgr, arc_vmf.clone(), k.as_str());
                // 用新的代码替换
                *v = Arc::new(jsgray);
                SHELL_MANAGER.write().unwrap().set_factory(arc_vmf.clone());
            } else {
                let deps = get_depends(&js, k.as_str());
                for dep in deps {
                    vmf = vmf.append_depend(dep);
                }
                let arc_vmf = Arc::new(vmf);

                let jsgray = JSGray::new(&mgr, arc_vmf.clone(), k.as_str());
                *v = Arc::new(jsgray);
                SHELL_MANAGER.write().unwrap().set_factory(arc_vmf.clone());
            }
            let _ = NOTIFY_CHAN.0.send((current_version - 1, k.clone()));
        }
    }
}

fn get_depends(js: &Arc<JS>, vmf_name: &str) -> Vec<String> {
    let cur_dir = env_var("PROJECT_ROOT").unwrap();

    if js.get_link_function("Module.require".to_string()) {
        js.new_str(vmf_name.clone().to_string()).unwrap();
        js.new_str(cur_dir).unwrap();
        js.call(2);
    } else {
        panic!("Module.require function is not exist");
    }

    js.get_js_function("getDepends".to_string());
    let depends = js.invoke(0);
    depends.get_str().split(" ").filter(|dep| dep.len() > 0).map(|dep|dep.to_string()).collect()
}

fn normalize_module_id(mod_id: &str) -> String {
    let root = match env_var("PROJECT_ROOT") {
        Ok(root) => root,
        Err(e) => {
            error!("Can't get PROJECT_ROOT env, mod_id: {:?}, error: {:?}", mod_id, e);
            return "".to_string();
        }
    };
    mod_id.replace("\\", "/")
        .as_str()
        .trim_start_matches(&(root + "/"))
        .to_string()
}

// 平台启动了哪些项目，根据是否有 ptconfig.json 配置文件来判断
// 如果没有 ptocnfig.json 文件就认为不是一个项目
fn launched_projects() -> Vec<String> {
    env_var("PROJECTS").unwrap()
        .split(" ")
        .filter(|p| {
            let path = PathBuf::from(p);
            if path.join("ptconfig.json").exists() {
                true
            } else {
                false
            }
        })
        .map(|s| s.to_string())
        .collect()
}

// 从被修改文件的路径得到是哪个项目被修改
// path 参数是文件监控得到的绝对路径
fn get_proj_name_from_path(path: &PathBuf) -> String {
    let projs = launched_projects();

    let comps = Path::new(path).canonicalize().unwrap();
    let comps = comps.components();
    let comps = comps.filter(|comp| if let Normal(_) = comp { true } else { false }).map(|p| if let Normal(c) = p { c.to_str().unwrap() } else { "" }).collect::<Vec<&str>>();
    
    // project name without relative path
    for proj in projs {
        let p = Path::new(&proj).canonicalize().unwrap();
        let p = p.components();
        let p = p.filter(|comp| if let Normal(_) = comp { true } else { false }).map(|p| if let Normal(c) = p { c.to_str().unwrap() } else { "" }).collect::<Vec<&str>>();

        let last = p.last().unwrap();
        if comps.contains(last) {
            return last.to_string()
        } else {
            return "".to_string()
        }
    }

    return "".to_string()
}

