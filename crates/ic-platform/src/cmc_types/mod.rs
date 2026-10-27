//! CMC top-up, mint notification and certified conversion-rate contracts.
//!
//! Passive external Candid contracts; see docs/external-contracts.md for pinned
//! schemas and provenance. Decoding does not authenticate a response or authorize
//! a payment, retry, topology decision or resource allocation.

use candid::{CandidType, Nat, Principal};
use serde::Deserialize;

/// NotifyTopUpArg.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub struct NotifyTopUpArg {
    /// External Candid `block_index` field.
    pub block_index: u64,
    /// External Candid `canister_id` field.
    pub canister_id: Principal,
}

/// NotifyError.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub enum NotifyError {
    /// External Candid `Refunded` alternative.
    Refunded {
        /// External Candid `reason` field.
        reason: String,
        /// External Candid `block_index` field.
        block_index: Option<u64>,
    },
    /// External Candid `Processing` alternative.
    Processing,
    /// External Candid `TransactionTooOld` alternative.
    TransactionTooOld(u64),
    /// External Candid `InvalidTransaction` alternative.
    InvalidTransaction(String),
    /// External Candid `Other` alternative.
    Other {
        /// External Candid `error_code` field.
        error_code: u64,
        /// External Candid `error_message` field.
        error_message: String,
    },
}

/// External Candid NotifyTopUpResult.
pub type NotifyTopUpResult = Result<Nat, NotifyError>;

/// IcpXdrConversionRate.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub struct IcpXdrConversionRate {
    /// External Candid `timestamp_seconds` field.
    pub timestamp_seconds: u64,
    /// External Candid `xdr_permyriad_per_icp` field.
    pub xdr_permyriad_per_icp: u64,
}

/// IcpXdrConversionRateResponse.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub struct IcpXdrConversionRateResponse {
    /// External Candid `data` field.
    pub data: IcpXdrConversionRate,
    /// External Candid `hash_tree` field.
    pub hash_tree: Vec<u8>,
    /// External Candid `certificate` field.
    pub certificate: Vec<u8>,
}

/// NotifyMintCyclesArg.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub struct NotifyMintCyclesArg {
    /// External Candid `block_index` field.
    pub block_index: u64,
    /// External Candid `to_subaccount` field.
    pub to_subaccount: Option<Vec<u8>>,
    /// External Candid `deposit_memo` field.
    pub deposit_memo: Option<Vec<u8>>,
}

/// NotifyMintCyclesSuccess.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub struct NotifyMintCyclesSuccess {
    /// External Candid `block_index` field.
    pub block_index: Nat,
    /// External Candid `minted` field.
    pub minted: Nat,
    /// External Candid `balance` field.
    pub balance: Nat,
}

/// External Candid NotifyMintCyclesResult.
pub type NotifyMintCyclesResult = Result<NotifyMintCyclesSuccess, NotifyError>;
