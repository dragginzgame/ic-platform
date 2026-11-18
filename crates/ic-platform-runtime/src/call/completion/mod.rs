//! Owned response observations; construction does not authenticate their origin.

#[cfg(test)]
mod tests;

use std::num::NonZeroU32;

/// Known protocol classifications, without retry or settlement policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RejectCode {
    /// Fatal system error (1).
    SysFatal = 1,
    /// Transient system error (2).
    SysTransient = 2,
    /// Invalid destination (3).
    DestinationInvalid = 3,
    /// Explicit canister rejection (4).
    CanisterReject = 4,
    /// Canister execution error (5).
    CanisterError = 5,
    /// The system stopped waiting; remote execution is uncertain (6).
    SysUnknown = 6,
}

/// Exact nonzero protocol code and rejection message.
///
/// Unknown codes are retained without interpreting their settlement semantics.
#[derive(Debug, PartialEq, Eq)]
pub struct Rejection {
    code: NonZeroU32,
    message: String,
}

impl Rejection {
    /// Returns the original protocol number, including unknown values.
    #[must_use]
    pub const fn code(&self) -> u32 {
        self.code.get()
    }

    /// Borrows the exact supplied text without normalization or truncation.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Classifies known codes; an unknown code remains available through `code`.
    #[must_use]
    pub const fn kind(&self) -> Option<RejectCode> {
        match self.code.get() {
            1 => Some(RejectCode::SysFatal),
            2 => Some(RejectCode::SysTransient),
            3 => Some(RejectCode::DestinationInvalid),
            4 => Some(RejectCode::CanisterReject),
            5 => Some(RejectCode::CanisterError),
            6 => Some(RejectCode::SysUnknown),
            _ => None,
        }
    }
}

/// A response observation, separate from submission refusal or local termination.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Owned, uninterpreted reply bytes. Application success is not inferred.
    Reply(Vec<u8>),
    /// A rejection other than SYS_UNKNOWN, including unrecognized codes.
    /// This variant does not promise the remote operation had no effects.
    Rejected(Rejection),
    /// SYS_UNKNOWN: the system stopped waiting without establishing the outcome.
    OutcomeUnknown(Rejection),
}

/// An immutable owned response and its captured refund observation.
///
/// This is a data contract, not authenticated evidence. Public constructors
/// accept supplied observations; a future backend must capture them in the
/// matching callback before invoking a waker. No constructor queries IC state.
#[derive(Debug, PartialEq, Eq)]
pub struct Completion {
    outcome: Outcome,
    refunded_cycles: Option<u128>,
}

impl Completion {
    /// Owns exact reply bytes and the already captured refund, without decoding.
    /// No arbitrary reply-size limit is imposed by this observation type.
    #[must_use]
    pub fn from_reply(bytes: Vec<u8>, refunded_cycles: Option<u128>) -> Self {
        Self {
            outcome: Outcome::Reply(bytes),
            refunded_cycles,
        }
    }

    /// Owns a rejection observation, preserving its code and text.
    ///
    /// Code zero denotes a reply and is excluded by the argument type.
    /// SYS_UNKNOWN becomes `OutcomeUnknown`; unrecognized codes remain raw
    /// rejections with no inferred retry or settlement guarantee.
    #[must_use]
    pub fn from_rejection(
        code: NonZeroU32,
        message: String,
        refunded_cycles: Option<u128>,
    ) -> Self {
        let rejection = Rejection { code, message };
        let outcome = if code.get() == 6 {
            Outcome::OutcomeUnknown(rejection)
        } else {
            Outcome::Rejected(rejection)
        };
        Self {
            outcome,
            refunded_cycles,
        }
    }

    /// Borrows the captured response without permitting mutation.
    #[must_use]
    pub const fn outcome(&self) -> &Outcome {
        &self.outcome
    }

    /// Returns the captured refund: absence differs from an observed zero.
    /// This observation is not a complete cost or settlement ledger.
    #[must_use]
    pub const fn refunded_cycles(&self) -> Option<u128> {
        self.refunded_cycles
    }

    /// Consumes the record, transferring its owned payload without copying.
    #[must_use]
    pub fn into_parts(self) -> (Outcome, Option<u128>) {
        (self.outcome, self.refunded_cycles)
    }
}
