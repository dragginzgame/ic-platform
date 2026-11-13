//! Public lifecycle qualification derived from the reviewed executor contract.

use ic_platform_runtime::executor::{self, MethodHandle, TaskHandle};
use std::cell::{Cell, RefCell};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

struct Probe<F> {
    poll: F,
    on_drop: Option<Box<dyn FnOnce()>>,
}

impl<F> Probe<F> {
    fn on_drop(mut self, action: impl FnOnce() + 'static) -> Self {
        self.on_drop = Some(Box::new(action));
        self
    }
}

impl<F> Future for Probe<F>
where
    F: FnMut(&mut Context<'_>) -> Poll<()> + Unpin,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<()> {
        (self.get_mut().poll)(context)
    }
}

impl<F> Drop for Probe<F> {
    fn drop(&mut self) {
        if let Some(action) = self.on_drop.take() {
            action();
        }
    }
}

fn probe<F: FnMut(&mut Context<'_>) -> Poll<()>>(poll: F) -> Probe<F> {
    Probe {
        poll,
        on_drop: None,
    }
}

fn isolated(action: impl FnOnce() + Send + 'static) {
    std::thread::spawn(action)
        .join()
        .expect("consumer scenario");
}

fn panics(action: impl FnOnce() + Send + 'static) {
    assert!(std::thread::spawn(action).join().is_err());
}

fn wake(slot: &RefCell<Option<Waker>>) {
    slot.borrow()
        .as_ref()
        .expect("polled task waker")
        .wake_by_ref();
}

#[test]
fn public_handles_retain_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<MethodHandle>();
    assert_send_sync::<TaskHandle>();
}

#[test]
fn context_exit_allows_destructor_reentry_after_return_and_unwind() {
    isolated(|| {
        for unwind in [false, true] {
            let drops = Rc::new(Cell::new(0));
            let observed = drops.clone();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                executor::in_tracking_executor_context(|| {
                    executor::spawn_protected(probe(|_| Poll::Pending).on_drop(move || {
                        assert!(TaskHandle::current().is_none());
                        executor::in_tracking_executor_context(|| {
                            observed.set(observed.get() + 1);
                        });
                    }));
                    assert!(!unwind, "exercise native context unwinding");
                });
            }));
            assert_eq!(result.is_err(), unwind);
            assert_eq!(drops.get(), 1);
            executor::in_tracking_executor_context(|| {});
        }
    });
}

#[test]
fn foreign_poll_wakes_wait_for_the_protected_owner() {
    isolated(|| {
        let polls = Rc::new(Cell::new(0));
        let saved = Rc::new(RefCell::new(None));
        let owner = executor::in_tracking_executor_context(|| {
            let task_polls = polls.clone();
            let saved = saved.clone();
            executor::spawn_protected(probe(move |context| {
                task_polls.set(task_polls.get() + 1);
                *saved.borrow_mut() = Some(context.waker().clone());
                if task_polls.get() == 2 {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            }));
            assert_eq!(polls.get(), 0);
            executor::extend_current_method_context()
        });
        assert_eq!(polls.get(), 1);
        executor::in_tracking_executor_context(|| {
            let saved = saved.clone();
            executor::spawn_protected(async move {
                wake(&saved);
                wake(&saved);
            });
        });
        assert_eq!(polls.get(), 1);
        let value = executor::in_callback_executor_context_for(owner, || {
            assert_eq!(polls.get(), 1);
            73
        });
        assert_eq!(value, 73);
        assert_eq!(polls.get(), 2);
    });
}

#[test]
fn query_self_wakes_coalesce_and_keep_waker_identity() {
    isolated(|| {
        let polls = Rc::new(Cell::new(0));
        let first_waker = Rc::new(RefCell::new(None::<Waker>));
        assert!(TaskHandle::current().is_none());
        let answer = executor::in_tracking_query_executor_context(|| {
            let polls = polls.clone();
            let first_waker = first_waker.clone();
            executor::spawn_protected(probe(move |context| {
                assert!(TaskHandle::current().is_some());
                assert!(!executor::is_recovering_from_trap());
                polls.set(polls.get() + 1);
                if let Some(first) = first_waker.borrow().as_ref() {
                    assert!(first.will_wake(context.waker()));
                }
                *first_waker.borrow_mut() = Some(context.waker().clone());
                if polls.get() < 3 {
                    let before_wake = polls.get();
                    context.waker().wake_by_ref();
                    context.waker().wake_by_ref();
                    assert_eq!(polls.get(), before_wake);
                    Poll::Pending
                } else {
                    Poll::Ready(())
                }
            }));
            19
        });
        assert_eq!(answer, 19);
        assert_eq!(polls.get(), 3);
        assert!(TaskHandle::current().is_none());
        executor::in_tracking_executor_context(|| wake(&first_waker));
        assert_eq!(polls.get(), 3);
    });
}

#[test]
fn protected_fifo_runs_before_migratory_fifo() {
    isolated(|| {
        let log = Rc::new(RefCell::new(Vec::new()));
        executor::in_tracking_executor_context(|| {
            for name in ["m1", "m2"] {
                let log = log.clone();
                executor::spawn_migratory(async move { log.borrow_mut().push(name) });
            }
            for name in ["p1", "p2"] {
                let log = log.clone();
                executor::spawn_protected(async move { log.borrow_mut().push(name) });
            }
            assert!(log.borrow().is_empty());
        });
        assert_eq!(*log.borrow(), ["p1", "p2", "m1", "m2"]);
    });
}

#[test]
fn stale_handles_and_wakers_cannot_reach_replacement_tasks() {
    isolated(|| {
        for completes in [false, true] {
            for _ in 0..32 {
                let stale_waker = Rc::new(RefCell::new(None));
                let stale_handle = Rc::new(RefCell::new(None));
                let owner = executor::in_tracking_executor_context(|| {
                    let saved = stale_waker.clone();
                    let handle = stale_handle.clone();
                    executor::spawn_protected(probe(move |context| {
                        *saved.borrow_mut() = Some(context.waker().clone());
                        *handle.borrow_mut() = TaskHandle::current();
                        if completes {
                            Poll::Ready(())
                        } else {
                            Poll::Pending
                        }
                    }));
                    executor::extend_current_method_context()
                });
                let old = stale_handle.borrow_mut().take().expect("current handle");
                executor::cancel_task(&old);
                let polls = Rc::new(Cell::new(0));
                let replacement_waker = Rc::new(RefCell::new(None));
                let owner = executor::in_callback_executor_context_for(owner, || {
                    let polls = polls.clone();
                    let saved = replacement_waker.clone();
                    executor::spawn_protected(probe(move |context| {
                        polls.set(polls.get() + 1);
                        *saved.borrow_mut() = Some(context.waker().clone());
                        if polls.get() == 2 {
                            Poll::Ready(())
                        } else {
                            Poll::Pending
                        }
                    }));
                    executor::extend_current_method_context()
                });
                assert_eq!(polls.get(), 1);
                let owner = executor::in_callback_executor_context_for(owner, || {
                    executor::cancel_task(&old);
                    wake(&stale_waker);
                    wake(&stale_waker);
                    executor::extend_current_method_context()
                });
                assert_eq!(polls.get(), 1);
                executor::in_callback_executor_context_for(owner, || wake(&replacement_waker));
                assert_eq!(polls.get(), 2);
            }
        }
    });
}

#[test]
fn self_cancellation_drops_once_and_allows_destructor_spawning() {
    isolated(|| {
        let polls = Rc::new(Cell::new(0));
        let drops = Rc::new(Cell::new(0));
        let successor_polls = Rc::new(Cell::new(0));
        executor::in_tracking_executor_context(|| {
            let polls = polls.clone();
            let drops = drops.clone();
            let successor_polls = successor_polls.clone();
            executor::spawn_protected(
                probe(move |context| {
                    polls.set(polls.get() + 1);
                    context.waker().wake_by_ref();
                    executor::cancel_task(&TaskHandle::current().expect("running task"));
                    Poll::Pending
                })
                .on_drop(move || {
                    assert!(TaskHandle::current().is_none());
                    drops.set(drops.get() + 1);
                    executor::spawn_protected(async move {
                        successor_polls.set(successor_polls.get() + 1);
                    });
                }),
            );
        });
        assert_eq!(polls.get(), 1);
        assert_eq!(drops.get(), 1);
        assert_eq!(successor_polls.get(), 1);
    });
}

#[test]
fn bulk_cancellation_preserves_survivor_order_and_new_destructor_work() {
    isolated(|| {
        let drops = Rc::new(RefCell::new(Vec::new()));
        let successor_polls = Rc::new(Cell::new(0));
        executor::in_tracking_executor_context(|| {
            let mut handles = Vec::new();
            for index in 0..4 {
                let drops = drops.clone();
                let successor_polls = successor_polls.clone();
                handles.push(executor::spawn_protected(probe(|_| Poll::Pending).on_drop(
                    move || {
                        drops.borrow_mut().push(index);
                        if index == 0 {
                            executor::spawn_protected(async move {
                                successor_polls.set(successor_polls.get() + 1);
                            });
                        }
                    },
                )));
            }
            executor::cancel_task(&handles[1]);
            assert_eq!(*drops.borrow(), [1]);
            drops.borrow_mut().clear();
            executor::cancel_all_tasks_attached_to_current_method();
            assert_eq!(*drops.borrow(), [0, 2, 3]);
            assert_eq!(successor_polls.get(), 0);
            for handle in &handles {
                executor::cancel_task(handle);
            }
        });
        assert_eq!(*drops.borrow(), [0, 2, 3]);
        assert_eq!(successor_polls.get(), 1);
    });
}

#[test]
fn retention_inside_poll_keeps_method_until_last_handle_drops() {
    isolated(|| {
        let retained = Rc::new(RefCell::new(Vec::new()));
        let drops = Rc::new(Cell::new(0));
        executor::in_tracking_executor_context(|| {
            let retained = retained.clone();
            let drops = drops.clone();
            executor::spawn_protected(
                probe(move |_| {
                    retained
                        .borrow_mut()
                        .push(executor::extend_current_method_context());
                    retained
                        .borrow_mut()
                        .push(executor::extend_current_method_context());
                    Poll::Pending
                })
                .on_drop(move || {
                    assert!(TaskHandle::current().is_none());
                    drops.set(drops.get() + 1);
                }),
            );
        });
        assert_eq!(drops.get(), 0);
        drop(retained.borrow_mut().pop());
        assert_eq!(drops.get(), 0);
        let last = retained.borrow_mut().pop().expect("second retention");
        drop(last);
        assert_eq!(drops.get(), 1);
    });
}

#[test]
fn migratory_survives_method_exit_and_retained_queries_cannot_drain_it() {
    isolated(|| {
        let migratory_polls = Rc::new(Cell::new(0));
        let migratory_waker = Rc::new(RefCell::new(None));
        executor::in_tracking_executor_context(|| {
            let polls = migratory_polls.clone();
            let saved = migratory_waker.clone();
            executor::spawn_migratory(probe(move |context| {
                polls.set(polls.get() + 1);
                *saved.borrow_mut() = Some(context.waker().clone());
                if polls.get() == 2 {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            }));
        });
        let query_polls = Rc::new(Cell::new(0));
        let query_waker = Rc::new(RefCell::new(None));
        let query = executor::in_tracking_query_executor_context(|| {
            wake(&migratory_waker);
            let polls = query_polls.clone();
            let saved = query_waker.clone();
            executor::spawn_protected(probe(move |context| {
                polls.set(polls.get() + 1);
                *saved.borrow_mut() = Some(context.waker().clone());
                if polls.get() == 2 {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            }));
            executor::extend_current_method_context()
        });
        executor::in_callback_executor_context_for(query, || {
            wake(&query_waker);
            wake(&migratory_waker);
        });
        assert_eq!(query_polls.get(), 2);
        assert_eq!(migratory_polls.get(), 1);
        executor::in_tracking_executor_context(|| {});
        assert_eq!(migratory_polls.get(), 2);
    });
}

#[test]
fn retained_migratory_context_resumes_without_a_tracked_owner() {
    isolated(|| {
        let retained = Rc::new(RefCell::new(None));
        executor::in_tracking_executor_context(|| {
            let retained = retained.clone();
            executor::spawn_migratory(async move {
                *retained.borrow_mut() = Some(executor::extend_current_method_context());
            });
        });
        let polls = Rc::new(Cell::new(0));
        let method = retained.borrow_mut().take().expect("untracked retention");
        executor::in_callback_executor_context_for(method, || {
            let polls = polls.clone();
            executor::spawn_migratory(async move { polls.set(polls.get() + 1) });
        });
        assert_eq!(polls.get(), 1);
    });
}

#[test]
fn recovery_marks_destruction_and_never_drains_queued_work() {
    isolated(|| {
        let polls = Rc::new(Cell::new(0));
        let saved = Rc::new(RefCell::new(None));
        let drops = Rc::new(Cell::new(0));
        let method = executor::in_tracking_executor_context(|| {
            let polls = polls.clone();
            let saved = saved.clone();
            let drops = drops.clone();
            executor::spawn_protected(
                probe(move |context| {
                    polls.set(polls.get() + 1);
                    *saved.borrow_mut() = Some(context.waker().clone());
                    Poll::Pending
                })
                .on_drop(move || {
                    assert!(executor::is_recovering_from_trap());
                    drops.set(drops.get() + 1);
                }),
            );
            executor::extend_current_method_context()
        });
        wake(&saved);
        assert!(!executor::is_recovering_from_trap());
        let method = executor::in_trap_recovery_context_for(method, || {
            assert!(executor::is_recovering_from_trap());
            assert!(TaskHandle::current().is_none());
            executor::extend_current_method_context()
        });
        assert_eq!(polls.get(), 1);
        assert!(!executor::is_recovering_from_trap());
        executor::in_trap_recovery_context_for(method, || {
            executor::cancel_all_tasks_attached_to_current_method();
        });
        assert_eq!(polls.get(), 1);
        assert_eq!(drops.get(), 1);
        assert!(!executor::is_recovering_from_trap());
    });
}

#[test]
#[expect(
    unused_must_use,
    reason = "There must be no handle returned when context admission panics"
)]
fn context_requirements_reject_misuse_in_fresh_threads() {
    panics(|| {
        executor::extend_current_method_context();
    });
    panics(|| {
        executor::spawn_protected(async {});
    });
    panics(|| {
        executor::spawn_migratory(async {});
    });
    panics(executor::cancel_all_tasks_attached_to_current_method);
    panics(|| {
        executor::in_tracking_executor_context(|| executor::in_tracking_executor_context(|| {}))
    });
    panics(|| {
        executor::in_tracking_executor_context(|| {
            executor::in_tracking_query_executor_context(|| {})
        })
    });
    panics(|| {
        executor::in_tracking_executor_context(|| {
            executor::spawn_protected(async {
                executor::in_tracking_executor_context(|| {});
            });
        })
    });
    panics(|| {
        executor::in_tracking_query_executor_context(|| {
            executor::spawn_migratory(async {});
        })
    });
    panics(|| {
        executor::in_tracking_executor_context(|| {
            executor::spawn_migratory(async {
                executor::spawn_protected(async {});
            });
        })
    });
    panics(|| {
        let method =
            executor::in_tracking_query_executor_context(executor::extend_current_method_context);
        executor::in_callback_executor_context_for(method, || {
            executor::spawn_migratory(async {});
        });
    });
    panics(|| {
        let method =
            executor::in_tracking_executor_context(executor::extend_current_method_context);
        executor::in_tracking_executor_context(|| {
            executor::in_callback_executor_context_for(method, || {})
        });
    });
    panics(|| {
        let method =
            executor::in_tracking_executor_context(executor::extend_current_method_context);
        executor::in_tracking_executor_context(|| {
            executor::in_trap_recovery_context_for(method, || {})
        });
    });
}

#[test]
fn recovery_and_final_reclamation_refuse_spawning() {
    for protected in [false, true] {
        panics(move || {
            let method =
                executor::in_tracking_executor_context(executor::extend_current_method_context);
            executor::in_trap_recovery_context_for(method, || {
                if protected {
                    executor::spawn_protected(async {});
                } else {
                    executor::spawn_migratory(async {});
                }
            });
        });
    }
    panics(|| {
        let method = executor::in_tracking_executor_context(|| {
            executor::spawn_protected(probe(|_| Poll::Pending).on_drop(|| {
                assert!(executor::is_recovering_from_trap());
                executor::spawn_protected(async {});
            }));
            executor::extend_current_method_context()
        });
        executor::in_trap_recovery_context_for(
            method,
            executor::cancel_all_tasks_attached_to_current_method,
        );
    });
    panics(|| {
        executor::in_tracking_executor_context(|| {
            executor::spawn_protected(probe(|_| Poll::Pending).on_drop(|| {
                executor::spawn_protected(async {});
            }));
        })
    });
}
