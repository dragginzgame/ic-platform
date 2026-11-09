//! Native install/create/log DTOs maintained from DFINITY 0.11.0 (Apache-2.0).
//! See client-provenance.json and ../maintained/LICENSE-APACHE.
use super::super::maintained::{CanisterId, CanisterInstallMode, CanisterSettings};
use crate::candid_codec::Nat;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
/// Native management protocol contract; fields preserve upstream 0.11.0.
pub struct InstallCodeArgs {
    /// Canister install mode.
    pub mode: CanisterInstallMode,
    /// Canister ID.
    pub canister_id: CanisterId,
    /// Code to be installed.
    pub wasm_module: Vec<u8>,
    /// The argument to be passed to `canister_init` or `canister_post_upgrade`.
    #[serde(with = "serde_bytes")]
    pub arg: Vec<u8>,
    /// Must match the canister's [`canister_version`](https://docs.internetcomputer.org/references/ic-interface-spec/canister-interface/#system-api-canister-version) value when specified.
    pub sender_canister_version: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default)]
/// Native management protocol contract; fields preserve upstream 0.11.0.
pub struct ProvisionalCreateCanisterWithCyclesArgs {
    /// The created canister will have this amount of cycles.
    pub amount: Option<Nat>,
    /// Canister settings.
    pub settings: Option<CanisterSettings>,
    /// If set, the canister will be created under this id.
    pub specified_id: Option<CanisterId>,
    /// Must match the canister's [`canister_version`](https://docs.internetcomputer.org/references/ic-interface-spec/canister-interface/#system-api-canister-version) value when specified.
    pub sender_canister_version: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
/// Native management protocol contract; fields preserve upstream 0.11.0.
pub struct CanisterLogRecord {
    /// The index of the log record.
    pub idx: u64,
    /// The timestamp of the log record.
    pub timestamp_nanos: u64,
    /// The content of the log record.
    #[serde(with = "serde_bytes")]
    pub content: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
/// Native management protocol contract; fields preserve upstream 0.11.0.
pub struct FetchCanisterLogsResult {
    /// The logs of the canister.
    pub canister_log_records: Vec<CanisterLogRecord>,
}
