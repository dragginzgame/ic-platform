//! Caller-owned exports; no macro package, Types, guard or executor is selected.

use candidate_runtime::raw as ic0;
use std::cell::Cell;

thread_local! { static VALUE: Cell<u8> = const { Cell::new(0) }; }
thread_local! { static CONTEXT_MUTATIONS: Cell<u8> = const { Cell::new(0) }; }

fn context_guard() {
    let replicated = ic0::in_replicated_execution();
    // Same ordering required by Entropy: context refusal precedes all custody
    // mutation and call submission. No dummy root or transport is involved.
    if replicated {
        CONTEXT_MUTATIONS.set(CONTEXT_MUTATIONS.get() + 1);
    }
    ic0::msg_reply_data_append(&[u8::from(replicated), CONTEXT_MUTATIONS.get()]);
    ic0::msg_reply();
}

#[expect(
    unsafe_code,
    reason = "Caller-owned IC export in the approved export fixture"
)]
#[unsafe(export_name = "canister_query context")]
extern "C" fn context_query() {
    context_guard();
}

#[expect(
    unsafe_code,
    reason = "Caller-owned IC export in the approved export fixture"
)]
#[unsafe(export_name = "canister_update context_update")]
extern "C" fn context_update() {
    context_guard();
}

fn caller() -> ([u8; 29], usize) {
    let size = ic0::msg_caller_size();
    assert!(size <= 29);
    let mut bytes = [0; 29];
    ic0::msg_caller_copy(&mut bytes[..size], 0);
    if size > 0 {
        let mut tail = [0; 29];
        ic0::msg_caller_copy(&mut tail[..size - 1], 1);
        assert_eq!(&tail[..size - 1], &bytes[1..size]);
    }
    (bytes, size)
}

fn observe() {
    let before = ic0::instruction_counter();
    let (bytes, size) = caller();
    ic0::msg_reply_data_append(&[VALUE.get(), ic0::is_controller(&bytes[..size]) as u8]);
    ic0::msg_reply_data_append(&bytes[..size]);
    let after = ic0::instruction_counter();
    assert!(after >= before);
    ic0::msg_reply_data_append(&before.to_le_bytes());
    ic0::msg_reply_data_append(&after.to_le_bytes());
    ic0::msg_reply();
}

#[expect(
    unsafe_code,
    reason = "Caller-owned IC export in the approved export fixture"
)]
#[unsafe(export_name = "canister_query observe")]
extern "C" fn query() {
    observe();
}

#[expect(
    unsafe_code,
    reason = "Caller-owned IC export in the approved export fixture"
)]
#[unsafe(export_name = "canister_update observe_update")]
extern "C" fn update_observe() {
    observe();
}

#[expect(
    unsafe_code,
    reason = "Caller-owned IC export in the approved export fixture"
)]
#[unsafe(export_name = "canister_update probe")]
extern "C" fn update() {
    let size = ic0::msg_arg_data_size();
    if size == 0 {
        ic0::msg_reply();
        return;
    }
    let mut mode = [0];
    ic0::msg_arg_data_copy(&mut mode, 0);
    match mode[0] {
        0 => {
            assert_eq!(size, 5);
            let mut bytes = [0; 4];
            ic0::msg_arg_data_copy(&mut bytes, 1);
            ic0::msg_arg_data_copy(&mut [], size);
            ic0::msg_reply_data_append(&bytes[..2]);
            ic0::msg_reply_data_append(&[]);
            ic0::msg_reply_data_append(&bytes[2..]);
            ic0::msg_reply();
        }
        1 => {
            let (bytes, size) = caller();
            if ic0::is_controller(&bytes[..size]) != 1 {
                ic0::trap(b"controller required");
            }
            VALUE.set(VALUE.get() + 1);
            ic0::msg_reply();
        }
        2 => {
            VALUE.set(99);
            ic0::msg_reply_data_append(b"discarded");
            ic0::msg_reply();
            ic0::trap(b"rollback diagnostic\0bytes");
        }
        3 => ic0::msg_arg_data_copy(&mut mode, size),
        4 => ic0::msg_arg_data_copy(&mut [], usize::MAX),
        5 => ic0::msg_caller_copy(&mut mode, ic0::msg_caller_size()),
        6 => ic0::msg_caller_copy(&mut [], usize::MAX),
        7 => {
            ic0::msg_reply();
            ic0::msg_reply();
        }
        8 => {
            ic0::msg_reply();
            ic0::msg_reply_data_append(&[]);
        }
        _ => ic0::trap(b"unknown mode"),
    }
}

#[expect(
    unsafe_code,
    reason = "Caller-owned lifecycle export in the approved fixture"
)]
#[unsafe(export_name = "canister_init")]
extern "C" fn initialize() {
    let _ = ic0::instruction_counter();
    let (bytes, size) = caller();
    assert_eq!(ic0::is_controller(&bytes[..size]), 1);
    assert_eq!(ic0::msg_arg_data_size(), 0);
    assert_eq!(ic0::stable64_size(), 0);
    assert_eq!(ic0::stable64_grow(2), Some(0));
    let mut zero = [1; 3];
    ic0::stable64_read(65535, &mut zero);
    assert_eq!(zero, [0; 3]);
    VALUE.set(7);
}

#[expect(
    unsafe_code,
    reason = "Caller-owned lifecycle export in the approved fixture"
)]
#[unsafe(export_name = "canister_pre_upgrade")]
extern "C" fn before_upgrade() {
    ic0::stable64_write(0, &[VALUE.get()]);
    let _ = ic0::instruction_counter();
    let _ = caller();
}

#[expect(
    unsafe_code,
    reason = "Caller-owned lifecycle export in the approved fixture"
)]
#[unsafe(export_name = "canister_post_upgrade")]
extern "C" fn after_upgrade() {
    let _ = ic0::instruction_counter();
    assert_eq!(ic0::msg_arg_data_size(), 0);
    let _ = caller();
    let mut value = [0];
    ic0::stable64_read(0, &mut value);
    VALUE.set(value[0]);
}

#[expect(
    unsafe_code,
    reason = "Caller-owned IC export in the approved export fixture"
)]
#[unsafe(export_name = "canister_update system_probe")]
extern "C" fn system_probe() {
    let mut mode = [0];
    ic0::msg_arg_data_copy(&mut mode, 0);
    match mode[0] {
        0 => {
            assert_eq!(ic0::stable64_grow(u64::MAX), None);
            let pages = ic0::stable64_size();
            ic0::stable64_write(pages * 65536, &[]);
            ic0::stable64_read(pages * 65536, &mut []);
            ic0::stable64_write(65535, &[0, 128, 255]);
            let mut bytes = [0; 3];
            ic0::stable64_read(65535, &mut bytes);
            assert_eq!(bytes, [0, 128, 255]);
            ic0::msg_reply_data_append(&pages.to_le_bytes());
        }
        1 => {
            assert_eq!(ic0::stable64_grow(1), Some(2));
            ic0::stable64_write(65535, &[9, 9, 9]);
            assert_eq!(ic0::cycles_burn(1u128 << 64), 1u128 << 64);
            ic0::trap(b"stable and cycle rollback");
        }
        2 => ic0::stable64_read(u64::MAX, &mut [0]),
        3 => ic0::stable64_write(1u64 << 32, &[1]),
        4 => ic0::stable64_read(u64::MAX, &mut []),
        5 => {
            let before = ic0::liquid_cycle_balance();
            let requested = (1u128 << 64) + 7;
            let burned = ic0::cycles_burn(requested);
            let after = ic0::liquid_cycle_balance();
            assert_eq!(burned, requested);
            assert!(after <= before - burned);
            assert_eq!(ic0::cycles_burn(0), 0);
            ic0::msg_reply_data_append(&burned.to_le_bytes());
        }
        6 => {
            let before = ic0::liquid_cycle_balance();
            let burned = ic0::cycles_burn(u128::MAX);
            assert!(burned <= before);
            assert!(burned > 0);
            ic0::msg_reply_data_append(&burned.to_le_bytes());
        }
        7 => {
            let cost = ic0::cost_http_request_v1(64, 256);
            assert!(cost > 0);
            assert!(ic0::cost_http_request_v1(64, 512) > cost);
            ic0::msg_reply_data_append(&cost.to_le_bytes());
        }
        _ => ic0::trap(b"unknown system mode"),
    }
    ic0::msg_reply();
}

#[expect(
    unsafe_code,
    reason = "Caller-owned IC export in the approved export fixture"
)]
#[unsafe(export_name = "canister_query stable_observe")]
extern "C" fn stable_observe() {
    ic0::msg_reply_data_append(&ic0::stable64_size().to_le_bytes());
    let mut bytes = [0; 3];
    ic0::stable64_read(65535, &mut bytes);
    ic0::msg_reply_data_append(&bytes);
    ic0::msg_reply();
}
