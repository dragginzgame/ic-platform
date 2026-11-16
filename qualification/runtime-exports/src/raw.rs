//! Caller-owned exports; no macro package, Types, guard or executor is selected.
use candidate_runtime::raw as ic0;
use std::cell::Cell;

thread_local! { static VALUE: Cell<u8> = const { Cell::new(0) }; }

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
    VALUE.set(7);
}

#[expect(
    unsafe_code,
    reason = "Caller-owned lifecycle export in the approved fixture"
)]
#[unsafe(export_name = "canister_pre_upgrade")]
extern "C" fn before_upgrade() {
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
    VALUE.set(9);
}
