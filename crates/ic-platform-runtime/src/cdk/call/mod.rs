//! Raw IC call requests, owned responses, and transport failures.
//!
//! Consumers own codecs, budgets, retry decisions, and application policy.

use crate::Principal;
use std::{
    borrow::{Borrow, Cow},
    future::IntoFuture,
    ops::Deref,
};

mod future;
mod system;

pub use crate::cdk::reject_code::RejectCode;

/// Selects a bounded best-effort wait or an unbounded response wait.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WaitPolicy {
    /// Uses this timeout unchanged, including zero.
    Bounded {
        /// The best-effort timeout passed unchanged to the IC.
        timeout_seconds: u32,
    },
    /// Omits a best-effort timeout.
    Unbounded,
}

/// A borrowed method name and borrowed or owned raw argument payload.
#[derive(Clone, Debug)]
pub struct Call<'m, 'a> {
    callee: Principal,
    method: &'m str,
    args: Cow<'a, [u8]>,
    cycles: u128,
    wait: WaitPolicy,
}

impl<'m, 'a> Call<'m, 'a> {
    /// Creates a call with a 300-second bounded wait, empty bytes, and zero cycles.
    #[must_use]
    pub fn bounded_wait(callee: Principal, method: &'m str) -> Self {
        Self {
            callee,
            method,
            args: Cow::Borrowed(&[]),
            cycles: 0,
            wait: WaitPolicy::Bounded {
                timeout_seconds: 300,
            },
        }
    }

    /// Creates an unbounded call with empty bytes and zero cycles.
    #[must_use]
    pub fn unbounded_wait(callee: Principal, method: &'m str) -> Self {
        Self::bounded_wait(callee, method).with_wait_policy(WaitPolicy::Unbounded)
    }

    /// Replaces the payload with borrowed transport bytes without encoding.
    #[must_use]
    pub fn with_raw_args(mut self, args: &'a [u8]) -> Self {
        self.args = Cow::Borrowed(args);
        self
    }

    /// Replaces the payload by taking ownership of transport bytes.
    #[must_use]
    pub fn take_raw_args(mut self, args: Vec<u8>) -> Self {
        self.args = Cow::Owned(args);
        self
    }

    /// Replaces the attached cycle amount.
    #[must_use]
    pub fn with_cycles(mut self, cycles: u128) -> Self {
        self.cycles = cycles;
        self
    }

    /// Replaces the wait policy without clamping its timeout.
    #[must_use]
    pub fn with_wait_policy(mut self, wait: WaitPolicy) -> Self {
        self.wait = wait;
        self
    }

    /// Returns the saturated sum of request cost and attached cycles.
    #[must_use]
    pub fn get_cost(&self) -> u128 {
        system::cost_call(self.method.len() as u64, self.args.len() as u64)
            .saturating_add(self.cycles)
    }

    fn check_balance(&self) -> Result<(), InsufficientLiquidCycleBalance> {
        let required = self.get_cost();
        let available = system::canister_liquid_cycle_balance();
        if available < required {
            Err(InsufficientLiquidCycleBalance {
                available,
                required,
            })
        } else {
            Ok(())
        }
    }

    fn configure(&self) {
        if !self.args.is_empty() {
            system::call_data_append(&self.args);
        }
        if self.cycles != 0 {
            system::call_cycles_add128(self.cycles);
        }
        if let WaitPolicy::Bounded { timeout_seconds } = self.wait {
            system::call_with_best_effort_response(timeout_seconds);
        }
    }

    /// Submits without callbacks or method retention, leaving this builder usable.
    pub fn oneway(&self) -> Result<(), OnewayError> {
        self.check_balance()?;
        system::call_new_oneway(self.callee.as_slice(), self.method);
        self.configure();
        if system::call_perform() == 0 {
            Ok(())
        } else {
            Err(CallPerformFailed.into())
        }
    }
}

impl<'m, 'a> IntoFuture for Call<'m, 'a> {
    type Output = Result<Response, CallFailed>;
    type IntoFuture = future::CallFuture<'m, 'a>;

    fn into_future(self) -> Self::IntoFuture {
        future::CallFuture::new(self)
    }
}

/// Opaque owned response bytes, without a codec or decoding policy.
#[derive(Debug, PartialEq)]
pub struct Response(Vec<u8>);

impl Response {
    /// Takes ownership of the response payload.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

impl Deref for Response {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for Response {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Borrow<[u8]> for Response {
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}

impl PartialEq<Vec<u8>> for Response {
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.0 == *other
    }
}

impl PartialEq<&[u8]> for Response {
    fn eq(&self, other: &&[u8]) -> bool {
        self.0.as_slice() == *other
    }
}

/// Liquid cycles cannot cover the request's saturated required amount.
#[derive(Debug, Clone, thiserror::Error)]
#[error("insufficient liquid cycles balance, available: {available}, required: {required}")]
pub struct InsufficientLiquidCycleBalance {
    /// The liquid balance reported before construction.
    pub available: u128,
    /// The saturated request cost plus attached cycles.
    pub required: u128,
}

/// The IC declined to enqueue the constructed call.
#[derive(Debug, Clone, thiserror::Error)]
#[error("call perform failed")]
pub struct CallPerformFailed;

/// A raw rejection code and exact rejection text from the IC.
#[derive(Debug, Clone, thiserror::Error)]
#[error("call rejected: {code} - {message}")]
pub struct CallRejected {
    code: u32,
    message: String,
}

impl CallRejected {
    /// Retains a raw rejection code and its text without interpretation.
    #[must_use]
    pub fn with_rejection(code: u32, message: String) -> Self {
        Self { code, message }
    }

    /// Returns the complete supplier code.
    #[must_use]
    pub fn raw_reject_code(&self) -> u32 {
        self.code
    }

    /// Borrows the exact rejection text.
    #[must_use]
    pub fn reject_message(&self) -> &str {
        &self.message
    }

    /// Resolves known protocol codes while preserving unknown numeric values.
    pub fn reject_code(&self) -> Result<RejectCode, UnrecognizedRejectCode> {
        RejectCode::try_from(u64::from(self.code))
            .map_err(|_| UnrecognizedRejectCode { code: self.code })
    }
}

/// A rejection code outside the known Runtime catalogue.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unrecognized reject code: {code}")]
pub struct UnrecognizedRejectCode {
    code: u32,
}

/// An awaited call failed preflight, submission, or remote delivery.
#[derive(Debug, Clone, thiserror::Error)]
pub enum CallFailed {
    /// The liquid balance cannot cover the request.
    #[error(transparent)]
    InsufficientLiquidCycleBalance(#[from] InsufficientLiquidCycleBalance),
    /// The IC did not enqueue the request.
    #[error(transparent)]
    CallPerformFailed(#[from] CallPerformFailed),
    /// The delivered response was a rejection.
    #[error(transparent)]
    CallRejected(#[from] CallRejected),
}

/// A callback-free call failed preflight or submission.
#[derive(Debug, Clone, thiserror::Error)]
pub enum OnewayError {
    /// The liquid balance cannot cover the request.
    #[error(transparent)]
    InsufficientLiquidCycleBalance(#[from] InsufficientLiquidCycleBalance),
    /// The IC did not enqueue the request.
    #[error(transparent)]
    CallPerformFailed(#[from] CallPerformFailed),
}

#[cfg(test)]
mod tests;
