//! CMC top-up, mint notification and certified conversion-rate contracts.
//!
//! Passive external Candid contracts; see docs/external-contracts.md for pinned
//! schemas and provenance. Decoding does not authenticate a response or authorize
//! a payment, retry, topology decision or resource allocation.

use crate::{candid_codec::Nat, principal::Principal};
use serde::Deserialize;

/// NotifyTopUpArg.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct NotifyTopUpArg {
    /// External Candid `block_index` field.
    pub block_index: u64,
    /// External Candid `canister_id` field.
    pub canister_id: Principal,
}

/// NotifyError.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct IcpXdrConversionRate {
    /// External Candid `timestamp_seconds` field.
    pub timestamp_seconds: u64,
    /// External Candid `xdr_permyriad_per_icp` field.
    pub xdr_permyriad_per_icp: u64,
}

/// IcpXdrConversionRateResponse.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct IcpXdrConversionRateResponse {
    /// External Candid `data` field.
    pub data: IcpXdrConversionRate,
    /// External Candid `hash_tree` field.
    pub hash_tree: Vec<u8>,
    /// External Candid `certificate` field.
    pub certificate: Vec<u8>,
}

/// NotifyMintCyclesArg.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct NotifyMintCyclesArg {
    /// External Candid `block_index` field.
    pub block_index: u64,
    /// External Candid `to_subaccount` field.
    pub to_subaccount: Option<Vec<u8>>,
    /// External Candid `deposit_memo` field.
    pub deposit_memo: Option<Vec<u8>>,
}

/// NotifyMintCyclesSuccess.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
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

// Numeric labels are checked against the independent pinned Candid schemas.
crate::candid_record!(NotifyTopUpArg {
    624086880 => block_index: u64,
    1313628723 => canister_id: Principal,
});
crate::candid_record!(IcpXdrConversionRate {
    2214623967 => xdr_permyriad_per_icp: u64,
    4174818006 => timestamp_seconds: u64,
});
crate::candid_record!(IcpXdrConversionRateResponse {
    457361687 => certificate: Vec<u8>,
    1113806378 => data: IcpXdrConversionRate,
    2709945007 => hash_tree: Vec<u8>,
});
crate::candid_record!(NotifyMintCyclesArg {
    624086880 => block_index: u64,
    982470779 => deposit_memo: Option<Vec<u8>>,
    1077262001 => to_subaccount: Option<Vec<u8>>,
});
crate::candid_record!(NotifyMintCyclesSuccess {
    596483356 => balance: Nat,
    624086880 => block_index: Nat,
    1392972673 => minted: Nat,
});
crate::candid_variant!(NotifyError {
    694863607 => Refunded{ 624086880 => block_index: Option<u64>, 4238151620 => reason: String },
    1650214983 => InvalidTransaction(String),
    3382957744 => Other{ 1389388560 => error_message: String, 3601615940 => error_code: u64 },
    3454426611 => Processing,
    4113796817 => TransactionTooOld(u64),
});
