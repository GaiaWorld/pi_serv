// 后台定时任务
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use atom::Atom;
use pi_db::mgr::Mgr;
use pi_vm::adapter::JS;
use pi_vm::bonmgr::ptr_jstype;
use pi_vm::pi_vm_impl::VMFactory;
use time::run_millis;
use wheel::slab_wheel::Wheel;
use wheel::wheel::Item;

struct TimerTask(Arc<VMFactory>, Box<dyn FnOnce(Arc<JS>) -> usize>);

unsafe impl Send for TimerTask {}
unsafe impl Sync for TimerTask {}

lazy_static! {
    // 定时任务的虚拟机工厂
    static ref FACTORY: Arc<RwLock<HashMap<String, (Arc<VMFactory>, Mgr)>>> =
        Arc::new(RwLock::new(HashMap::new()));
    // 所有的定时任务集合，用来支持取消任务
    static ref TASK_MAP: Arc<RwLock<HashMap<String, usize>>> =
        Arc::new(RwLock::new(HashMap::new()));

    // 全局定时轮
    static ref WHELL: Arc<RwLock<Wheel<String>>> = {
        let mut wheel = Wheel::new();
        wheel.set_time(run_millis());
        Arc::new(RwLock::new(wheel))
    };

    // 周期性任务
    static ref PERIOIDC_TASKS: Arc<RwLock<HashMap<String, usize>>> = Arc::new(RwLock::new(HashMap::new()));
}

pub fn register_timer_task_vm_factory(topic: String, vmf: Arc<VMFactory>, dbmgr: Mgr) {
    FACTORY.write().insert(topic, (vmf, dbmgr));
}

pub fn insert_oneshot_task(topic: String, after: usize) {
    let mut wheel = WHELL.write();
    let cur_time = wheel.get_time();
    let index = wheel.insert(Item {
        elem: topic.clone(),
        time_point: cur_time + after as u64,
    });
    TASK_MAP.write().insert(topic, index);
}

pub fn insert_periodic_task(topic: String, period: usize) {
    insert_oneshot_task(topic.clone(), period);
    PERIOIDC_TASKS.write().insert(topic, period);
}

pub fn cancel_task(topic: String) {
    let mut wheel = WHELL.write();
    TASK_MAP
        .write()
        .remove(&topic)
        .and_then(|index| wheel.remove(index));
    PERIOIDC_TASKS.write().remove(&topic);
}

pub fn tick() {
    let mut wheel = WHELL.write();
    let factory = FACTORY.read();
    let expired = wheel.roll();
    for (Item { elem, .. }, _) in expired {
        // 如果执行的是周期性任务, 再次添加到时间轮里面
        if let Some(period) = PERIOIDC_TASKS.read().get(&elem) {
            let cur_time = wheel.get_time();
            let _index = wheel.insert(Item {
                elem: elem.clone(),
                time_point: cur_time + *period as u64,
            });
        }

        // 通过 elem 找到虚拟机工厂
        let fact = factory.get(&elem).cloned();
        if let Some((vmf, dbmgr)) = fact {
            let real_args = Box::new(move |vm: Arc<JS>| {
                let ptr = Box::into_raw(Box::new(dbmgr.clone())) as usize;
                ptr_jstype(vm.get_objs(), vm.clone(), ptr, 2976191628);
                let _ = vm.new_str(elem.to_string()); // topic 参数
                2
            });
            vmf.call(
                None,
                Atom::from("_$timer_task"),
                real_args,
                Atom::from("timer task"),
            );
        }
    }
}