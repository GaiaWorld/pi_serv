use pi_vm::bonmgr::{BonMgr, StructMeta, FnMeta, jstype_ptr,ptr_jstype, CallResult};
use pi_vm::adapter::{JSType, JS};
use std::sync::Arc;
use pi_lib;
use https;



fn call_3779679042(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in new";

	let jst0 = &v[0];
	if !jst0.is_string(){ return Some(CallResult::Err(String::from(param_error)));}
	let jst0 = jst0.get_str();


    let result = https::file::StaticFile::new(jst0);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,369829824);


    Some(CallResult::Ok)
}


fn call_3011830990(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in new";

	let jst0 = &v[0];
	if !jst0.is_string(){ return Some(CallResult::Err(String::from(param_error)));}
	let jst0 = jst0.get_str();


    let result = https::files::StaticFileBatch::new(jst0);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,2592534340);


    Some(CallResult::Ok)
}


fn call_1576795673(js: Arc<JS>) -> Option<CallResult>{

    let result = https::mount::Mount::new();
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,969075058);


    Some(CallResult::Ok)
}


fn call_3977181471(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in mount";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 969075058, false, param_error).expect("");
	let jst0 = unsafe { &mut *(ptr as *mut https::mount::Mount) };


	let jst1 = &v[1];
	if !jst1.is_string(){ return Some(CallResult::Err(String::from(param_error)));}
	let jst1 = &jst1.get_str();


	let jst2 = &v[2];
    let ptr = jstype_ptr(&jst2, js.clone(), 369829824, true, param_error).expect("");
	let jst2 = *unsafe { Box::from_raw(ptr as *mut https::file::StaticFile) };


    let result = https::mount::Mount::mount(jst0,jst1,jst2);
    let ptr = result as *const https::mount::Mount as usize;let mut result = ptr_jstype(js.get_objs_ref(), js.clone(), ptr,969075058);


    Some(CallResult::Ok)
}


fn call_4128314446(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in mount";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 969075058, false, param_error).expect("");
	let jst0 = unsafe { &mut *(ptr as *mut https::mount::Mount) };


	let jst1 = &v[1];
	if !jst1.is_string(){ return Some(CallResult::Err(String::from(param_error)));}
	let jst1 = &jst1.get_str();


	let jst2 = &v[2];
    let ptr = jstype_ptr(&jst2, js.clone(), 2592534340, true, param_error).expect("");
	let jst2 = *unsafe { Box::from_raw(ptr as *mut https::files::StaticFileBatch) };


    let result = https::mount::Mount::mount(jst0,jst1,jst2);
    let ptr = result as *const https::mount::Mount as usize;let mut result = ptr_jstype(js.get_objs_ref(), js.clone(), ptr,969075058);


    Some(CallResult::Ok)
}


fn call_374744388(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in start_http";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 969075058, true, param_error).expect("");
	let jst0 = *unsafe { Box::from_raw(ptr as *mut https::mount::Mount) };


	let jst1 = &v[1];
    let ptr = jstype_ptr(&jst1, js.clone(), 1411051473, true, param_error).expect("");
	let jst1 = *unsafe { Box::from_raw(ptr as *mut pi_lib::atom::Atom) };


	let jst2 = &v[2];
	if !jst2.is_number(){ return Some(CallResult::Err(String::from(param_error)));}
	let jst2 = jst2.get_u16();


	let jst3 = &v[3];
	if !jst3.is_number(){ return Some(CallResult::Err(String::from(param_error)));}
	let jst3 = jst3.get_u32() as usize;


	let jst4 = &v[4];
	if !jst4.is_number(){ return Some(CallResult::Err(String::from(param_error)));}
	let jst4 = jst4.get_u32();


    https::https_impl::start_http(jst0,jst1,jst2,jst3,jst4);
    Some(CallResult::Ok)
}

fn drop_369829824(ptr: usize){
    unsafe { Box::from_raw(ptr as *mut https::file::StaticFile) };
}

fn drop_2592534340(ptr: usize){
    unsafe { Box::from_raw(ptr as *mut https::files::StaticFileBatch) };
}

fn drop_969075058(ptr: usize){
    unsafe { Box::from_raw(ptr as *mut https::mount::Mount) };
}

fn drop_1411051473(ptr: usize){
    unsafe { Box::from_raw(ptr as *mut pi_lib::atom::Atom) };
}
pub fn register(mgr: &BonMgr){
    mgr.regist_struct_meta(StructMeta{name:String::from("https::file::StaticFile"), drop_fn: drop_369829824}, 369829824);
    mgr.regist_struct_meta(StructMeta{name:String::from("https::files::StaticFileBatch"), drop_fn: drop_2592534340}, 2592534340);
    mgr.regist_struct_meta(StructMeta{name:String::from("https::mount::Mount"), drop_fn: drop_969075058}, 969075058);
    mgr.regist_struct_meta(StructMeta{name:String::from("pi_lib::atom::Atom"), drop_fn: drop_1411051473}, 1411051473);
    mgr.regist_fun_meta(FnMeta::CallArg(call_3779679042), 3779679042);
    mgr.regist_fun_meta(FnMeta::CallArg(call_3011830990), 3011830990);
    mgr.regist_fun_meta(FnMeta::Call(call_1576795673), 1576795673);
    mgr.regist_fun_meta(FnMeta::CallArg(call_3977181471), 3977181471);
    mgr.regist_fun_meta(FnMeta::CallArg(call_4128314446), 4128314446);
    mgr.regist_fun_meta(FnMeta::CallArg(call_374744388), 374744388);
}