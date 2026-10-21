//! A standalone raw executor/system consumer without application identity types.
//!
//! ```compile_fail
//! use ic_platform::Principal;
//! ```

/// Submit a caller-owned task to the single registry executor.
pub fn submit(task: impl std::future::Future<Output = ()> + 'static) {
    ic_platform::executor::spawn_protected(task);
}

/// Copy raw caller bytes into caller-owned storage without constructing a Principal.
pub fn caller(bytes: &mut [u8]) {
    ic_platform::ic0::msg_caller_copy(bytes, 0);
}
