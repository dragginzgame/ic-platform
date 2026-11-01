//! Compatibility paths delegating to the independently released runtime owner.

#[cfg(feature = "cdk")]
pub use ic_platform_macros::{
    heartbeat, init, inspect_message, on_low_wasm_memory, post_upgrade, pre_upgrade, query, update,
};
pub use ic_platform_runtime::cdk::{api, call, futures, println, reject_code, stable};
