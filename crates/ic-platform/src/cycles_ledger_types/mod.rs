//! Cycles Ledger creation, withdrawal and provider block contracts, without its service engine.
//!
//! Passive external Candid contracts; see docs/external-contracts.md for pinned
//! schemas and provenance. Decoding does not authenticate a response or authorize
//! a payment, retry, topology decision or resource allocation.

use crate::{candid_codec::Nat, principal::Principal};
use serde::Deserialize;

/// CreateCanisterArgs.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct CreateCanisterArgs {
    /// External Candid `from_subaccount` field.
    pub from_subaccount: Option<Vec<u8>>,
    /// External Candid `created_at_time` field.
    pub created_at_time: Option<u64>,
    /// External Candid `amount` field.
    pub amount: Nat,
    /// External Candid `creation_args` field.
    pub creation_args: Option<CmcCreateCanisterArgs>,
}

/// Cycles Ledger CMC creation arguments; distinct from the direct CMC method.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct CmcCreateCanisterArgs {
    /// External Candid `settings` field.
    pub settings: Option<CanisterSettings>,
    /// External Candid `subnet_selection` field.
    pub subnet_selection: Option<SubnetSelection>,
}

/// EnvironmentVariable.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct EnvironmentVariable {
    /// External Candid `name` field.
    pub name: String,
    /// External Candid `value` field.
    pub value: String,
}

/// LogVisibility.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum LogVisibility {
    /// External Candid `Controllers` alternative.
    #[serde(rename = "controllers")]
    Controllers,
    /// External Candid `Public` alternative.
    #[serde(rename = "public")]
    Public,
    /// External Candid `AllowedViewers` alternative.
    #[serde(rename = "allowed_viewers")]
    AllowedViewers(Vec<Principal>),
}

/// Cycles Ledger creation settings, not management-canister settings.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct CanisterSettings {
    /// External Candid `controllers` field.
    pub controllers: Option<Vec<Principal>>,
    /// External Candid `compute_allocation` field.
    pub compute_allocation: Option<Nat>,
    /// External Candid `memory_allocation` field.
    pub memory_allocation: Option<Nat>,
    /// External Candid `freezing_threshold` field.
    pub freezing_threshold: Option<Nat>,
    /// External Candid `reserved_cycles_limit` field.
    pub reserved_cycles_limit: Option<Nat>,
    /// External Candid `wasm_memory_limit` field.
    pub wasm_memory_limit: Option<Nat>,
    /// External Candid `wasm_memory_threshold` field.
    pub wasm_memory_threshold: Option<Nat>,
    /// External Candid `log_visibility` field.
    pub log_visibility: Option<LogVisibility>,
    /// External Candid `environment_variables` field.
    pub environment_variables: Option<Vec<EnvironmentVariable>>,
}

/// SubnetFilter.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct SubnetFilter {
    /// External Candid `subnet_type` field.
    pub subnet_type: Option<String>,
}

/// SubnetSelection.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum SubnetSelection {
    /// External Candid `Subnet` alternative.
    Subnet {
        /// External Candid `subnet` field.
        subnet: Principal,
    },
    /// External Candid `Filter` alternative.
    Filter(SubnetFilter),
}

/// CreateCanisterSuccess.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct CreateCanisterSuccess {
    /// External Candid `block_id` field.
    pub block_id: Nat,
    /// External Candid `canister_id` field.
    pub canister_id: Principal,
}

/// CreateCanisterError.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum CreateCanisterError {
    /// External Candid `InsufficientFunds` alternative.
    InsufficientFunds {
        /// External Candid `balance` field.
        balance: Nat,
    },
    /// External Candid `TooOld` alternative.
    TooOld,
    /// External Candid `CreatedInFuture` alternative.
    CreatedInFuture {
        /// External Candid `ledger_time` field.
        ledger_time: u64,
    },
    /// External Candid `TemporarilyUnavailable` alternative.
    TemporarilyUnavailable,
    /// External Candid `Duplicate` alternative.
    Duplicate {
        /// External Candid `duplicate_of` field.
        duplicate_of: Nat,
        /// External Candid `canister_id` field.
        canister_id: Option<Principal>,
    },
    /// External Candid `FailedToCreate` alternative.
    FailedToCreate {
        /// External Candid `fee_block` field.
        fee_block: Option<Nat>,
        /// External Candid `refund_block` field.
        refund_block: Option<Nat>,
        /// External Candid `error` field.
        error: String,
    },
    /// External Candid `GenericError` alternative.
    GenericError {
        /// External Candid `message` field.
        message: String,
        /// External Candid `error_code` field.
        error_code: Nat,
    },
}

/// External Candid CreateCanisterResult.
pub type CreateCanisterResult = Result<CreateCanisterSuccess, CreateCanisterError>;

/// WithdrawArgs.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct WithdrawArgs {
    /// External Candid `amount` field.
    pub amount: Nat,
    /// External Candid `from_subaccount` field.
    pub from_subaccount: Option<Vec<u8>>,
    /// External Candid `to` field.
    pub to: Principal,
    /// External Candid `created_at_time` field.
    pub created_at_time: Option<u64>,
}

/// RejectionCode.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum RejectionCode {
    /// External Candid `NoError` alternative.
    NoError,
    /// External Candid `CanisterError` alternative.
    CanisterError,
    /// External Candid `SysTransient` alternative.
    SysTransient,
    /// External Candid `DestinationInvalid` alternative.
    DestinationInvalid,
    /// External Candid `Unknown` alternative.
    Unknown,
    /// External Candid `SysFatal` alternative.
    SysFatal,
    /// External Candid `CanisterReject` alternative.
    CanisterReject,
}

/// WithdrawError.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum WithdrawError {
    /// External Candid `GenericError` alternative.
    GenericError {
        /// External Candid `message` field.
        message: String,
        /// External Candid `error_code` field.
        error_code: Nat,
    },
    /// External Candid `TemporarilyUnavailable` alternative.
    TemporarilyUnavailable,
    /// External Candid `FailedToWithdraw` alternative.
    FailedToWithdraw {
        /// External Candid `fee_block` field.
        fee_block: Option<Nat>,
        /// External Candid `rejection_code` field.
        rejection_code: RejectionCode,
        /// External Candid `rejection_reason` field.
        rejection_reason: String,
    },
    /// External Candid `Duplicate` alternative.
    Duplicate {
        /// External Candid `duplicate_of` field.
        duplicate_of: Nat,
    },
    /// External Candid `BadFee` alternative.
    BadFee {
        /// External Candid `expected_fee` field.
        expected_fee: Nat,
    },
    /// External Candid `InvalidReceiver` alternative.
    InvalidReceiver {
        /// External Candid `receiver` field.
        receiver: Principal,
    },
    /// External Candid `CreatedInFuture` alternative.
    CreatedInFuture {
        /// External Candid `ledger_time` field.
        ledger_time: u64,
    },
    /// External Candid `TooOld` alternative.
    TooOld,
    /// External Candid `InsufficientFunds` alternative.
    InsufficientFunds {
        /// External Candid `balance` field.
        balance: Nat,
    },
}

/// External Candid WithdrawResult.
pub type WithdrawResult = Result<Nat, WithdrawError>;

/// Cycles Ledger's provider block retains legacy `Value::Nat64`, unlike the
/// narrower standard ICRC-3 `ICRC3Value` projection. No value conversion is made.
#[cfg(feature = "icrc-ledger-types")]
#[derive(Clone, Debug, Deserialize)]
pub struct BlockWithId {
    /// Exact provider block index.
    pub id: Nat,
    /// Exact provider value, including the legacy Nat64 alternative.
    pub block: crate::icrc_ledger_types::Value,
}

/// Provider archive callback identity; admission and following stay caller-owned.
#[cfg(feature = "icrc-ledger-types")]
#[derive(Clone, Debug, Deserialize)]
pub struct ArchivedBlocks {
    /// Exact provider ranges, without normalization or automatic merging.
    pub args: Vec<crate::icrc_ledger_types::GetBlocksRequest>,
    /// Typed query function with this provider's complete result schema.
    pub callback: crate::icrc_ledger_types::QueryArchiveFn<
        Vec<crate::icrc_ledger_types::GetBlocksRequest>,
        GetBlocksResult,
    >,
}

/// Exact Cycles Ledger `icrc3_get_blocks` result, including legacy Nat64 values.
/// Both `cycles-ledger-types` and `icrc-ledger-types` select this provider schema.
#[cfg(feature = "icrc-ledger-types")]
#[derive(Clone, Debug, Deserialize)]
pub struct GetBlocksResult {
    /// Provider-reported total log length; not an authenticated observation.
    pub log_length: Nat,
    /// Provider blocks without standard-projection narrowing.
    pub blocks: Vec<BlockWithId>,
    /// Untrusted provider callback declarations, never followed automatically.
    pub archived_blocks: Vec<ArchivedBlocks>,
}

// Numeric labels are checked against the independent pinned Candid schema.
crate::candid_record!(CreateCanisterArgs {
    1835347746 => from_subaccount: Option<Vec<u8>>,
    3258775938 => created_at_time: Option<u64>,
    3573748184 => amount: Nat,
    4132524221 => creation_args: Option<CmcCreateCanisterArgs>,
});
crate::candid_record!(CmcCreateCanisterArgs {
    1337893162 => subnet_selection: Option<SubnetSelection>,
    2336062691 => settings: Option<CanisterSettings>,
});
crate::candid_record!(EnvironmentVariable {
    834174833 => value: String,
    1224700491 => name: String,
});
crate::candid_record!(CanisterSettings {
    238856128 => freezing_threshold: Option<Nat>,
    241629288 => wasm_memory_threshold: Option<Nat>,
    356749323 => environment_variables: Option<Vec<EnvironmentVariable>>,
    570880087 => controllers: Option<Vec<Principal>>,
    1095112320 => reserved_cycles_limit: Option<Nat>,
    2033331437 => log_visibility: Option<LogVisibility>,
    3380736376 => wasm_memory_limit: Option<Nat>,
    3844961758 => memory_allocation: Option<Nat>,
    4174053672 => compute_allocation: Option<Nat>,
});
crate::candid_record!(SubnetFilter {
    3816945308 => subnet_type: Option<String>,
});
crate::candid_record!(CreateCanisterSuccess {
    430475981 => block_id: Nat,
    1313628723 => canister_id: Principal,
});
crate::candid_record!(WithdrawArgs {
    25979 => to: Principal,
    1835347746 => from_subaccount: Option<Vec<u8>>,
    3258775938 => created_at_time: Option<u64>,
    3573748184 => amount: Nat,
});
crate::candid_variant!(LogVisibility {
    570880087 => Controllers,
    1727234217 => Public,
    2235716810 => AllowedViewers(Vec<Principal>),
});
crate::candid_variant!(SubnetSelection {
    3116995544 => Filter(SubnetFilter),
    3466300157 => Subnet{ 2584155869 => subnet: Principal },
});
crate::candid_variant!(CreateCanisterError {
    260448849 => GenericError{ 2584819143 => message: String, 3601615940 => error_code: Nat },
    658180290 => TemporarilyUnavailable,
    1122632043 => Duplicate{ 326934155 => duplicate_of: Nat, 1313628723 => canister_id: Option<Principal> },
    2608432112 => CreatedInFuture{ 2709806499 => ledger_time: u64 },
    3210455508 => FailedToCreate{ 1932118984 => error: String, 2817723142 => refund_block: Option<Nat>, 3052565332 => fee_block: Option<Nat> },
    3373249171 => TooOld,
    4206284395 => InsufficientFunds{ 596483356 => balance: Nat },
});
crate::candid_variant!(RejectionCode {
    327055271 => NoError,
    424852481 => CanisterError,
    2231315613 => SysTransient,
    2600479273 => DestinationInvalid,
    2932624234 => Unknown,
    3163375575 => SysFatal,
    3224277766 => CanisterReject,
});
crate::candid_variant!(WithdrawError {
    189253090 => FailedToWithdraw{ 2800423587 => rejection_code: RejectionCode, 3052565332 => fee_block: Option<Nat>, 3241622202 => rejection_reason: String },
    260448849 => GenericError{ 2584819143 => message: String, 3601615940 => error_code: Nat },
    658180290 => TemporarilyUnavailable,
    1122632043 => Duplicate{ 326934155 => duplicate_of: Nat },
    2142953889 => BadFee{ 3725446591 => expected_fee: Nat },
    2446930726 => InvalidReceiver{ 3808249135 => receiver: Principal },
    2608432112 => CreatedInFuture{ 2709806499 => ledger_time: u64 },
    3373249171 => TooOld,
    4206284395 => InsufficientFunds{ 596483356 => balance: Nat },
});

#[cfg(feature = "icrc-ledger-types")]
crate::candid_record!(BlockWithId {23515 => id: Nat, 3036443981 => block: crate::icrc_ledger_types::Value});
#[cfg(feature = "icrc-ledger-types")]
crate::candid_record!(ArchivedBlocks {1081380189 => args: Vec<crate::icrc_ledger_types::GetBlocksRequest>, 2131139013 => callback: crate::icrc_ledger_types::QueryArchiveFn<Vec<crate::icrc_ledger_types::GetBlocksRequest>, GetBlocksResult>});
#[cfg(feature = "icrc-ledger-types")]
crate::candid_record!(GetBlocksResult {2799807105 => log_length: Nat, 2817142406 => blocks: Vec<BlockWithId>, 4171053571 => archived_blocks: Vec<ArchivedBlocks>});
