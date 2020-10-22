use std::path::Path;
use std::fs::read;
use time::{start_secs, run_second, now_second};
use guid::Guid;
use chrono::Utc;

/**
* 同步的读取指定文件的数据
* @param path 文件的路径
* @returns 返回文件的数据
*/
pub fn read_file(path: &str) -> Vec<u8>{
    let r = read(Path::new(path));
    let data = r.expect(&(String::from("file is not exist, path:") + path));
    return data;
}

pub fn start_seconds() -> u64 {
    start_secs()
}

pub fn run_seconds() -> u64 {
    run_second()
}

pub fn now_seconds() -> u64 {
    now_second()
}

pub fn now_nano() -> u64 {
    let dt = Utc::now();
    dt.timestamp_nanos() as u64
}
