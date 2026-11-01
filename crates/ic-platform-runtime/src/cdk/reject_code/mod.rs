//! Runtime reject codes adapted from ic-error-types 0.2.0 (Apache-2.0).
//! External Agent/PocketIC reject codes retain their separate upstream identities.
use serde::{Deserialize, Serialize};

/// IC transport rejection classification; numeric values are protocol-defined.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum RejectCode {
    /// Fatal system rejection.
    SysFatal = 1,
    /// Transient system rejection.
    SysTransient = 2,
    /// Destination does not exist.
    DestinationInvalid = 3,
    /// Explicit canister rejection.
    CanisterReject = 4,
    /// Canister execution failed.
    CanisterError = 5,
    /// Outcome is unknown (best-effort calls).
    SysUnknown = 6,
}

/// A numeric rejection code unknown to this runtime version.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnknownRejectCode(pub u64);

impl TryFrom<u64> for RejectCode {
    type Error = UnknownRejectCode;
    fn try_from(code: u64) -> Result<Self, Self::Error> {
        match code {
            1 => Ok(Self::SysFatal),
            2 => Ok(Self::SysTransient),
            3 => Ok(Self::DestinationInvalid),
            4 => Ok(Self::CanisterReject),
            5 => Ok(Self::CanisterError),
            6 => Ok(Self::SysUnknown),
            _ => Err(UnknownRejectCode(code)),
        }
    }
}
impl std::fmt::Display for RejectCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::SysFatal => "SYS_FATAL",
            Self::SysTransient => "SYS_TRANSIENT",
            Self::DestinationInvalid => "DESTINATION_INVALID",
            Self::CanisterReject => "CANISTER_REJECT",
            Self::CanisterError => "CANISTER_ERROR",
            Self::SysUnknown => "SYS_UNKNOWN",
        })
    }
}

#[cfg(test)]
mod tests;
