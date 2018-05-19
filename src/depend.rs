use std::collections::HashMap;
use json::JsonValue;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Depend{
	file_map: HashMap<String, RcFileDes>,
	pub root: String
}

impl Depend{
	pub fn new(list: Vec<FileDes>, root: &str) -> Depend{
		let mut file_map = HashMap::new();
		for fd in list.into_iter() {
			let rc = Rc::new(RefCell::new(fd));
			file_map.insert(String::from(rc.borrow().path.as_str()), rc.clone());
			Depend::init_dir(rc, &mut file_map);
		}
		Depend{
			file_map: file_map,
			root: String::from(root),
		}
	}

	pub fn get(&self, path: &str) -> Option<&RcFileDes>{
		self.file_map.get(path)
	}

	pub fn get_path(&self, path: &str) -> String{
		if path.starts_with("./") || path.starts_with("../"){
			String::from(&path[self.root.len()+1..path.len()])
		}else{
			String::from(path)
		}
	}

	fn init_dir(fd: RcFileDes, map: &mut HashMap<String, RcFileDes>){
		let path = String::from(fd.borrow().path.as_str());
		let mut file_map = HashMap::new();
		Depend::init_child_dir(map, fd.clone(), &path, "", &mut file_map);
		let iter = file_map.into_iter();
		map.extend(iter);
		map.insert(path, fd);
	}

	//文件路径不能以/开始
	fn init_child_dir(map: &mut HashMap<String, RcFileDes>, fd: RcFileDes, path:&str, root: &str,  ev_map: &mut HashMap<String, RcFileDes>){
		let fd_ref = fd.borrow();
		if path == "" {
			return;
		}
		let mut si = match path.find("/"){
			Some(v) => {v},
			None => {path.len()},
		};

		let dir =  &path[0..si];
		let full_dir;
		if root == ""{
			full_dir = String::from(dir);
		}else{
			full_dir = root.to_owned() + "/" + dir;
		}
		let info = match map.contains_key(dir) {
			true => {map.get(dir).unwrap().clone()},
			false => {
				if si < path.len(){
					let f = Rc::new(RefCell::new(FileDes{children:Some(HashMap::new()), size:0, path: full_dir.clone(), sign:None, time:None, depend:None}));
					map.insert(String::from(dir), f.clone());
					ev_map.insert(String::from(full_dir.clone()), f.clone());
					f
				}else{
					let f = Rc::new(RefCell::new(FileDes{children:None, size:0, path: full_dir.clone(), sign:None, time:None, depend:None}));
					map.insert(String::from(dir), f.clone());
					f
				}
				
			},
		};
		let mut info_ref = info.borrow_mut();
		info_ref.size += fd_ref.size;

		if si < path.len(){
			let m = info_ref.children.as_mut().unwrap();
			si = si + 1;
			let child_path = &path[si..path.len()];
			Depend::init_child_dir(m, fd.clone(), child_path, &full_dir, ev_map);
		}
	}
}

pub type RcFileDes = Rc<RefCell<FileDes>>;

pub struct FileDes{
	pub path: String,
	pub sign: Option<String>,
	pub time: Option<u64>,
	pub size: u64,
	pub depend: Option<HashMap<String, Vec<String>>>,
	pub children:Option<HashMap<String, RcFileDes>>,
}

impl FileDes {
	pub fn from(jv: JsonValue, per_dir: &str) -> Self{
		let mut obj = match jv {
			JsonValue::Object(v) => {v},
			_ => {panic!("Json不是一个Object，无法转换为FileDes")},
		};
		let path = match un_jvalue(obj.remove("path")){
			JsonValue::Short(v) => {String::from(per_dir) + v.as_str()},
			JsonValue::String(v) => {String::from(per_dir) + v.as_str()},
			_ => {panic!("path不是一个String，无法转换为FileDes.path")},
		};

		let sign = match un_jvalue(obj.remove("sign")){
			JsonValue::Short(v) => {Some(String::from(v.as_str()))},
            JsonValue::String(v) => {Some(String::from(v.as_str()))},
            JsonValue::Number(_) => {Some(String::from("0"))},
			_ => {panic!("sign不是一个String，无法转换为FileDes.sign")},
		};

		let time = match un_jvalue(obj.remove("time")){
			JsonValue::Number(v) => {Some(v.into())},
			_ => {panic!("time不是一个number，无法转换为FileDes.time")},
		};

		let size = match un_jvalue(obj.remove("size")){
			JsonValue::Number(v) => {v.into()},
			_ => {panic!("time不是一个number，无法转换为FileDes.size")},
		};

        let d = obj.remove("depend");
        let depend = match d {
            Some(de) => {
                match de{
                    JsonValue::Array(_) => {
                        None
                    },
                    JsonValue::Object(v) => {
                        let iter = v.iter();
                        let mut map = HashMap::new();
                        for (k, v) in iter{
                            let v = match v{
                                JsonValue::Array(v) => {
                                    let iter1 = v.into_iter();
                                    let mut arr = Vec::new();
                                    for s in iter1{
                                        match s {
                                            JsonValue::String(v) => {arr.push(v.clone())},
                                            JsonValue::Short(v) => {arr.push(String::from(v.as_str()))},
                                            _ => {panic!("{}中元素不是一个String，无法转换为FileDes.depend", k)},
                                        }
                                    }
                                    arr
                                }
                                _ => {panic!("{}不是一个数组，无法转换为FileDes.depend", k)}
                            };
                            map.insert(String::from(k), v);
                        }
                        Some(map)
                    },
                    _ => {panic!("depend不是一个Object，无法转换为FileDes.depend")},
                }
            },
            None => None,
        };


		FileDes{
			path, size, sign, time, depend, children:None
		}
	}
}

fn un_jvalue(o: Option<JsonValue>) -> JsonValue{
	match o {
		Some(v) => {v},
		None => {panic!("Option is None, can not unpack");},
	}
}

pub struct Built;
impl Built{
	pub fn relative_path(file_path: &str, dir: &str) -> String {
		if !file_path.starts_with("./") && !file_path.starts_with("../"){
			return String::from(file_path);
		}
		let last;
		let mut fv: Vec<&str> = file_path.split('/').collect();
		let mut dv: Vec<&str> = dir.split('/').collect();
		let dlen = dv.len();
		if dlen != 0{
			dv.remove(dlen - 1);
		}

		fn c(fv: &mut Vec<&str>,dv: &mut Vec<&str>) -> Option<String>{
			if fv.len() == 0{
				panic!("file_path不符合相对路劲格式规范");
			}
			let elem = fv.remove(0);
			if elem ==".."{
				dv.pop();
				c(fv, dv)
			}else if elem !="."{
				Some(String::from(elem))
			}else{
				None
			}
		};
		let l = c(&mut fv, &mut dv);
		if l.is_some(){
			last = l.unwrap();
			dv.push(last.as_str());
		}

		return Built::join(dv.as_slice(), "/") + "/" + Built::join(fv.as_slice(), "/").as_str();
	}

	fn join(v: &[&str], jstr: &str) -> String{
		if v.len() > 0 {
			let mut s = String::from(v[0]);
			for i in 1..v.len(){
				s = s + jstr + v[i];
			}
			return s;
		}else{
			return String::from("");
		}
	}

	
}

// impl BonCode for FileDes {
// 	fn bon_encode(&self, bb: &mut BonBuffer, next: fn(&mut BonBuffer,  &Self)){
// 		let mut buf = BonBuffer::new();
// 		buf.write_utf8(self.path);
// 		buf.write_utf8(self.sign);
// 		buf.write_u64(self.time);
// 		buf.write_u64(self.size);
// 		buf.write_utf8(self.path);
// 		let next = fn(){
// 			let bc: Struct = new meta.construct();
// 			bc.binDecode(bb, getAllReadNext(mgr));
// 		}
// 		buf.write_container::<FileDes>(&(self.depend), fn())
// 		<T: BonCode>(&mut self, o: &T, write_next: fn(&mut BonBuffer,  &T), estimated_size: Option<usize>) 
// 	}
// 	fn bon_decode(&mut self, bb: &BonBuffer, next: fn(&BonBuffer,  &u32) -> Self){

// 	};
// }

	
#[test]
fn from_jvalue(){
	use json::parse;
	let s = r#"{
		"path":"pi_serv",
		"sign":"FFFFFFFFFFFFFFFFFFFFFF",
		"time":324343435,
		"size": 50, 
		"depend": {"js":["../../pi/test.js", "../../pi/test1.js"], "png":["../../pi/test.png"]}
		}"#;
	let jvalue = parse(s);
	match jvalue {
		Ok(v) => {
			let fd = FileDes::from(v, "../pi/");
			assert_eq!("../pi/pi_serv",fd.path);
			assert_eq!("FFFFFFFFFFFFFFFFFFFFFF",fd.sign.unwrap());
			assert_eq!(324343435,fd.time.unwrap());
			assert_eq!(50,fd.size);
			match fd.depend {
				Some(depend) => {
					let iter = depend.iter();
					for (k, v) in iter{
						let iter_elem = v.iter();
						for elem in iter_elem{
							println!("{}:{}", k, elem);
						}
					}
				},
				None => {},
			}
		},
		_ => {},
	};
	
}

#[test]
fn new_depend(){
	use json::parse;
	let s = r#"{
		"path":"pi_serv/util/hash.js",
		"sign":"FFFFFFFFFFFFFFFFFFFFFF",
		"time":324343435,
		"size": 50, 
		"depend": [],
		}"#;
	let jvalue = parse(s);
	match jvalue {
		Ok(v) => {
			let mut vec = Vec::new();
			let fd = FileDes::from(v, "pi/");
			vec.push(fd);
			let depend = Depend::new(vec, "../");
			let map = depend.file_map;
			assert_eq!(map.len(), 3);
		},
		_ => {},
	};
	
}

#[test]
fn relative_path(){
	let p1 = "../pi/util/hash.js";
	let p2 = "../widget/util.js";
	let r = Built::relative_path(p2, p1);
	assert_eq!(r, "../pi/widget/util.js");
}
