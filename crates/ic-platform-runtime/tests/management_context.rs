//! The same management and Entropy callers type-check on native and Wasm.
#![cfg(all(feature = "call-ic0", feature = "system-raw"))]

use ic_platform_runtime::{
    call::{Call, CostOverflow, Wait},
    executor::{self, EntryError, TaskHandle},
    raw, system,
};
use ic_platform_types::Principal;
use std::panic::catch_unwind;

#[test]
fn outside_canister_refuses_before_observation_or_task_poll() {
    let balance: fn() -> u128 = raw::liquid_cycle_balance;
    let pages: fn() -> u64 = raw::stable64_size;
    let version: fn() -> u64 = system::canister_version;
    let price_http: fn(u64, u64) -> u128 = raw::cost_http_request_v1;
    let call = Call::new(Principal::management_canister(), "status", Wait::Unbounded);
    let price_call = || -> Result<u128, CostOverflow> { call.required_cycles() };

    assert!(catch_unwind(raw::cost_create_canister).is_err());
    assert!(catch_unwind(raw::msg_cycles_available).is_err());
    assert!(catch_unwind(|| raw::msg_cycles_accept(0)).is_err());
    assert!(catch_unwind(balance).is_err());
    assert!(catch_unwind(pages).is_err());
    assert!(catch_unwind(version).is_err());
    assert!(catch_unwind(|| price_http(64, 256)).is_err());
    assert!(catch_unwind(price_call).is_err());
    assert!(!raw::in_replicated_execution());
    let entry: Result<TaskHandle, EntryError> =
        executor::run_update(async { panic!("refused task was polled") });
    assert!(matches!(entry, Err(EntryError::NonReplicated)));
}
