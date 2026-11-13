//! Task lifetime policies over Runtime's sole executor.
//!
//! These adapters select cancellation policy; they do not create execution
//! contexts or provide scheduling, transport, or retry policy.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use pin_project_lite::pin_project;

pub mod internals;

pin_project! {
    struct ProtectedTask<F> {
        #[pin]
        future: F,
        completed: bool,
    }

    impl<F> PinnedDrop for ProtectedTask<F> {
        fn drop(this: Pin<&mut Self>) {
            let this = this.project();
            if !*this.completed && !crate::executor::is_recovering_from_trap() {
                panic!(
                    "protected task outlived its canister method (did you mean to use spawn_weak or spawn_migratory?)"
                );
            }
        }
    }
}

impl<F: Future<Output = ()>> Future for ProtectedTask<F> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<()> {
        let this = self.project();
        let result = this.future.poll(context);
        if result.is_ready() {
            // Record completion before any inner-future destructor can run.
            *this.completed = true;
        }
        result
    }
}

/// Spawns a method-owned task that must finish before ordinary cancellation.
///
/// Trap recovery may cancel unfinished work without an additional panic. Use
/// [`spawn_weak`] when ordinary cancellation is acceptable, or [`spawn_migratory`]
/// when the task must survive its originating method.
///
/// # Panics
///
/// Panics without an active tracked method or during trap recovery. Dropping an
/// unfinished task outside recovery also panics, including before its first poll.
pub fn spawn<F: Future<Output = ()> + 'static>(future: F) {
    crate::executor::spawn_protected(ProtectedTask {
        future,
        completed: false,
    });
}

/// Spawns a method-owned task whose unfinished future may be cancelled.
///
/// # Panics
///
/// Panics without an active tracked method or during trap recovery.
pub fn spawn_weak<F: Future<Output = ()> + 'static>(future: F) {
    crate::executor::spawn_protected(future);
}

/// Spawns a task that may continue in later update contexts after its method ends.
///
/// # Panics
///
/// Panics outside an executor context, in a query context, or during trap recovery.
pub fn spawn_migratory<F: Future<Output = ()> + 'static>(future: F) {
    crate::executor::spawn_migratory(future);
}

/// Reports whether the current context is explicit trap recovery.
#[must_use]
pub fn is_recovering_from_trap() -> bool {
    crate::executor::is_recovering_from_trap()
}

#[cfg(test)]
mod tests;
