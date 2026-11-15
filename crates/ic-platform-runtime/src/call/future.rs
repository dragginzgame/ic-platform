//! Lazy transport completion with integer callback custody, not an executor.

use super::{Call, Completion, SubmitError, ic0, registry::Registry};
use crate::executor::{self, TaskToken};
use std::{
    cell::RefCell,
    future::Future,
    num::NonZeroU32,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

enum State {
    Pending(Option<Waker>),
    Complete(Completion),
    Abandoned,
    Cleanup,
    Consumed,
}

type Shared = Arc<Mutex<State>>;
struct Registration {
    shared: Shared,
    task: Option<TaskToken>,
}
thread_local! {
    static CALLBACKS: RefCell<Registry<Registration>> = const { RefCell::new(Registry::new()) };
}

/// A lazy, one-shot IC transport future. Construction sends nothing.
///
/// First poll submits once. Dropping local interest does not cancel the remote
/// operation or release the physical callback registration. Response callbacks
/// capture data/refunds and wake outside internal borrows. Cleanup retires the
/// registration and local wake interest without fabricating a remote rejection.
/// A cleaned-up future never yields a completion. If first polled by the update
/// executor, its callback resumes that task and cleanup terminates that task.
/// Installed rollback qualification remains pending.
///
/// An unpolled future may move into a task. Once first polled, it must remain
/// in the same executor task (or remain outside the executor for manual polling).
/// Moving submitted interest does not move the IC callback's method context.
///
/// # Panics
/// Polling after completion or from a different executor task panics before
/// replacing the registered waker or consuming a response. IC execution-context
/// and resource-limit violations may trap at submission.
#[must_use = "an unpolled call future sends nothing"]
pub struct CallFuture<'m, 'a> {
    request: Option<Call<'m, 'a>>,
    shared: Shared,
    owner: Option<TaskToken>,
    terminal: bool,
}

impl<'m, 'a> Call<'m, 'a> {
    /// Consumes this request into an inert transport future.
    ///
    /// The first poll requires an IC call-capable execution context. The current
    /// transport captures the current executor task at first poll, when present.
    /// Manual polling outside the executor supplies no task-termination owner;
    /// installed callback/cleanup qualification remains pending.
    pub fn execute(self) -> CallFuture<'m, 'a> {
        CallFuture {
            request: Some(self),
            shared: Arc::new(Mutex::new(State::Pending(None))),
            owner: None,
            terminal: false,
        }
    }
}

fn retire(shared: &Shared, replacement: State) {
    let previous = {
        let mut state = shared.lock().unwrap();
        std::mem::replace(&mut *state, replacement)
    };
    // Waker destruction may execute arbitrary code.
    drop(previous);
}

fn take(id: u32) -> Registration {
    CALLBACKS
        .with(|registry| registry.borrow_mut().take(id))
        .expect("stale callback handle")
}

pub(super) extern "C" fn response(id: u32) {
    let Registration { shared, task } = take(id);
    // Capture before publishing or waking; these observations belong to this
    // callback, never to whichever callback happens to precede a later poll.
    let mut completion = Some(ic0::capture());
    let previous = {
        let mut state = shared.lock().unwrap();
        if matches!(*state, State::Pending(_)) {
            Some(std::mem::replace(
                &mut *state,
                State::Complete(completion.take().unwrap()),
            ))
        } else {
            assert!(matches!(*state, State::Abandoned));
            None
        }
    };
    if let Some(State::Pending(Some(waker))) = previous {
        waker.wake();
    }
    drop(completion);
    if let Some(task) = task {
        executor::drive(task);
    }
}

pub(super) extern "C" fn cleanup(id: u32) {
    // Replica rollback must restore the registration after a trapped callback.
    // Cleanup itself can trap; no committed task-termination claim is made here.
    let Registration { shared, task } = take(id);
    retire(&shared, State::Cleanup);
    if let Some(task) = task {
        executor::terminate(task);
    }
}

impl Future for CallFuture<'_, '_> {
    type Output = Result<Completion, SubmitError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        assert!(!this.terminal, "call future polled after completion");
        let current = executor::current();
        if this.request.is_some() {
            this.owner = current;
        } else {
            // Callback custody stays attached to its first polling context.
            // Reject before invoking any replacement-waker clone/drop hooks.
            assert!(
                this.owner == current,
                "call future moved between executor tasks"
            );
        }
        let mut replacement = Some(cx.waker().clone());
        let old = {
            let mut state = this.shared.lock().unwrap();
            match &mut *state {
                State::Pending(waker) => std::mem::replace(waker, replacement.take()),
                _ => None,
            }
        };
        drop(old);
        drop(replacement);

        if let Some(request) = this.request.take() {
            let result = (|| {
                request.check_balance()?;
                let shared = Arc::clone(&this.shared);
                let value = Registration {
                    shared,
                    task: this.owner,
                };
                let registration = CALLBACKS.with(|registry| registry.borrow_mut().insert(value));
                let id = registration.map_err(|value| {
                    drop(value);
                    SubmitError::CallbackCapacity
                })?;
                if let Some(code) = NonZeroU32::new(ic0::enqueue(&request, id, response, cleanup)) {
                    drop(take(id));
                    return Err(SubmitError::Refused(code));
                }
                Ok(())
            })();
            if let Err(error) = result {
                retire(&this.shared, State::Consumed);
                this.terminal = true;
                return Poll::Ready(Err(error));
            }
        }
        let result = {
            let mut state = this.shared.lock().unwrap();
            if matches!(*state, State::Complete(_)) {
                Some(std::mem::replace(&mut *state, State::Consumed))
            } else {
                None
            }
        };
        if let Some(State::Complete(completion)) = result {
            this.terminal = true;
            Poll::Ready(Ok(completion))
        } else {
            Poll::Pending
        }
    }
}

impl Drop for CallFuture<'_, '_> {
    fn drop(&mut self) {
        retire(&self.shared, State::Abandoned);
    }
}
