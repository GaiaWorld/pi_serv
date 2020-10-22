use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use mqtt3::QoS;

use atom::Atom;
use bon::{Decode, Encode, ReadBonErr, ReadBuffer, WriteBuffer};
use hash_value::hex::ToHex;
use mqtt_tmp::data::Server;
use mqtt_tmp::server::ServerNode;
use pi_db::db::{Bin, Event, EventType, Iter, TabKV, TabMeta, Ware};
use pi_db::memery_db::DB;
use pi_db::mgr::{Mgr, Monitor, Tr};
use pi_db::util::{dump as db_dump, restore as db_restore};
use pi_store::file_mem_db::FileMemDB;
use pi_store::lmdb_file::{DB as Lmdb};
use pi_store::log_file_db::LogFileDB;
use pi_vm::adapter::{dukc_pop, JSType, JS};
use pi_vm::bonmgr::ptr_jstype;
use pi_vm::pi_vm_impl::{block_set_global_var, BlockError, VMFactory};

//use pi_base::util::now_millisecond;
//use pi_vm::adapter::dukc_top;

use js_util::{decode_by_tabkv, decode_by_type, decode_bin_by_tabkv};

type DBIterTrait = Box<Iter<Item = (Bin, Bin)>>;
/**
* 封装类db迭代器， 是其由traiobj转化为具体类型（构建工具暂时不支持traitobj的构建）
*/
pub struct DBIter(DBIterTrait, Arc<TabMeta>);

impl DBIter {
    /**
     * 迭代下一个记录
     * @param cb 迭代下一个记录的结果的异步回调，成功返回下一个记录的键值对，如果没有下一个记录，则返回空，失败返回原因描述
     * @returns 返回同步迭代下一个记录的结果，成功返回下一个记录的键值对，如果没有下一个记录，则返回空
     * @throws 失败则抛出原因描述
     */
    pub fn next(
        &mut self,
        cb: Arc<Fn(Result<Option<(Arc<Vec<u8>>, Arc<Vec<u8>>)>, String>)>,
    ) -> Option<Result<Option<(Arc<Vec<u8>>, Arc<Vec<u8>>)>, String>> {
        self.0.next(cb.clone())
    }

    /**
     * 迭代下一个对象
     * @param cb 迭代下一个对象的结果的异步回调，成功返回下一个记录的反序列化对象，如果没有下一个对象，则返回空，失败返回原因描述
     * @param js 当前虚拟机
     * @returns 返回同步迭代下一个对象的结果的异步回调，成功返回下一个记录的反序列化对象，如果没有下一个对象，则返回空
     * @throws 失败则抛出原因描述
     */
    pub fn next_elem(
        &mut self,
        cb: Arc<Fn(Result<Option<JSType>, String>)>,
        js: &Arc<JS>,
    ) -> Option<Result<Option<JSType>, String>> {
        let js = js.clone();
        let js1 = js.clone();
        let js2 = js.clone();
        let meta = self.1.clone();
        let meta1 = self.1.clone();
        let cb1 = cb.clone();
        let call_back = move |r: Result<Option<(Arc<Vec<u8>>, Arc<Vec<u8>>)>, String>| {
            // let meta = meta1;
            let meta = meta.clone();
            let cb = cb1.clone();
            let cb1 = cb1.clone();
            match r {
                Ok(v) => {
                    match v {
                        Some(value) => {
                            // let cb = cb1;
                            block_set_global_var(
                                js2.clone(),
                                "_$rust_r".to_string(),
                                Box::new(move |js: Arc<JS>| -> Result<JSType, String> {
                                    let arr = js.new_array();
                                    let mut k = match decode_by_type(
                                        &js,
                                        &mut ReadBuffer::new(&value.0, 0),
                                        &meta.k,
                                    ) {
                                        Ok(v) => v,
                                        Err(s) => {
                                            unsafe { dukc_pop(js.get_vm()) };
                                            return Err(s);
                                        }
                                    };
                                    js.set_index(&arr, 0, &mut k);
                                    let mut v = match decode_by_type(
                                        &js,
                                        &mut ReadBuffer::new(&value.1, 0),
                                        &meta.v,
                                    ) {
                                        Ok(v) => v,
                                        Err(s) => {
                                            unsafe { dukc_pop(js.get_vm()) };
                                            return Err(s);
                                        }
                                    };
                                    js.set_index(&arr, 1, &mut v);
                                    Ok(arr)
                                }),
                                Box::new(move |r: Result<Arc<JS>, BlockError>| match r {
                                    Ok(js) => cb1(Ok(Some(js.new_undefined()))),
                                    Err(s) => cb1(Err(format!("{:?}", s))),
                                }),
                                Atom::from("next_elem"),
                            );
                            // js.set_global_var("_$rust_r".to_string(), arr);
                            // cb(Ok(Some(js.new_undefined())));
                        }
                        None => cb(Ok(None)),
                    };
                }
                Err(s) => cb(Err(s)),
            }
        };

        match self.0.next(Arc::new(call_back)) {
            Some(v) => match v {
                Ok(v) => match v {
                    Some(value) => {
                        let arr = js1.new_array();
                        let mut k =
                            match decode_by_type(&js1, &mut ReadBuffer::new(&value.0, 0), &meta1.k)
                            {
                                Ok(v) => v,
                                Err(s) => {
                                    unsafe { dukc_pop(js.get_vm()) };
                                    return Some(Err(s));
                                }
                            };
                        js1.set_index(&arr, 0, &mut k);
                        let mut v =
                            match decode_by_type(&js1, &mut ReadBuffer::new(&value.1, 0), &meta1.v)
                            {
                                Ok(v) => v,
                                Err(s) => {
                                    unsafe { dukc_pop(js.get_vm()) };
                                    return Some(Err(s));
                                }
                            };
                        js1.set_index(&arr, 1, &mut v);
                        Some(Ok(Some(arr)))
                    }
                    None => Some(Ok(None)),
                },
                Err(s) => Some(Err(s)),
            },
            None => None,
        }
    }

    // pub fn next_elem(&mut self, cb: Arc<Fn(Result<Option<(Arc<Vec<u8>>, Arc<Vec<u8>>, Vec<u8>)>, String>)>, js: &Arc<JS>) -> Option<Result<Option<(Arc<Vec<u8>>, Arc<Vec<u8>>, Vec<u8>)>, String>> {
    //     let meta = self.1.clone();
    //     let mut wb = WriteBuffer::new();
    //     meta.encode(&mut wb);

    //     let buf = Arc::new(wb);
    //     let buf1 = buf.clone();

    //     let call_back = move|r: Result<Option<(Arc<Vec<u8>>, Arc<Vec<u8>>)>, String>|{
    //         match r {
    //             Ok(v) => {
    //                 match v {
    //                     Some(value) => {
    //                         cb(Ok(Some((value.0, value.1, buf.to_vec()))));
    //                     },
    //                     None => cb(Ok(None)),
    //                 };
    //             },
    //             Err(s) => cb(Err(s)),
    //         }
    //     };

    //     match self.0.next(Arc::new(call_back)) {
    //         Some(v) => {
    //             match v {
    //                 Ok(v) => {
    //                     match v {
    //                         Some(value) => Some(Ok(Some((value.0, value.1, buf1.to_vec())))),
    //                         None => Some(Ok(None)),
    //                     }
    //                 },
    //                 Err(s) => Some(Err(s)),
    //             }
    //         },
    //         None => None,
    //     }
    // }
}

pub struct DBWare(Arc<Ware>);

/**
* 获取数据库的迭代器
* @param tr 事务
* @param ware 库名
* @param tab 表名
* @param key 启始关键字
* @param descending 是否倒序
* @param _filter 过滤器，暂未使用
* @param cb 获取迭代器的结果的异步回调，成功返回数据库的迭代器，失败返回原因描述
* @returns 返回同步获取迭代器的结果，成功返回数据库的迭代器
* @throws 失败则抛出原因描述
*/
pub fn iter_db(
    tr: &Tr,
    ware: String,
    tab: String,
    key: Option<&[u8]>,
    descending: bool,
    _filter: Option<String>,
    cb: Arc<Fn(Result<DBIter, String>)>,
) -> Option<Result<DBIter, String>> {
    let ware = Atom::from(ware);
    let tab = Atom::from(tab);
    let key = match key {
        Some(v) => Some(Arc::new(Vec::from(v))),
        None => None,
    };

    //取元信息
    let meta = match tr.tab_info(&ware, &tab) {
        Some(v) => v,
        None => return Some(Err(String::from("meta is not exist"))), //元信息不存在，不可能生成迭代器， 因此直接返回None
    };
    let meta1 = meta.clone();

    let cb1 = move |r: Result<DBIterTrait, String>| match r {
        Ok(v) => cb(Ok(DBIter(v, meta.clone()))),
        Err(s) => cb(Err(s)),
    };
    let cb1 = Arc::new(cb1);
    match tr.iter(&ware, &tab, key, descending, None, cb1.clone()) {
        Some(v) => match v {
            Ok(v) => Some(Ok(DBIter(v, meta1))),
            Err(s) => Some(Err(s)),
        },
        None => None,
    }
}

pub fn clone_db_mgr(mgr: &Mgr) -> Mgr {
    mgr.clone()
}

// 注册内存数据库
pub fn register_memery_db(mgr: &Mgr, prefix: String, ware: DB) -> bool {
    if let Some(w) = mgr.find(&Atom::from(prefix.clone())) {
        false
    } else {
        mgr.register(Atom::from(prefix), Arc::new(ware))
    }
}

// 注册文件数据库
pub fn register_file_db(mgr: &Mgr, prefix: String, ware: Lmdb) -> bool {
    if let Some(w) = mgr.find(&Atom::from(prefix.clone())) {
        false
    } else {
        mgr.register(Atom::from(prefix), Arc::new(ware))
    }
}

pub fn register_file_mem_db(mgr: &Mgr, prefix: String, ware: FileMemDB) -> bool {
    mgr.register(Atom::from(prefix), Arc::new(ware))
}

pub fn register_log_file_db(mgr: &Mgr, prefix: String, ware: LogFileDB) -> bool {
    if let Some(w) = mgr.find(&Atom::from(prefix.clone())) {
        false
    } else {
        mgr.register(Atom::from(prefix), Arc::new(ware))
    }
}

pub fn get_all_wares(mgr: &Mgr) -> Vec<String> {
    mgr.ware_name_list()
}

pub fn get_tabmeta_buffer(meta: Arc<TabMeta>) -> Vec<u8> {
    let mut bon = WriteBuffer::new();
    meta.encode(&mut bon);
    bon.unwrap()
}

//new TabKV
pub fn tabkv_with_value(ware: &str, tab: &str, key: &[u8], value: &[u8]) -> TabKV {
    TabKV {
        ware: Atom::from(ware),
        tab: Atom::from(tab),
        key: Arc::new(Vec::from(key)),
        index: 0,
        value: Some(Arc::new(Vec::from(value))),
    }
}

//new TabKV
pub fn tabkv_new(ware: &str, tab: &str, key: &[u8]) -> TabKV {
    TabKV {
        ware: Atom::from(ware),
        tab: Atom::from(tab),
        key: Arc::new(Vec::from(key)),
        index: 0,
        value: None,
    }
}

//TabKV get_value
pub fn tabkv_get_value(tabkv: &TabKV) -> Option<Arc<Vec<u8>>> {
    tabkv.value.clone()
}

pub fn list_all_tables(tr: &Tr, ware: String) -> Vec<String> {
    match tr.list(&Atom::from(ware)) {
        Some(tabs) => {
            let mut v = vec![];
            for t in tabs {
                v.push(t.to_string());
            }
            v
        }

        None => vec![],
    }
}

/**
* 修改数据库元信息
* @param tr 事务
* @param ware 库名
* @param tab 表名
* @param meta_buf 元信息
* @param cb 创建、修改或删除表的结果的异步回调，成功返回空，失败返回原因描述
* @returns 返回同步创建、修改或删除表的结果，成功返回空
* @throws 失败则抛出原因描述
*/
pub fn alter(
    tr: &Tr,
    ware: String,
    tab: String,
    meta_buf: Option<&[u8]>,
    cb: Arc<Fn(Result<(), String>)>,
) -> Option<Result<(), String>> {
    let meta = match meta_buf {
        Some(buf) => {
            let r = match TabMeta::decode(&mut ReadBuffer::new(buf, 0)) {
                Ok(o) => o,
                Err(e) => return Some(Err(e.to_string())),
            };
            Some(Arc::new(r))
        }
        None => None,
    };
    let r = tr.alter(&Atom::from(ware), &Atom::from(tab), meta, cb);
    r
}

/**
* 修改数据库记录
* @param tr 事务
* @param items 对象数组
* @param lock_time 修改的锁超时时长，单位毫秒
* @param read_lock 是否读锁
* @param cb 修改结果的异步回调，成功返回空，失败返回原因描述
* @returns 返回同步修改结果，成功返回空
* @throws 失败则抛出原因描述
*/
pub fn modify(
    tr: &Tr,
    items: &JSType,
    lock_time: Option<usize>,
    read_lock: bool,
    cb: Arc<Fn(Result<(), String>)>,
) -> Option<Result<(), String>> {
    let param_error = String::from("param error in modify");
    if !items.is_array() {
        return Some(Err(param_error));
    }
    let a_len = items.get_array_length();
    let mut arr = Vec::new();
    for i in 0..a_len {
        let elem_e = items.get_index(i as u32);
        if !elem_e.is_array() {
            return Some(Err(param_error));
        }
        let elem_e_e = elem_e.get_index(0);
        if !elem_e_e.is_string() {
            return Some(Err(param_error));
        }
        let elem_e_0 = elem_e_e.get_str();
        let elem_e_e = elem_e.get_index(1);
        if !elem_e_e.is_string() {
            return Some(Err(param_error));
        }
        let elem_e_1 = elem_e_e.get_str();

        let elem_e_e = elem_e.get_index(2);
        if !elem_e_e.is_uint8_array() && !elem_e_e.is_array_buffer() {
            return Some(Err(param_error));
        }
        let elem_e_2 = elem_e_e.to_bytes();

        let elem_e_e = elem_e.get_index(3);
        let elem_e_3 = if elem_e_e.is_null() || elem_e_e.is_undefined() {
            None
        } else if !elem_e_e.is_uint8_array() && !elem_e_e.is_array_buffer() {
            return Some(Err(param_error));
        } else {
            Some(Arc::new(elem_e_e.into_vec()))
        };

        arr.push(TabKV {
            ware: Atom::from(elem_e_0.clone()),
            tab: Atom::from(elem_e_1.clone()),
            key: Arc::new(Vec::from(elem_e_2)),
            value: elem_e_3,
            index: 0,
        });
    }

    tr.modify(arr, lock_time, read_lock, cb)
}

/**
* 查询数据库记录
* @param tr 事务
* @param items 关键字数组
* @param lock_time 修改的锁超时时长，单位毫秒
* @param read_lock 是否读锁
* @param cb 查询结果的异步回调，成功查询到关键字对应的记录，则返回对象数组，否则返回空，失败返回原因描述
* @returns 返回同步查询结果，成功查询到关键字对应的记录，则返回对象数组，否则返回空
* @throws 失败则抛出原因描述
*/
pub fn query(
    tr: &Tr,
    items: &JSType,
    lock_time: Option<usize>,
    read_lock: bool,
    cb: Arc<Fn(Result<JSType, String>)>,
    js: &Arc<JS>,
) -> Option<Result<JSType, String>> {
    let param_error = String::from("param error in query");
    if !items.is_array() {
        return Some(Err(param_error));
    }
    let a_len = items.get_array_length();
    let mut arr = Vec::new();
    for i in 0..a_len {
        let elem_e = items.get_index(i as u32);
        if !elem_e.is_array() {
            return Some(Err(param_error));
        }
        let elem_e_e = elem_e.get_index(0);
        if !elem_e_e.is_string() {
            return Some(Err(param_error));
        }
        let elem_e_0 = elem_e_e.get_str();

        let elem_e_e = elem_e.get_index(1);
        if !elem_e_e.is_string() {
            return Some(Err(param_error));
        }
        let elem_e_1 = elem_e_e.get_str();

        let elem_e_e = elem_e.get_index(2);
        if !elem_e_e.is_uint8_array() && !elem_e_e.is_array_buffer() {
            return Some(Err(param_error));
        }
        let elem_e_2 = Arc::new(elem_e_e.into_vec());

        arr.push(TabKV {
            ware: Atom::from(elem_e_0.clone()),
            tab: Atom::from(elem_e_1.clone()),
            key: elem_e_2,
            value: None,
            index: 0,
        });
    }

    let js1 = js.clone();
    let tr1 = tr.clone();
    let call_back = move |r: Result<Vec<TabKV>, String>| {
        let cb = cb.clone();
        let cb1 = cb.clone();
        let js1 = js1.clone();
        let tr1 = tr1.clone();
        match r {
            Ok(v) => {
                block_set_global_var(
                    js1.clone(),
                    "_$rust_r".to_string(),
                    Box::new(move |js: Arc<JS>| -> Result<JSType, String> {
                        let arr = js.new_array();
                        for i in 0..v.len() {
                            let elem = &v[i];
                            let mut r = match decode_by_tabkv(
                                &js,
                                elem,
                                &tr1.tab_info(&elem.ware, &elem.tab).unwrap(),
                            ) {
                                Ok(v) => v,
                                Err(s) => {
                                    unsafe { dukc_pop(js.get_vm()) };
                                    return Err(s);
                                }
                            };
                            js.set_index(&arr, i as u32, &mut r);
                        }
                        Ok(arr)
                    }),
                    Box::new(move |r: Result<Arc<JS>, BlockError>| match r {
                        Ok(js) => cb1(Ok(js.new_undefined())),
                        Err(s) => cb1(Err(format!("{:?}", s))),
                    }),
                    Atom::from("query"),
                );
                // js1.set_global_var("_$rust_r".to_string(), arr);
                // cb(Ok(js1.new_undefined()));
            }
            Err(s) => cb(Err(s)),
        }
    };
    match tr.query(arr, lock_time, read_lock, Arc::new(call_back)) {
        Some(r) => match r {
            Ok(v) => {
                let arr = js.new_array();
                for i in 0..v.len() {
                    let elem = &v[i];
                    let mut r = match decode_by_tabkv(
                        &js,
                        elem,
                        &tr.tab_info(&elem.ware, &elem.tab).unwrap(),
                    ) {
                        Ok(v) => v,
                        Err(s) => {
                            unsafe { dukc_pop(js.get_vm()) };
                            return Some(Err(s));
                        }
                    };
                    js.set_index(&arr, i as u32, &mut r);
                }
                Some(Ok(arr))
            }
            Err(s) => Some(Err(s)),
        },
        None => None,
    }
}

/**
* 查询数据库记录
* @param tr 事务
* @param items 关键字数组
* @param lock_time 修改的锁超时时长，单位毫秒
* @param read_lock 是否读锁
* @param cb 查询结果的异步回调，成功查询到关键字对应的记录，则返回对象数组，否则返回空，失败返回原因描述
* @returns 返回同步查询结果，成功查询到关键字对应的记录，则返回对象数组，否则返回空
* @throws 失败则抛出原因描述
*/
pub fn query_bin(
    tr: &Tr,
    items: &JSType,
    lock_time: Option<usize>,
    read_lock: bool,
    cb: Arc<Fn(Result<JSType, String>)>,
    js: &Arc<JS>,
) -> Option<Result<JSType, String>> {
    let param_error = String::from("param error in query");
    if !items.is_array() {
        return Some(Err(param_error));
    }
    let a_len = items.get_array_length();
    let mut arr = Vec::new();
    for i in 0..a_len {
        let elem_e = items.get_index(i as u32);
        if !elem_e.is_array() {
            return Some(Err(param_error));
        }
        let elem_e_e = elem_e.get_index(0);
        if !elem_e_e.is_string() {
            return Some(Err(param_error));
        }
        let elem_e_0 = elem_e_e.get_str();

        let elem_e_e = elem_e.get_index(1);
        if !elem_e_e.is_string() {
            return Some(Err(param_error));
        }
        let elem_e_1 = elem_e_e.get_str();

        let elem_e_e = elem_e.get_index(2);
        if !elem_e_e.is_uint8_array() && !elem_e_e.is_array_buffer() {
            return Some(Err(param_error));
        }
        let elem_e_2 = Arc::new(elem_e_e.into_vec());

        arr.push(TabKV {
            ware: Atom::from(elem_e_0.clone()),
            tab: Atom::from(elem_e_1.clone()),
            key: elem_e_2,
            value: None,
            index: 0,
        });
    }

    let js1 = js.clone();
    let tr1 = tr.clone();
    let call_back = move |r: Result<Vec<TabKV>, String>| {
        let cb = cb.clone();
        let cb1 = cb.clone();
        let js1 = js1.clone();
        let tr1 = tr1.clone();
        match r {
            Ok(v) => {
                block_set_global_var(
                    js1.clone(),
                    "_$rust_r".to_string(),
                    Box::new(move |js: Arc<JS>| -> Result<JSType, String> {
                        let arr = js.new_array();
                        for i in 0..v.len() {
                            let elem = &v[i];
                            let mut r = match decode_bin_by_tabkv(
                                &js,
                                elem,
                                &tr1.tab_info(&elem.ware, &elem.tab).unwrap(),
                            ) {
                                Ok(v) => v,
                                Err(s) => {
                                    unsafe { dukc_pop(js.get_vm()) };
                                    return Err(s);
                                }
                            };
                            js.set_index(&arr, i as u32, &mut r);
                        }
                        Ok(arr)
                    }),
                    Box::new(move |r: Result<Arc<JS>, BlockError>| match r {
                        Ok(js) => cb1(Ok(js.new_undefined())),
                        Err(s) => cb1(Err(format!("{:?}", s))),
                    }),
                    Atom::from("query"),
                );
                // js1.set_global_var("_$rust_r".to_string(), arr);
                // cb(Ok(js1.new_undefined()));
            }
            Err(s) => cb(Err(s)),
        }
    };
    match tr.query(arr, lock_time, read_lock, Arc::new(call_back)) {
        Some(r) => match r {
            Ok(v) => {
                let arr = js.new_array();
                for i in 0..v.len() {
                    let elem = &v[i];
                    let mut r = match decode_bin_by_tabkv(
                        &js,
                        elem,
                        &tr.tab_info(&elem.ware, &elem.tab).unwrap(),
                    ) {
                        Ok(v) => v,
                        Err(s) => {
                            unsafe { dukc_pop(js.get_vm()) };
                            return Some(Err(s));
                        }
                    };
                    js.set_index(&arr, i as u32, &mut r);
                }
                Some(Ok(arr))
            }
            Err(s) => Some(Err(s)),
        },
        None => None,
    }
}

/**
* 获取表的记录数量
* @param tr 事务
* @param ware_name 库名
* @param tab_name 表名
* @param cb 获取表的记录数量结果的异步回调，成功返回记录数量，失败返回原因描述
* @returns 返回同步获取表的记录数量的结果，成功返回记录数量
* @throws 失败则抛出原因描述
*/
pub fn tab_size(
    tr: &Tr,
    ware_name: &str,
    tab_name: &str,
    cb: Arc<Fn(Result<usize, String>)>,
) -> Option<Result<usize, String>> {
    tr.tab_size(&Atom::from(ware_name), &Atom::from(tab_name), cb)
}

/**
* 将指定表的所有记录备份到指定的文件
* @param mgr 表库及事务管理器
* @param ware 库名
* @param tab 表名
* @param file 文件的路径
* @param cb 将指定表的所有记录保存到指定的文件的结果的异步回调，成功返回空，失败返回原因描述
* @returns 返回同步将指定表的所有记录保存到指定的文件的结果，成功返回空
* @throws 失败则抛出原因描述
*/
pub fn dump(mgr: &Mgr, ware: String, tab: String, file: String, cb: Arc<Fn(Result<(), String>)>) {
    let dir = match file.as_str().rfind("/") {
        Some(v) => &file[0..v],
        None => {
            panic!("restore file Invalid:{}", file);
        }
    };

    if !Path::new(&dir).exists() {
        fs::DirBuilder::new().recursive(true).create(dir).unwrap();
    }
    db_dump(mgr, Atom::from(ware), Atom::from(tab), file.clone(), cb);
}

/**
* 将指定表的备份文件的所有记录恢复到指定的表
* @param mgr 表库及事务管理器
* @param ware 库名
* @param tab 表名
* @param file 文件的路径
* @param cb 将将指定表的备份文件的所有记录恢复到指定的表的结果的异步回调，成功返回空，失败返回原因描述
* @returns 返回同步将指定表的备份文件的所有记录恢复到指定的表的结果，成功返回空
* @throws 失败则抛出原因描述
*/
pub fn restore(
    mgr: &Mgr,
    ware: String,
    tab: String,
    file: String,
    cb: Box<FnOnce(Result<(), String>)>,
) {
    let dir = match file.as_str().rfind("/") {
        Some(v) => &file[0..v],
        None => {
            panic!("restore file Invalid:{}", file);
        }
    };
    if !Path::new(&dir).exists() {
        fs::DirBuilder::new().recursive(true).create(dir).unwrap();
    }

    if !Path::new(&file).exists() {
        fs::File::create(&file).expect("");
    }
    db_restore(
        mgr,
        Atom::from(ware),
        Atom::from(tab),
        Atom::from(file.clone()),
        cb,
    );
}

// 表的元信息
// pub fn tab_info(mgr: &Mgr, ware_name:String, tab_name: String) -> Option<Arc<TabMeta>> {
//     match mgr.tab_info(&Atom::from(ware_name), &Atom::from(tab_name)) {
//         Some(b) => b.tab_info(tab_name),
//         _ => None
//     }
// }

/*
* 数据库监听器
*/
pub struct JSDBMonitor {
    handler: Atom, //处理函数名称（js函数）
    factory: Arc<VMFactory>,
}

pub fn register_db_js_db_monitor(mgr: &Mgr, monitor: JSDBMonitor) {
    mgr.listen(Arc::new(monitor));
}

impl Monitor for JSDBMonitor {
    fn notify(&self, e: Event, mgr: Mgr) {
        //否则，将该事件投递到mqtt TODO
        if e.ware.as_str() != "file" {
            return;
        }
        match &e.other {
            &EventType::Tab {
                key: ref k,
                value: ref v,
            } => {
                let k = k.clone();
                let v = v.clone();
                let ware = e.ware.clone();
                let tab = e.tab.clone();
                let real_args = Box::new(move |vm: Arc<JS>| -> usize {
                    let event = vm.new_object();
                    vm.set_field(
                        &event,
                        String::from("event_name"),
                        &mut vm.new_str("db_change".to_string()).unwrap(),
                    );
                    vm.set_field(
                        &event,
                        String::from("ware"),
                        &mut vm.new_str(ware.as_str().to_string()).unwrap(),
                    ); // ware
                    vm.set_field(
                        &event,
                        String::from("tab"),
                        &mut vm.new_str(tab.as_str().to_string()).unwrap(),
                    ); // tab
                    vm.set_field(
                        &event,
                        String::from("key"),
                        &mut ptr_jstype(
                            vm.get_objs(),
                            vm.clone(),
                            Box::into_raw(Box::new(k)) as usize,
                            2886438122,
                        ),
                    ); //key
                    match v {
                        Some(v) => {
                            vm.set_field(
                                &event,
                                String::from("value"),
                                &mut ptr_jstype(
                                    vm.get_objs(),
                                    vm.clone(),
                                    Box::into_raw(Box::new(v)) as usize,
                                    2886438122,
                                ),
                            );
                        } //value,
                        None => (),
                    };
                    //mgr
                    ptr_jstype(
                        vm.get_objs(),
                        vm.clone(),
                        Box::into_raw(Box::new(mgr.clone())) as usize,
                        2976191628,
                    );
                    vm.new_undefined();
                    vm.new_object();
                    4
                });
                self.factory.call(
                    None,
                    self.handler.clone(),
                    real_args,
                    Atom::from("db_change".to_string() + " db task"),
                );
            }
            &EventType::Meta(ref _info) => (),
        }
    }
}

impl JSDBMonitor {
    //构建一个监听器
    pub fn new(handler: String, factory: VMFactory) -> JSDBMonitor {
        JSDBMonitor {
            handler: Atom::from(handler),
            factory: Arc::new(factory),
        }
    }
}
