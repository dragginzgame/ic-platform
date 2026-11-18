use candidate_runtime::{
    entrypoints::{self, UpdateError},
    executor::EntryError,
    system,
};
use std::cell::Cell;

thread_local! {
    static COMMITTED: Cell<u8> = const { Cell::new(0) };
}

/// Admit every caller to this qualification fixture.
fn admit() -> Result<(), u8> {
    Ok(())
}

fn decode(bytes: &[u8]) -> Result<u8, u8> {
    match bytes {
        [value @ 0..=3] => Ok(*value),
        _ => Err(1),
    }
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "The generated admission callback owns its error"
)]
fn fail(error: UpdateError<u8, u8>) -> ! {
    panic!("input refused: {error:?}")
}

/// Observe update and reply rollback for the selected trap mode.
#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "probe",
    guard = admit,
    decode = decode,
    max_bytes = 1,
    on_error = fail,
    manual_reply
)]
async fn probe(mode: u8) {
    let before = COMMITTED.with(Cell::get);
    if mode != 0 {
        COMMITTED.with(|value| value.set(before + 1));
    }
    assert_ne!(mode, 2, "initial-handler-trap");
    system::reply(&[COMMITTED.with(Cell::get)]).unwrap();
    assert_eq!(system::reply(&[]), Err(system::ReplyError::AlreadyReplied));
    assert_ne!(mode, 3, "after-reply-trap");
}

pub(super) fn assert_entry_refusal(expected: EntryError) {
    let result = entrypoints::update_manual(
        0,
        || -> Result<(), u8> { panic!("refused entry must not invoke guard") },
        |_| -> Result<(), u8> { panic!("refused entry must not invoke decoder") },
        |()| async { panic!("refused entry must not poll handler") },
    );
    assert!(matches!(result, Err(UpdateError::Entry(error)) if error == expected));
}

#[expect(
    unsafe_code,
    reason = "Fixed-ABI IC query export testing update-entry refusal"
)]
#[unsafe(export_name = "canister_query refuse_update")]
pub extern "C" fn refuse_update() {
    assert_entry_refusal(EntryError::NonReplicated);
    // No update task owns reply authority here. Normal return deliberately
    // yields DidNotReply, distinct from a failed assertion's CanisterTrapped.
}

/// Verify nested update refusal while an outer task owns the reply.
#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "nested_probe",
    guard = admit,
    decode = decode,
    max_bytes = 1,
    on_error = fail,
    manual_reply
)]
async fn nested_probe(mode: u8) {
    assert_eq!(mode, 0);
    assert_entry_refusal(EntryError::NestedEntry);
    system::reply(b"nested-refused").unwrap();
}
