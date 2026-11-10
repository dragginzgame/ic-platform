//! Lifecycle tests informed only by the replacement implementation and contract.

use super::*;
use crate::Principal;
use crate::executor::{in_tracking_executor_context, spawn_migratory, spawn_protected};
use std::cell::{Cell, RefCell};
use std::future::{IntoFuture, pending, poll_fn};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;
use std::sync::Weak;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::Wake;

thread_local! {
    static COMPLETION_DROPS: Cell<usize> = const { Cell::new(0) };
}

impl Drop for Completion {
    fn drop(&mut self) {
        COMPLETION_DROPS.with(|count| count.set(count.get() + 1));
    }
}

fn isolated(action: impl FnOnce() + Send + 'static) {
    std::thread::spawn(action)
        .join()
        .expect("call lifecycle scenario");
}

fn request() -> CallFuture<'static, 'static> {
    Call::bounded_wait(Principal::anonymous(), "lifecycle").into_future()
}

fn poll(future: &mut CallFuture<'_, '_>, waker: &Waker) -> Poll<Result<Response, CallFailed>> {
    Pin::new(future).poll(&mut Context::from_waker(waker))
}

fn completion(future: &CallFuture<'_, '_>) -> Weak<Completion> {
    Arc::downgrade(future.completion.as_ref().expect("submitted completion"))
}

// The seam transfers each successful environment to this guard exactly once.
// Upgrade the independently saved Weak before releasing that raw reference,
// then exercise ordinary helpers, never an extern callback that could unwind.
struct Delivery {
    environment: Environment,
    completion: Weak<Completion>,
}

impl Delivery {
    fn capture(future: &CallFuture<'_, '_>) -> Self {
        let raw = system::with_mock(|mock| mock.environments.remove(0));
        Self {
            environment: Environment { raw: Some(raw) },
            completion: completion(future),
        }
    }

    fn run(self, action: fn(Arc<Completion>)) {
        let owned = self
            .completion
            .upgrade()
            .expect("callback ownership remains");
        drop(self.environment);
        action(owned);
    }
}

struct DropCount(Rc<Cell<usize>>);

impl Drop for DropCount {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

fn pending_task(drops: Rc<Cell<usize>>) -> impl Future<Output = ()> {
    let marker = DropCount(drops);
    async move {
        pending::<()>().await;
        drop(marker);
    }
}

fn submitted_task(
    captured: Rc<RefCell<Option<Delivery>>>,
    drops: Rc<Cell<usize>>,
) -> impl Future<Output = ()> {
    let marker = DropCount(drops);
    async move {
        let mut future = request();
        let _result = poll_fn(|context| {
            let result = Pin::new(&mut future).poll(context);
            if result.is_pending() && captured.borrow().is_none() {
                *captured.borrow_mut() = Some(Delivery::capture(&future));
            }
            result
        })
        .await;
        drop(marker);
    }
}

fn assert_trap(action: impl FnOnce(), expected: &str) {
    let failure =
        catch_unwind(AssertUnwindSafe(action)).expect_err("expected native trap substitute");
    let text = failure
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| failure.downcast_ref::<&str>().copied());
    assert_eq!(text, Some(expected));
}

#[test]
fn failed_first_poll_reclaims_environment_without_a_context() {
    isolated(|| {
        for (index, status) in [1, u32::MAX].into_iter().enumerate() {
            system::with_mock(|mock| mock.status = status);
            let mut future = request();
            assert!(matches!(
                poll(&mut future, Waker::noop()),
                Poll::Ready(Err(CallFailed::CallPerformFailed(_)))
            ));
            assert!(future.completion.is_none());
            assert_eq!(COMPLETION_DROPS.with(Cell::get), index + 1);
            system::with_mock(|mock| assert!(mock.environments.is_empty()));
            assert_trap(
                || {
                    let _ = poll(&mut future, Waker::noop());
                },
                "CallFuture polled after completing",
            );
        }
        system::with_mock(|mock| {
            mock.events.clear();
            mock.balance = 0;
        });
        let mut future = request();
        assert!(matches!(
            poll(&mut future, Waker::noop()),
            Poll::Ready(Err(CallFailed::InsufficientLiquidCycleBalance(_)))
        ));
        assert_eq!(COMPLETION_DROPS.with(Cell::get), 2);
        system::with_mock(|mock| {
            assert_eq!(
                mock.events,
                [system::Event::Cost(9, 0), system::Event::Balance]
            );
        });
    });
}

#[test]
fn ordinary_pending_drop_preserves_callback_and_method_ownership() {
    isolated(|| {
        let method = String::from("borrowed");
        let args = vec![0, 255, 8];
        let task_drops = Rc::new(Cell::new(0));
        let (future, delivery) = in_tracking_executor_context(|| {
            spawn_protected(pending_task(task_drops.clone()));
            let mut future = Call::bounded_wait(Principal::anonymous(), &method)
                .with_raw_args(&args)
                .into_future();
            assert!(poll(&mut future, Waker::noop()).is_pending());
            let delivery = Delivery::capture(&future);
            (future, delivery)
        });
        let weak = delivery.completion.clone();
        drop(future);
        drop(args);
        drop(method);
        assert!(weak.upgrade().is_some());
        assert_eq!(task_drops.get(), 0);
        delivery.run(deliver);
        assert!(weak.upgrade().is_none());
        assert_eq!(task_drops.get(), 1);
        assert_eq!(COMPLETION_DROPS.with(Cell::get), 1);
    });
}

#[derive(Default)]
struct WakeCounts {
    wakes: AtomicUsize,
    drops: AtomicUsize,
}

struct ObservedWake {
    target: Mutex<Option<Weak<Completion>>>,
    counts: Arc<WakeCounts>,
    reenter: bool,
}

impl ObservedWake {
    fn inspect(&self, completed: bool) {
        let target = self.target.lock().unwrap().clone();
        if let Some(target) = target.and_then(|target| target.upgrade()) {
            let state = target
                .state
                .try_lock()
                .expect("user Waker code runs outside completion lock");
            if completed {
                assert!(matches!(&*state, State::Completed(_)));
            }
        }
    }
}

impl Wake for ObservedWake {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.inspect(true);
        self.counts.wakes.fetch_add(1, Ordering::SeqCst);
        if self.reenter {
            Call::unbounded_wait(Principal::anonymous(), "reentrant")
                .oneway()
                .unwrap();
            let counts = self.counts.clone();
            spawn_protected(async move {
                counts.wakes.fetch_add(1, Ordering::SeqCst);
            });
        }
    }
}

impl Drop for ObservedWake {
    fn drop(&mut self) {
        self.inspect(false);
        self.counts.drops.fetch_add(1, Ordering::SeqCst);
    }
}

fn observed_waker(
    target: Option<Weak<Completion>>,
    reenter: bool,
) -> (Arc<ObservedWake>, Waker, Arc<WakeCounts>) {
    let counts = Arc::new(WakeCounts::default());
    let observed = Arc::new(ObservedWake {
        target: Mutex::new(target),
        counts: counts.clone(),
        reenter,
    });
    (observed.clone(), Waker::from(observed), counts)
}

#[test]
fn latest_waker_and_reentrant_wake_and_drop_run_outside_locks() {
    isolated(|| {
        let (first, first_waker, first_counts) = observed_waker(None, false);
        let mut future = request();
        let (method, delivery) = in_tracking_executor_context(|| {
            assert!(poll(&mut future, &first_waker).is_pending());
            (extend_current_method_context(), Delivery::capture(&future))
        });
        *first.target.lock().unwrap() = Some(completion(&future));
        drop(first);
        drop(first_waker);
        let (latest, latest_waker, latest_counts) = observed_waker(Some(completion(&future)), true);
        in_callback_executor_context_for(method, || {
            assert!(poll(&mut future, &latest_waker).is_pending());
        });
        assert_eq!(first_counts.drops.load(Ordering::SeqCst), 1);
        assert_eq!(first_counts.wakes.load(Ordering::SeqCst), 0);
        drop(latest);
        drop(latest_waker);
        system::with_mock(|mock| mock.reply_data = vec![0, 255, 17]);
        delivery.run(deliver);
        assert_eq!(latest_counts.wakes.load(Ordering::SeqCst), 2);
        assert_eq!(latest_counts.drops.load(Ordering::SeqCst), 1);
        match poll(&mut future, Waker::noop()) {
            Poll::Ready(Ok(response)) => assert_eq!(response.into_bytes(), [0, 255, 17]),
            other => panic!("unexpected reply result: {other:?}"),
        }
        assert_trap(
            || {
                let _ = poll(&mut future, Waker::noop());
            },
            "CallFuture polled after completing",
        );
        system::with_mock(|mock| {
            assert!(mock.environments.is_empty());
            assert_eq!(
                mock.events
                    .iter()
                    .filter(|event| matches!(event, system::Event::Perform))
                    .count(),
                2
            );
        });
    });
}

#[test]
fn cleanup_cancels_sibling_calls_but_cancelled_cleanup_spares_later_tasks() {
    isolated(|| {
        let first_slot = Rc::new(RefCell::new(None));
        let second_slot = Rc::new(RefCell::new(None));
        let first_drops = Rc::new(Cell::new(0));
        let second_drops = Rc::new(Cell::new(0));
        let method = in_tracking_executor_context(|| {
            spawn_protected(submitted_task(first_slot.clone(), first_drops.clone()));
            spawn_protected(submitted_task(second_slot.clone(), second_drops.clone()));
            extend_current_method_context()
        });
        let first = first_slot.borrow_mut().take().unwrap();
        let second = second_slot.borrow_mut().take().unwrap();
        let first_weak = first.completion.clone();
        let second_weak = second.completion.clone();
        first.run(cleanup);
        assert_eq!(first_drops.get(), 1);
        assert_eq!(second_drops.get(), 1);
        assert!(first_weak.upgrade().is_none());
        assert!(matches!(
            &*second_weak.upgrade().unwrap().lock(),
            State::Cancelled
        ));

        let survivor_drops = Rc::new(Cell::new(0));
        let method = in_callback_executor_context_for(method, || {
            spawn_protected(pending_task(survivor_drops.clone()));
            extend_current_method_context()
        });
        second.run(cleanup);
        assert!(second_weak.upgrade().is_none());
        assert_eq!(survivor_drops.get(), 0);
        system::with_mock(|mock| assert_eq!(mock.message_reads, 0));
        drop(method);
        assert_eq!(survivor_drops.get(), 1);
        assert_eq!(COMPLETION_DROPS.with(Cell::get), 2);
    });
}

#[test]
fn delivery_to_recovery_cancelled_call_traps_and_releases_ownership() {
    isolated(|| {
        let mut future = request();
        let (method, delivery) = in_tracking_executor_context(|| {
            assert!(poll(&mut future, Waker::noop()).is_pending());
            (extend_current_method_context(), Delivery::capture(&future))
        });
        let weak = delivery.completion.clone();
        in_trap_recovery_context_for(method, || drop(future));
        assert!(weak.upgrade().is_some());
        assert_trap(|| delivery.run(deliver), "Call already trapped");
        assert!(weak.upgrade().is_none());
        assert_eq!(COMPLETION_DROPS.with(Cell::get), 1);
    });
}

#[test]
fn external_future_observes_cleanup_without_a_current_task_or_message_reads() {
    isolated(|| {
        let mut future = request();
        let delivery = in_tracking_executor_context(|| {
            assert!(TaskHandle::current().is_none());
            assert!(poll(&mut future, Waker::noop()).is_pending());
            Delivery::capture(&future)
        });
        let weak = delivery.completion.clone();
        delivery.run(cleanup);
        match poll(&mut future, Waker::noop()) {
            Poll::Ready(Err(CallFailed::CallRejected(error))) => {
                assert_eq!(error.raw_reject_code(), 4);
                assert_eq!(error.reject_message(), "cleanup");
            }
            other => panic!("unexpected cleanup result: {other:?}"),
        }
        system::with_mock(|mock| assert_eq!(mock.message_reads, 0));
        drop(future);
        assert!(weak.upgrade().is_none());
    });
}

#[test]
fn cleanup_preserves_original_migratory_task_when_another_task_repolls() {
    isolated(|| {
        let shared_future = Rc::new(RefCell::new(None));
        let delivery_slot = Rc::new(RefCell::new(None));
        let origin_drops = Rc::new(Cell::new(0));
        let replacement_drops = Rc::new(Cell::new(0));
        in_tracking_executor_context(|| {
            let shared_future = shared_future.clone();
            let delivery_slot = delivery_slot.clone();
            let marker = DropCount(origin_drops.clone());
            spawn_migratory(async move {
                let mut future = request();
                poll_fn(|context| {
                    assert!(Pin::new(&mut future).poll(context).is_pending());
                    Poll::Ready(())
                })
                .await;
                *delivery_slot.borrow_mut() = Some(Delivery::capture(&future));
                *shared_future.borrow_mut() = Some(future);
                pending::<()>().await;
                drop(marker);
            });
        });
        let replacement = in_tracking_executor_context(|| {
            let shared_future = shared_future.clone();
            let marker = DropCount(replacement_drops.clone());
            spawn_migratory(async move {
                poll_fn(|context| {
                    let mut future = shared_future.borrow_mut().take().unwrap();
                    assert!(Pin::new(&mut future).poll(context).is_pending());
                    *shared_future.borrow_mut() = Some(future);
                    Poll::Ready(())
                })
                .await;
                pending::<()>().await;
                drop(marker);
            })
        });
        delivery_slot.borrow_mut().take().unwrap().run(cleanup);
        assert_eq!(origin_drops.get(), 1);
        assert_eq!(replacement_drops.get(), 0);
        let mut future = shared_future.borrow_mut().take().unwrap();
        assert!(matches!(
            poll(&mut future, Waker::noop()),
            Poll::Ready(Err(CallFailed::CallRejected(_)))
        ));
        cancel_task(&replacement);
        assert_eq!(replacement_drops.get(), 1);
        system::with_mock(|mock| assert_eq!(mock.message_reads, 0));
    });
}

#[test]
fn rejection_delivery_preserves_raw_unknown_code_and_text() {
    isolated(|| {
        let mut future = request();
        let delivery = in_tracking_executor_context(|| {
            assert!(poll(&mut future, Waker::noop()).is_pending());
            Delivery::capture(&future)
        });
        system::with_mock(|mock| {
            mock.reply_code = u32::MAX;
            mock.reject_message = "raw\0rejection\nλ".to_owned();
        });
        delivery.run(deliver);
        match poll(&mut future, Waker::noop()) {
            Poll::Ready(Err(CallFailed::CallRejected(error))) => {
                assert_eq!(error.raw_reject_code(), u32::MAX);
                assert_eq!(error.reject_message(), "raw\0rejection\nλ");
                assert!(error.reject_code().is_err());
            }
            other => panic!("unexpected rejection result: {other:?}"),
        }
        system::with_mock(|mock| assert_eq!(mock.message_reads, 2));
    });
}
