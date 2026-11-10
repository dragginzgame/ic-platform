use super::*;
use std::{cell::Cell, future::poll_fn, rc::Rc};

fn isolated(test: impl FnOnce() + Send + 'static) {
    std::thread::spawn(test)
        .join()
        .expect("isolated executor test failed");
}

fn assert_reclaimed() {
    EXECUTOR.with(|executor| {
        let executor = executor.borrow();
        assert!(executor.tasks.is_empty());
        assert!(executor.methods.is_empty());
        assert!(executor.migratory_ready.is_empty());
        assert!(executor.context.is_none());
        assert!(executor.current_task.is_none());
    });
}

#[test]
fn completed_tasks_leave_no_live_records_members_or_queued_keys() {
    isolated(|| {
        for _ in 0..64 {
            let method = in_tracking_executor_context(|| {
                for _ in 0..64 {
                    spawn_protected(async {});
                    spawn_migratory(async {});
                }
                extend_current_method_context()
            });
            EXECUTOR.with(|executor| {
                let executor = executor.borrow();
                assert!(executor.tasks.is_empty());
                assert!(executor.migratory_ready.is_empty());
                assert_eq!(executor.methods.len(), 1);
                let retained = executor.methods.get(method.method.unwrap()).unwrap();
                assert!(retained.members.is_empty());
                assert!(retained.ready.is_empty());
                assert_eq!(retained.retentions, 1);
            });
            drop(method);
            assert_reclaimed();
        }
    });
}

struct DropProbe {
    tag: usize,
    order: Rc<RefCell<Vec<usize>>>,
    child_polls: Rc<Cell<usize>>,
    spawn_child: bool,
}

impl Future for DropProbe {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<()> {
        Poll::Pending
    }
}

impl Drop for DropProbe {
    fn drop(&mut self) {
        self.order.borrow_mut().push(self.tag);
        if self.spawn_child {
            let child_polls = self.child_polls.clone();
            spawn_protected(async move {
                child_polls.set(child_polls.get() + 1);
            });
        }
    }
}

#[test]
fn cancellation_preserves_member_order_and_bulk_detaches_before_destructors() {
    isolated(|| {
        let order = Rc::new(RefCell::new(Vec::new()));
        let child_polls = Rc::new(Cell::new(0));
        in_tracking_executor_context(|| {
            let first = spawn_protected(DropProbe {
                tag: 1,
                order: order.clone(),
                child_polls: child_polls.clone(),
                spawn_child: true,
            });
            let middle = spawn_protected(DropProbe {
                tag: 2,
                order: order.clone(),
                child_polls: child_polls.clone(),
                spawn_child: false,
            });
            let last = spawn_protected(DropProbe {
                tag: 3,
                order: order.clone(),
                child_polls: child_polls.clone(),
                spawn_child: false,
            });
            cancel_task(&middle);
            EXECUTOR.with(|executor| {
                let executor = executor.borrow();
                let owner = executor.context.unwrap().method.unwrap();
                let method = executor.methods.get(owner).unwrap();
                assert_eq!(method.members, [first.id, last.id]);
                assert_eq!(
                    method.ready.iter().copied().collect::<Vec<_>>(),
                    [first.id, last.id]
                );
                assert_eq!(executor.tasks.len(), 2);
            });
            cancel_all_tasks_attached_to_current_method();
            assert_eq!(*order.borrow(), [2, 1, 3]);
            EXECUTOR.with(|executor| {
                let executor = executor.borrow();
                let owner = executor.context.unwrap().method.unwrap();
                let method = executor.methods.get(owner).unwrap();
                assert_eq!(method.members.len(), 1);
                assert_eq!(method.ready.len(), 1);
                assert_eq!(executor.tasks.len(), 1);
            });
            // Handles from the detached batch cannot cancel destructor-spawned work.
            cancel_task(&first);
            cancel_task(&middle);
            cancel_task(&last);
        });
        assert_eq!(child_polls.get(), 1);
        assert_reclaimed();
    });
}

struct PollDropProbe {
    drops: Rc<Cell<usize>>,
    saw_current_task: Rc<Cell<bool>>,
    saw_surrounding_context: Rc<Cell<bool>>,
}

impl Drop for PollDropProbe {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
        self.saw_current_task.set(TaskHandle::current().is_some());
        self.saw_surrounding_context.set(EXECUTOR.with(|executor| {
            let executor = executor.borrow();
            executor
                .context
                .is_some_and(|context| context.method.is_some())
        }));
    }
}

#[test]
fn self_cancellation_while_pending_reclaims_once_after_restoring_context() {
    isolated(|| {
        let drops = Rc::new(Cell::new(0));
        let saw_current_task = Rc::new(Cell::new(false));
        let saw_surrounding_context = Rc::new(Cell::new(false));
        in_tracking_executor_context(|| {
            let guard = PollDropProbe {
                drops: drops.clone(),
                saw_current_task: saw_current_task.clone(),
                saw_surrounding_context: saw_surrounding_context.clone(),
            };
            spawn_protected(poll_fn(move |context| {
                let _keep_drop_probe = &guard;
                context.waker().wake_by_ref();
                cancel_task(&TaskHandle::current().unwrap());
                Poll::Pending
            }));
        });
        assert_eq!(drops.get(), 1);
        assert!(!saw_current_task.get());
        assert!(saw_surrounding_context.get());
        assert_reclaimed();
    });
}

#[test]
fn stale_task_generations_do_not_requeue_replacement_slots() {
    isolated(|| {
        let old_waker = Rc::new(RefCell::new(None::<Waker>));
        let first = in_tracking_executor_context(|| {
            let old_waker = old_waker.clone();
            spawn_migratory(poll_fn(move |context| {
                *old_waker.borrow_mut() = Some(context.waker().clone());
                Poll::Pending
            }))
        });
        cancel_task(&first);
        assert_reclaimed();

        let new_waker = Rc::new(RefCell::new(None::<Waker>));
        let polls = Rc::new(Cell::new(0));
        let replacement = in_tracking_executor_context(|| {
            let new_waker = new_waker.clone();
            let polls = polls.clone();
            spawn_migratory(poll_fn(move |context| {
                polls.set(polls.get() + 1);
                *new_waker.borrow_mut() = Some(context.waker().clone());
                if polls.get() == 2 {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            }))
        });
        assert_ne!(first.id, replacement.id);
        for _ in 0..1024 {
            cancel_task(&first);
            old_waker.borrow().as_ref().unwrap().wake_by_ref();
        }
        EXECUTOR.with(|executor| {
            let executor = executor.borrow();
            assert_eq!(executor.tasks.len(), 1);
            assert!(executor.migratory_ready.is_empty());
        });
        in_tracking_executor_context(|| {});
        assert_eq!(polls.get(), 1);
        new_waker.borrow().as_ref().unwrap().wake_by_ref();
        in_tracking_executor_context(|| {});
        assert_eq!(polls.get(), 2);
        cancel_task(&replacement);
        assert_reclaimed();
    });
}

#[test]
fn final_retention_release_reclaims_protected_tasks_and_method_storage() {
    isolated(|| {
        for _ in 0..128 {
            let (first, second) = in_tracking_executor_context(|| {
                for _ in 0..16 {
                    spawn_protected(std::future::pending());
                }
                (
                    extend_current_method_context(),
                    extend_current_method_context(),
                )
            });
            drop(first);
            EXECUTOR.with(|executor| {
                let executor = executor.borrow();
                assert_eq!(executor.methods.len(), 1);
                assert_eq!(executor.tasks.len(), 16);
                let method = executor.methods.get(second.method.unwrap()).unwrap();
                assert_eq!(method.retentions, 1);
                assert_eq!(method.members.len(), 16);
                assert!(method.ready.is_empty());
            });
            drop(second);
            assert_reclaimed();
        }
    });
}
