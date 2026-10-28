//! Cycles Ledger creation, withdrawal and provider block contracts, without its service engine.
//!
//! Passive external Candid contracts; see docs/external-contracts.md for pinned
//! schemas and provenance. Decoding does not authenticate a response or authorize
//! a payment, retry, topology decision or resource allocation.

use candid::{CandidType, Nat, Principal};
use serde::Deserialize;

/// CreateCanisterArgs.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub struct CmcCreateCanisterArgs {
    /// External Candid `settings` field.
    pub settings: Option<CanisterSettings>,
    /// External Candid `subnet_selection` field.
    pub subnet_selection: Option<SubnetSelection>,
}

/// EnvironmentVariable.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub struct EnvironmentVariable {
    /// External Candid `name` field.
    pub name: String,
    /// External Candid `value` field.
    pub value: String,
}

/// LogVisibility.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub struct SubnetFilter {
    /// External Candid `subnet_type` field.
    pub subnet_type: Option<String>,
}

/// SubnetSelection.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub struct CreateCanisterSuccess {
    /// External Candid `block_id` field.
    pub block_id: Nat,
    /// External Candid `canister_id` field.
    pub canister_id: Principal,
}

/// CreateCanisterError.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
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
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct BlockWithId {
    /// Exact provider block index.
    pub id: Nat,
    /// Exact provider value, including the legacy Nat64 alternative.
    pub block: crate::icrc_ledger_types::Value,
}

/// Provider archive callback identity; admission and following stay caller-owned.
#[cfg(feature = "icrc-ledger-types")]
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ArchivedBlocks {
    /// Exact provider ranges, without normalization or automatic merging.
    pub args: Vec<crate::icrc_ledger_types::GetBlocksRequest>,
    /// Upstream function identity with this provider's complete result schema.
    pub callback: crate::icrc_ledger_types::QueryArchiveFn<
        Vec<crate::icrc_ledger_types::GetBlocksRequest>,
        GetBlocksResult,
    >,
}

/// Exact Cycles Ledger `icrc3_get_blocks` result, including legacy Nat64 values.
/// Both `cycles-ledger-types` and `icrc-ledger-types` select this provider schema.
#[cfg(feature = "icrc-ledger-types")]
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct GetBlocksResult {
    /// Provider-reported total log length; not an authenticated observation.
    pub log_length: Nat,
    /// Provider blocks without standard-projection narrowing.
    pub blocks: Vec<BlockWithId>,
    /// Untrusted provider callback declarations, never followed automatically.
    pub archived_blocks: Vec<ArchivedBlocks>,
}
