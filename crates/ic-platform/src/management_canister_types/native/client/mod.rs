//! Additional native management request and log response declarations.

use super::super::maintained::{CanisterId, CanisterInstallMode, CanisterSettings};
use crate::candid_codec::Nat;
use serde::{Deserialize, Serialize};

/// Code-installation parameters supplied by a native client.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct InstallCodeArgs {
    /// The requested installation mode.
    pub mode: CanisterInstallMode,
    /// The destination canister.
    pub canister_id: CanisterId,
    /// Wasm module bytes using ordinary sequence Serde.
    pub wasm_module: Vec<u8>,
    /// Raw initialization bytes using byte-buffer Serde.
    #[serde(with = "serde_bytes")]
    pub arg: Vec<u8>,
    /// The sender's optional canister-version value.
    pub sender_canister_version: Option<u64>,
}

/// Optional parameters for a provisional canister-creation request.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default)]
pub struct ProvisionalCreateCanisterWithCyclesArgs {
    /// The optional unbounded cycle amount.
    pub amount: Option<Nat>,
    /// Optional canister settings supplied by the caller.
    pub settings: Option<CanisterSettings>,
    /// An optional requested canister identifier.
    pub specified_id: Option<CanisterId>,
    /// The sender's optional canister-version value.
    pub sender_canister_version: Option<u64>,
}

/// One log record returned by a canister.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct CanisterLogRecord {
    /// The reported record index.
    pub idx: u64,
    /// The reported timestamp in nanoseconds.
    pub timestamp_nanos: u64,
    /// The opaque record content using byte-buffer Serde.
    #[serde(with = "serde_bytes")]
    pub content: Vec<u8>,
}

/// Log records returned by a fetch request.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct FetchCanisterLogsResult {
    /// Records in the order supplied by the canister.
    pub canister_log_records: Vec<CanisterLogRecord>,
}
