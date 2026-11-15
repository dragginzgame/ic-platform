use candidate_runtime::{entrypoints::UpdateError, system};
use std::cell::Cell;

thread_local! {
    static COMMITTED: Cell<u8> = const { Cell::new(0) };
}

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

#[candidate_runtime::update(runtime = candidate_runtime, name = "probe", guard = admit,
    decode = decode, max_bytes = 1, on_error = fail, manual_reply)]
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
