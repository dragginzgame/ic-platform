//! Small reviewed IC system operations, independent of codecs and authorization.

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
