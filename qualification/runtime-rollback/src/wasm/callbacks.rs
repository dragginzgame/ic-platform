use candidate_runtime::{
    call::{Call, Outcome, RejectCode, SubmitError, Wait},
    entrypoints::{self, UpdateError},
    executor::{EntryError, TaskHandle, TaskState},
    system::{self, ReplyError},
};
use candidate_types::Principal;
use std::{
    cell::{Cell, RefCell},
    future::{Future, poll_fn},
    pin::Pin,
    rc::{Rc, Weak},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll, Wake, Waker},
};

thread_local! {
    static STALE: RefCell<Option<Waker>> = const { RefCell::new(None) };
    static CAPTURE_POLLS: Cell<u8> = const { Cell::new(0) };
    static DISPATCH: RefCell<Vec<bool>> = const { RefCell::new(Vec::new()) };
    static CUSTODY: RefCell<Vec<Weak<()>>> = const { RefCell::new(Vec::new()) };
    static HANDLES: RefCell<Vec<TaskHandle>> = const { RefCell::new(Vec::new()) };
    static STARTED: Cell<u8> = const { Cell::new(0) };
    static COMPLETED: Cell<u8> = const { Cell::new(0) };
    static DROPPED: Cell<u8> = const { Cell::new(0) };
}

/// Admit every caller to this qualification fixture.
fn admit() -> Result<(), u8> {
    Ok(())
}

fn decode(bytes: &[u8]) -> Result<(u8, Principal), u8> {
    let (&mode, target) = bytes.split_first().ok_or(1)?;
    if mode > 6 || target.is_empty() {
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

struct ObserveDrop(Option<ReplyError>, bool, Rc<()>);
impl Drop for ObserveDrop {
    fn drop(&mut self) {
        assert_eq!(Rc::strong_count(&self.2), 1);
        // On cleanup the restored suspended future cannot claim reply authority.
        if let Some(expected) = self.0 {
            assert_eq!(system::reply(b"destructor"), Err(expected));
        }
        // Re-enter the real adapter from user destruction. It must refuse
        // before invoking admission callbacks, without a RefCell borrow trap.
        super::exports::assert_entry_refusal(EntryError::NestedEntry);
        DROPPED.with(|value| value.set(value.get() + 1));
        assert!(!self.1, "trap-during-cleanup");
    }
}

// Retain the public handle returned by the same adapter used by generated
// exports. The generated wrapper intentionally drops it.
#[expect(
    unsafe_code,
    reason = "Fixed-ABI IC update export in the installed qualification canister"
)]
#[unsafe(export_name = "canister_update await_probe")]
pub extern "C" fn tracked_await_probe() {
    let handle = entrypoints::update_manual(30, admit, decode, await_probe)
        .unwrap_or_else(|error| fail(error));
    assert_eq!(handle.state(), TaskState::Running);
    HANDLES.with(|handles| {
        let mut handles = handles.borrow_mut();
        assert!(handles.len() < 8, "bounded qualification corpus");
        handles.push(handle);
    });
}

async fn await_probe((mode, target): (u8, Principal)) {
    // Consumer-owned record: false = dispatched; true = reply observed.
    // The prefix commits at suspension; a trapping callback rolls its update back.
    let dispatch = DISPATCH.with(|records| {
        let mut records = records.borrow_mut();
        assert!(records.len() < 8, "bounded qualification corpus");
        let id = records.len();
        records.push(false);
        id
    });
    let custody = Rc::new(());
    CUSTODY.with(|records| {
        let mut records = records.borrow_mut();
        assert!(records.len() < 8, "bounded qualification corpus");
        records.push(Rc::downgrade(&custody));
    });
    let mut observer = ObserveDrop(Some(ReplyError::NoActiveTask), mode == 3, custody);
    STARTED.with(|value| value.set(value.get() + 1));
    if mode == 4 {
        // An unpolled call must not increment the peer. A first-polled call
        // must still reach it after its borrowed storage and local interest die.
        drop(
            Call::new(target, "probe", Wait::Unbounded)
                .owned_args(vec![1])
                .execute(),
        );
        let method = String::from("probe");
        let args = vec![1];
        let mut abandoned = Call::new(target, &method, Wait::Unbounded)
            .args(&args)
            .execute();
        poll_fn(|cx| {
            assert!(Pin::new(&mut abandoned).poll(cx).is_pending());
            Poll::Ready(())
        })
        .await;
        drop(abandoned);
    }
    let mut first = Call::new(target, "probe", Wait::Unbounded)
        .owned_args(vec![1])
        .execute();
    let mut orphan = None;
    if mode == 5 {
        poll_fn(|cx| {
            assert!(Pin::new(&mut first).poll(cx).is_pending());
            Poll::Ready(())
        })
        .await;
        let mut args = vec![0];
        args.extend_from_slice(target.as_slice());
        let mut second = Call::new(target, "delayed_reply", Wait::Unbounded)
            .owned_args(args)
            .execute();
        poll_fn(|cx| {
            assert!(Pin::new(&mut second).poll(cx).is_pending());
            Poll::Ready(())
        })
        .await;
        orphan = Some(second);
    }
    let completion = first.await.unwrap();
    if let Some(second) = orphan.as_mut() {
        let pending = poll_fn(|cx| Poll::Ready(Pin::new(&mut *second).poll(cx).is_pending())).await;
        if !pending {
            // A failed ordering check must not look like the intentional trap.
            system::reply(b"unexpected-callback-order").unwrap();
            observer.0 = Some(ReplyError::AlreadyReplied);
            return;
        }
    }
    let Outcome::Reply(bytes) = completion.outcome() else {
        panic!("peer must reply")
    };
    assert_eq!(bytes.len(), 1);
    DISPATCH.with(|records| records.borrow_mut()[dispatch] = true);
    COMPLETED.with(|value| value.set(value.get() + 1));
    assert!(!matches!(mode, 1 | 3 | 5), "trap-after-await");
    if mode == 6 {
        // Exercise missing application reply without the destructor supplying one.
        observer.0 = None;
        return;
    }
    system::reply(bytes).unwrap();
    observer.0 = Some(ReplyError::AlreadyReplied);
    assert_ne!(mode, 2, "trap-after-callback-reply");
    drop(orphan);
}

/// Read the fixture’s committed counters and task outcomes.
#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "observe",
    guard = admit,
    decode = empty,
    max_bytes = 0,
    on_error = fail,
    manual_reply
)]
async fn observe((): ()) {
    let mut observation = vec![
        STARTED.with(Cell::get),
        COMPLETED.with(Cell::get),
        DROPPED.with(Cell::get),
    ];
    HANDLES.with(|handles| {
        observation.extend(handles.borrow().iter().map(|handle| match handle.state() {
            TaskState::Running => 0,
            TaskState::Completed => 1,
            TaskState::TerminatedByCleanup => 2,
        }));
    });
    system::reply(&observation).unwrap();
}

/// Read the response-observed flag for each dispatched handler.
#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "observe_dispatch",
    guard = admit,
    decode = empty,
    max_bytes = 0,
    on_error = fail,
    manual_reply
)]
async fn observe_dispatch((): ()) {
    let records: Vec<u8> =
        DISPATCH.with(|records| records.borrow().iter().copied().map(u8::from).collect());
    system::reply(&records).unwrap();
}

// Weak records cannot themselves retain the handler-owned allocation.
/// Read retained handler ownership after completion or cleanup.
#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "observe_custody",
    guard = admit,
    decode = empty,
    max_bytes = 0,
    on_error = fail,
    manual_reply
)]
async fn observe_custody((): ()) {
    let counts: Vec<u8> = CUSTODY.with(|records| {
        records
            .borrow()
            .iter()
            .map(|record| u8::try_from(record.strong_count()).unwrap())
            .collect()
    });
    system::reply(&counts).unwrap();
}

// Keep the first completion alive across a different callback context. Neither
// peer path accepts attached cycles, so both attachments must be returned.
/// Observe the call result and cycles returned by the callback.
#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "refund_probe",
    guard = admit,
    decode = decode,
    max_bytes = 30,
    on_error = fail,
    manual_reply
)]
async fn refund_probe((mode, target): (u8, Principal)) {
    assert!(mode <= 2);
    let reply_cycles = if mode == 2 { (1_u128 << 64) + 17 } else { 17 };
    let reject_cycles = if mode == 2 { (1_u128 << 65) + 29 } else { 29 };
    let mut first = Call::new(target, "probe", Wait::Unbounded)
        .owned_args(vec![0])
        .cycles(reply_cycles)
        .execute();
    struct ObservedWake(AtomicBool);
    impl Wake for ObservedWake {
        fn wake(self: Arc<Self>) {
            self.0.store(true, Ordering::SeqCst);
        }
    }
    let observed = Arc::new(ObservedWake(AtomicBool::new(false)));
    let early = if mode == 0 {
        Some((&mut first).await.unwrap())
    } else {
        // Submit now, but do not consume this completion until after the
        // second callback. The waker proves publication happened first.
        let waker = Waker::from(Arc::clone(&observed));
        assert!(
            Pin::new(&mut first)
                .poll(&mut Context::from_waker(&waker))
                .is_pending()
        );
        None
    };
    let rejected = Call::new(target, "probe", Wait::Unbounded)
        .owned_args(vec![2])
        .cycles(reject_cycles)
        .execute()
        .await
        .unwrap();
    let reply = early.unwrap_or_else(|| {
        assert!(
            observed.0.load(Ordering::SeqCst),
            "first callback must have published before delayed polling"
        );
        let Poll::Ready(completion) =
            Pin::new(&mut first).poll(&mut Context::from_waker(Waker::noop()))
        else {
            panic!("published completion must be ready without another callback")
        };
        completion.unwrap()
    });
    let Outcome::Rejected(error) = rejected.outcome() else {
        panic!("peer trap must be an ordinary rejected completion")
    };
    assert_eq!(error.code(), 5);
    assert_eq!(error.kind(), Some(RejectCode::CanisterError));
    assert!(!error.message().is_empty());
    assert_eq!(rejected.refunded_cycles(), Some(reject_cycles));
    assert_eq!(reply.refunded_cycles(), Some(reply_cycles));
    assert!(matches!(reply.outcome(), Outcome::Reply(bytes) if bytes == &[9]));
    let mut result = reply.refunded_cycles().unwrap().to_le_bytes().to_vec();
    result.extend_from_slice(&rejected.refunded_cycles().unwrap().to_le_bytes());
    result.push(5);
    system::reply(&result).unwrap();
}

/// Exercise call resource refusals and report the result.
#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "resource_probe",
    guard = admit,
    decode = decode,
    max_bytes = 30,
    on_error = fail,
    manual_reply
)]
async fn resource_probe((mode, target): (u8, Principal)) {
    assert_eq!(mode, 0);
    for attachment in [u128::MAX, 1_u128 << 100] {
        let request = || {
            Call::new(target, "probe", Wait::Unbounded)
                .owned_args(vec![1])
                .cycles(attachment)
        };
        let required = request().required_cycles();
        let notify_error = request()
            .notify()
            .expect_err("oversized attachment must refuse");
        let mut future = request().execute();
        let call_error = poll_fn(|cx| {
            let Poll::Ready(result) = Pin::new(&mut future).poll(cx) else {
                panic!("resource refusal must be synchronous on first poll")
            };
            Poll::Ready(result.expect_err("oversized attachment must refuse"))
        })
        .await;
        for error in [notify_error, call_error] {
            if attachment == u128::MAX {
                assert!(required.is_err());
                assert_eq!(error, SubmitError::CostOverflow);
            } else {
                let SubmitError::InsufficientBalance {
                    available,
                    required: observed,
                } = error
                else {
                    panic!("expected typed balance refusal")
                };
                assert_eq!(observed, required.unwrap());
                assert!(observed > attachment);
                assert!(available < observed);
            }
        }
    }
    let completion = Call::new(target, "probe", Wait::Unbounded)
        .owned_args(vec![0])
        .execute()
        .await
        .unwrap();
    let Outcome::Reply(bytes) = completion.outcome() else {
        panic!("ordinary call after refusal must succeed")
    };
    assert_eq!(bytes, &[9]);
    system::reply(bytes).unwrap();
}

/// Exercise one-way submission across commit and rollback.
#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "notify_probe",
    guard = admit,
    decode = decode,
    max_bytes = 30,
    on_error = fail,
    manual_reply
)]
async fn notify_probe((mode, target): (u8, Principal)) {
    assert!(mode <= 2);
    let peer_mode = if mode == 2 { 2 } else { 1 };
    // The receipt is local queue admission even when the peer will trap.
    Call::new(target, "probe", Wait::Unbounded)
        .owned_args(vec![peer_mode])
        .notify()
        .unwrap();
    assert_ne!(mode, 1, "trap-after-notify-before-commit");
    let read = Call::new(target, "probe", Wait::Unbounded)
        .owned_args(vec![0])
        .execute()
        .await
        .unwrap();
    let Outcome::Reply(bytes) = read.outcome() else {
        panic!("independent read after notification must succeed")
    };
    system::reply(bytes).unwrap();
}

/// Exercise a saved wake after its original task has retired.
#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "stale_probe",
    guard = admit,
    decode = decode,
    max_bytes = 30,
    on_error = fail,
    manual_reply
)]
async fn stale_probe((mode, target): (u8, Principal)) {
    assert!(mode <= 1);
    if mode == 0 {
        poll_fn(|cx| {
            CAPTURE_POLLS.with(|value| value.set(value.get() + 1));
            STALE.with(|stored| assert!(stored.borrow_mut().replace(cx.waker().clone()).is_none()));
            Poll::Ready(())
        })
        .await;
        system::reply(&[CAPTURE_POLLS.with(Cell::get)]).unwrap();
        return;
    }
    let mut stale = STALE.with(|stored| stored.borrow_mut().take());
    assert!(stale.is_some(), "capture task must complete first");
    let mut call = Call::new(target, "probe", Wait::Unbounded)
        .owned_args(vec![0])
        .execute();
    let mut polls = 0_u8;
    let completion = poll_fn(|cx| {
        polls += 1;
        assert!(polls <= 2, "stale wake must not schedule the live task");
        let result = Pin::new(&mut call).poll(cx);
        if polls == 1 {
            assert!(result.is_pending());
            let old = stale.take().unwrap();
            old.wake_by_ref();
            old.wake();
        }
        result
    })
    .await
    .unwrap();
    assert_eq!(polls, 2);
    let Outcome::Reply(bytes) = completion.outcome() else {
        panic!("live call must succeed after stale wakes")
    };
    assert_eq!(bytes, &[11]);
    system::reply(&[CAPTURE_POLLS.with(Cell::get), polls, bytes[0]]).unwrap();
}

/// Observe the selected bounded-wait outcome.
#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "bounded_probe",
    guard = admit,
    decode = decode,
    max_bytes = 30,
    on_error = fail,
    manual_reply
)]
async fn bounded_probe((mode, target): (u8, Principal)) {
    assert!(mode <= 1);
    let seconds = if mode == 0 { 1 } else { 30 };
    let completion = Call::new(target, "probe", Wait::Bounded { seconds })
        .owned_args(vec![1])
        .execute()
        .await
        .unwrap();
    if mode == 0 {
        let Outcome::OutcomeUnknown(error) = completion.outcome() else {
            panic!("expired bounded call must expose uncertain outcome")
        };
        assert_eq!(error.code(), 6);
        assert_eq!(error.kind(), Some(RejectCode::SysUnknown));
        system::reply(&[6]).unwrap();
    } else {
        let Outcome::Reply(bytes) = completion.outcome() else {
            panic!("bounded call with sufficient time must reply")
        };
        system::reply(bytes).unwrap();
    }
}

/// Fill the request queue and return its accepted count and refusal code.
#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "submission_probe",
    guard = admit,
    decode = decode,
    max_bytes = 30,
    on_error = fail,
    manual_reply
)]
async fn submission_probe((mode, target): (u8, Principal)) {
    assert_eq!(mode, 0);
    for accepted in 0_u32..1024 {
        let result = Call::new(target, "probe", Wait::Unbounded)
            .owned_args(vec![0])
            .notify();
        match result {
            Ok(_) => {}
            Err(SubmitError::Refused(code)) => {
                let mut future = Call::new(target, "probe", Wait::Unbounded)
                    .owned_args(vec![0])
                    .execute();
                poll_fn(|cx| {
                    let Poll::Ready(result) = Pin::new(&mut future).poll(cx) else {
                        panic!("full queue must refuse on first poll")
                    };
                    assert_eq!(result, Err(SubmitError::Refused(code)));
                    Poll::Ready(())
                })
                .await;
                let mut bytes = accepted.to_le_bytes().to_vec();
                bytes.extend_from_slice(&code.get().to_le_bytes());
                system::reply(&bytes).unwrap();
                return;
            }
            Err(error) => panic!("expected system refusal, not preflight error: {error:?}"),
        }
    }
    panic!("bounded corpus did not reach system queue refusal");
}

/// Delay the peer response by one inter-canister hop so the first callback can trap.
#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "delayed_reply",
    guard = admit,
    decode = decode,
    max_bytes = 30,
    on_error = fail,
    manual_reply
)]
async fn delayed_reply((mode, target): (u8, Principal)) {
    assert_eq!(mode, 0);
    let completion = Call::new(target, "probe", Wait::Unbounded)
        .owned_args(vec![1])
        .execute()
        .await
        .unwrap();
    let Outcome::Reply(bytes) = completion.outcome() else {
        panic!("peer hop must reply")
    };
    system::reply(bytes).unwrap();
}

/// Observe the refund after partial cycle acceptance.
#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "partial_refund_probe",
    guard = admit,
    decode = decode,
    max_bytes = 30,
    on_error = fail,
    manual_reply
)]
async fn partial_refund_probe((mode, target): (u8, Principal)) {
    assert_eq!(mode, 0);
    let accepted = Call::new(target, "accept", Wait::Unbounded)
        .cycles(17)
        .execute()
        .await
        .unwrap();
    let Outcome::Reply(bytes) = accepted.outcome() else {
        panic!("accepting peer must reply")
    };
    assert_eq!(bytes, &5_u128.to_le_bytes());
    assert_eq!(accepted.refunded_cycles(), Some(12));
    let trapped = Call::new(target, "accept_trap", Wait::Unbounded)
        .cycles(29)
        .execute()
        .await
        .unwrap();
    assert!(matches!(trapped.outcome(), Outcome::Rejected(error) if error.code() == 5));
    assert_eq!(trapped.refunded_cycles(), Some(29));
    assert_eq!(accepted.refunded_cycles(), Some(12));
    let mut result = bytes.clone();
    result.extend_from_slice(&accepted.refunded_cycles().unwrap().to_le_bytes());
    result.extend_from_slice(&trapped.refunded_cycles().unwrap().to_le_bytes());
    system::reply(&result).unwrap();
}

/// Share a held endpoint across two independent ingress calls, one with a deadline.
/// Await an independently releasable peer with the selected wait policy.
#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "held_probe",
    guard = admit,
    decode = decode,
    max_bytes = 30,
    on_error = fail,
    manual_reply
)]
async fn held_probe((mode, peer): (u8, Principal)) {
    assert!(mode <= 1);
    let wait = if mode == 0 {
        Wait::Bounded { seconds: 10 }
    } else {
        Wait::Unbounded
    };
    let completion = Call::new(peer, "hold_upgrade", wait)
        .owned_args(system::canister_self().as_slice().to_vec())
        .execute()
        .await
        .unwrap();
    match completion.outcome() {
        Outcome::OutcomeUnknown(error) => {
            system::reply(&[u8::try_from(error.code()).unwrap()]).unwrap()
        }
        Outcome::Reply(bytes) => system::reply(bytes).unwrap(),
        Outcome::Rejected(error) => panic!("unexpected held-call rejection: {error:?}"),
    }
}
