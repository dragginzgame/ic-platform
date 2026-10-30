use super::*;
use std::{future::poll_fn, rc::Rc};

#[test]
fn protected_query_self_wakeup_completes_and_reuses_waker() {
    let polls = Rc::new(Cell::new(0));
    let observed = polls.clone();
    in_tracking_query_executor_context(|| {
        let mut first_waker: Option<Waker> = None;
        spawn_protected(poll_fn(move |cx| {
            observed.set(observed.get() + 1);
            if let Some(first) = &first_waker {
                assert!(first.will_wake(cx.waker()));
                Poll::Ready(())
            } else {
                first_waker = Some(cx.waker().clone());
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }));
    });
    assert_eq!(polls.get(), 2);
    assert!(TASKS.with_borrow(|tasks| tasks.is_empty()));
    assert!(MIGRATORY_WAKEUPS.with_borrow(|queue| queue.is_empty()));
    assert!(METHODS.with_borrow(|methods| methods.is_empty()));
}

#[test]
fn wake_storm_is_coalesced_without_losing_a_wake_during_poll() {
    let polls = Rc::new(Cell::new(0));
    let observed = polls.clone();
    in_tracking_executor_context(|| {
        spawn_protected(poll_fn(move |cx| {
            observed.set(observed.get() + 1);
            if observed.get() <= 2 {
                for _ in 0..1000 {
                    cx.waker().wake_by_ref();
                }
            }
            Poll::Pending
        }));
    });
    assert_eq!(polls.get(), 3);
}

#[test]
fn completed_tasks_do_not_accumulate_in_a_live_method() {
    in_tracking_executor_context(|| {
        for _ in 0..1000 {
            spawn_protected(async {});
            poll_all();
        }
        assert!(TASKS.with_borrow(|tasks| tasks.is_empty()));
        assert!(
            METHODS.with_borrow(|methods| methods.values().all(|method| method.tasks.is_empty()))
        );
    });
}

#[test]
fn foreign_method_wakeup_waits_for_the_owning_callback() {
    let polls = Rc::new(Cell::new(0));
    let observed = polls.clone();
    let saved = Rc::new(RefCell::new(None::<Waker>));
    let captured = saved.clone();
    let method = in_tracking_executor_context(|| {
        spawn_protected(poll_fn(move |cx| {
            observed.set(observed.get() + 1);
            *captured.borrow_mut() = Some(cx.waker().clone());
            if observed.get() == 1 {
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        }));
        extend_current_method_context()
    });
    in_tracking_executor_context(|| saved.borrow().as_ref().unwrap().wake_by_ref());
    assert_eq!(polls.get(), 1);
    in_callback_executor_context_for(method, || {});
    assert_eq!(polls.get(), 2);
}

#[test]
fn stale_waker_cannot_wake_a_reused_slot() {
    let saved = Rc::new(RefCell::new(None::<Waker>));
    let captured = saved.clone();
    in_tracking_executor_context(|| {
        let task = spawn_protected(poll_fn(move |cx| {
            *captured.borrow_mut() = Some(cx.waker().clone());
            Poll::Pending
        }));
        poll_all();
        cancel_task(&task);
        let polls = Rc::new(Cell::new(0));
        let observed = polls.clone();
        spawn_protected(poll_fn(move |_| {
            observed.set(observed.get() + 1);
            Poll::Pending
        }));
        poll_all();
        saved.borrow().as_ref().unwrap().wake_by_ref();
        poll_all();
        assert_eq!(polls.get(), 1);
    });
}

struct DropProbe(Rc<Cell<usize>>, bool);
impl Drop for DropProbe {
    fn drop(&mut self) {
        assert!(TASKS.with(|tasks| tasks.try_borrow_mut().is_ok()));
        assert!(METHODS.with(|methods| methods.try_borrow_mut().is_ok()));
        assert_eq!(is_recovering_from_trap(), self.1);
        self.0.set(self.0.get() + 1);
    }
}

#[test]
fn self_cancellation_drops_future_outside_borrows() {
    let drops = Rc::new(Cell::new(0));
    let probe = DropProbe(drops.clone(), false);
    in_tracking_executor_context(|| {
        spawn_protected(async move {
            let _probe = probe;
            poll_fn(|cx| {
                cx.waker().wake_by_ref();
                cancel_task(&TaskHandle::current().unwrap());
                Poll::<()>::Pending
            })
            .await;
        });
    });
    assert_eq!(drops.get(), 1);
    assert!(TASKS.with_borrow(|tasks| tasks.is_empty()));
}

#[test]
fn recovery_cancels_attached_tasks_outside_borrows() {
    let drops = Rc::new(Cell::new(0));
    let method = in_tracking_executor_context(|| {
        for _ in 0..5 {
            let probe = DropProbe(drops.clone(), true);
            spawn_protected(async move {
                let _probe = probe;
                std::future::pending::<()>().await;
            });
        }
        extend_current_method_context()
    });
    in_trap_recovery_context_for(method, cancel_all_tasks_attached_to_current_method);
    assert_eq!(drops.get(), 5);
    assert!(!is_recovering_from_trap());
    assert!(TASKS.with_borrow(|tasks| tasks.is_empty()));
}

#[test]
fn migratory_task_survives_its_spawning_method() {
    let polls = Rc::new(Cell::new(0));
    let observed = polls.clone();
    let saved = Rc::new(RefCell::new(None::<Waker>));
    let captured = saved.clone();
    in_tracking_executor_context(|| {
        spawn_migratory(poll_fn(move |cx| {
            observed.set(observed.get() + 1);
            *captured.borrow_mut() = Some(cx.waker().clone());
            if observed.get() == 1 {
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        }));
    });
    in_tracking_executor_context(|| saved.borrow().as_ref().unwrap().wake_by_ref());
    assert_eq!(polls.get(), 2);
}

#[test]
fn query_does_not_drain_migratory_wakeups() {
    let polls = Rc::new(Cell::new(0));
    let observed = polls.clone();
    let saved = Rc::new(RefCell::new(None::<Waker>));
    let captured = saved.clone();
    in_tracking_executor_context(|| {
        spawn_migratory(poll_fn(move |cx| {
            observed.set(observed.get() + 1);
            *captured.borrow_mut() = Some(cx.waker().clone());
            if observed.get() == 1 {
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        }));
    });
    in_tracking_query_executor_context(|| {
        saved.borrow().as_ref().unwrap().wake_by_ref();
        spawn_protected(async {});
    });
    assert_eq!(polls.get(), 1);
    in_tracking_executor_context(|| {});
    assert_eq!(polls.get(), 2);
}

#[test]
fn bulk_cancellation_clears_old_wakeups_and_keeps_destructor_spawns() {
    struct SpawnOnDrop(Rc<Cell<bool>>);
    impl Drop for SpawnOnDrop {
        fn drop(&mut self) {
            // Every cancelled task must be detached before any destructor runs.
            assert!(TASKS.with_borrow(|tasks| tasks.is_empty()));
            let completed = self.0.clone();
            spawn_protected(async move { completed.set(true) });
        }
    }

    let completed = Rc::new(Cell::new(false));
    let probe = SpawnOnDrop(completed.clone());
    in_tracking_executor_context(|| {
        spawn_protected(async move {
            let _probe = probe;
            std::future::pending::<()>().await;
        });
        spawn_protected(std::future::pending());
        cancel_all_tasks_attached_to_current_method();
        let method_id = CURRENT_METHOD.get().unwrap();
        METHODS.with_borrow(|methods| {
            let method = &methods[method_id];
            assert_eq!(method.tasks.len(), 1);
            assert_eq!(method.wakeups.len(), 1);
        });
    });
    assert!(completed.get());
    assert!(TASKS.with_borrow(|tasks| tasks.is_empty()));
    assert!(METHODS.with_borrow(|methods| methods.is_empty()));
}
