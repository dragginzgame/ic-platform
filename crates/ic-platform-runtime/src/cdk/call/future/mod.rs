//! Lazy submission and owned callback completion state.

use super::{Call, CallFailed, CallPerformFailed, CallRejected, Response, system};
use crate::executor::{
    MethodHandle, TaskHandle, cancel_all_tasks_attached_to_current_method, cancel_task,
    extend_current_method_context, in_callback_executor_context_for, in_trap_recovery_context_for,
    is_recovering_from_trap,
};
use std::{
    future::Future,
    mem::DropGuard,
    pin::Pin,
    sync::{Arc, Mutex, MutexGuard},
    task::{Context, Poll, Waker},
};

#[derive(Debug)]
struct Registration {
    method: MethodHandle,
    task: Option<TaskHandle>,
    waker: Waker,
}

#[derive(Debug)]
enum State {
    Unarmed,
    Pending(Registration),
    Completed(Result<Response, CallFailed>),
    Cancelled,
    Consumed,
}

#[derive(Debug)]
struct Completion {
    state: Mutex<State>,
}

impl Completion {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(|_| system::trap("Call already trapped"))
    }

    fn arm(&self, registration: Registration) {
        let previous = {
            let mut state = self.lock();
            std::mem::replace(&mut *state, State::Pending(registration))
        };
        if !matches!(previous, State::Unarmed) {
            drop(previous);
            system::trap("Call already trapped");
        }
    }

    fn finish(&self, result: Result<Response, CallFailed>) -> Result<Registration, FinishError> {
        let mut result = Some(result);
        let outcome = {
            let mut state = self.lock();
            match &*state {
                State::Pending(_) => {
                    let previous = std::mem::replace(
                        &mut *state,
                        State::Completed(result.take().expect("the result is available")),
                    );
                    let State::Pending(registration) = previous else {
                        unreachable!("the pending state was just selected")
                    };
                    Ok(registration)
                }
                State::Cancelled => Err(FinishError::Cancelled),
                _ => Err(FinishError::Unexpected),
            }
        };
        drop(result);
        outcome
    }

    fn cancel(&self) {
        let previous = {
            let mut state = self.lock();
            std::mem::replace(&mut *state, State::Cancelled)
        };
        // Registration owns arbitrary Waker and method-handle destruction.
        drop(previous);
    }
}

enum FinishError {
    Cancelled,
    Unexpected,
}

/// The concrete future associated with a raw Call; construction submits nothing.
#[derive(Debug)]
pub struct CallFuture<'m, 'a> {
    request: Option<Call<'m, 'a>>,
    completion: Option<Arc<Completion>>,
    terminal: bool,
}

impl<'m, 'a> CallFuture<'m, 'a> {
    pub(super) fn new(request: Call<'m, 'a>) -> Self {
        Self {
            request: Some(request),
            completion: None,
            terminal: false,
        }
    }
}

impl Future for CallFuture<'_, '_> {
    type Output = Result<Response, CallFailed>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if this.terminal {
            system::trap("CallFuture polled after completing");
        }
        if let Some(request) = this.request.take() {
            return match submit(&request, context.waker()) {
                Ok(completion) => {
                    this.completion = Some(completion);
                    Poll::Pending
                }
                Err(error) => {
                    this.terminal = true;
                    Poll::Ready(Err(error))
                }
            };
        }

        let completion = this
            .completion
            .as_ref()
            .expect("submitted futures own completion state");
        // Waker clone/drop may execute arbitrary code. Neither occurs under the
        // completion lock, including when delivery raced with the cloning code.
        let mut replacement = Some(context.waker().clone());
        let mut previous_waker = None;
        let outcome = {
            let mut state = completion.lock();
            match &mut *state {
                State::Pending(registration) => {
                    previous_waker = Some(std::mem::replace(
                        &mut registration.waker,
                        replacement.take().expect("the replacement is available"),
                    ));
                    FuturePoll::Pending
                }
                State::Completed(_) => {
                    let previous = std::mem::replace(&mut *state, State::Consumed);
                    let State::Completed(result) = previous else {
                        unreachable!("the completed state was just selected")
                    };
                    FuturePoll::Ready(result)
                }
                _ => FuturePoll::Trapped,
            }
        };
        drop(previous_waker);
        drop(replacement);
        match outcome {
            FuturePoll::Pending => Poll::Pending,
            FuturePoll::Ready(result) => {
                this.terminal = true;
                Poll::Ready(result)
            }
            FuturePoll::Trapped => system::trap("Call already trapped"),
        }
    }
}

enum FuturePoll {
    Pending,
    Ready(Result<Response, CallFailed>),
    Trapped,
}

impl Drop for CallFuture<'_, '_> {
    fn drop(&mut self) {
        if !self.terminal
            && is_recovering_from_trap()
            && let Some(completion) = &self.completion
        {
            completion.cancel();
        }
    }
}

#[expect(
    unsafe_code,
    reason = "The callback ABI transfers one raw Arc ownership reference through usize"
)]
unsafe fn reclaim_environment(raw: usize) -> Arc<Completion> {
    // SAFETY: The caller must own the one reference produced by into_raw and
    // consume it exactly once, either on submission failure or committed delivery.
    unsafe { Arc::from_raw(raw as *const Completion) }
}

fn submit(request: &Call<'_, '_>, waker: &Waker) -> Result<Arc<Completion>, CallFailed> {
    request.check_balance()?;
    let completion = Arc::new(Completion {
        state: Mutex::new(State::Unarmed),
    });
    let environment = DropGuard::new(Arc::into_raw(completion.clone()) as usize, |raw| {
        // SAFETY: No successful perform transferred this environment, so
        // the supplier cannot call it and this guard is its sole owner.
        #[expect(
            unsafe_code,
            reason = "An untransferred environment still owns exactly one raw Arc reference"
        )]
        drop(unsafe { reclaim_environment(raw) });
    });
    // SAFETY: Only owned/static Completion data crosses the ABI. Reply/reject
    // share one environment with cleanup; the supplier commits exactly one
    // invocation, and cannot invoke it before perform returns.
    #[expect(
        unsafe_code,
        reason = "Registering owned completion callbacks at the reviewed IC0 ABI boundary"
    )]
    unsafe {
        system::call_new(
            request.callee.as_slice(),
            request.method,
            reply_callback,
            *environment,
            reply_callback,
            *environment,
        );
        system::call_on_cleanup(cleanup_callback, *environment);
    }
    request.configure();
    if system::call_perform() != 0 {
        return Err(CallPerformFailed.into());
    }
    // Successful perform owns the environment even if subsequent execution traps.
    // The replica's rollback, not native unwinding, governs that trap lifecycle.
    let _ = DropGuard::dismiss(environment);
    let registration = Registration {
        method: extend_current_method_context(),
        task: TaskHandle::current(),
        waker: waker.clone(),
    };
    completion.arm(registration);
    Ok(completion)
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "The callback consumes its owned completion reference and releases it after wake routing"
)]
fn deliver(completion: Arc<Completion>) {
    let code = system::msg_reject_code();
    let result = if code == 0 {
        Ok(Response(system::msg_arg_data()))
    } else {
        Err(CallRejected::with_rejection(code, system::msg_reject_msg()).into())
    };
    let registration = match completion.finish(result) {
        Ok(registration) => registration,
        Err(_) => system::trap("Call already trapped"),
    };
    let Registration {
        method,
        task: _,
        waker,
    } = registration;
    in_callback_executor_context_for(method, move || {
        waker.wake();
    });
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "Cleanup consumes its owned completion reference through cancellation and wake routing"
)]
fn cleanup(completion: Arc<Completion>) {
    let result = Err(CallRejected::with_rejection(4, "cleanup".to_owned()).into());
    let registration = match completion.finish(result) {
        Ok(registration) => registration,
        Err(FinishError::Cancelled) => return,
        Err(FinishError::Unexpected) => system::trap("Call already trapped"),
    };
    let Registration {
        method,
        task,
        waker,
    } = registration;
    in_trap_recovery_context_for(method, move || {
        drop(waker);
        cancel_all_tasks_attached_to_current_method();
        if let Some(task) = task {
            cancel_task(&task);
        }
    });
}

#[expect(
    unsafe_code,
    reason = "IC0 invokes this ABI entrypoint with one transferred completion reference"
)]
unsafe extern "C" fn reply_callback(raw: usize) {
    // SAFETY: A committed reply/reject exclusively consumes the environment.
    // If this callback traps, replica rollback restores it for cleanup instead.
    deliver(unsafe { reclaim_environment(raw) });
}

#[expect(
    unsafe_code,
    reason = "IC0 cleanup receives the environment restored by callback rollback"
)]
unsafe extern "C" fn cleanup_callback(raw: usize) {
    // SAFETY: Cleanup consumes the restored reference once. A previous trapped
    // callback has no committed ownership effects and is not another live owner.
    cleanup(unsafe { reclaim_environment(raw) });
}

#[cfg(test)]
mod tests;
