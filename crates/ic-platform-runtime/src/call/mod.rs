//! One consuming request shape, with independent method and argument ownership.

use candidate_types::Principal;
use std::borrow::Cow;

mod completion;
pub use completion::{Completion, Outcome, RejectCode, Rejection};

#[cfg(all(feature = "call-ic0", target_arch = "wasm32"))]
mod ic0;

#[cfg(all(feature = "call-ic0", target_arch = "wasm32"))]
pub(crate) use ic0::replicated_execution;

#[cfg(all(feature = "call-ic0", target_arch = "wasm32"))]
pub(crate) use ic0::read_args;

#[cfg(all(feature = "call-ic0", target_arch = "wasm32"))]
mod submission;
#[cfg(all(feature = "call-ic0", target_arch = "wasm32"))]
pub use submission::{SubmitError, Submitted};

#[cfg(all(feature = "call-ic0", target_arch = "wasm32"))]
mod future;
#[cfg(any(test, all(feature = "call-ic0", target_arch = "wasm32")))]
mod registry;
#[cfg(all(feature = "call-ic0", target_arch = "wasm32"))]
pub use future::CallFuture;

/// The requested attachment plus the system cost cannot fit in u128.
/// No call was submitted or cycles reserved by the observation.
#[cfg(all(feature = "call-ic0", target_arch = "wasm32"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CostOverflow;

/// Explicit response-wait selection for a future submission backend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wait {
    /// Request bounded waiting; expiry does not establish remote nonexecution.
    Bounded {
        /// Requested timeout, preserved verbatim including zero.
        seconds: u32,
    },
    /// Request unbounded waiting.
    Unbounded,
}

/// An inert, non-cloneable request with no implicit encoding or submission.
///
/// Construction and drop perform no system calls. The opt-in wasm32 IC0 feature
/// provides consuming one-way submission and an experimental awaited transport.
/// Preparation does not establish spending or endpoint authority.
#[derive(Debug)]
#[must_use = "a prepared request has no effect by itself"]
pub struct Call<'m, 'a> {
    target: Principal,
    method: Cow<'m, str>,
    args: Cow<'a, [u8]>,
    wait: Wait,
    cycles: u128,
}

impl<'m> Call<'m, 'static> {
    /// Prepares an inert request with empty raw arguments and no attached cycles.
    ///
    /// Empty transport bytes are not an encoded unit value. The caller owns codecs.
    pub fn new(target: Principal, method: impl Into<Cow<'m, str>>, wait: Wait) -> Self {
        Self {
            target,
            method: method.into(),
            args: Cow::Borrowed(&[]),
            wait,
            cycles: 0,
        }
    }
}

impl<'m, 'a> Call<'m, 'a> {
    /// Observes the current system cost plus the requested cycle attachment.
    ///
    /// Available only with `call-ic0` on wasm32. This reads cost in an IC
    /// execution context; it does not submit, reserve cycles, validate the
    /// endpoint, or guarantee the observation remains current. Inputs must meet
    /// the system's method/payload limits for the estimate to be meaningful.
    ///
    /// # Errors
    /// Returns [`CostOverflow`] if the sum cannot be represented.
    ///
    /// # Panics
    /// The IC traps if invoked in an execution context that disallows `cost_call`.
    #[cfg(all(feature = "call-ic0", target_arch = "wasm32"))]
    pub fn required_cycles(&self) -> Result<u128, CostOverflow> {
        ic0::cost(self.method.len(), self.args.len())
            .checked_add(self.cycles)
            .ok_or(CostOverflow)
    }

    /// Replaces the argument storage with a borrow, consuming this request.
    pub fn args<'b>(self, bytes: &'b [u8]) -> Call<'m, 'b> {
        Call {
            target: self.target,
            method: self.method,
            args: Cow::Borrowed(bytes),
            wait: self.wait,
            cycles: self.cycles,
        }
    }

    /// Takes ownership of exact argument bytes, releasing any former argument borrow.
    pub fn owned_args(self, bytes: Vec<u8>) -> Call<'m, 'static> {
        Call {
            target: self.target,
            method: self.method,
            args: Cow::Owned(bytes),
            wait: self.wait,
            cycles: self.cycles,
        }
    }

    /// Sets the requested cycle attachment without reserving or spending it.
    pub const fn cycles(mut self, amount: u128) -> Self {
        self.cycles = amount;
        self
    }

    /// Returns the destination identity without resolving or authenticating it.
    #[must_use]
    pub const fn target(&self) -> Principal {
        self.target
    }

    /// Borrows the exact method name, without normalization.
    #[must_use]
    pub fn method(&self) -> &str {
        self.method.as_ref()
    }

    /// Borrows the exact argument bytes for caller-owned inspection or journaling.
    #[must_use]
    pub fn raw_args(&self) -> &[u8] {
        self.args.as_ref()
    }

    /// Returns the requested attachment, not a cost or refund observation.
    #[must_use]
    pub const fn attached_cycles(&self) -> u128 {
        self.cycles
    }

    /// Returns the explicit response-wait selection.
    #[must_use]
    pub const fn wait(&self) -> Wait {
        self.wait
    }
}

#[cfg(test)]
mod tests;
