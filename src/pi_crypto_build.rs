use pi_vm::bonmgr::{BonMgr, StructMeta, FnMeta, jstype_ptr,ptr_jstype, CallResult};
use pi_vm::adapter::{JSType, JS};
use std::sync::Arc;
use std::mem::transmute;
use atom::Atom;
use hash_value;
use std::ops::Drop;
use pi_crypto;



fn call_266558349(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in exchange";

	let jst0 = &v[0];
	if !jst0.is_uint8_array() && !jst0.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst0 = jst0.to_bytes();



	let jst1 = &v[1];
	if !jst1.is_uint8_array() && !jst1.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst1 = jst1.to_bytes();



    let result = pi_crypto::ed25519::exchange(jst0,jst1);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,1035403249);


    Some(CallResult::Ok)
}


fn call_2282179587(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in keypair";

	let jst0 = &v[0];
	if !jst0.is_uint8_array() && !jst0.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst0 = jst0.to_bytes();



    let result = pi_crypto::ed25519::keypair(jst0);
	let array = js.new_array();
    let mut result_elem = result.0;
    let ptr = Box::into_raw(Box::new(result_elem)) as usize;let mut result_elem = ptr_jstype(js.get_objs(), js.clone(), ptr,3223866506);

js.set_index(&array, 0, &mut result_elem);
    let mut result_elem = result.1;
    let ptr = Box::into_raw(Box::new(result_elem)) as usize;let mut result_elem = ptr_jstype(js.get_objs(), js.clone(), ptr,1035403249);

js.set_index(&array, 1, &mut result_elem);

    Some(CallResult::Ok)
}


fn call_1005885597(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in sign";

	let jst0 = &v[0];
	if !jst0.is_uint8_array() && !jst0.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst0 = jst0.to_bytes();



	let jst1 = &v[1];
	if !jst1.is_uint8_array() && !jst1.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst1 = jst1.to_bytes();



    let result = pi_crypto::ed25519::sign(jst0,jst1);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,3223866506);


    Some(CallResult::Ok)
}


fn call_1115867356(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in verify";

	let jst0 = &v[0];
	if !jst0.is_uint8_array() && !jst0.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst0 = jst0.to_bytes();



	let jst1 = &v[1];
	if !jst1.is_uint8_array() && !jst1.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst1 = jst1.to_bytes();



	let jst2 = &v[2];
	if !jst2.is_uint8_array() && !jst2.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst2 = jst2.to_bytes();



    let result = pi_crypto::ed25519::verify(jst0,jst1,jst2);let mut result = js.new_boolean(result);

    Some(CallResult::Ok)
}


fn call_3937242908(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in digest";

	let jst0 = &v[0];
    if !jst0.is_number(){return Some(CallResult::Err(String::from(param_error)));}
    let jst0 = match jst0.get_u32(){
        0 => pi_crypto::digest::DigestAlgorithm::MD5,
        1 => pi_crypto::digest::DigestAlgorithm::SHA1,
        2 => pi_crypto::digest::DigestAlgorithm::SHA256,
        3 => pi_crypto::digest::DigestAlgorithm::SHA384,
        4 => pi_crypto::digest::DigestAlgorithm::SHA512,
        _ => panic!("enum type error")
    };


	let jst1 = &v[1];
	if !jst1.is_uint8_array() && !jst1.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst1 = jst1.to_bytes();



    let result = pi_crypto::digest::digest(jst0,jst1);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,104530634);


    Some(CallResult::Ok)
}


fn call_3800356447(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in sign";

	let jst0 = &v[0];
    if !jst0.is_number(){return Some(CallResult::Err(String::from(param_error)));}
    let jst0 = match jst0.get_u32(){
        0 => pi_crypto::hmac::DigestAlgorithm::SHA1,
        1 => pi_crypto::hmac::DigestAlgorithm::SHA256,
        2 => pi_crypto::hmac::DigestAlgorithm::SHA384,
        3 => pi_crypto::hmac::DigestAlgorithm::SHA512,
        _ => panic!("enum type error")
    };


	let jst1 = &v[1];
	if !jst1.is_uint8_array() && !jst1.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst1 = jst1.to_bytes();



	let jst2 = &v[2];
	if !jst2.is_uint8_array() && !jst2.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst2 = jst2.to_bytes();



    let result = pi_crypto::hmac::Hmac::sign(jst0,jst1,jst2);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,104530634);


    Some(CallResult::Ok)
}


fn call_2199666057(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in verify";

	let jst0 = &v[0];
    if !jst0.is_number(){return Some(CallResult::Err(String::from(param_error)));}
    let jst0 = match jst0.get_u32(){
        0 => pi_crypto::hmac::DigestAlgorithm::SHA1,
        1 => pi_crypto::hmac::DigestAlgorithm::SHA256,
        2 => pi_crypto::hmac::DigestAlgorithm::SHA384,
        3 => pi_crypto::hmac::DigestAlgorithm::SHA512,
        _ => panic!("enum type error")
    };


	let jst1 = &v[1];
	if !jst1.is_uint8_array() && !jst1.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst1 = jst1.to_bytes();



	let jst2 = &v[2];
	if !jst2.is_uint8_array() && !jst2.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst2 = jst2.to_bytes();



	let jst3 = &v[3];
	if !jst3.is_uint8_array() && !jst3.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst3 = jst3.to_bytes();



    let result = pi_crypto::hmac::Hmac::verify(jst0,jst1,jst2,jst3);let mut result = js.new_boolean(result);

    Some(CallResult::Ok)
}


fn call_3054628822(js: Arc<JS>) -> Option<CallResult>{

    let result = pi_crypto::signature::ECDSASecp256k1::new();
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,3761116463);


    Some(CallResult::Ok)
}


fn call_1378957447(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in sign";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 3761116463, false, param_error).expect("");
	let jst0 = unsafe { &*(ptr as *const pi_crypto::signature::ECDSASecp256k1) };


	let jst1 = &v[1];
	if !jst1.is_uint8_array() && !jst1.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst1 = jst1.to_bytes();



	let jst2 = &v[2];
	if !jst2.is_uint8_array() && !jst2.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst2 = jst2.to_bytes();



    let result = pi_crypto::signature::ECDSASecp256k1::sign(jst0,jst1,jst2);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,104530634);


    Some(CallResult::Ok)
}


fn call_758410087(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in verify";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 3761116463, false, param_error).expect("");
	let jst0 = unsafe { &*(ptr as *const pi_crypto::signature::ECDSASecp256k1) };


	let jst1 = &v[1];
	if !jst1.is_uint8_array() && !jst1.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst1 = jst1.to_bytes();



	let jst2 = &v[2];
	if !jst2.is_uint8_array() && !jst2.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst2 = jst2.to_bytes();



	let jst3 = &v[3];
	if !jst3.is_uint8_array() && !jst3.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst3 = jst3.to_bytes();



    let result = pi_crypto::signature::ECDSASecp256k1::verify(jst0,jst1,jst2,jst3);let mut result = js.new_boolean(result);

    Some(CallResult::Ok)
}


fn call_835933247(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in fromPKCS8";

	let jst0 = &v[0];
	if !jst0.is_uint8_array() && !jst0.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst0 = jst0.to_bytes();



    let result = pi_crypto::signature::Rsa::fromPKCS8(jst0);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,1545795468);


    Some(CallResult::Ok)
}


fn call_1239750690(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in public_key";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 1545795468, false, param_error).expect("");
	let jst0 = unsafe { &*(ptr as *const pi_crypto::signature::Rsa) };


    let result = pi_crypto::signature::Rsa::public_key(jst0);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,104530634);


    Some(CallResult::Ok)
}


fn call_2850709748(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in sign";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 1545795468, false, param_error).expect("");
	let jst0 = unsafe { &*(ptr as *const pi_crypto::signature::Rsa) };


	let jst1 = &v[1];
    if !jst1.is_number(){return Some(CallResult::Err(String::from(param_error)));}
    let jst1 = match jst1.get_u32(){
        0 => pi_crypto::signature::PaddingAlg::RSA_PKCS1_SHA256,
        1 => pi_crypto::signature::PaddingAlg::RSA_PKCS1_SHA384,
        2 => pi_crypto::signature::PaddingAlg::RSA_PKCS1_SHA512,
        3 => pi_crypto::signature::PaddingAlg::RSA_PSS_SHA256,
        4 => pi_crypto::signature::PaddingAlg::RSA_PSS_SHA384,
        5 => pi_crypto::signature::PaddingAlg::RSA_PSS_SHA512,
        _ => panic!("enum type error")
    };


	let jst2 = &v[2];
	if !jst2.is_uint8_array() && !jst2.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst2 = jst2.to_bytes();



    let result = pi_crypto::signature::Rsa::sign(jst0,jst1,jst2);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,104530634);


    Some(CallResult::Ok)
}


fn call_901229592(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in verify";

	let jst0 = &v[0];
    if !jst0.is_number(){return Some(CallResult::Err(String::from(param_error)));}
    let jst0 = match jst0.get_u32(){
        0 => pi_crypto::signature::PaddingAlg::RSA_PKCS1_SHA256,
        1 => pi_crypto::signature::PaddingAlg::RSA_PKCS1_SHA384,
        2 => pi_crypto::signature::PaddingAlg::RSA_PKCS1_SHA512,
        3 => pi_crypto::signature::PaddingAlg::RSA_PSS_SHA256,
        4 => pi_crypto::signature::PaddingAlg::RSA_PSS_SHA384,
        5 => pi_crypto::signature::PaddingAlg::RSA_PSS_SHA512,
        _ => panic!("enum type error")
    };


	let jst1 = &v[1];
	if !jst1.is_uint8_array() && !jst1.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst1 = jst1.to_bytes();



	let jst2 = &v[2];
	if !jst2.is_uint8_array() && !jst2.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst2 = jst2.to_bytes();



	let jst3 = &v[3];
	if !jst3.is_uint8_array() && !jst3.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst3 = jst3.to_bytes();



    let result = pi_crypto::signature::Rsa::verify(jst0,jst1,jst2,jst3);let mut result = js.new_boolean(result);

    Some(CallResult::Ok)
}


fn call_2878910515(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in alipay_verify";

	let jst0 = &v[0];
	if !jst0.is_uint8_array() && !jst0.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst0 = jst0.to_bytes();



	let jst1 = &v[1];
	if !jst1.is_uint8_array() && !jst1.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst1 = jst1.to_bytes();



	let jst2 = &v[2];
	if !jst2.is_uint8_array() && !jst2.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst2 = jst2.to_bytes();



    let result = pi_crypto::signature::Rsa::alipay_verify(jst0,jst1,jst2);let mut result = js.new_boolean(result);

    Some(CallResult::Ok)
}


fn call_2041863833(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in generate_pkcs8";

	let jst0 = &v[0];
    if !jst0.is_number(){return Some(CallResult::Err(String::from(param_error)));}
    let jst0 = match jst0.get_u32(){
        0 => pi_crypto::signature::EcdsaAlg::ECDSA_P256_SHA256_ASN1,
        1 => pi_crypto::signature::EcdsaAlg::ECDSA_P384_SHA384_ASN1,
        _ => panic!("enum type error")
    };


    let result = pi_crypto::signature::EcdsaKeyPair::generate_pkcs8(jst0);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,104530634);


    Some(CallResult::Ok)
}


fn call_3809028580(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in from_private_key_and_public_key";

	let jst0 = &v[0];
    if !jst0.is_number(){return Some(CallResult::Err(String::from(param_error)));}
    let jst0 = match jst0.get_u32(){
        0 => pi_crypto::signature::EcdsaAlg::ECDSA_P256_SHA256_ASN1,
        1 => pi_crypto::signature::EcdsaAlg::ECDSA_P384_SHA384_ASN1,
        _ => panic!("enum type error")
    };


	let jst1 = &v[1];
	if !jst1.is_uint8_array() && !jst1.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst1 = jst1.to_bytes();



	let jst2 = &v[2];
	if !jst2.is_uint8_array() && !jst2.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst2 = jst2.to_bytes();



    let result = pi_crypto::signature::EcdsaKeyPair::from_private_key_and_public_key(jst0,jst1,jst2);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,1190264576);


    Some(CallResult::Ok)
}


fn call_3841573761(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in from_pkcs8";

	let jst0 = &v[0];
    if !jst0.is_number(){return Some(CallResult::Err(String::from(param_error)));}
    let jst0 = match jst0.get_u32(){
        0 => pi_crypto::signature::EcdsaAlg::ECDSA_P256_SHA256_ASN1,
        1 => pi_crypto::signature::EcdsaAlg::ECDSA_P384_SHA384_ASN1,
        _ => panic!("enum type error")
    };


	let jst1 = &v[1];
	if !jst1.is_uint8_array() && !jst1.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst1 = jst1.to_bytes();



    let result = pi_crypto::signature::EcdsaKeyPair::from_pkcs8(jst0,jst1);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,1190264576);


    Some(CallResult::Ok)
}


fn call_1413668685(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in sign";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 1190264576, false, param_error).expect("");
	let jst0 = unsafe { &*(ptr as *const pi_crypto::signature::EcdsaKeyPair) };


	let jst1 = &v[1];
	if !jst1.is_uint8_array() && !jst1.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst1 = jst1.to_bytes();



    let result = pi_crypto::signature::EcdsaKeyPair::sign(jst0,jst1);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,104530634);


    Some(CallResult::Ok)
}


fn call_1006720087(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in public_key";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 1190264576, false, param_error).expect("");
	let jst0 = unsafe { &*(ptr as *const pi_crypto::signature::EcdsaKeyPair) };


    let result = pi_crypto::signature::EcdsaKeyPair::public_key(jst0);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,104530634);


    Some(CallResult::Ok)
}


fn call_2103960165(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in ecdsa_verify";

	let jst0 = &v[0];
    if !jst0.is_number(){return Some(CallResult::Err(String::from(param_error)));}
    let jst0 = match jst0.get_u32(){
        0 => pi_crypto::signature::EcdsaAlg::ECDSA_P256_SHA256_ASN1,
        1 => pi_crypto::signature::EcdsaAlg::ECDSA_P384_SHA384_ASN1,
        _ => panic!("enum type error")
    };


	let jst1 = &v[1];
	if !jst1.is_uint8_array() && !jst1.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst1 = jst1.to_bytes();



	let jst2 = &v[2];
	if !jst2.is_uint8_array() && !jst2.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst2 = jst2.to_bytes();



	let jst3 = &v[3];
	if !jst3.is_uint8_array() && !jst3.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst3 = jst3.to_bytes();



    let result = pi_crypto::signature::ecdsa_verify(jst0,jst1,jst2,jst3);let mut result = js.new_boolean(result);

    Some(CallResult::Ok)
}


fn call_3851862966(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in genSecureRandBytes";

	let jst0 = &v[0];
	if !jst0.is_number(){ return Some(CallResult::Err(String::from(param_error)));}
	let jst0 = jst0.get_u32() as usize;


    let result = pi_crypto::random::genSecureRandBytes(jst0);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,104530634);


    Some(CallResult::Ok)
}


fn call_1252421489(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in new";

	let jst0 = &v[0];
	if !jst0.is_number(){ return Some(CallResult::Err(String::from(param_error)));}
	let jst0 = jst0.get_u32() as usize;


    let result = pi_crypto::bls::BlsIdVec::new(jst0);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,1875205449);


    Some(CallResult::Ok)
}


fn call_2592527877(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in new";

	let jst0 = &v[0];
	if !jst0.is_number(){ return Some(CallResult::Err(String::from(param_error)));}
	let jst0 = jst0.get_u32() as usize;


    let result = pi_crypto::bls::BlsSecKeyVec::new(jst0);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,2934268916);


    Some(CallResult::Ok)
}


fn call_3404883075(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in new";

	let jst0 = &v[0];
	if !jst0.is_number(){ return Some(CallResult::Err(String::from(param_error)));}
	let jst0 = jst0.get_u32() as usize;


    let result = pi_crypto::bls::BlsPubKeyVec::new(jst0);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,3840517932);


    Some(CallResult::Ok)
}


fn call_2903230657(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in new";

	let jst0 = &v[0];
	if !jst0.is_number(){ return Some(CallResult::Err(String::from(param_error)));}
	let jst0 = jst0.get_u32() as usize;


    let result = pi_crypto::bls::BlsSigVec::new(jst0);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,4060246115);


    Some(CallResult::Ok)
}


fn call_2498464569(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_init";

	let jst0 = &v[0];
    if !jst0.is_number(){return Some(CallResult::Err(String::from(param_error)));}
    let jst0 = match jst0.get_u32(){
        0 => pi_crypto::bls::Curve::MclBnCurveFp254BNb,
        1 => pi_crypto::bls::Curve::MclBnCurveFp382_1,
        2 => pi_crypto::bls::Curve::MclBnCurveFp382_2,
        3 => pi_crypto::bls::Curve::MclBnCurveFp462,
        4 => pi_crypto::bls::Curve::MclBnCurveSNARK1,
        5 => pi_crypto::bls::Curve::MclBls12CurveFp381,
        _ => panic!("enum type error")
    };


    let result = pi_crypto::bls::bls_init(jst0);let mut result = js.new_boolean(result);

    Some(CallResult::Ok)
}


fn call_1295262082(js: Arc<JS>) -> Option<CallResult>{

    let result = pi_crypto::bls::bls_get_op_unit_size();let mut result = js.new_u32(result as u32);

    Some(CallResult::Ok)
}


fn call_2496411899(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_get_curve_order";

	let jst0 = &v[0];
	if !jst0.is_number(){ return Some(CallResult::Err(String::from(param_error)));}
	let jst0 = jst0.get_u32() as usize;


    let result = pi_crypto::bls::bls_get_curve_order(jst0);let mut result = match result{
        Some(v) => { let mut v = js.new_str(v).unwrap();
 v}
        None => js.new_null()
    };

    Some(CallResult::Ok)
}


fn call_755737870(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_get_field_order";

	let jst0 = &v[0];
	if !jst0.is_number(){ return Some(CallResult::Err(String::from(param_error)));}
	let jst0 = jst0.get_u32() as usize;


    let result = pi_crypto::bls::bls_get_field_order(jst0);let mut result = match result{
        Some(v) => { let mut v = js.new_str(v).unwrap();
 v}
        None => js.new_null()
    };

    Some(CallResult::Ok)
}


fn call_3253072797(js: Arc<JS>) -> Option<CallResult>{

    let result = pi_crypto::bls::bls_get_generator_of_g2();
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,1617625763);


    Some(CallResult::Ok)
}


fn call_4280890483(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_id_set_int";

	let jst0 = &v[0];
	if !jst0.is_number(){ return Some(CallResult::Err(String::from(param_error)));}
	let jst0 = jst0.get_i32();


    let result = pi_crypto::bls::bls_id_set_int(jst0);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,3094164306);


    Some(CallResult::Ok)
}


fn call_2402380511(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_id_set_dec_str";

	let jst0 = &v[0];
	if !jst0.is_string(){ return Some(CallResult::Err(String::from(param_error)));}
    let jst0 = jst0.get_str();


    let result = pi_crypto::bls::bls_id_set_dec_str(jst0);let mut result = match result{
        Some(v) => { 
    let ptr = Box::into_raw(Box::new(v)) as usize;let mut v = ptr_jstype(js.get_objs(), js.clone(), ptr,3094164306);

 v}
        None => js.new_null()
    };

    Some(CallResult::Ok)
}


fn call_2426850537(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_id_set_hex_str";

	let jst0 = &v[0];
	if !jst0.is_string(){ return Some(CallResult::Err(String::from(param_error)));}
    let jst0 = jst0.get_str();


    let result = pi_crypto::bls::bls_id_set_hex_str(jst0);let mut result = match result{
        Some(v) => { 
    let ptr = Box::into_raw(Box::new(v)) as usize;let mut v = ptr_jstype(js.get_objs(), js.clone(), ptr,3094164306);

 v}
        None => js.new_null()
    };

    Some(CallResult::Ok)
}


fn call_3075954650(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_id_get_dec_str";

	let jst0 = &v[0];
	if !jst0.is_number(){ return Some(CallResult::Err(String::from(param_error)));}
	let jst0 = jst0.get_u32() as usize;


	let jst1 = &v[1];
    let ptr = jstype_ptr(&jst1, js.clone(), 3094164306, false, param_error).expect("");
	let jst1 = unsafe { &*(ptr as *const pi_crypto::bls::BlsId) };


    let result = pi_crypto::bls::bls_id_get_dec_str(jst0,jst1);let mut result = match result{
        Some(v) => { let mut v = js.new_str(v).unwrap();
 v}
        None => js.new_null()
    };

    Some(CallResult::Ok)
}


fn call_3801863647(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_id_get_hex_str";

	let jst0 = &v[0];
	if !jst0.is_number(){ return Some(CallResult::Err(String::from(param_error)));}
	let jst0 = jst0.get_u32() as usize;


	let jst1 = &v[1];
    let ptr = jstype_ptr(&jst1, js.clone(), 3094164306, false, param_error).expect("");
	let jst1 = unsafe { &*(ptr as *const pi_crypto::bls::BlsId) };


    let result = pi_crypto::bls::bls_id_get_hex_str(jst0,jst1);let mut result = match result{
        Some(v) => { let mut v = js.new_str(v).unwrap();
 v}
        None => js.new_null()
    };

    Some(CallResult::Ok)
}


fn call_1719604587(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_hash_to_secret_key";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 104530634, true, param_error).expect("");
	let jst0 = *unsafe { Box::from_raw(ptr as *mut Vec<u8>) };


    let result = pi_crypto::bls::bls_hash_to_secret_key(jst0);let mut result = match result{
        Some(v) => { 
    let ptr = Box::into_raw(Box::new(v)) as usize;let mut v = ptr_jstype(js.get_objs(), js.clone(), ptr,187111440);

 v}
        None => js.new_null()
    };

    Some(CallResult::Ok)
}


fn call_3025531400(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_get_public_key";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 187111440, false, param_error).expect("");
	let jst0 = unsafe { &*(ptr as *const pi_crypto::bls::BlsSecretKey) };


    let result = pi_crypto::bls::bls_get_public_key(jst0);let mut result = match result{
        Some(v) => { 
    let ptr = Box::into_raw(Box::new(v)) as usize;let mut v = ptr_jstype(js.get_objs(), js.clone(), ptr,1617625763);

 v}
        None => js.new_null()
    };

    Some(CallResult::Ok)
}


fn call_3723291352(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_get_pop";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 187111440, false, param_error).expect("");
	let jst0 = unsafe { &*(ptr as *const pi_crypto::bls::BlsSecretKey) };


    let result = pi_crypto::bls::bls_get_pop(jst0);let mut result = match result{
        Some(v) => { 
    let ptr = Box::into_raw(Box::new(v)) as usize;let mut v = ptr_jstype(js.get_objs(), js.clone(), ptr,3966088300);

 v}
        None => js.new_null()
    };

    Some(CallResult::Ok)
}


fn call_1669774542(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_verify_pop";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 3966088300, false, param_error).expect("");
	let jst0 = unsafe { &*(ptr as *const pi_crypto::bls::BlsSignature) };


	let jst1 = &v[1];
    let ptr = jstype_ptr(&jst1, js.clone(), 1617625763, false, param_error).expect("");
	let jst1 = unsafe { &*(ptr as *const pi_crypto::bls::BlsPublicKey) };


    let result = pi_crypto::bls::bls_verify_pop(jst0,jst1);let mut result = js.new_boolean(result);

    Some(CallResult::Ok)
}


fn call_1235807017(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_id_serialize";

	let jst0 = &v[0];
	if !jst0.is_number(){ return Some(CallResult::Err(String::from(param_error)));}
	let jst0 = jst0.get_u32() as usize;


	let jst1 = &v[1];
    let ptr = jstype_ptr(&jst1, js.clone(), 3094164306, false, param_error).expect("");
	let jst1 = unsafe { &*(ptr as *const pi_crypto::bls::BlsId) };


    let result = pi_crypto::bls::bls_id_serialize(jst0,jst1);let mut result = match result{
        Some(v) => { 
    let ptr = Box::into_raw(Box::new(v)) as usize;let mut v = ptr_jstype(js.get_objs(), js.clone(), ptr,104530634);

 v}
        None => js.new_null()
    };

    Some(CallResult::Ok)
}


fn call_3671848448(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_secret_key_serialize";

	let jst0 = &v[0];
	if !jst0.is_number(){ return Some(CallResult::Err(String::from(param_error)));}
	let jst0 = jst0.get_u32() as usize;


	let jst1 = &v[1];
    let ptr = jstype_ptr(&jst1, js.clone(), 187111440, false, param_error).expect("");
	let jst1 = unsafe { &*(ptr as *const pi_crypto::bls::BlsSecretKey) };


    let result = pi_crypto::bls::bls_secret_key_serialize(jst0,jst1);let mut result = match result{
        Some(v) => { 
    let ptr = Box::into_raw(Box::new(v)) as usize;let mut v = ptr_jstype(js.get_objs(), js.clone(), ptr,104530634);

 v}
        None => js.new_null()
    };

    Some(CallResult::Ok)
}


fn call_1900424700(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_public_key_serialize";

	let jst0 = &v[0];
	if !jst0.is_number(){ return Some(CallResult::Err(String::from(param_error)));}
	let jst0 = jst0.get_u32() as usize;


	let jst1 = &v[1];
    let ptr = jstype_ptr(&jst1, js.clone(), 1617625763, false, param_error).expect("");
	let jst1 = unsafe { &*(ptr as *const pi_crypto::bls::BlsPublicKey) };


    let result = pi_crypto::bls::bls_public_key_serialize(jst0,jst1);let mut result = match result{
        Some(v) => { 
    let ptr = Box::into_raw(Box::new(v)) as usize;let mut v = ptr_jstype(js.get_objs(), js.clone(), ptr,104530634);

 v}
        None => js.new_null()
    };

    Some(CallResult::Ok)
}


fn call_2045530324(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_signature_serialize";

	let jst0 = &v[0];
	if !jst0.is_number(){ return Some(CallResult::Err(String::from(param_error)));}
	let jst0 = jst0.get_u32() as usize;


	let jst1 = &v[1];
    let ptr = jstype_ptr(&jst1, js.clone(), 3966088300, false, param_error).expect("");
	let jst1 = unsafe { &*(ptr as *const pi_crypto::bls::BlsSignature) };


    let result = pi_crypto::bls::bls_signature_serialize(jst0,jst1);let mut result = match result{
        Some(v) => { 
    let ptr = Box::into_raw(Box::new(v)) as usize;let mut v = ptr_jstype(js.get_objs(), js.clone(), ptr,104530634);

 v}
        None => js.new_null()
    };

    Some(CallResult::Ok)
}


fn call_298607248(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_id_deserialize";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 104530634, true, param_error).expect("");
	let jst0 = *unsafe { Box::from_raw(ptr as *mut Vec<u8>) };


    let result = pi_crypto::bls::bls_id_deserialize(jst0);let mut result = match result{
        Some(v) => { 
    let ptr = Box::into_raw(Box::new(v)) as usize;let mut v = ptr_jstype(js.get_objs(), js.clone(), ptr,3094164306);

 v}
        None => js.new_null()
    };

    Some(CallResult::Ok)
}


fn call_2029782143(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_secret_key_deserialize";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 104530634, true, param_error).expect("");
	let jst0 = *unsafe { Box::from_raw(ptr as *mut Vec<u8>) };


    let result = pi_crypto::bls::bls_secret_key_deserialize(jst0);let mut result = match result{
        Some(v) => { 
    let ptr = Box::into_raw(Box::new(v)) as usize;let mut v = ptr_jstype(js.get_objs(), js.clone(), ptr,187111440);

 v}
        None => js.new_null()
    };

    Some(CallResult::Ok)
}


fn call_1922268706(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_public_key_deserialize";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 104530634, true, param_error).expect("");
	let jst0 = *unsafe { Box::from_raw(ptr as *mut Vec<u8>) };


    let result = pi_crypto::bls::bls_public_key_deserialize(jst0);let mut result = match result{
        Some(v) => { 
    let ptr = Box::into_raw(Box::new(v)) as usize;let mut v = ptr_jstype(js.get_objs(), js.clone(), ptr,1617625763);

 v}
        None => js.new_null()
    };

    Some(CallResult::Ok)
}


fn call_760927771(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_signature_deserialize";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 104530634, true, param_error).expect("");
	let jst0 = *unsafe { Box::from_raw(ptr as *mut Vec<u8>) };


    let result = pi_crypto::bls::bls_signature_deserialize(jst0);let mut result = match result{
        Some(v) => { 
    let ptr = Box::into_raw(Box::new(v)) as usize;let mut v = ptr_jstype(js.get_objs(), js.clone(), ptr,3966088300);

 v}
        None => js.new_null()
    };

    Some(CallResult::Ok)
}


fn call_1304117942(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_id_is_equal";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 3094164306, false, param_error).expect("");
	let jst0 = unsafe { &*(ptr as *const pi_crypto::bls::BlsId) };


	let jst1 = &v[1];
    let ptr = jstype_ptr(&jst1, js.clone(), 3094164306, false, param_error).expect("");
	let jst1 = unsafe { &*(ptr as *const pi_crypto::bls::BlsId) };


    let result = pi_crypto::bls::bls_id_is_equal(jst0,jst1);let mut result = js.new_boolean(result);

    Some(CallResult::Ok)
}


fn call_1202562609(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_secret_key_is_equal";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 187111440, false, param_error).expect("");
	let jst0 = unsafe { &*(ptr as *const pi_crypto::bls::BlsSecretKey) };


	let jst1 = &v[1];
    let ptr = jstype_ptr(&jst1, js.clone(), 187111440, false, param_error).expect("");
	let jst1 = unsafe { &*(ptr as *const pi_crypto::bls::BlsSecretKey) };


    let result = pi_crypto::bls::bls_secret_key_is_equal(jst0,jst1);let mut result = js.new_boolean(result);

    Some(CallResult::Ok)
}


fn call_1494397139(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_public_key_is_equal";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 1617625763, false, param_error).expect("");
	let jst0 = unsafe { &*(ptr as *const pi_crypto::bls::BlsPublicKey) };


	let jst1 = &v[1];
    let ptr = jstype_ptr(&jst1, js.clone(), 1617625763, false, param_error).expect("");
	let jst1 = unsafe { &*(ptr as *const pi_crypto::bls::BlsPublicKey) };


    let result = pi_crypto::bls::bls_public_key_is_equal(jst0,jst1);let mut result = js.new_boolean(result);

    Some(CallResult::Ok)
}


fn call_1251457612(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_signature_is_equal";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 3966088300, false, param_error).expect("");
	let jst0 = unsafe { &*(ptr as *const pi_crypto::bls::BlsSignature) };


	let jst1 = &v[1];
    let ptr = jstype_ptr(&jst1, js.clone(), 3966088300, false, param_error).expect("");
	let jst1 = unsafe { &*(ptr as *const pi_crypto::bls::BlsSignature) };


    let result = pi_crypto::bls::bls_signature_is_equal(jst0,jst1);let mut result = js.new_boolean(result);

    Some(CallResult::Ok)
}


fn call_863200741(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_secret_key_add";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 187111440, false, param_error).expect("");
	let jst0 = unsafe { &*(ptr as *const pi_crypto::bls::BlsSecretKey) };


	let jst1 = &v[1];
    let ptr = jstype_ptr(&jst1, js.clone(), 187111440, false, param_error).expect("");
	let jst1 = unsafe { &*(ptr as *const pi_crypto::bls::BlsSecretKey) };


    pi_crypto::bls::bls_secret_key_add(jst0,jst1);
    Some(CallResult::Ok)
}


fn call_3082139465(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_public_key_add";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 1617625763, false, param_error).expect("");
	let jst0 = unsafe { &*(ptr as *const pi_crypto::bls::BlsPublicKey) };


	let jst1 = &v[1];
    let ptr = jstype_ptr(&jst1, js.clone(), 1617625763, false, param_error).expect("");
	let jst1 = unsafe { &*(ptr as *const pi_crypto::bls::BlsPublicKey) };


    pi_crypto::bls::bls_public_key_add(jst0,jst1);
    Some(CallResult::Ok)
}


fn call_3576086575(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_signature_add";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 3966088300, false, param_error).expect("");
	let jst0 = unsafe { &*(ptr as *const pi_crypto::bls::BlsSignature) };


	let jst1 = &v[1];
    let ptr = jstype_ptr(&jst1, js.clone(), 3966088300, false, param_error).expect("");
	let jst1 = unsafe { &*(ptr as *const pi_crypto::bls::BlsSignature) };


    pi_crypto::bls::bls_signature_add(jst0,jst1);
    Some(CallResult::Ok)
}


fn call_3750445483(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_secret_key_share";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 187111440, false, param_error).expect("");
	let jst0 = unsafe { &*(ptr as *const pi_crypto::bls::BlsSecretKey) };


	let jst1 = &v[1];
	if !jst1.is_number(){ return Some(CallResult::Err(String::from(param_error)));}
	let jst1 = jst1.get_u32() as usize;


	let jst2 = &v[2];
    let ptr = jstype_ptr(&jst2, js.clone(), 3094164306, false, param_error).expect("");
	let jst2 = unsafe { &*(ptr as *const pi_crypto::bls::BlsId) };


    let result = pi_crypto::bls::bls_secret_key_share(jst0,jst1,jst2);let mut result = match result{
        Some(v) => { 
    let ptr = Box::into_raw(Box::new(v)) as usize;let mut v = ptr_jstype(js.get_objs(), js.clone(), ptr,187111440);

 v}
        None => js.new_null()
    };

    Some(CallResult::Ok)
}


fn call_3551222567(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_public_key_share";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 1617625763, false, param_error).expect("");
	let jst0 = unsafe { &*(ptr as *const pi_crypto::bls::BlsPublicKey) };


	let jst1 = &v[1];
	if !jst1.is_number(){ return Some(CallResult::Err(String::from(param_error)));}
	let jst1 = jst1.get_u32() as usize;


	let jst2 = &v[2];
    let ptr = jstype_ptr(&jst2, js.clone(), 3094164306, false, param_error).expect("");
	let jst2 = unsafe { &*(ptr as *const pi_crypto::bls::BlsId) };


    let result = pi_crypto::bls::bls_public_key_share(jst0,jst1,jst2);let mut result = match result{
        Some(v) => { 
    let ptr = Box::into_raw(Box::new(v)) as usize;let mut v = ptr_jstype(js.get_objs(), js.clone(), ptr,1617625763);

 v}
        None => js.new_null()
    };

    Some(CallResult::Ok)
}


fn call_4217857181(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_get_id_from_vec";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 1875205449, false, param_error).expect("");
	let jst0 = unsafe { &*(ptr as *const pi_crypto::bls::BlsIdVec) };


	let jst1 = &v[1];
	if !jst1.is_number(){ return Some(CallResult::Err(String::from(param_error)));}
	let jst1 = jst1.get_u32() as usize;


    let result = pi_crypto::bls::bls_get_id_from_vec(jst0,jst1);let mut result = match result{
        Some(v) => { 
    let ptr = Box::into_raw(Box::new(v)) as usize;let mut v = ptr_jstype(js.get_objs(), js.clone(), ptr,3094164306);

 v}
        None => js.new_null()
    };

    Some(CallResult::Ok)
}


fn call_3778283533(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_add_id_to_vec";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 1875205449, false, param_error).expect("");
	let jst0 = unsafe { &mut *(ptr as *mut pi_crypto::bls::BlsIdVec) };


	let jst1 = &v[1];
    let ptr = jstype_ptr(&jst1, js.clone(), 3094164306, false, param_error).expect("");
	let jst1 = unsafe { &*(ptr as *const pi_crypto::bls::BlsId) };


    pi_crypto::bls::bls_add_id_to_vec(jst0,jst1);
    Some(CallResult::Ok)
}


fn call_1905417019(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_get_secret_key_from_vec";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 2934268916, false, param_error).expect("");
	let jst0 = unsafe { &*(ptr as *const pi_crypto::bls::BlsSecKeyVec) };


	let jst1 = &v[1];
	if !jst1.is_number(){ return Some(CallResult::Err(String::from(param_error)));}
	let jst1 = jst1.get_u32() as usize;


    let result = pi_crypto::bls::bls_get_secret_key_from_vec(jst0,jst1);let mut result = match result{
        Some(v) => { 
    let ptr = Box::into_raw(Box::new(v)) as usize;let mut v = ptr_jstype(js.get_objs(), js.clone(), ptr,187111440);

 v}
        None => js.new_null()
    };

    Some(CallResult::Ok)
}


fn call_2172313629(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_add_secret_key_to_vec";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 2934268916, false, param_error).expect("");
	let jst0 = unsafe { &mut *(ptr as *mut pi_crypto::bls::BlsSecKeyVec) };


	let jst1 = &v[1];
    let ptr = jstype_ptr(&jst1, js.clone(), 187111440, false, param_error).expect("");
	let jst1 = unsafe { &*(ptr as *const pi_crypto::bls::BlsSecretKey) };


    pi_crypto::bls::bls_add_secret_key_to_vec(jst0,jst1);
    Some(CallResult::Ok)
}


fn call_2861556416(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_get_secret_key_vec";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 2934268916, false, param_error).expect("");
	let jst0 = unsafe { &*(ptr as *const pi_crypto::bls::BlsSecKeyVec) };


    let result = pi_crypto::bls::bls_get_secret_key_vec(jst0);let mut result = match result{
        Some(v) => { 
    let ptr = Box::into_raw(Box::new(v)) as usize;let mut v = ptr_jstype(js.get_objs(), js.clone(), ptr,187111440);

 v}
        None => js.new_null()
    };

    Some(CallResult::Ok)
}


fn call_4054179525(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_get_public_key_from_vec";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 3840517932, false, param_error).expect("");
	let jst0 = unsafe { &*(ptr as *const pi_crypto::bls::BlsPubKeyVec) };


	let jst1 = &v[1];
	if !jst1.is_number(){ return Some(CallResult::Err(String::from(param_error)));}
	let jst1 = jst1.get_u32() as usize;


    let result = pi_crypto::bls::bls_get_public_key_from_vec(jst0,jst1);let mut result = match result{
        Some(v) => { 
    let ptr = Box::into_raw(Box::new(v)) as usize;let mut v = ptr_jstype(js.get_objs(), js.clone(), ptr,1617625763);

 v}
        None => js.new_null()
    };

    Some(CallResult::Ok)
}


fn call_3718730423(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_add_public_key_to_vec";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 3840517932, false, param_error).expect("");
	let jst0 = unsafe { &mut *(ptr as *mut pi_crypto::bls::BlsPubKeyVec) };


	let jst1 = &v[1];
    let ptr = jstype_ptr(&jst1, js.clone(), 1617625763, false, param_error).expect("");
	let jst1 = unsafe { &*(ptr as *const pi_crypto::bls::BlsPublicKey) };


    pi_crypto::bls::bls_add_public_key_to_vec(jst0,jst1);
    Some(CallResult::Ok)
}


fn call_2864459653(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_get_public_key_vec";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 3840517932, false, param_error).expect("");
	let jst0 = unsafe { &*(ptr as *const pi_crypto::bls::BlsPubKeyVec) };


    let result = pi_crypto::bls::bls_get_public_key_vec(jst0);let mut result = match result{
        Some(v) => { 
    let ptr = Box::into_raw(Box::new(v)) as usize;let mut v = ptr_jstype(js.get_objs(), js.clone(), ptr,1617625763);

 v}
        None => js.new_null()
    };

    Some(CallResult::Ok)
}


fn call_376820189(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_get_signature_from_vec";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 4060246115, false, param_error).expect("");
	let jst0 = unsafe { &*(ptr as *const pi_crypto::bls::BlsSigVec) };


	let jst1 = &v[1];
	if !jst1.is_number(){ return Some(CallResult::Err(String::from(param_error)));}
	let jst1 = jst1.get_u32() as usize;


    let result = pi_crypto::bls::bls_get_signature_from_vec(jst0,jst1);let mut result = match result{
        Some(v) => { 
    let ptr = Box::into_raw(Box::new(v)) as usize;let mut v = ptr_jstype(js.get_objs(), js.clone(), ptr,3966088300);

 v}
        None => js.new_null()
    };

    Some(CallResult::Ok)
}


fn call_263952757(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_add_signature_to_vec";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 4060246115, false, param_error).expect("");
	let jst0 = unsafe { &mut *(ptr as *mut pi_crypto::bls::BlsSigVec) };


	let jst1 = &v[1];
    let ptr = jstype_ptr(&jst1, js.clone(), 3966088300, false, param_error).expect("");
	let jst1 = unsafe { &*(ptr as *const pi_crypto::bls::BlsSignature) };


    pi_crypto::bls::bls_add_signature_to_vec(jst0,jst1);
    Some(CallResult::Ok)
}


fn call_2039602097(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_get_signature_key_vec";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 4060246115, false, param_error).expect("");
	let jst0 = unsafe { &*(ptr as *const pi_crypto::bls::BlsSigVec) };


    let result = pi_crypto::bls::bls_get_signature_key_vec(jst0);let mut result = match result{
        Some(v) => { 
    let ptr = Box::into_raw(Box::new(v)) as usize;let mut v = ptr_jstype(js.get_objs(), js.clone(), ptr,3966088300);

 v}
        None => js.new_null()
    };

    Some(CallResult::Ok)
}


fn call_1087017908(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_secret_key_recover";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 2934268916, false, param_error).expect("");
	let jst0 = unsafe { &*(ptr as *const pi_crypto::bls::BlsSecKeyVec) };


	let jst1 = &v[1];
    let ptr = jstype_ptr(&jst1, js.clone(), 1875205449, false, param_error).expect("");
	let jst1 = unsafe { &*(ptr as *const pi_crypto::bls::BlsIdVec) };


	let jst2 = &v[2];
	if !jst2.is_number(){ return Some(CallResult::Err(String::from(param_error)));}
	let jst2 = jst2.get_u32() as usize;


    let result = pi_crypto::bls::bls_secret_key_recover(jst0,jst1,jst2);let mut result = match result{
        Some(v) => { 
    let ptr = Box::into_raw(Box::new(v)) as usize;let mut v = ptr_jstype(js.get_objs(), js.clone(), ptr,187111440);

 v}
        None => js.new_null()
    };

    Some(CallResult::Ok)
}


fn call_993477813(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_public_key_recover";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 3840517932, false, param_error).expect("");
	let jst0 = unsafe { &*(ptr as *const pi_crypto::bls::BlsPubKeyVec) };


	let jst1 = &v[1];
    let ptr = jstype_ptr(&jst1, js.clone(), 1875205449, false, param_error).expect("");
	let jst1 = unsafe { &*(ptr as *const pi_crypto::bls::BlsIdVec) };


	let jst2 = &v[2];
	if !jst2.is_number(){ return Some(CallResult::Err(String::from(param_error)));}
	let jst2 = jst2.get_u32() as usize;


    let result = pi_crypto::bls::bls_public_key_recover(jst0,jst1,jst2);let mut result = match result{
        Some(v) => { 
    let ptr = Box::into_raw(Box::new(v)) as usize;let mut v = ptr_jstype(js.get_objs(), js.clone(), ptr,1617625763);

 v}
        None => js.new_null()
    };

    Some(CallResult::Ok)
}


fn call_3587763353(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_signature_recover";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 4060246115, false, param_error).expect("");
	let jst0 = unsafe { &*(ptr as *const pi_crypto::bls::BlsSigVec) };


	let jst1 = &v[1];
    let ptr = jstype_ptr(&jst1, js.clone(), 1875205449, false, param_error).expect("");
	let jst1 = unsafe { &*(ptr as *const pi_crypto::bls::BlsIdVec) };


	let jst2 = &v[2];
	if !jst2.is_number(){ return Some(CallResult::Err(String::from(param_error)));}
	let jst2 = jst2.get_u32() as usize;


    let result = pi_crypto::bls::bls_signature_recover(jst0,jst1,jst2);let mut result = match result{
        Some(v) => { 
    let ptr = Box::into_raw(Box::new(v)) as usize;let mut v = ptr_jstype(js.get_objs(), js.clone(), ptr,3966088300);

 v}
        None => js.new_null()
    };

    Some(CallResult::Ok)
}


fn call_3188209906(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_sign";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 187111440, false, param_error).expect("");
	let jst0 = unsafe { &*(ptr as *const pi_crypto::bls::BlsSecretKey) };


	let jst1 = &v[1];
    let ptr = jstype_ptr(&jst1, js.clone(), 2886438122, true, param_error).expect("");
	let jst1 = *unsafe { Box::from_raw(ptr as *mut Arc<Vec<u8>>)}.clone();


    let result = pi_crypto::bls::bls_sign(jst0,jst1);let mut result = match result{
        Some(v) => { 
    let ptr = Box::into_raw(Box::new(v)) as usize;let mut v = ptr_jstype(js.get_objs(), js.clone(), ptr,3966088300);

 v}
        None => js.new_null()
    };

    Some(CallResult::Ok)
}


fn call_2084703123(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in bls_verify";

	let jst0 = &v[0];
    let ptr = jstype_ptr(&jst0, js.clone(), 3966088300, false, param_error).expect("");
	let jst0 = unsafe { &*(ptr as *const pi_crypto::bls::BlsSignature) };


	let jst1 = &v[1];
    let ptr = jstype_ptr(&jst1, js.clone(), 1617625763, false, param_error).expect("");
	let jst1 = unsafe { &*(ptr as *const pi_crypto::bls::BlsPublicKey) };


	let jst2 = &v[2];
    let ptr = jstype_ptr(&jst2, js.clone(), 2886438122, true, param_error).expect("");
	let jst2 = *unsafe { Box::from_raw(ptr as *mut Arc<Vec<u8>>)}.clone();


    let result = pi_crypto::bls::bls_verify(jst0,jst1,jst2);let mut result = js.new_boolean(result);

    Some(CallResult::Ok)
}


fn call_890057462(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in from_secret";

	let jst0 = &v[0];
	if !jst0.is_uint8_array() && !jst0.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst0 = jst0.to_bytes();



    let result = pi_crypto::jwt::SignKey::from_secret(jst0);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,4244548360);


    Some(CallResult::Ok)
}


fn call_1518526824(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in from_base64_secret";

	let jst0 = &v[0];
	if !jst0.is_string(){ return Some(CallResult::Err(String::from(param_error)));}
    let jst0 = &jst0.get_str();


    let result = pi_crypto::jwt::SignKey::from_base64_secret(jst0);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,4244548360);


    Some(CallResult::Ok)
}


fn call_985222615(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in from_rsa_pem";

	let jst0 = &v[0];
	if !jst0.is_uint8_array() && !jst0.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst0 = jst0.to_bytes();



    let result = pi_crypto::jwt::SignKey::from_rsa_pem(jst0);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,4244548360);


    Some(CallResult::Ok)
}


fn call_1277908099(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in from_rsa_der";

	let jst0 = &v[0];
	if !jst0.is_uint8_array() && !jst0.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst0 = jst0.to_bytes();



    let result = pi_crypto::jwt::SignKey::from_rsa_der(jst0);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,4244548360);


    Some(CallResult::Ok)
}


fn call_2975386969(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in from_ec_pem";

	let jst0 = &v[0];
	if !jst0.is_uint8_array() && !jst0.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst0 = jst0.to_bytes();



    let result = pi_crypto::jwt::SignKey::from_ec_pem(jst0);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,4244548360);


    Some(CallResult::Ok)
}


fn call_1021127516(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in from_ec_der";

	let jst0 = &v[0];
	if !jst0.is_uint8_array() && !jst0.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst0 = jst0.to_bytes();



    let result = pi_crypto::jwt::SignKey::from_ec_der(jst0);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,4244548360);


    Some(CallResult::Ok)
}


fn call_4078596132(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in from_secret";

	let jst0 = &v[0];
	if !jst0.is_uint8_array() && !jst0.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst0 = jst0.to_bytes();



    let result = pi_crypto::jwt::VerifyKey::from_secret(jst0);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,768518599);


    Some(CallResult::Ok)
}


fn call_1558442167(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in from_base64_secret";

	let jst0 = &v[0];
	if !jst0.is_string(){ return Some(CallResult::Err(String::from(param_error)));}
    let jst0 = &jst0.get_str();


    let result = pi_crypto::jwt::VerifyKey::from_base64_secret(jst0);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,768518599);


    Some(CallResult::Ok)
}


fn call_284043717(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in from_rsa_components";

	let jst0 = &v[0];
	if !jst0.is_string(){ return Some(CallResult::Err(String::from(param_error)));}
    let jst0 = &jst0.get_str();


	let jst1 = &v[1];
	if !jst1.is_string(){ return Some(CallResult::Err(String::from(param_error)));}
    let jst1 = &jst1.get_str();


    let result = pi_crypto::jwt::VerifyKey::from_rsa_components(jst0,jst1);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,768518599);


    Some(CallResult::Ok)
}


fn call_1162834309(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in from_rsa_pem";

	let jst0 = &v[0];
	if !jst0.is_uint8_array() && !jst0.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst0 = jst0.to_bytes();



    let result = pi_crypto::jwt::VerifyKey::from_rsa_pem(jst0);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,768518599);


    Some(CallResult::Ok)
}


fn call_1055468812(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in from_rsa_der";

	let jst0 = &v[0];
	if !jst0.is_uint8_array() && !jst0.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst0 = jst0.to_bytes();



    let result = pi_crypto::jwt::VerifyKey::from_rsa_der(jst0);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,768518599);


    Some(CallResult::Ok)
}


fn call_1163872385(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in from_ec_pem";

	let jst0 = &v[0];
	if !jst0.is_uint8_array() && !jst0.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst0 = jst0.to_bytes();



    let result = pi_crypto::jwt::VerifyKey::from_ec_pem(jst0);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,768518599);


    Some(CallResult::Ok)
}


fn call_3565773478(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in from_ec_der";

	let jst0 = &v[0];
	if !jst0.is_uint8_array() && !jst0.is_array_buffer(){return Some(CallResult::Err(String::from(param_error))); }
    let jst0 = jst0.to_bytes();



    let result = pi_crypto::jwt::VerifyKey::from_ec_der(jst0);
    let ptr = Box::into_raw(Box::new(result)) as usize;let mut result = ptr_jstype(js.get_objs(), js.clone(), ptr,768518599);


    Some(CallResult::Ok)
}


fn call_1219230175(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in jwt_sign";

	let jst0 = &v[0];
	if !jst0.is_string(){ return Some(CallResult::Err(String::from(param_error)));}
    let jst0 = &jst0.get_str();


	let jst1 = &v[1];
    let ptr = jstype_ptr(&jst1, js.clone(), 4244548360, true, param_error).expect("");
	let jst1 = *unsafe { Box::from_raw(ptr as *mut pi_crypto::jwt::SignKey) };


	let jst2 = &v[2];
    if !jst2.is_number(){return Some(CallResult::Err(String::from(param_error)));}
    let jst2 = match jst2.get_u32(){
        0 => pi_crypto::jwt::JwtAlg::HS256,
        1 => pi_crypto::jwt::JwtAlg::HS384,
        2 => pi_crypto::jwt::JwtAlg::HS512,
        3 => pi_crypto::jwt::JwtAlg::ES256,
        4 => pi_crypto::jwt::JwtAlg::ES384,
        5 => pi_crypto::jwt::JwtAlg::RS256,
        6 => pi_crypto::jwt::JwtAlg::RS384,
        7 => pi_crypto::jwt::JwtAlg::RS512,
        8 => pi_crypto::jwt::JwtAlg::PS256,
        9 => pi_crypto::jwt::JwtAlg::PS384,
        10 => pi_crypto::jwt::JwtAlg::PS512,
        _ => panic!("enum type error")
    };


    let result = pi_crypto::jwt::jwt_sign(jst0,jst1,jst2);let mut result = js.new_str(result).unwrap();

    Some(CallResult::Ok)
}


fn call_2615574485(js: Arc<JS>, v:Vec<JSType>) -> Option<CallResult>{
	let param_error = "param error in jwt_verify";

	let jst0 = &v[0];
	if !jst0.is_string(){ return Some(CallResult::Err(String::from(param_error)));}
    let jst0 = &jst0.get_str();


	let jst1 = &v[1];
	if !jst1.is_string(){ return Some(CallResult::Err(String::from(param_error)));}
    let jst1 = &jst1.get_str();


	let jst2 = &v[2];
    let ptr = jstype_ptr(&jst2, js.clone(), 768518599, false, param_error).expect("");
	let jst2 = unsafe { &*(ptr as *const pi_crypto::jwt::VerifyKey) };


	let jst3 = &v[3];
    if !jst3.is_number(){return Some(CallResult::Err(String::from(param_error)));}
    let jst3 = match jst3.get_u32(){
        0 => pi_crypto::jwt::JwtAlg::HS256,
        1 => pi_crypto::jwt::JwtAlg::HS384,
        2 => pi_crypto::jwt::JwtAlg::HS512,
        3 => pi_crypto::jwt::JwtAlg::ES256,
        4 => pi_crypto::jwt::JwtAlg::ES384,
        5 => pi_crypto::jwt::JwtAlg::RS256,
        6 => pi_crypto::jwt::JwtAlg::RS384,
        7 => pi_crypto::jwt::JwtAlg::RS512,
        8 => pi_crypto::jwt::JwtAlg::PS256,
        9 => pi_crypto::jwt::JwtAlg::PS384,
        10 => pi_crypto::jwt::JwtAlg::PS512,
        _ => panic!("enum type error")
    };


    let result = pi_crypto::jwt::jwt_verify(jst0,jst1,jst2,jst3);let mut result = js.new_boolean(result);

    Some(CallResult::Ok)
}

fn drop_1035403249(ptr: usize){
    unsafe { Box::from_raw(ptr as *mut hash_value::H256) };
}

fn drop_3223866506(ptr: usize){
    unsafe { Box::from_raw(ptr as *mut hash_value::H512) };
}

fn drop_4116332428(ptr: usize){
    unsafe { Box::from_raw(ptr as *mut pi_crypto::digest::DigestAlgorithm) };
}

fn drop_104530634(ptr: usize){
    unsafe { Box::from_raw(ptr as *mut Vec<u8>) };
}

fn drop_3042099183(ptr: usize){
    unsafe { Box::from_raw(ptr as *mut pi_crypto::hmac::DigestAlgorithm) };
}

fn drop_3761116463(ptr: usize){
    unsafe { Box::from_raw(ptr as *mut pi_crypto::signature::ECDSASecp256k1) };
}

fn drop_1545795468(ptr: usize){
    unsafe { Box::from_raw(ptr as *mut pi_crypto::signature::Rsa) };
}

fn drop_142204220(ptr: usize){
    unsafe { Box::from_raw(ptr as *mut pi_crypto::signature::PaddingAlg) };
}

fn drop_3559558627(ptr: usize){
    unsafe { Box::from_raw(ptr as *mut pi_crypto::signature::EcdsaAlg) };
}

fn drop_1190264576(ptr: usize){
    unsafe { Box::from_raw(ptr as *mut pi_crypto::signature::EcdsaKeyPair) };
}

fn drop_1875205449(ptr: usize){
    unsafe { Box::from_raw(ptr as *mut pi_crypto::bls::BlsIdVec) };
}

fn drop_2934268916(ptr: usize){
    unsafe { Box::from_raw(ptr as *mut pi_crypto::bls::BlsSecKeyVec) };
}

fn drop_3840517932(ptr: usize){
    unsafe { Box::from_raw(ptr as *mut pi_crypto::bls::BlsPubKeyVec) };
}

fn drop_4060246115(ptr: usize){
    unsafe { Box::from_raw(ptr as *mut pi_crypto::bls::BlsSigVec) };
}

fn drop_2254569071(ptr: usize){
    unsafe { Box::from_raw(ptr as *mut pi_crypto::bls::Curve) };
}

fn drop_1617625763(ptr: usize){
    unsafe { Box::from_raw(ptr as *mut pi_crypto::bls::BlsPublicKey) };
}

fn drop_3094164306(ptr: usize){
    unsafe { Box::from_raw(ptr as *mut pi_crypto::bls::BlsId) };
}

fn drop_187111440(ptr: usize){
    unsafe { Box::from_raw(ptr as *mut pi_crypto::bls::BlsSecretKey) };
}

fn drop_3966088300(ptr: usize){
    unsafe { Box::from_raw(ptr as *mut pi_crypto::bls::BlsSignature) };
}

fn drop_2886438122(ptr: usize){
    unsafe { Box::from_raw(ptr as *mut Arc<Vec<u8>>) };
}

fn drop_4244548360(ptr: usize){
    unsafe { Box::from_raw(ptr as *mut pi_crypto::jwt::SignKey) };
}

fn drop_768518599(ptr: usize){
    unsafe { Box::from_raw(ptr as *mut pi_crypto::jwt::VerifyKey) };
}

fn drop_1145754379(ptr: usize){
    unsafe { Box::from_raw(ptr as *mut pi_crypto::jwt::JwtAlg) };
}
pub fn register(mgr: &BonMgr){
    mgr.regist_struct_meta(StructMeta{name:String::from("hash_value::H256"), drop_fn: drop_1035403249}, 1035403249);
    mgr.regist_struct_meta(StructMeta{name:String::from("hash_value::H512"), drop_fn: drop_3223866506}, 3223866506);
    mgr.regist_struct_meta(StructMeta{name:String::from("pi_crypto::digest::DigestAlgorithm"), drop_fn: drop_4116332428}, 4116332428);
    mgr.regist_struct_meta(StructMeta{name:String::from("Vec<u8>"), drop_fn: drop_104530634}, 104530634);
    mgr.regist_struct_meta(StructMeta{name:String::from("pi_crypto::hmac::DigestAlgorithm"), drop_fn: drop_3042099183}, 3042099183);
    mgr.regist_struct_meta(StructMeta{name:String::from("pi_crypto::signature::ECDSASecp256k1"), drop_fn: drop_3761116463}, 3761116463);
    mgr.regist_struct_meta(StructMeta{name:String::from("pi_crypto::signature::Rsa"), drop_fn: drop_1545795468}, 1545795468);
    mgr.regist_struct_meta(StructMeta{name:String::from("pi_crypto::signature::PaddingAlg"), drop_fn: drop_142204220}, 142204220);
    mgr.regist_struct_meta(StructMeta{name:String::from("pi_crypto::signature::EcdsaAlg"), drop_fn: drop_3559558627}, 3559558627);
    mgr.regist_struct_meta(StructMeta{name:String::from("pi_crypto::signature::EcdsaKeyPair"), drop_fn: drop_1190264576}, 1190264576);
    mgr.regist_struct_meta(StructMeta{name:String::from("pi_crypto::bls::BlsIdVec"), drop_fn: drop_1875205449}, 1875205449);
    mgr.regist_struct_meta(StructMeta{name:String::from("pi_crypto::bls::BlsSecKeyVec"), drop_fn: drop_2934268916}, 2934268916);
    mgr.regist_struct_meta(StructMeta{name:String::from("pi_crypto::bls::BlsPubKeyVec"), drop_fn: drop_3840517932}, 3840517932);
    mgr.regist_struct_meta(StructMeta{name:String::from("pi_crypto::bls::BlsSigVec"), drop_fn: drop_4060246115}, 4060246115);
    mgr.regist_struct_meta(StructMeta{name:String::from("pi_crypto::bls::Curve"), drop_fn: drop_2254569071}, 2254569071);
    mgr.regist_struct_meta(StructMeta{name:String::from("pi_crypto::bls::BlsPublicKey"), drop_fn: drop_1617625763}, 1617625763);
    mgr.regist_struct_meta(StructMeta{name:String::from("pi_crypto::bls::BlsId"), drop_fn: drop_3094164306}, 3094164306);
    mgr.regist_struct_meta(StructMeta{name:String::from("pi_crypto::bls::BlsSecretKey"), drop_fn: drop_187111440}, 187111440);
    mgr.regist_struct_meta(StructMeta{name:String::from("pi_crypto::bls::BlsSignature"), drop_fn: drop_3966088300}, 3966088300);
    mgr.regist_struct_meta(StructMeta{name:String::from("Arc<Vec<u8>>"), drop_fn: drop_2886438122}, 2886438122);
    mgr.regist_struct_meta(StructMeta{name:String::from("pi_crypto::jwt::SignKey"), drop_fn: drop_4244548360}, 4244548360);
    mgr.regist_struct_meta(StructMeta{name:String::from("pi_crypto::jwt::VerifyKey"), drop_fn: drop_768518599}, 768518599);
    mgr.regist_struct_meta(StructMeta{name:String::from("pi_crypto::jwt::JwtAlg"), drop_fn: drop_1145754379}, 1145754379);
    mgr.regist_fun_meta(FnMeta::CallArg(call_266558349), 266558349);
    mgr.regist_fun_meta(FnMeta::CallArg(call_2282179587), 2282179587);
    mgr.regist_fun_meta(FnMeta::CallArg(call_1005885597), 1005885597);
    mgr.regist_fun_meta(FnMeta::CallArg(call_1115867356), 1115867356);
    mgr.regist_fun_meta(FnMeta::CallArg(call_3937242908), 3937242908);
    mgr.regist_fun_meta(FnMeta::CallArg(call_3800356447), 3800356447);
    mgr.regist_fun_meta(FnMeta::CallArg(call_2199666057), 2199666057);
    mgr.regist_fun_meta(FnMeta::Call(call_3054628822), 3054628822);
    mgr.regist_fun_meta(FnMeta::CallArg(call_1378957447), 1378957447);
    mgr.regist_fun_meta(FnMeta::CallArg(call_758410087), 758410087);
    mgr.regist_fun_meta(FnMeta::CallArg(call_835933247), 835933247);
    mgr.regist_fun_meta(FnMeta::CallArg(call_1239750690), 1239750690);
    mgr.regist_fun_meta(FnMeta::CallArg(call_2850709748), 2850709748);
    mgr.regist_fun_meta(FnMeta::CallArg(call_901229592), 901229592);
    mgr.regist_fun_meta(FnMeta::CallArg(call_2878910515), 2878910515);
    mgr.regist_fun_meta(FnMeta::CallArg(call_2041863833), 2041863833);
    mgr.regist_fun_meta(FnMeta::CallArg(call_3809028580), 3809028580);
    mgr.regist_fun_meta(FnMeta::CallArg(call_3841573761), 3841573761);
    mgr.regist_fun_meta(FnMeta::CallArg(call_1413668685), 1413668685);
    mgr.regist_fun_meta(FnMeta::CallArg(call_1006720087), 1006720087);
    mgr.regist_fun_meta(FnMeta::CallArg(call_2103960165), 2103960165);
    mgr.regist_fun_meta(FnMeta::CallArg(call_3851862966), 3851862966);
    mgr.regist_fun_meta(FnMeta::CallArg(call_1252421489), 1252421489);
    mgr.regist_fun_meta(FnMeta::CallArg(call_2592527877), 2592527877);
    mgr.regist_fun_meta(FnMeta::CallArg(call_3404883075), 3404883075);
    mgr.regist_fun_meta(FnMeta::CallArg(call_2903230657), 2903230657);
    mgr.regist_fun_meta(FnMeta::CallArg(call_2498464569), 2498464569);
    mgr.regist_fun_meta(FnMeta::Call(call_1295262082), 1295262082);
    mgr.regist_fun_meta(FnMeta::CallArg(call_2496411899), 2496411899);
    mgr.regist_fun_meta(FnMeta::CallArg(call_755737870), 755737870);
    mgr.regist_fun_meta(FnMeta::Call(call_3253072797), 3253072797);
    mgr.regist_fun_meta(FnMeta::CallArg(call_4280890483), 4280890483);
    mgr.regist_fun_meta(FnMeta::CallArg(call_2402380511), 2402380511);
    mgr.regist_fun_meta(FnMeta::CallArg(call_2426850537), 2426850537);
    mgr.regist_fun_meta(FnMeta::CallArg(call_3075954650), 3075954650);
    mgr.regist_fun_meta(FnMeta::CallArg(call_3801863647), 3801863647);
    mgr.regist_fun_meta(FnMeta::CallArg(call_1719604587), 1719604587);
    mgr.regist_fun_meta(FnMeta::CallArg(call_3025531400), 3025531400);
    mgr.regist_fun_meta(FnMeta::CallArg(call_3723291352), 3723291352);
    mgr.regist_fun_meta(FnMeta::CallArg(call_1669774542), 1669774542);
    mgr.regist_fun_meta(FnMeta::CallArg(call_1235807017), 1235807017);
    mgr.regist_fun_meta(FnMeta::CallArg(call_3671848448), 3671848448);
    mgr.regist_fun_meta(FnMeta::CallArg(call_1900424700), 1900424700);
    mgr.regist_fun_meta(FnMeta::CallArg(call_2045530324), 2045530324);
    mgr.regist_fun_meta(FnMeta::CallArg(call_298607248), 298607248);
    mgr.regist_fun_meta(FnMeta::CallArg(call_2029782143), 2029782143);
    mgr.regist_fun_meta(FnMeta::CallArg(call_1922268706), 1922268706);
    mgr.regist_fun_meta(FnMeta::CallArg(call_760927771), 760927771);
    mgr.regist_fun_meta(FnMeta::CallArg(call_1304117942), 1304117942);
    mgr.regist_fun_meta(FnMeta::CallArg(call_1202562609), 1202562609);
    mgr.regist_fun_meta(FnMeta::CallArg(call_1494397139), 1494397139);
    mgr.regist_fun_meta(FnMeta::CallArg(call_1251457612), 1251457612);
    mgr.regist_fun_meta(FnMeta::CallArg(call_863200741), 863200741);
    mgr.regist_fun_meta(FnMeta::CallArg(call_3082139465), 3082139465);
    mgr.regist_fun_meta(FnMeta::CallArg(call_3576086575), 3576086575);
    mgr.regist_fun_meta(FnMeta::CallArg(call_3750445483), 3750445483);
    mgr.regist_fun_meta(FnMeta::CallArg(call_3551222567), 3551222567);
    mgr.regist_fun_meta(FnMeta::CallArg(call_4217857181), 4217857181);
    mgr.regist_fun_meta(FnMeta::CallArg(call_3778283533), 3778283533);
    mgr.regist_fun_meta(FnMeta::CallArg(call_1905417019), 1905417019);
    mgr.regist_fun_meta(FnMeta::CallArg(call_2172313629), 2172313629);
    mgr.regist_fun_meta(FnMeta::CallArg(call_2861556416), 2861556416);
    mgr.regist_fun_meta(FnMeta::CallArg(call_4054179525), 4054179525);
    mgr.regist_fun_meta(FnMeta::CallArg(call_3718730423), 3718730423);
    mgr.regist_fun_meta(FnMeta::CallArg(call_2864459653), 2864459653);
    mgr.regist_fun_meta(FnMeta::CallArg(call_376820189), 376820189);
    mgr.regist_fun_meta(FnMeta::CallArg(call_263952757), 263952757);
    mgr.regist_fun_meta(FnMeta::CallArg(call_2039602097), 2039602097);
    mgr.regist_fun_meta(FnMeta::CallArg(call_1087017908), 1087017908);
    mgr.regist_fun_meta(FnMeta::CallArg(call_993477813), 993477813);
    mgr.regist_fun_meta(FnMeta::CallArg(call_3587763353), 3587763353);
    mgr.regist_fun_meta(FnMeta::CallArg(call_3188209906), 3188209906);
    mgr.regist_fun_meta(FnMeta::CallArg(call_2084703123), 2084703123);
    mgr.regist_fun_meta(FnMeta::CallArg(call_890057462), 890057462);
    mgr.regist_fun_meta(FnMeta::CallArg(call_1518526824), 1518526824);
    mgr.regist_fun_meta(FnMeta::CallArg(call_985222615), 985222615);
    mgr.regist_fun_meta(FnMeta::CallArg(call_1277908099), 1277908099);
    mgr.regist_fun_meta(FnMeta::CallArg(call_2975386969), 2975386969);
    mgr.regist_fun_meta(FnMeta::CallArg(call_1021127516), 1021127516);
    mgr.regist_fun_meta(FnMeta::CallArg(call_4078596132), 4078596132);
    mgr.regist_fun_meta(FnMeta::CallArg(call_1558442167), 1558442167);
    mgr.regist_fun_meta(FnMeta::CallArg(call_284043717), 284043717);
    mgr.regist_fun_meta(FnMeta::CallArg(call_1162834309), 1162834309);
    mgr.regist_fun_meta(FnMeta::CallArg(call_1055468812), 1055468812);
    mgr.regist_fun_meta(FnMeta::CallArg(call_1163872385), 1163872385);
    mgr.regist_fun_meta(FnMeta::CallArg(call_3565773478), 3565773478);
    mgr.regist_fun_meta(FnMeta::CallArg(call_1219230175), 1219230175);
    mgr.regist_fun_meta(FnMeta::CallArg(call_2615574485), 2615574485);
}