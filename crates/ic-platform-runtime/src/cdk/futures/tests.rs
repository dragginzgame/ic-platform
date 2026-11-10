use super::{internals, is_recovering_from_trap, spawn, spawn_migratory, spawn_weak};
use crate::executor::{self, TaskHandle};
use std::cell::{Cell, RefCell};
use std::future::{Future, pending, poll_fn};
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll};

struct DropCount(Arc<AtomicUsize>);

impl Drop for DropCount {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

fn unfinished(drops: Arc<AtomicUsize>) -> impl Future<Output = ()> {
    let marker = DropCount(drops);
    async move {
        pending::<()>().await;
        drop(marker);
    }
}

fn isolated(action: impl FnOnce() + Send + 'static) {
    std::thread::spawn(action)
        .join()
        .expect("task policy scenario");
}

fn policy_panics(action: impl FnOnce() + Send + 'static) {
    let panic = std::thread::spawn(action)
        .join()
        .expect_err("unfinished protected task must panic");
    let message = panic
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| panic.downcast_ref::<String>().map(String::as_str));
    assert_eq!(
        message,
        Some(
            "protected task outlived its canister method (did you mean to use spawn_weak or spawn_migratory?)"
        )
    );
}

#[test]
fn completed_protected_task_drops_inner_future_once() {
    isolated(|| {
        let drops = Arc::new(AtomicUsize::new(0));
        let marker = DropCount(drops.clone());
        let value = internals::in_executor_context(|| {
            spawn(async move { drop(marker) });
            assert_eq!(drops.load(Ordering::SeqCst), 0);
            31
        });
        assert_eq!(value, 31);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        assert!(!is_recovering_from_trap());
    });
}

struct InnerDropFailure;

struct ReadyWithPanickingDrop(Arc<AtomicUsize>);

impl Future for ReadyWithPanickingDrop {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<()> {
        Poll::Ready(())
    }
}

impl Drop for ReadyWithPanickingDrop {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
        std::panic::panic_any(InnerDropFailure);
    }
}

#[test]
fn ready_inner_drop_panic_preserves_original_failure() {
    isolated(|| {
        let drops = Arc::new(AtomicUsize::new(0));
        let failure = std::panic::catch_unwind(|| {
            internals::in_executor_context(|| {
                spawn(ReadyWithPanickingDrop(drops.clone()));
            });
        })
        .expect_err("inner Drop must panic");
        assert!(failure.is::<InnerDropFailure>());
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    });
}

#[test]
fn never_polled_protected_cancellation_panics_and_drops_once() {
    let drops = Arc::new(AtomicUsize::new(0));
    let captured = drops.clone();
    policy_panics(move || {
        internals::in_executor_context(|| {
            spawn(unfinished(captured));
            executor::cancel_all_tasks_attached_to_current_method();
        });
    });
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn pending_protected_cancellation_panics_and_drops_once() {
    let drops = Arc::new(AtomicUsize::new(0));
    let captured = drops.clone();
    policy_panics(move || {
        let method = internals::in_executor_context(|| {
            spawn(unfinished(captured));
            executor::extend_current_method_context()
        });
        executor::in_callback_executor_context_for(
            method,
            executor::cancel_all_tasks_attached_to_current_method,
        );
    });
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn cancellation_during_poll_keeps_protected_policy() {
    let drops = Arc::new(AtomicUsize::new(0));
    let captured = drops.clone();
    policy_panics(move || {
        let marker = DropCount(captured);
        internals::in_executor_context(|| {
            spawn(async move {
                executor::cancel_task(&TaskHandle::current().expect("polled task"));
                pending::<()>().await;
                drop(marker);
            });
        });
    });
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn weak_cancellation_accepts_unpolled_and_pending_futures() {
    isolated(|| {
        for before_poll in [true, false] {
            let drops = Arc::new(AtomicUsize::new(0));
            internals::in_executor_context(|| {
                spawn_weak(unfinished(drops.clone()));
                if before_poll {
                    executor::cancel_all_tasks_attached_to_current_method();
                    assert_eq!(drops.load(Ordering::SeqCst), 1);
                }
            });
            assert_eq!(drops.load(Ordering::SeqCst), 1);
        }
    });
}

#[test]
fn recovery_cancellation_suppresses_protected_policy_panic() {
    isolated(|| {
        let drops = Arc::new(AtomicUsize::new(0));
        let method = internals::in_executor_context(|| {
            spawn(unfinished(drops.clone()));
            executor::extend_current_method_context()
        });
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        let value = executor::in_trap_recovery_context_for(method, || {
            assert!(is_recovering_from_trap());
            executor::cancel_all_tasks_attached_to_current_method();
            17
        });
        assert_eq!(value, 17);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        assert!(!is_recovering_from_trap());
    });
}

struct PinnedProbe {
    polls: Rc<Cell<usize>>,
    drops: Arc<AtomicUsize>,
    address: Cell<Option<*const Self>>,
    _pin: PhantomPinned,
}

impl Future for PinnedProbe {
    type Output = ();

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<()> {
        let this = self.as_ref().get_ref();
        let address = std::ptr::from_ref(this);
        if let Some(previous) = this.address.replace(Some(address)) {
            assert_eq!(previous, address);
        }
        assert!(TaskHandle::current().is_some());
        assert!(!is_recovering_from_trap());
        this.polls.set(this.polls.get() + 1);
        if this.polls.get() == 1 {
            context.waker().wake_by_ref();
            context.waker().wake_by_ref();
            assert_eq!(this.polls.get(), 1);
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

impl Drop for PinnedProbe {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn protected_query_polls_non_unpin_future_in_its_task_context() {
    isolated(|| {
        let polls = Rc::new(Cell::new(0));
        let drops = Arc::new(AtomicUsize::new(0));
        assert!(TaskHandle::current().is_none());
        let value = internals::in_query_executor_context(|| {
            spawn(PinnedProbe {
                polls: polls.clone(),
                drops: drops.clone(),
                address: Cell::new(None),
                _pin: PhantomPinned,
            });
            59
        });
        assert_eq!(value, 59);
        assert_eq!(polls.get(), 2);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        assert!(TaskHandle::current().is_none());
    });
}

#[test]
fn migratory_adapter_retains_executor_lifetime_and_query_refusal() {
    isolated(|| {
        let polls = Rc::new(Cell::new(0));
        let drops = Arc::new(AtomicUsize::new(0));
        let saved = Rc::new(RefCell::new(None));
        let marker = DropCount(drops.clone());
        internals::in_executor_context(|| {
            let polls = polls.clone();
            let saved = saved.clone();
            spawn_migratory(async move {
                poll_fn(move |context| {
                    polls.set(polls.get() + 1);
                    *saved.borrow_mut() = Some(context.waker().clone());
                    if polls.get() == 1 {
                        Poll::Pending
                    } else {
                        Poll::Ready(())
                    }
                })
                .await;
                drop(marker);
            });
        });
        assert_eq!(polls.get(), 1);
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        internals::in_query_executor_context(|| {
            saved
                .borrow()
                .as_ref()
                .expect("migratory waker")
                .wake_by_ref();
        });
        assert_eq!(polls.get(), 1);
        internals::in_executor_context(|| {});
        assert_eq!(polls.get(), 2);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    });
    assert!(
        std::thread::spawn(|| {
            internals::in_query_executor_context(|| spawn_migratory(async {}));
        })
        .join()
        .is_err()
    );
}

#[test]
fn context_aliases_preserve_nested_entry_refusal() {
    assert!(
        std::thread::spawn(|| {
            internals::in_executor_context(|| internals::in_query_executor_context(|| {}));
        })
        .join()
        .is_err()
    );
}
