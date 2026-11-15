//! Small reviewed IC system operations, independent of codecs and authorization.

pub use crate::executor::ReplyError;

/// Sends exact raw reply bytes once from the current live update task.
///
/// The caller owns encoding. Empty bytes mean an empty raw reply, not encoded
/// unit. This operation appends and replies synchronously, retaining no borrow.
/// Success is conditional on the enclosing IC execution committing; it does not
/// terminate the task or guarantee delivery. Completing a task without calling
/// this function still does not issue a reply automatically.
///
/// This ordinary path shares one reply authority per task, including resumed
/// callbacks. It cannot police external raw IC0 calls or multiple manual roots
/// created for one export; generated entrypoint/reply ownership remains pending.
///
/// # Errors
/// Refuses outside a live task, during cleanup, or after this task already
/// replied. Refusal performs no IC0 reply imports.
///
/// # Panics
/// Invalid system contexts, oversized responses or an earlier raw reply can
/// trap. Installed reply/rollback qualification remains pending.
pub fn reply(bytes: &[u8]) -> Result<(), ReplyError> {
    crate::executor::claim_reply()?;
    crate::call::reply(bytes);
    Ok(())
}

/// The current message exceeds the caller's raw argument-byte allowance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArgumentTooLarge {
    /// Exact message size reported by the system.
    pub actual: usize,
    /// Caller-selected maximum, inclusive.
    pub maximum: usize,
}

/// Copies the current message's raw arguments after checking their byte length.
///
/// Run identity-only guards before calling this operation. The maximum is
/// checked before allocation and copying; equality is admitted. Returned bytes
/// are owned and survive later callbacks. This operation does not decode bytes,
/// choose a codec, enforce authorization, submit a call or issue a reply.
/// The limit bounds this argument buffer, not the caller's total heap usage.
/// Available on wasm32 with `call-ic0` during argument-capable IC executions.
/// In a reply callback the system exposes that callback's reply, not the
/// original method arguments; capture method arguments before awaiting.
///
/// # Errors
/// Returns [`ArgumentTooLarge`] without copying when the size exceeds `maximum`.
///
/// # Panics
/// System context violations or an allocation failure can trap. This function
/// does not turn traps into successful admission or an encoded error response.
pub fn read_args(maximum: usize) -> Result<Vec<u8>, ArgumentTooLarge> {
    crate::call::read_args(maximum)
}
