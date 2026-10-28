//! Prepared, one-shot inter-canister calls with owned or borrowed input bytes.
//!
//! Construction is inert. Execution delegates to IC Platform's qualified raw
//! call implementation and shared executor; no callback runtime is installed.
//! Supply Wire frames or separately encoded required external Candid. Codecs,
//! resource budgets, authentication, retry and settlement policy remain caller-owned.
//! A rejected or malformed reply never grants permission to repeat an effect.

use std::borrow::Cow;

pub use ic_platform::Principal;
use ic_platform::cdk::call::Call as RawCall;
pub use ic_platform::cdk::call::{
    CallFailed, CallPerformFailed, CallRejected, InsufficientLiquidCycleBalance, RejectCode,
    Response, UnrecognizedRejectCode, WaitPolicy,
};

/// An inert request, consumed by one explicit [`execute`](Self::execute).
///
/// Method names and argument bytes may independently borrow or own their storage.
/// Owned requests can be returned after their source locals have been dropped.
/// No codec, timeout, retry or resource budget is inferred from the endpoint.
#[derive(Debug)]
#[must_use = "a prepared call does nothing until execute is polled"]
pub struct Call<'m, 'a> {
    target: Principal,
    method: Cow<'m, str>,
    args: Cow<'a, [u8]>,
    wait: WaitPolicy,
    cycles: u128,
}

impl<'m> Call<'m, 'static> {
    /// Prepare a call with an explicit wait policy, no attachment and empty raw bytes.
    ///
    /// Empty transport bytes are not an encoded unit request. Encode the selected
    /// protocol explicitly before using [`with_raw_args`](Self::with_raw_args) or
    /// [`take_raw_args`](Self::take_raw_args). Pass a `String` to own the method.
    pub fn new(target: Principal, method: impl Into<Cow<'m, str>>, wait: WaitPolicy) -> Self {
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
    /// Borrow the exact request bytes without validation, encoding or copying.
    pub fn with_raw_args<'b>(self, args: &'b [u8]) -> Call<'m, 'b> {
        Call {
            target: self.target,
            method: self.method,
            args: Cow::Borrowed(args),
            wait: self.wait,
            cycles: self.cycles,
        }
    }

    /// Own the exact request bytes, releasing any previous argument borrow.
    pub fn take_raw_args(self, args: Vec<u8>) -> Call<'m, 'static> {
        Call {
            target: self.target,
            method: self.method,
            args: Cow::Owned(args),
            wait: self.wait,
            cycles: self.cycles,
        }
    }

    /// Replace the attachment. This is a request fact, not spending authority.
    pub const fn with_cycles(mut self, cycles: u128) -> Self {
        self.cycles = cycles;
        self
    }

    /// Replace the wait policy, forwarding the timeout unchanged to the runtime.
    pub const fn with_wait_policy(mut self, wait: WaitPolicy) -> Self {
        self.wait = wait;
        self
    }

    /// Selected destination, without resolving topology or authenticating it.
    pub const fn target(&self) -> Principal {
        self.target
    }
    /// Exact method name, without normalization.
    pub fn method(&self) -> &str {
        self.method.as_ref()
    }
    /// Exact bytes available to caller-owned journals before submission.
    pub fn raw_args(&self) -> &[u8] {
        self.args.as_ref()
    }
    /// Requested attachment, which is not a refund or settlement observation.
    pub const fn attached_cycles(&self) -> u128 {
        self.cycles
    }
    /// Explicit selected wait, including zero as a bounded timeout.
    pub const fn wait_policy(&self) -> WaitPolicy {
        self.wait
    }

    /// Delegate the exact current cost query, including the attachment.
    ///
    /// This observes the current context, does not submit the request, and grants
    /// no spending or retry authority. It is not a lifetime-price guarantee.
    ///
    /// # Panics
    /// Requires an IC canister context; native construction alone does not provide one.
    pub fn get_cost(&self) -> u128 {
        self.raw().get_cost()
    }

    /// Execute once, returning the existing raw response or structured failure.
    ///
    /// The call is submitted only when this future is polled. Dropping the future
    /// retains the existing runtime's cancellation/cleanup semantics, not a claim
    /// that a remote effect was cancelled. Bounded-wait uncertainty stays uncertain.
    /// Capture matching-callback refund observations before another await; this
    /// function adds no await after the underlying call completes.
    ///
    /// # Errors
    /// Preserves Platform's submission, balance and raw rejection identities.
    /// Decode a successful response separately; a codec error is not non-submission.
    ///
    /// # Panics
    /// Must be polled in Platform's existing canister executor context. Native
    /// construction and configuration are supported, not native IC execution.
    pub async fn execute(self) -> Result<Response, CallFailed> {
        self.raw().await
    }

    fn raw(&self) -> RawCall<'_, '_> {
        RawCall::unbounded_wait(self.target, self.method.as_ref())
            .with_wait_policy(self.wait)
            .with_cycles(self.cycles)
            .with_raw_args(self.args.as_ref())
    }
}

#[cfg(test)]
mod tests;
