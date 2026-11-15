use candidate_runtime::{
    call::{Call, Outcome, Wait},
    entrypoints::UpdateError,
    system::{self, ReplyError},
};
use candidate_types::Principal;
use std::cell::Cell;

thread_local! {
    static STARTED: Cell<u8> = const { Cell::new(0) };
    static COMPLETED: Cell<u8> = const { Cell::new(0) };
    static DROPPED: Cell<u8> = const { Cell::new(0) };
}

fn admit() -> Result<(), u8> {
    Ok(())
}
fn decode(bytes: &[u8]) -> Result<(u8, Principal), u8> {
    let (&mode, target) = bytes.split_first().ok_or(1)?;
    if mode > 2 || target.is_empty() {
        return Err(2);
    }
    Ok((mode, Principal::try_from_slice(target).map_err(|_| 3)?))
}
fn empty(bytes: &[u8]) -> Result<(), u8> {
    if bytes.is_empty() { Ok(()) } else { Err(1) }
}
#[expect(
    clippy::needless_pass_by_value,
    reason = "Generated failure handlers own admission errors"
)]
fn fail(error: UpdateError<u8, u8>) -> ! {
    panic!("admission refused: {error:?}")
}

struct ObserveDrop(ReplyError);
impl Drop for ObserveDrop {
    fn drop(&mut self) {
        // On cleanup the restored suspended future cannot claim reply authority.
        assert_eq!(system::reply(b"destructor"), Err(self.0));
        DROPPED.with(|value| value.set(value.get() + 1));
    }
}

#[candidate_runtime::update(runtime = candidate_runtime, name = "await_probe", guard = admit,
    decode = decode, max_bytes = 30, on_error = fail, manual_reply)]
async fn await_probe((mode, target): (u8, Principal)) {
    let mut observer = ObserveDrop(ReplyError::NoActiveTask);
    STARTED.with(|value| value.set(value.get() + 1));
    let completion = Call::new(target, "probe", Wait::Unbounded)
        .owned_args(vec![1])
        .execute()
        .await
        .unwrap();
    let Outcome::Reply(bytes) = completion.outcome() else {
        panic!("peer must reply")
    };
    assert_eq!(bytes.len(), 1);
    COMPLETED.with(|value| value.set(value.get() + 1));
    assert_ne!(mode, 1, "trap-after-await");
    system::reply(bytes).unwrap();
    observer.0 = ReplyError::AlreadyReplied;
    assert_ne!(mode, 2, "trap-after-callback-reply");
}

#[candidate_runtime::update(runtime = candidate_runtime, name = "observe", guard = admit,
    decode = empty, max_bytes = 0, on_error = fail, manual_reply)]
async fn observe((): ()) {
    system::reply(&[
        STARTED.with(Cell::get),
        COMPLETED.with(Cell::get),
        DROPPED.with(Cell::get),
    ])
    .unwrap();
}
