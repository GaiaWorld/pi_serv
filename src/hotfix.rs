use std::env;
use std::ffi::OsString;
use std::mem::forget;
use std::path::PathBuf;
use std::sync::Arc;

use crossbeam_channel::bounded;
use dunce::canonicalize;
use json::stringify;

use atom::Atom;
use file::fs_monitor::{FSChangeEvent, FSListener, FSMonitor, FSMonitorOptions};
use pi_serv_lib::js_gray::GRAY_MGR;
use vm_builtin::ContextHandle;

use crate::js_net::HTTP_STATIC_CACHES;
use crate::MAIN_ASYNC_RUNTIME;
use crate::VID_CONTEXTS;

const INIT_FILE: &str = "pi_pt/init.js";

fn module_changed(change_path: PathBuf, prefix: PathBuf) {
    for (vid, ctxs) in VID_CONTEXTS.lock().iter() {
        let path = canonicalize(&change_path).unwrap();
        let path = path
            .strip_prefix(&prefix)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let path = path.replace("\\", "/");

        // init.js 的改变不应该热更，因为init.js中会启动网络服务，导致冲突
        if path == INIT_FILE {
            break;
        }

        match GRAY_MGR.read().vm_instance(0, vid.clone()) {
            Some(vm) => {
                // 虚拟机的每个context重新require该模块
                for context in ctxs {
                    let change_path = change_path.clone();
                    let vm = vm.clone();
                    let path = path.clone();
                    let context = context.clone();
                    let _ = MAIN_ASYNC_RUNTIME.spawn(MAIN_ASYNC_RUNTIME.alloc(), async move {
                        if let Ok(Some(func)) = vm
                            .get_property(context.clone(), "self.Module.require")
                            .await
                        {
                            let path = vm
                                .to_js_value(context.clone(), stringify(path))
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
                                .call(context.clone(), &func, vec![path, dir, force])
                                .await
                            {
                                warn!("hotfix call require error");
                            } else {
                                debug!("module_changed success path: {:?}", change_path);
                            }
                        } else {
                            warn!("hotfix get vm context failed");
                        }
                    });
                }
            }
            None => {
                warn!("hotfix create module, can't find vm, path = {:?}", path);
            }
        }
    }
}

// 删除模块
fn module_removed(change_path: PathBuf) {
    for (vid, ctxs) in VID_CONTEXTS.lock().iter() {
        let path = change_path.to_str().unwrap();
        let path = path.replace("\\", "/");

        match GRAY_MGR.read().vm_instance(0, vid.clone()) {
            Some(vm) => {
                for context in ctxs {
                    let change_path = change_path.clone();
                    let vm = vm.clone();
                    let path = path.clone();
                    let context = context.clone();
                    let _ = MAIN_ASYNC_RUNTIME.spawn(MAIN_ASYNC_RUNTIME.alloc(), async move {
                        if let Ok(Some(func)) = vm
                            .get_property(context.clone(), "self.Module.delete_module")
                            .await
                        {
                            let path = vm
                                .to_js_value(context.clone(), stringify(path))
                                .await
                                .unwrap()
                                .unwrap();
                            if let Err(_e) = vm.call(context.clone(), &func, vec![path]).await {
                                warn!("hotfix call delete_module error");
                            } else {
                                debug!("module_removed success path: {:?}", change_path);
                            }
                        } else {
                            warn!("hotfix get vm context failed");
                        }
                    });
                }
            }
            None => {
                warn!("hotfix create module, can't find vm, path = {:?}", path);
            }
        }
    }
}

/// 后端资源热更
pub fn hotfix_listen_backend(path: String) {
    let listener = FSListener(Arc::new(move |event: FSChangeEvent| {
        let proj_root = env::var("PROJECT_ROOT").unwrap();
        let prefix = canonicalize(proj_root).unwrap();

        match event {
            FSChangeEvent::Create(path) => {
                let path_clone = path.clone();
                path.extension().and_then(|p| {
                    p.to_str().and_then(|s| {
                        if s == "js" {
                            debug!("hotfix create path {:?}", path);
                            Some(module_changed(path_clone, prefix))
                        } else {
                            None
                        }
                    })
                });
            }
            FSChangeEvent::Write(path) => {
                let path_clone = path.clone();
                path.extension().and_then(|p| {
                    p.to_str().and_then(|s| {
                        if s == "js" {
                            debug!("hotfix modified path {:?}", path);
                            Some(module_changed(path_clone, prefix))
                        } else {
                            None
                        }
                    })
                });
            }
            FSChangeEvent::Remove(path) => {
                let path_clone = path.clone();
                path.extension().and_then(|p| {
                    p.to_str().and_then(|s| {
                        if s == "js" {
                            debug!("hotfix remove path {:?}", path);
                            Some(module_removed(path_clone))
                        } else {
                            None
                        }
                    })
                });
            }
            FSChangeEvent::Rename(old, new) => {
                let path_clone = new.clone();
                new.extension().and_then(|p| {
                    p.to_str().and_then(|s| {
                        if s == "js" {
                            debug!("hotfix rename path old = {:?}, new = {:?}", old, new);
                            Some(module_changed(path_clone.clone(), prefix));
                            Some(module_removed(path_clone))
                        } else {
                            None
                        }
                    })
                });
            }
        };
    }));
    let mut monitor = FSMonitor::new(
        FSMonitorOptions::Dir(Atom::from(path), true, 1000),
        listener,
    );
    monitor.run().expect("watch dir failed");
    forget(monitor);
}

/// 前端资源热更， 清除http缓存
pub fn hotfix_listen_frontend() {
    for cache in HTTP_STATIC_CACHES.read().iter() {
        cache.remove_all_cache();
    }
}
