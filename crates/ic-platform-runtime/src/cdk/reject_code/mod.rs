//! Rejection codes defined by the IC protocol.
//!
//! This catalogue identifies responses; callers own retry and recovery policy.

use std::fmt;

/// An IC rejection code, with its protocol number as the numeric cast.
///
/// Serde uses Rust unit-variant names and declaration indices, independently of
/// the protocol numbers. Conversion from an unsupported number preserves that
/// number in [`UnknownRejectCode`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub enum RejectCode {
    /// Protocol code `SYS_FATAL`.
    SysFatal = 1,
    /// Protocol code `SYS_TRANSIENT`.
    SysTransient = 2,
    /// Protocol code `DESTINATION_INVALID`.
    DestinationInvalid = 3,
    /// Protocol code `CANISTER_REJECT`.
    CanisterReject = 4,
    /// Protocol code `CANISTER_ERROR`.
    CanisterError = 5,
    /// A best-effort call has an unknown outcome.
    ///
    /// This code does not establish failure or permission to retry or replay.
    SysUnknown = 6,
}

impl TryFrom<u64> for RejectCode {
    type Error = UnknownRejectCode;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::SysFatal),
            2 => Ok(Self::SysTransient),
            3 => Ok(Self::DestinationInvalid),
            4 => Ok(Self::CanisterReject),
            5 => Ok(Self::CanisterError),
            6 => Ok(Self::SysUnknown),
            _ => Err(UnknownRejectCode(value)),
        }
    }
}

impl fmt::Display for RejectCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::SysFatal => "SYS_FATAL",
            Self::SysTransient => "SYS_TRANSIENT",
            Self::DestinationInvalid => "DESTINATION_INVALID",
            Self::CanisterReject => "CANISTER_REJECT",
            Self::CanisterError => "CANISTER_ERROR",
            Self::SysUnknown => "SYS_UNKNOWN",
        })
    }
}

/// A number outside the IC rejection catalogue, retained without truncation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnknownRejectCode(pub u64);

#[cfg(test)]
mod tests;
