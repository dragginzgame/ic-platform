//! Native builds share the method contracts. There is no IC message context.

#[cfg(all(test, feature = "call-ic0", feature = "system-raw"))]
mod tests;

#[cold]
#[track_caller]
pub(crate) fn requires_canister() -> ! {
    panic!("operation requires a canister execution")
}

/// IC message observations. Calling these outside a canister panics.
#[cfg(feature = "call-ic0")]
pub mod system {
    use super::requires_canister;
    pub use crate::execution_types::{ArgumentTooLarge, ReplyError};
    use ic_platform_types::Principal;

    /// # Panics
    /// A native process has no ingress payload.
    #[must_use]
    pub fn argument_size() -> usize {
        requires_canister()
    }

    /// Read the current message's arguments with an explicit bound.
    ///
    /// # Errors
    /// An oversized IC message is refused before copying.
    /// # Panics
    /// A native process has no IC message to read.
    pub fn read_args(_maximum: usize) -> Result<Vec<u8>, ArgumentTooLarge> {
        requires_canister()
    }

    /// # Errors
    /// Native execution has no current IC reply authority.
    pub fn reply(_bytes: &[u8]) -> Result<(), ReplyError> {
        Err(ReplyError::NoActiveTask)
    }

    /// # Panics
    /// A native process has no IC caller.
    #[must_use]
    pub fn caller() -> Principal {
        requires_canister()
    }

    /// # Panics
    /// A native process has no canister principal.
    #[must_use]
    pub fn canister_self() -> Principal {
        requires_canister()
    }

    /// # Panics
    /// A native process cannot observe IC message time.
    #[must_use]
    pub fn time() -> u64 {
        requires_canister()
    }

    /// # Panics
    /// A native process has no installed controller set.
    #[must_use]
    pub fn is_controller(_principal: &Principal) -> bool {
        requires_canister()
    }

    /// # Panics
    /// Ends native execution with the supplied diagnostic.
    pub fn trap(message: &str) -> ! {
        panic!("{message}")
    }
}

/// IC argument operations. Calling these outside a canister panics.
#[cfg(feature = "system-raw")]
pub mod raw {
    use super::requires_canister;

    /// # Panics
    /// A native process has no IC message argument.
    #[must_use]
    pub fn msg_arg_data_size() -> usize {
        requires_canister()
    }

    /// # Panics
    /// Refuses before writing to the destination.
    pub fn msg_arg_data_copy(_destination: &mut [u8], _offset: usize) {
        requires_canister()
    }
}

#[cfg(feature = "call-ic0")]
mod transport {
    use crate::call::{Call, Completion, SubmitError};
    use std::{
        future::Future,
        pin::Pin,
        task::{Context, Poll},
    };

    /// An unsubmitted request outside a canister. Polling reports local refusal.
    #[must_use = "an unpolled call future sends nothing"]
    pub struct CallFuture<'m, 'a> {
        _request: Call<'m, 'a>,
    }

    impl<'m, 'a> Call<'m, 'a> {
        /// Constructs a future that reports the absence of an IC on first poll.
        pub fn execute(self) -> CallFuture<'m, 'a> {
            CallFuture { _request: self }
        }
    }

    impl Future for CallFuture<'_, '_> {
        type Output = Result<Completion, SubmitError>;

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            Poll::Ready(Err(SubmitError::OutsideCanister))
        }
    }
}
#[cfg(feature = "call-ic0")]
pub use transport::CallFuture;
