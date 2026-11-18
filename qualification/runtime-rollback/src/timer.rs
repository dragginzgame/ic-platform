//! Timer roots exercise the same callback cleanup owner as update roots.
use candidate_runtime::{
    call::{Call, Outcome, Wait},
    executor::{self, EntryError, TaskHandle, TaskState},
    system::{self, ReplyError},
};
use std::cell::{Cell, RefCell};

thread_local! {
    static MODE: Cell<u8> = const { Cell::new(0) };
    static FIRED: Cell<u8> = const { Cell::new(0) };
    static DROPPED: Cell<u8> = const { Cell::new(0) };
    static HANDLE: RefCell<Option<TaskHandle>> = const { RefCell::new(None) };
}

fn admit() -> Result<(), u8> {
    Ok(())
}
fn decode(bytes: &[u8]) -> Result<u8, u8> {
    match bytes {
        [mode @ 0..=4] => Ok(*mode),
        _ => Err(1),
    }
}
fn empty(bytes: &[u8]) -> Result<(), u8> {
    if bytes.is_empty() { Ok(()) } else { Err(1) }
}
fn encode(bytes: Vec<u8>) -> Result<Vec<u8>, u8> {
    Ok(bytes)
}
fn fail<T: std::fmt::Debug>(error: T) -> ! {
    panic!("timer fixture: {error:?}")
}

#[candidate_runtime::update(runtime = candidate_runtime, name = "timer_arm", guard = admit,
    decode = decode, max_bytes = 1, encode = encode, on_error = fail)]
fn arm(mode: u8) -> Vec<u8> {
    // The arm update commits before the timer executes. One private stable page
    // belongs to this single-attempt fixture. Byte 0 is its schema version.
    assert_eq!(candidate_runtime::raw::stable64_grow(1), Some(0));
    candidate_runtime::raw::stable64_write(0, &[1, mode, 1]);
    MODE.set(mode);
    let deadline = system::time().checked_add(1_000_000_000).unwrap();
    assert_eq!(system::global_timer_set(deadline), 0);
    if mode == 3 {
        assert_eq!(system::global_timer_set(0), deadline);
    }
    system::canister_version().to_le_bytes().to_vec()
}

#[candidate_runtime::query(runtime = candidate_runtime, name = "timer_observe", guard = admit,
    decode = empty, max_bytes = 0, encode = encode, on_error = fail)]
fn observe((): ()) -> Vec<u8> {
    assert!(matches!(
        executor::run_global_timer(async { panic!("query must not poll") }),
        Err(EntryError::NonReplicated | EntryError::NestedEntry)
    ));
    let state = HANDLE.with(
        |handle| match handle.borrow().as_ref().map(TaskHandle::state) {
            None => 0,
            Some(TaskState::Running) => 1,
            Some(TaskState::Completed) => 2,
            Some(TaskState::TerminatedByCleanup) => 3,
        },
    );
    let mut bytes = vec![FIRED.get(), DROPPED.get(), state];
    bytes.extend_from_slice(&system::canister_version().to_le_bytes());
    bytes
}

#[candidate_runtime::update(runtime = candidate_runtime, name = "timer_peer", guard = admit,
    decode = empty, max_bytes = 0, encode = encode, on_error = fail)]
fn peer((): ()) -> Vec<u8> {
    vec![42]
}

struct Cleanup(bool);
impl Drop for Cleanup {
    fn drop(&mut self) {
        assert_eq!(system::reply(&[]), Err(ReplyError::NoActiveTask));
        assert!(matches!(
            executor::run_global_timer(async {}),
            Err(EntryError::NestedEntry)
        ));
        DROPPED.set(DROPPED.get() + 1);
        assert!(!self.0, "timer cleanup trap");
    }
}

#[expect(
    unsafe_code,
    reason = "installed fixture owns the IC global timer export"
)]
#[unsafe(export_name = "canister_global_timer")]
extern "C" fn timer() {
    FIRED.set(FIRED.get() + 1);
    assert_eq!(system::global_timer_set(0), 0);
    let mode = MODE.get();
    let handle = executor::run_global_timer(async move {
        assert_eq!(system::reply(&[]), Err(ReplyError::NoActiveTask));
        let _cleanup = Cleanup(mode == 2);
        assert_ne!(mode, 4, "initial timer poll trap");
        candidate_runtime::raw::stable64_write(2, &[2]);
        let response = Call::new(system::canister_self(), "timer_peer", Wait::Unbounded)
            .execute()
            .await
            .unwrap();
        assert!(matches!(response.outcome(), Outcome::Reply(bytes) if bytes == &[42]));
        assert_eq!(system::reply(&[]), Err(ReplyError::NoActiveTask));
        candidate_runtime::raw::stable64_write(2, &[3]);
        assert_eq!(mode, 0, "timer callback trap");
    })
    .unwrap();
    HANDLE.with(|slot| *slot.borrow_mut() = Some(handle));
}

#[candidate_runtime::query(runtime = candidate_runtime, name = "timer_record", guard = admit,
    decode = empty, max_bytes = 0, encode = encode, on_error = fail)]
fn record((): ()) -> Vec<u8> {
    let mut bytes = vec![0; 3];
    candidate_runtime::raw::stable64_read(0, &mut bytes);
    bytes
}

/// Upgrade only the fixture-owned record. Keep its observed dispatch stage.
pub(super) fn migrate_record() {
    if candidate_runtime::raw::stable64_size() == 0 {
        return;
    }
    let bytes = record(());
    match bytes[0] {
        1 => candidate_runtime::raw::stable64_write(0, &[2]),
        2 => {}
        version => panic!("unknown timer record version {version}"),
    }
}
