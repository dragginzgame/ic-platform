//! Executable ownership proposal, not a production transport or trap simulator.
//!
//! A successful transport retains one delivery token. It receives borrowed
//! request data only during submit; neither that token nor completion borrows it.

use super::{Call, Principal, Wait};
use crate::call::{Completion, Outcome};
use std::num::NonZeroU32;
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Wake, Waker},
};

enum State {
    Pending(Option<Waker>),
    Complete(Completion),
    Abandoned,
    Consumed,
}

struct Delivery(Arc<Mutex<State>>);

impl Delivery {
    fn complete(self, value: Completion) {
        let mut value = Some(value);
        let old = {
            let mut state = self.0.lock().unwrap();
            if matches!(*state, State::Pending(_)) {
                Some(std::mem::replace(
                    &mut *state,
                    State::Complete(value.take().unwrap()),
                ))
            } else {
                assert!(matches!(*state, State::Abandoned));
                None
            }
        };
        // Waking and dropping user-owned values must happen after unlocking.
        if let Some(State::Pending(Some(waker))) = old {
            waker.wake();
        }
        drop(value);
    }
}

trait Transport {
    fn submit(&mut self, request: &Call<'_, '_>, delivery: Delivery) -> Result<(), u32>;
}

struct Attempt<'m, 'a, T> {
    request: Option<Call<'m, 'a>>,
    state: Arc<Mutex<State>>,
    transport: T,
    terminal: bool,
}

impl<'m, 'a, T> Attempt<'m, 'a, T> {
    fn new(request: Call<'m, 'a>, transport: T) -> Self {
        Self {
            request: Some(request),
            state: Arc::new(Mutex::new(State::Pending(None))),
            transport,
            terminal: false,
        }
    }
}

impl<T: Transport + Unpin> Future for Attempt<'_, '_, T> {
    type Output = Result<Completion, u32>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        assert!(!this.terminal, "completed attempt cannot submit again");
        // Clone outside the lock: even cloning a RawWaker can reenter.
        let mut replacement = Some(cx.waker().clone());
        let previous = {
            let mut state = this.state.lock().unwrap();
            match &mut *state {
                State::Pending(waker) => std::mem::replace(waker, replacement.take()),
                _ => None,
            }
        };
        drop(previous);
        drop(replacement);
        if let Some(request) = this.request.take() {
            let delivery = Delivery(Arc::clone(&this.state));
            if let Err(code) = this.transport.submit(&request, delivery) {
                let old = {
                    let mut state = this.state.lock().unwrap();
                    std::mem::replace(&mut *state, State::Consumed)
                };
                drop(old);
                this.terminal = true;
                return Poll::Ready(Err(code));
            }
        }
        let completed = {
            let mut state = this.state.lock().unwrap();
            if matches!(*state, State::Complete(_)) {
                Some(std::mem::replace(&mut *state, State::Consumed))
            } else {
                None
            }
        };
        if let Some(State::Complete(value)) = completed {
            this.terminal = true;
            Poll::Ready(Ok(value))
        } else {
            Poll::Pending
        }
    }
}

impl<T> Drop for Attempt<'_, '_, T> {
    fn drop(&mut self) {
        let old = {
            let mut state = self.state.lock().unwrap();
            std::mem::replace(&mut *state, State::Abandoned)
        };
        drop(old);
    }
}

#[derive(Default)]
struct Harness {
    submissions: usize,
    callback: Option<Delivery>,
    refuse: Option<u32>,
    observed: Option<(String, Vec<u8>, u128, Wait)>,
}

impl Transport for Arc<Mutex<Harness>> {
    fn submit(&mut self, request: &Call<'_, '_>, delivery: Delivery) -> Result<(), u32> {
        let mut harness = self.lock().unwrap();
        harness.submissions += 1;
        harness.observed = Some((
            request.method().to_owned(),
            request.raw_args().to_vec(),
            request.attached_cycles(),
            request.wait(),
        ));
        if let Some(code) = harness.refuse {
            return Err(code);
        }
        harness.callback = Some(delivery);
        Ok(())
    }
}

fn request() -> Call<'static, 'static> {
    Call::new(Principal::anonymous(), "send", Wait::Bounded { seconds: 1 })
        .owned_args(vec![0, 255])
        .cycles(17)
}

fn poll<T: Transport + Unpin>(attempt: &mut Attempt<'_, '_, T>) -> Poll<Result<Completion, u32>> {
    Pin::new(attempt).poll(&mut Context::from_waker(Waker::noop()))
}

#[test]
fn construction_and_unpolled_drop_never_submit() {
    let harness = Arc::new(Mutex::new(Harness::default()));
    let attempt = Attempt::new(request(), Arc::clone(&harness));
    drop(attempt);
    assert_eq!(harness.lock().unwrap().submissions, 0);
}

#[test]
fn first_poll_submits_once_and_callback_owns_captured_refund() {
    let harness = Arc::new(Mutex::new(Harness::default()));
    let mut attempt = Attempt::new(request(), Arc::clone(&harness));
    assert!(poll(&mut attempt).is_pending());
    assert!(poll(&mut attempt).is_pending());
    let callback = {
        let mut h = harness.lock().unwrap();
        assert_eq!(h.submissions, 1);
        assert_eq!(
            h.observed,
            Some((
                "send".into(),
                vec![0, 255],
                17,
                Wait::Bounded { seconds: 1 }
            ))
        );
        h.callback.take().unwrap()
    };
    callback.complete(Completion::from_reply(vec![9], Some(0)));
    assert_eq!(
        poll(&mut attempt),
        Poll::Ready(Ok(Completion::from_reply(vec![9], Some(0))))
    );
    assert_eq!(harness.lock().unwrap().submissions, 1);
}

#[test]
fn refusal_reclaims_callback_authority_and_keeps_raw_code() {
    let harness = Arc::new(Mutex::new(Harness {
        refuse: Some(u32::MAX),
        ..Harness::default()
    }));
    let mut attempt = Attempt::new(request(), Arc::clone(&harness));
    assert_eq!(poll(&mut attempt), Poll::Ready(Err(u32::MAX)));
    assert!(harness.lock().unwrap().callback.is_none());
    assert_eq!(Arc::strong_count(&attempt.state), 1);
}

#[test]
fn dropped_interest_does_not_cancel_callback_custody() {
    let harness = Arc::new(Mutex::new(Harness::default()));
    let mut attempt = Attempt::new(request(), Arc::clone(&harness));
    let weak = Arc::downgrade(&attempt.state);
    assert!(poll(&mut attempt).is_pending());
    drop(attempt);
    assert!(weak.upgrade().is_some());
    let callback = harness.lock().unwrap().callback.take().unwrap();
    callback.complete(Completion::from_reply(vec![1], None));
    assert!(weak.upgrade().is_none());
}

struct Reentrant {
    state: Arc<Mutex<State>>,
    events: Arc<Mutex<Vec<&'static str>>>,
}

#[test]
fn delayed_poll_retains_callback_snapshot_and_raw_rejection() {
    let refund_register = std::cell::Cell::new(Some(37));
    for code in [4, 6, u32::MAX] {
        let harness = Arc::new(Mutex::new(Harness::default()));
        let mut attempt = Attempt::new(request(), Arc::clone(&harness));
        assert!(poll(&mut attempt).is_pending());
        let callback = harness.lock().unwrap().callback.take().unwrap();
        refund_register.set(Some(37));
        callback.complete(Completion::from_rejection(
            NonZeroU32::new(code).unwrap(),
            "exact\0拒否".into(),
            refund_register.get(),
        ));
        // Model an unrelated later callback overwriting the ambient observation.
        refund_register.set(Some(99));
        let Poll::Ready(Ok(completion)) = poll(&mut attempt) else {
            panic!("expected completion")
        };
        assert_eq!(completion.refunded_cycles(), Some(37));
        assert_eq!(
            matches!(completion.outcome(), Outcome::OutcomeUnknown(_)),
            code == 6
        );
        let (Outcome::Rejected(rejection) | Outcome::OutcomeUnknown(rejection)) =
            completion.outcome()
        else {
            panic!("expected rejection")
        };
        assert_eq!(rejection.code(), code);
        assert_eq!(rejection.message(), "exact\0拒否");
        assert_eq!(harness.lock().unwrap().submissions, 1);
    }
}

#[test]
fn callback_outlives_all_borrowed_request_storage() {
    let harness = Arc::new(Mutex::new(Harness::default()));
    {
        let method = String::from("temporary");
        let bytes = vec![7, 8, 9];
        let call = Call::new(Principal::anonymous(), method.as_str(), Wait::Unbounded).args(&bytes);
        let mut attempt = Attempt::new(call, Arc::clone(&harness));
        assert!(poll(&mut attempt).is_pending());
    }
    let callback = harness.lock().unwrap().callback.take().unwrap();
    callback.complete(Completion::from_reply(vec![42], Some(u128::MAX)));
    assert_eq!(harness.lock().unwrap().submissions, 1);
}

#[test]
fn terminal_repoll_cannot_resubmit() {
    let harness = Arc::new(Mutex::new(Harness {
        refuse: Some(19),
        ..Harness::default()
    }));
    let mut attempt = Attempt::new(request(), Arc::clone(&harness));
    assert_eq!(poll(&mut attempt), Poll::Ready(Err(19)));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| poll(&mut attempt)));
    assert!(result.is_err());
    assert_eq!(harness.lock().unwrap().submissions, 1);
}

impl Wake for Reentrant {
    fn wake(self: Arc<Self>) {
        assert!(self.state.try_lock().is_ok(), "wake under completion lock");
        self.events.lock().unwrap().push("wake");
    }
}

impl Drop for Reentrant {
    fn drop(&mut self) {
        assert!(
            self.state.try_lock().is_ok(),
            "destruction under completion lock"
        );
        self.events.lock().unwrap().push("drop");
    }
}

#[test]
fn wake_replacement_completion_and_abandonment_drop_outside_lock() {
    for mode in 0..3 {
        let harness = Arc::new(Mutex::new(Harness::default()));
        let mut attempt = Attempt::new(request(), Arc::clone(&harness));
        let events = Arc::new(Mutex::new(Vec::new()));
        let waker = Waker::from(Arc::new(Reentrant {
            state: Arc::clone(&attempt.state),
            events: Arc::clone(&events),
        }));
        assert!(
            Pin::new(&mut attempt)
                .poll(&mut Context::from_waker(&waker))
                .is_pending()
        );
        drop(waker);
        match mode {
            0 => {
                assert!(poll(&mut attempt).is_pending());
            }
            1 => {
                let callback = harness.lock().unwrap().callback.take().unwrap();
                callback.complete(Completion::from_reply(vec![], None));
            }
            _ => {
                drop(attempt);
            }
        }
        assert_eq!(
            *events.lock().unwrap(),
            if mode == 1 {
                vec!["wake", "drop"]
            } else {
                vec!["drop"]
            }
        );
    }
}
