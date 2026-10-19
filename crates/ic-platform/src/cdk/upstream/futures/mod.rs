// Imported from ic-cdk 0.20.3; Apache-2.0. See ../NOTICE.md.
//! Protected, weak and migratory tasks on the sole registry executor.
//!
//! Entrypoints enter the original tracking update/query contexts. Protected
//! tasks panic if unfinished when their method ends; weak tasks are dropped.
//! Callback traps cancel tasks through the original executor recovery context.
//! Consumer code retains rollback and retry policy. No executor is implemented here.

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub mod internals;

/// Spawn a protected asynchronous task to run during the current canister method.
///
/// The task will panic if it outlives the canister method. To cancel it instead, use [`spawn_weak`].
pub fn spawn<F: 'static + Future<Output = ()>>(future: F) {
    pin_project_lite::pin_project! {
        struct ProtectedTask<F> {
            #[pin]
            future: F,
            completed: bool,
        }
        impl<F> PinnedDrop for ProtectedTask<F> {
            #[track_caller]
            fn drop(this: Pin<&mut Self>) {
                if !this.completed && !ic_cdk_executor::is_recovering_from_trap() {
                    panic!("protected task outlived its canister method (did you mean to use spawn_weak or spawn_migratory?)")
                }
            }
        }
    }
    impl<F> Future for ProtectedTask<F>
    where
        F: Future<Output = ()>,
    {
        type Output = ();

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.project();
            let res = this.future.poll(cx);
            if res.is_ready() {
                *this.completed = true;
            }
            res
        }
    }
    ic_cdk_executor::spawn_protected(ProtectedTask {
        future,
        completed: false,
    });
}

/// Spawn a weak asynchronous task to run during the current canister method.
///
/// If the task outlives the canister method, it will be dropped.
pub fn spawn_weak<F: 'static + Future<Output = ()>>(future: F) {
    ic_cdk_executor::spawn_protected(future);
}

/// Spawn an asynchronous task that can outlive the current canister method.
pub fn spawn_migratory<F: 'static + Future<Output = ()>>(future: F) {
    ic_cdk_executor::spawn_migratory(future);
}

/// Tells you whether the current async fn is being canceled due to a trap/panic.
///
/// In a destructor, `is_recovering_from_trap` serves the same purpose as
/// [`std::thread::panicking`] - it tells you whether the destructor is executing *because* of a trap,
/// as opposed to just because the scope was exited, so you could e.g. implement mutex poisoning.
///
/// For information about when and how this occurs, see the task lifecycle documentation.
pub fn is_recovering_from_trap() -> bool {
    ic_cdk_executor::is_recovering_from_trap()
}
