//! Legacy ICP transfer and encoded-block/archive contracts.
//!
//! External Candid contracts and selected upstream account-identifier derivation;
//! see docs/external-contracts.md for pinned schemas and provenance.
//! Decoding does not authenticate a response or authorize
//! a payment, retry, topology decision or resource allocation.

use serde::Deserialize;

mod account_identifier;
pub use account_identifier::account_identifier;

/// Tokens.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Tokens {
    /// External Candid `e8s` field.
    pub e8s: u64,
}

/// TimeStamp.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct TimeStamp {
    /// External Candid `timestamp_nanos` field.
    pub timestamp_nanos: u64,
}

/// TransferArgs.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct GetBlocksArgs {
    /// External Candid `start` field.
    pub start: u64,
    /// External Candid `length` field.
    pub length: u64,
}

/// QueryArchiveError.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
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

mod callback;
/// ICP encoded-block archive callback with its exact query signature.
pub use callback::QueryEncodedBlocksFn;

/// ArchivedEncodedBlocksRange.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ArchivedEncodedBlocksRange {
    /// External Candid `callback` field.
    pub callback: QueryEncodedBlocksFn,
    /// External Candid `start` field.
    pub start: u64,
    /// External Candid `length` field.
    pub length: u64,
}

/// QueryEncodedBlocksResponse.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
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

// Numeric labels are checked against the independent pinned ledger DID.
crate::candid_record!(Tokens {
    5035232 => e8s: u64,
});
crate::candid_record!(TimeStamp {
    268680022 => timestamp_nanos: u64,
});
crate::candid_record!(TransferArgs {
    25979 => to: Vec<u8>,
    5094982 => fee: Tokens,
    1213809850 => memo: u64,
    1835347746 => from_subaccount: Option<Vec<u8>>,
    3258775938 => created_at_time: Option<TimeStamp>,
    3573748184 => amount: Tokens,
});
crate::candid_record!(GetBlocksArgs {
    2215343202 => start: u64,
    2668074214 => length: u64,
});
crate::candid_record!(ArchivedEncodedBlocksRange {
    2131139013 => callback: QueryEncodedBlocksFn,
    2215343202 => start: u64,
    2668074214 => length: u64,
});
crate::candid_record!(QueryEncodedBlocksResponse {
    457361687 => certificate: Option<Vec<u8>>,
    2817142406 => blocks: Vec<Vec<u8>>,
    3385467812 => chain_length: u64,
    3527769489 => first_block_index: u64,
    4171053571 => archived_blocks: Vec<ArchivedEncodedBlocksRange>,
});
crate::candid_variant!(TransferError {
    195874615 => TxTooOld{ 3919368871 => allowed_window_nanos: u64 },
    2142953889 => BadFee{ 3725446591 => expected_fee: Tokens },
    2932695879 => TxDuplicate{ 326934155 => duplicate_of: u64 },
    4060984012 => TxCreatedInFuture,
    4206284395 => InsufficientFunds{ 596483356 => balance: Tokens },
});
crate::candid_variant!(QueryArchiveError {
    1250105008 => BadFirstBlockIndex{ 1421906497 => requested_index: u64,
    1538714240 => first_valid_index: u64 },
    3382957744 => Other{ 1389388560 => error_message: String,
    3601615940 => error_code: u64 },
});
