if not defined PI_JS_PROXY_EXT_CRATES (
    set PI_JS_PROXY_EXT_CRATES=..\pi_core_lib;..\pi_serv_lib
)

if not defined PI_JS_PROXY_TS_PATH (
    set PI_JS_PROXY_TS_PATH=..\pi_pt
)

cargo clean --target-dir target\release\pi_serv.exe
cargo b --release

pause;
