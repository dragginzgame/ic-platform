//! Consuming one-way submission; no executor or Rust callback registration.

use super::{Call, ic0};
use std::num::NonZeroU32;

/// A locally queued one-way call, conditional on the enclosing execution commit.
///
/// This receipt does not prove delivery, remote execution or final settlement.
/// A later trap can roll back submission. No callback or retry authority is held.
#[derive(Debug, PartialEq, Eq)]
pub struct Submitted {
    _private: (),
}

/// A synchronous failure before a call was successfully queued.
///
/// IC context and request-limit violations may still trap at the system boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubmitError {
    /// All non-reused callback identifiers for this instance have been issued.
    CallbackCapacity,
    /// System cost plus attachment cannot be represented by u128.
    CostOverflow,
    /// The observed liquid balance cannot cover the required cost and attachment.
    InsufficientBalance {
        /// Available liquid cycles at admission time.
        available: u128,
        /// Required system cost plus attachment.
        required: u128,
    },
    /// Exact nonzero `call_perform` result; this is not a remote reject code.
    Refused(NonZeroU32),
}

impl Call<'_, '_> {
    /// Consumes this request and synchronously queues one one-way call.
    ///
    /// No executor, Rust callback, cleanup registration or future is created.
    /// Wait selection is intentionally ignored: there is no response wait.
    /// Success means local queue admission, conditional on this execution not
    /// subsequently trapping; it does not prove delivery or authorize retry.
    ///
    /// # Errors
    /// Returns [`SubmitError`] for cost overflow, insufficient liquid balance or
    /// a nonzero system submission result. No automatic retry is performed.
    ///
    /// # Panics
    /// The IC traps for invalid execution contexts, invalid system request
    /// limits, or a cycle balance change that prevents cycle attachment.
    pub fn notify(self) -> Result<Submitted, SubmitError> {
        self.check_balance()?;
        match NonZeroU32::new(ic0::enqueue_oneway(&self)) {
            Some(code) => Err(SubmitError::Refused(code)),
            None => Ok(Submitted { _private: () }),
        }
    }

    pub(super) fn check_balance(&self) -> Result<(), SubmitError> {
        let required = self
            .required_cycles()
            .map_err(|_| SubmitError::CostOverflow)?;
        let available = ic0::liquid_balance();
        if available < required {
            return Err(SubmitError::InsufficientBalance {
                available,
                required,
            });
        }
        Ok(())
    }
}
