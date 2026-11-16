use super::{
    EXECUTOR, EntryError, TaskHandle, TaskState, TaskToken, current, drive, start, terminate,
};
use std::{
    cell::{Cell, RefCell},
    future::{Future, poll_fn},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

#[test]
fn self_wakes_coalesce_and_nested_entry_refuses() {
    let polls = Rc::new(Cell::new(0));
    let observed = Rc::clone(&polls);
    let handle = start(Box::pin(poll_fn(move |cx| {
        observed.set(observed.get() + 1);
        assert!(matches!(
            start(Box::pin(async {})),
            Err(EntryError::NestedEntry)
        ));
        if observed.get() == 1 {
            cx.waker().wake_by_ref();
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    })))
    .unwrap();
    assert_eq!(polls.get(), 2);
    assert_eq!(handle.state(), TaskState::Completed);
    assert!(current().is_none());
}

struct PendingTask {
    token: Rc<Cell<Option<TaskToken>>>,
    wake: Rc<RefCell<Option<Waker>>>,
    polls: Rc<Cell<usize>>,
    handle: TaskHandle,
}
fn pending() -> PendingTask {
    let token = Rc::new(Cell::new(None));
    let wake = Rc::new(RefCell::new(None));
    let polls = Rc::new(Cell::new(0));
    let (t, w, p) = (Rc::clone(&token), Rc::clone(&wake), Rc::clone(&polls));
    let handle = start(Box::pin(poll_fn(move |cx| {
        t.set(current());
        *w.borrow_mut() = Some(cx.waker().clone());
        p.set(p.get() + 1);
        Poll::Pending
    })))
    .unwrap();
    PendingTask {
        token,
        wake,
        polls,
        handle,
    }
}

#[test]
fn only_owner_is_driven_and_stale_wake_cannot_poll_another_task() {
    let a = pending();
    let b = pending();
    a.wake.borrow().as_ref().unwrap().wake_by_ref();
    b.wake.borrow().as_ref().unwrap().wake_by_ref();
    drive(a.token.get().unwrap());
    assert_eq!(a.polls.get(), 2);
    assert_eq!(b.polls.get(), 1);
    terminate(a.token.get().unwrap());
    assert_eq!(a.handle.state(), TaskState::TerminatedByCleanup);
    a.wake.borrow().as_ref().unwrap().wake_by_ref();
    drive(a.token.get().unwrap());
    assert_eq!(b.polls.get(), 1);
    drive(b.token.get().unwrap());
    assert_eq!(b.polls.get(), 2);
    terminate(b.token.get().unwrap());
}

#[test]
fn dropping_handle_does_not_cancel_task() {
    let task = pending();
    drop(task.handle);
    task.wake.borrow().as_ref().unwrap().wake_by_ref();
    drive(task.token.get().unwrap());
    assert_eq!(task.polls.get(), 2);
    terminate(task.token.get().unwrap());
}

struct InspectDrop {
    token: Rc<Cell<Option<TaskToken>>>,
    observed: Rc<RefCell<Option<TaskHandle>>>,
    dropped: Rc<Cell<bool>>,
    stop_in_poll: bool,
    complete: Rc<Cell<bool>>,
}
impl Future for InspectDrop {
    type Output = ();
    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<()> {
        self.token.set(current());
        if self.stop_in_poll {
            terminate(current().unwrap());
            assert!(!self.dropped.get());
        }
        if self.complete.get() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
impl Drop for InspectDrop {
    fn drop(&mut self) {
        assert_eq!(current().unwrap().0, self.token.get().unwrap().0);
        // Querying executor state and attempting admission must not encounter
        // an outstanding RefCell borrow during user destruction.
        assert!(matches!(
            start(Box::pin(async {})),
            Err(EntryError::NestedEntry)
        ));
        if let Some(handle) = self.observed.borrow().as_ref() {
            assert_eq!(handle.state(), TaskState::Running);
        }
        assert_eq!(super::claim_reply(), Err(super::ReplyError::NoActiveTask));
        self.dropped.set(true);
    }
}

#[test]
fn cleanup_destroys_outside_borrows_before_publishing_termination() {
    let token = Rc::new(Cell::new(None));
    let observed = Rc::new(RefCell::new(None));
    let dropped = Rc::new(Cell::new(false));
    let handle = start(Box::pin(InspectDrop {
        token: Rc::clone(&token),
        observed: Rc::clone(&observed),
        dropped: Rc::clone(&dropped),
        stop_in_poll: false,
        complete: Rc::new(Cell::new(false)),
    }))
    .unwrap();
    *observed.borrow_mut() = Some(handle.clone());
    terminate(token.get().unwrap());
    assert!(dropped.get());
    assert_eq!(handle.state(), TaskState::TerminatedByCleanup);
    assert!(current().is_none());
    terminate(token.get().unwrap());
}

#[test]
fn reentrant_termination_defers_destruction_until_poll_returns() {
    let dropped = Rc::new(Cell::new(false));
    let handle = start(Box::pin(InspectDrop {
        token: Rc::new(Cell::new(None)),
        observed: Rc::new(RefCell::new(None)),
        dropped: Rc::clone(&dropped),
        stop_in_poll: true,
        complete: Rc::new(Cell::new(false)),
    }))
    .unwrap();
    assert!(dropped.get());
    assert_eq!(handle.state(), TaskState::TerminatedByCleanup);
    assert!(current().is_none());
}

#[test]
fn exhausted_ids_refuse_without_polling_or_wrapping() {
    let previous = EXECUTOR.with(|executor| {
        let mut executor = executor.borrow_mut();
        assert!(executor.tasks.is_empty());
        std::mem::replace(&mut executor.last, u64::MAX)
    });
    let result = start(Box::pin(async {
        panic!("refused future must not be polled")
    }));
    assert!(matches!(result, Err(EntryError::Capacity)));
    EXECUTOR.with(|executor| executor.borrow_mut().last = previous);
}

struct RefusedDrop {
    expected_context: Option<TaskToken>,
    expected_error: EntryError,
    drops: Rc<Cell<usize>>,
}

#[test]
fn reply_authority_is_per_task_and_unavailable_after_retirement() {
    use super::{ReplyError, claim_reply};
    assert_eq!(claim_reply(), Err(ReplyError::NoActiveTask));
    for _ in 0..2 {
        let handle = start(Box::pin(async {
            assert_eq!(claim_reply(), Ok(()));
            assert_eq!(claim_reply(), Err(ReplyError::AlreadyReplied));
        }))
        .unwrap();
        assert_eq!(handle.state(), TaskState::Completed);
        assert_eq!(claim_reply(), Err(ReplyError::NoActiveTask));
    }
}

impl Future for RefusedDrop {
    type Output = ();
    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<()> {
        panic!("refused future must not be polled");
    }
}

impl Drop for RefusedDrop {
    fn drop(&mut self) {
        assert!(current() == self.expected_context);
        // This admission reenters while the rejected outer future is being
        // destroyed. It must observe the same refusal, not a RefCell panic.
        assert!(matches!(start(Box::pin(async {})), Err(error) if error == self.expected_error));
        assert!(current() == self.expected_context);
        self.drops.set(self.drops.get() + 1);
    }
}

#[test]
fn nested_refusal_destroys_outside_borrow_and_preserves_active_context() {
    let drops = Rc::new(Cell::new(0));
    let observed = Rc::clone(&drops);
    let handle = start(Box::pin(async move {
        let owner = current();
        assert!(owner.is_some());
        let result = start(Box::pin(RefusedDrop {
            expected_context: owner,
            expected_error: EntryError::NestedEntry,
            drops: observed,
        }));
        assert!(matches!(result, Err(EntryError::NestedEntry)));
        assert!(current() == owner);
    }))
    .unwrap();
    assert_eq!(handle.state(), TaskState::Completed);
    assert_eq!(drops.get(), 1);
    assert!(current().is_none());
}

#[test]
fn capacity_refusal_destroys_outside_borrow_without_entering_a_task() {
    let previous = EXECUTOR.with(|executor| {
        let mut executor = executor.borrow_mut();
        assert!(executor.tasks.is_empty());
        std::mem::replace(&mut executor.last, u64::MAX)
    });
    let drops = Rc::new(Cell::new(0));
    let result = start(Box::pin(RefusedDrop {
        expected_context: None,
        expected_error: EntryError::Capacity,
        drops: Rc::clone(&drops),
    }));
    assert!(matches!(result, Err(EntryError::Capacity)));
    assert_eq!(drops.get(), 1);
    assert!(current().is_none());
    EXECUTOR.with(|executor| executor.borrow_mut().last = previous);
}

#[test]
fn completion_destroys_before_publishing_and_revokes_reply_authority() {
    let token = Rc::new(Cell::new(None));
    let observed = Rc::new(RefCell::new(None));
    let dropped = Rc::new(Cell::new(false));
    let complete = Rc::new(Cell::new(false));
    let handle = start(Box::pin(InspectDrop {
        token: Rc::clone(&token),
        observed: Rc::clone(&observed),
        dropped: Rc::clone(&dropped),
        stop_in_poll: false,
        complete: Rc::clone(&complete),
    }))
    .unwrap();
    *observed.borrow_mut() = Some(handle.clone());
    assert!(!dropped.get());
    complete.set(true);
    // Model the owning callback making its suspended task ready.
    EXECUTOR.with(|executor| {
        executor.borrow().tasks[&token.get().unwrap().0]
            .signal
            .ready
            .store(true, std::sync::atomic::Ordering::SeqCst)
    });
    drive(token.get().unwrap());
    assert!(dropped.get());
    assert_eq!(handle.state(), TaskState::Completed);
    assert!(current().is_none());
    terminate(token.get().unwrap());
    assert_eq!(handle.state(), TaskState::Completed);
}

#[test]
fn reentrant_cleanup_restores_enclosing_task_and_reply_authority() {
    let token = Rc::new(Cell::new(None));
    let observed = Rc::new(RefCell::new(None));
    let dropped = Rc::new(Cell::new(false));
    let suspended = start(Box::pin(InspectDrop {
        token: Rc::clone(&token),
        observed: Rc::clone(&observed),
        dropped: Rc::clone(&dropped),
        stop_in_poll: false,
        complete: Rc::new(Cell::new(false)),
    }))
    .unwrap();
    *observed.borrow_mut() = Some(suspended.clone());
    let enclosing = start(Box::pin(async move {
        let owner = current().unwrap();
        assert!(Some(owner) != token.get());
        terminate(token.get().unwrap());
        assert!(dropped.get());
        assert!(current() == Some(owner));
        assert_eq!(super::claim_reply(), Ok(()));
        assert_eq!(super::claim_reply(), Err(super::ReplyError::AlreadyReplied));
    }))
    .unwrap();
    assert_eq!(suspended.state(), TaskState::TerminatedByCleanup);
    assert_eq!(enclosing.state(), TaskState::Completed);
    assert!(current().is_none());
}
