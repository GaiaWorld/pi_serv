export PI_JS_PROXY_EXT_CRATES="../pi_core_lib;../pi_serv_lib"
export PI_JS_PROXY_TS_PATH="../pi_pt"

cargo clean --target-dir target/debug/pi_serv
cargo b
