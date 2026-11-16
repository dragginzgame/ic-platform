use candidate_runtime::call::{Call, CallFuture, Completion, Outcome, SubmitError, Wait};
use candidate_types::Principal;
use std::{
    cell::{Cell, RefCell},
    future::Future,
    num::NonZeroU32,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll, Wake, Waker},
};

mod executor;
mod handoff;
mod interleaved;
mod update;

thread_local! {
    static PHASE: Cell<u32> = const { Cell::new(0) };
    static PENDING: RefCell<Option<CallFuture<'static, 'static>>> = const { RefCell::new(None) };
    static TERMINAL: RefCell<Option<CallFuture<'static, 'static>>> = const { RefCell::new(None) };
}
static WAKES: AtomicUsize = AtomicUsize::new(0);
struct Counter;
impl Wake for Counter {
    fn wake(self: Arc<Self>) {
        WAKES.fetch_add(1, Ordering::SeqCst);
    }
}

fn poll(future: &mut CallFuture<'_, '_>) -> Poll<Result<Completion, SubmitError>> {
    let waker = Waker::from(Arc::new(Counter));
    Pin::new(future).poll(&mut Context::from_waker(&waker))
}

fn start(method: &'static str, wait: Wait) {
    let mut future = Call::new(Principal::anonymous(), method, wait)
        .owned_args(vec![0, 128, 255])
        .cycles(9)
        .execute();
    assert!(poll(&mut future).is_pending());
    assert!(poll(&mut future).is_pending());
    PENDING.with(|pending| {
        assert!(pending.borrow().is_none());
        *pending.borrow_mut() = Some(future);
    });
}

fn finish() -> Completion {
    // Never poll or drop a future while the fixture's own slot is borrowed.
    let mut future = PENDING.with(|pending| pending.borrow_mut().take().unwrap());
    let Poll::Ready(Ok(completion)) = poll(&mut future) else {
        panic!("expected completed call")
    };
    completion
}

pub(super) fn step() {
    let phase = PHASE.with(|phase| {
        let old = phase.get();
        phase.set(old + 1);
        old
    });
    match phase {
        0 => {
            drop(Call::new(Principal::anonymous(), "never", Wait::Unbounded).execute());
            start("reply", Wait::Bounded { seconds: 7 });
            assert_eq!(WAKES.load(Ordering::SeqCst), 0);
        }
        1 => {
            assert_eq!(WAKES.load(Ordering::SeqCst), 1);
            let completion = finish();
            assert_eq!(completion.refunded_cycles(), Some((1_u128 << 100) + 9));
            assert_eq!(completion.outcome(), &Outcome::Reply(vec![0, 128, 255]));
            start("reject", Wait::Unbounded);
        }
        2 => {
            assert_eq!(WAKES.load(Ordering::SeqCst), 2);
            let completion = finish();
            assert_eq!(completion.refunded_cycles(), None);
            let Outcome::OutcomeUnknown(rejection) = completion.outcome() else {
                panic!("expected unknown outcome")
            };
            assert_eq!(rejection.code(), 6);
            assert_eq!(rejection.message(), "拒否\0");
            // The callback must outlive the future and all borrowed request storage.
            let method = String::from("drop");
            let args = vec![0, 128, 255];
            let mut borrowed = Call::new(
                Principal::anonymous(),
                method.as_str(),
                Wait::Bounded { seconds: 0 },
            )
            .args(&args)
            .cycles(9)
            .execute();
            assert!(poll(&mut borrowed).is_pending());
        }
        3 => {
            assert_eq!(WAKES.load(Ordering::SeqCst), 2);
            start("cleanup", Wait::Unbounded);
        }
        4 => {
            assert_eq!(WAKES.load(Ordering::SeqCst), 2);
            let mut cleaned = PENDING.with(|pending| pending.borrow_mut().take().unwrap());
            assert!(poll(&mut cleaned).is_pending());
            drop(cleaned);
            let mut refused =
                Call::new(Principal::anonymous(), "refuse", Wait::Unbounded).execute();
            assert_eq!(
                poll(&mut refused),
                Poll::Ready(Err(SubmitError::Refused(
                    NonZeroU32::new(u32::MAX).unwrap()
                )))
            );
            TERMINAL.with(|slot| *slot.borrow_mut() = Some(refused));
            start("after", Wait::Bounded { seconds: u32::MAX });
        }
        5 => {
            assert_eq!(WAKES.load(Ordering::SeqCst), 3);
            let completion = finish();
            assert_eq!(completion.refunded_cycles(), Some(0));
            let Outcome::Rejected(rejection) = completion.outcome() else {
                panic!("expected raw rejection")
            };
            assert_eq!(rejection.code(), u32::MAX);
            assert_eq!(rejection.message(), "raw");
        }
        6..=9 => executor::step(phase - 6),
        10..=18 => interleaved::step(phase - 10),
        19..=22 => update::step(phase - 19),
        23..=24 => handoff::step(phase - 23),
        25 => {
            let mut terminal = TERMINAL.with(|slot| slot.borrow_mut().take().unwrap());
            // Must trap before any attempt to submit a second time.
            let _ = poll(&mut terminal);
        }
        _ => panic!("unexpected fixture entry"),
    }
}
