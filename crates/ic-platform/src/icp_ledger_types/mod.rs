//! Legacy ICP transfer and encoded-block/archive contracts.
//!
//! Passive external Candid contracts; see docs/external-contracts.md for pinned
//! schemas and provenance. Decoding does not authenticate a response or authorize
//! a payment, retry, topology decision or resource allocation.

use candid::CandidType;
use serde::Deserialize;

/// Tokens.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub struct Tokens {
    /// External Candid `e8s` field.
    pub e8s: u64,
}

/// TimeStamp.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub struct TimeStamp {
    /// External Candid `timestamp_nanos` field.
    pub timestamp_nanos: u64,
}

/// TransferArgs.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub struct TransferArgs {
    /// External Candid `memo` field.
    pub memo: u64,
    /// External Candid `amount` field.
    pub amount: Tokens,
    /// External Candid `fee` field.
    pub fee: Tokens,
    /// External Candid `from_subaccount` field.
    pub from_subaccount: Option<Vec<u8>>,
    /// External Candid `to` field.
    pub to: Vec<u8>,
    /// External Candid `created_at_time` field.
    pub created_at_time: Option<TimeStamp>,
}

/// TransferError.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub enum TransferError {
    /// External Candid `BadFee` alternative.
    BadFee {
        /// External Candid `expected_fee` field.
        expected_fee: Tokens,
    },
    /// External Candid `InsufficientFunds` alternative.
    InsufficientFunds {
        /// External Candid `balance` field.
        balance: Tokens,
    },
    /// External Candid `TxTooOld` alternative.
    TxTooOld {
        /// External Candid `allowed_window_nanos` field.
        allowed_window_nanos: u64,
    },
    /// External Candid `TxCreatedInFuture` alternative.
    TxCreatedInFuture,
    /// External Candid `TxDuplicate` alternative.
    TxDuplicate {
        /// External Candid `duplicate_of` field.
        duplicate_of: u64,
    },
}

/// External Candid TransferResult.
pub type TransferResult = Result<u64, TransferError>;

/// GetBlocksArgs.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub struct GetBlocksArgs {
    /// External Candid `start` field.
    pub start: u64,
    /// External Candid `length` field.
    pub length: u64,
}

/// QueryArchiveError.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub enum QueryArchiveError {
    /// External Candid `BadFirstBlockIndex` alternative.
    BadFirstBlockIndex {
        /// External Candid `requested_index` field.
        requested_index: u64,
        /// External Candid `first_valid_index` field.
        first_valid_index: u64,
    },
    /// External Candid `Other` alternative.
    Other {
        /// External Candid `error_code` field.
        error_code: u64,
        /// External Candid `error_message` field.
        error_message: String,
    },
}

/// External Candid QueryEncodedBlocksResult.
pub type QueryEncodedBlocksResult = Result<Vec<Vec<u8>>, QueryArchiveError>;

mod callback {
    #![allow(
        missing_docs,
        reason = "upstream macro generates undocumented function-reference members"
    )]
    use super::{GetBlocksArgs, QueryEncodedBlocksResult};
    candid::define_function!(pub QueryEncodedBlocksFn : (GetBlocksArgs) -> (QueryEncodedBlocksResult) query);
}
/// ICP encoded-block archive callback with its exact query signature.
pub use callback::QueryEncodedBlocksFn;

/// ArchivedEncodedBlocksRange.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub struct ArchivedEncodedBlocksRange {
    /// External Candid `callback` field.
    pub callback: QueryEncodedBlocksFn,
    /// External Candid `start` field.
    pub start: u64,
    /// External Candid `length` field.
    pub length: u64,
}

/// QueryEncodedBlocksResponse.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub struct QueryEncodedBlocksResponse {
    /// External Candid `certificate` field.
    pub certificate: Option<Vec<u8>>,
    /// External Candid `blocks` field.
    pub blocks: Vec<Vec<u8>>,
    /// External Candid `chain_length` field.
    pub chain_length: u64,
    /// External Candid `first_block_index` field.
    pub first_block_index: u64,
    /// External Candid `archived_blocks` field.
    pub archived_blocks: Vec<ArchivedEncodedBlocksRange>,
}
