use pi_vm::bonmgr::{BonMgr, StructMeta, FnMeta, jstype_ptr,ptr_jstype, CallResult};
use pi_vm::adapter::{JSType, JS};
use std::sync::Arc;
use mqtt;
use rpc;



fn call_193751450(js: Arc<JS>, mgr: &BonMgr, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in rpc::server::RPCServer";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, mgr, 1751456239, true, param_error).expect("");
	let jst0 = *unsafe { Box::from_raw(ptr as *mut mqtt::server::ServerNode) };


    let result = rpc::server::RPCServer::new(jst0);
    let ptr = Box::into_raw(Box::new(result)) as usize;let result = ptr_jstype(mgr.objs.clone(), js.clone(), ptr,1285687456);


    Some(CallResult::Ok)
}
pub fn register(mgr: &BonMgr){
    mgr.regist_struct_meta(StructMeta{name:String::from("mqtt::server::ServerNode")}, 1751456239);
    mgr.regist_struct_meta(StructMeta{name:String::from("rpc::server::RPCServer")}, 1285687456);
    mgr.regist_fun_meta(FnMeta::CallArgNobj(call_193751450), 193751450);
}