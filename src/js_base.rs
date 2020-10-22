use std::sync::{Arc, Mutex};
use std::ops::Deref;
use std::sync::atomic::{AtomicUsize};

use rand::rngs::OsRng;
use rand::RngCore;

use pi_vm::pi_vm_impl::{VMFactory, register_async_request, push_callback};
use pi_vm::adapter::{JSType, JS};
use pi_vm::bonmgr::{BON_MGR};
use pi_db::db::TabKV;
use atom::Atom;
use sinfo::StructInfo;
use bon::{ReadBuffer, Decode};
use timer::{TIMER, FuncRuner};

use js_async::AsyncRequestHandler;
use bon::ReadBonErr;

/**
* 创建一个自定义对象序列化元信息的引用计数
* @param data 元信息的二进制数据
* @returns 返回创建的结果，成功返回自定义对象序列化元信息
* @throws 失败则抛出原因描述
*/
pub fn create_sinfo(data: &[u8]) -> Result<Arc<StructInfo>, ReadBonErr>{
	let mut buf = ReadBuffer::new(data, 0);
	Ok(Arc::new(StructInfo::decode(&mut buf)?))
}

/**
* Copy指定的虚拟机工厂
* @param factory 待Copy的虚拟机工厂
* @returns 返回Copy的虚拟机工厂
*/
pub fn clone_vm_factory(factory: &VMFactory) -> VMFactory{
    factory.clone()
}

/**
* 创建一个异步请求处理器的引用计数
* @param arh 异步请求处理器
* @returns 返回异步请求处理器的引用计数
*/
pub fn arc_new_async_request_handler(arh: AsyncRequestHandler) -> Arc<AsyncRequestHandler> {
    Arc::new(arh)
}

/**
* 为一个指定topic的异步请求，注册处理器
* @param topic 异步请求的topic
* @param handler 异步请求处理器的引用计数
*/
pub fn register_async_handler(topic: String, handler: &Arc<AsyncRequestHandler>){
    register_async_request(Atom::from(topic), handler.clone());
}

/**
* 创建一个指定对象的引用计数
* @param v 对象
* @returns 返回指定对象的引用计数
*/
pub fn arc_new<T>(v: T) -> Arc<T>{
    Arc::new(v)
}

/**
* 解引用一个指定对象的引用计数
* @param v 指定对象的引用计数
* @returns 返回指定对象
*/
pub fn arc_deref< T>(v: &Arc<T>) -> &T{
    v.deref()
}

/**
* 创建一个vec
* @param v 数组
* @returns 返回Vec
*/
pub fn vec_from<T: Clone>(v: &[T]) -> Vec<T>{
	Vec::from(v)
}

/**
* 创建一个指定对象的指针
* @param v 对象
* @returns 返回指定对象的指针
*/
pub fn box_new<T>(v: T) -> Box<T>{
    Box::new(v)
}

/**
* 同步阻塞的暂停当前虚拟机的执行
* @param ms 暂停的时长，单位毫秒，等于0表示空调度当前虚拟机，大于0则表示在指定时间内暂停虚拟机运行
*/
pub fn sleep(ms: u32, f: Box<FnOnce()>){
	TIMER.set_timeout(FuncRuner::new(f), ms);
}

// /**
// * 同步的设置定时异步回调
// * @param ms 间隔的时长，单位毫秒
// * @param cb 异步回调
// * @returns 返回定时任务的编号
// */
// pub fn set_timeout(js: Arc<JS>, callback: u32, timeout: u32, info: Atom, args: Box<FnOnce(Arc<JS>) -> usize>) -> Option<isize> {
//     push_callback(js, callback, args, Some(timeout), info)
// }

pub fn clear_timeout(index: usize){
	// TIMER.cancel(index);
}

/**
* 随机数生成器
*/
pub struct Rand(OsRng);

/**
* 创建一个随机数生成器
* @returns 返回随机数生成器
*/
pub fn create_rand() -> Rand{
	Rand(OsRng::new().expect("create_osrng fail"))
}

/**
* 取到一个随机32位整数
* @param or 随机数生成器
* @returns 返回随机32位整数
*/
pub fn next_u32(or: &mut Rand) -> u32{
	or.0.next_u32()
}

/**
* 取到一个随机64位整数
* @param or 随机数生成器
* @returns 返回随机64位整数
*/
pub fn next_u64(or: &mut Rand) -> u64{
	or.0.next_u64()
}

/**
* 取到一个指定长度的随机buffer
* @param or 随机数生成器
* @param len 长度
* @returns 返回随机buffer
*/
pub fn fill_bytes(or: &mut Rand, len: usize) -> Vec<u8>{
    let mut arr = Vec::with_capacity(len);
    unsafe{arr.set_len(len);};
	or.0.fill_bytes(arr.as_mut_slice());
    arr
}

/**
* 取到一个指定长度的随机buffer
* @param or 随机数生成器
* @param len 长度
* @returns 返回结果，成功返回随机buffer
* @throws 失败抛出原因描述
*/
pub fn try_fill_bytes(or: &mut Rand, len: usize) -> Result<Vec<u8>, String> {
    let mut arr = Vec::new();
    unsafe{arr.set_len(len);};
	match or.0.try_fill_bytes(arr.as_mut_slice()) {
        Ok(_) => Ok(arr),
        Err(e) => Err(String::from(e.msg)),
    }
}

pub fn drop_native_obj(t: &JSType, js: &Arc<JS>) -> Result<bool, String> {
    if !t.is_native_object(){
        return Err(String::from("drop_native_obj err, param is not NativeObject!"))
    }

    let ptr = t.get_native_object();
    let objs = js.get_objs();
    let mut objs = objs.borrow_mut();
    let struct_metas = BON_MGR.struct_metas.lock().unwrap();
    match objs.remove(&ptr){
        Some(v) => {
            let meta = struct_metas.get(&v.meta_hash).unwrap();
            //println!("drop_native_obj---------------------------------------------{}", meta.name);
            (meta.drop_fn)(ptr);
            Ok(true)
        },
        None => {
            //println!("drop_native_obj fail---------------------------------------------");
            Ok(false)
        }
    }
}
