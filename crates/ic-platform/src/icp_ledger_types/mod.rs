//! ICP ledger transfer and encoded-block declarations.

use serde::Deserialize;

mod account_identifier;
mod callback;

pub use account_identifier::account_identifier;
pub use callback::QueryEncodedBlocksFn;

/// An ICP amount represented in hundred-millionths of a token.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Tokens {
    /// The amount in e8s.
    pub e8s: u64,
}

/// A ledger timestamp in nanoseconds.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct TimeStamp {
    /// The timestamp value.
    pub timestamp_nanos: u64,
}

/// Parameters supplied for an ICP ledger transfer.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct TransferArgs {
    /// The supplied transfer memo.
    pub memo: u64,
    /// The amount to transfer.
    pub amount: Tokens,
    /// The supplied transfer fee.
    pub fee: Tokens,
    /// The optional source subaccount bytes.
    pub from_subaccount: Option<Vec<u8>>,
    /// The destination account bytes.
    pub to: Vec<u8>,
    /// The optional creation timestamp.
    pub created_at_time: Option<TimeStamp>,
}

/// An error reported for an ICP ledger transfer.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum TransferError {
    /// The supplied fee was not accepted.
    BadFee {
        /// The reported expected fee.
        expected_fee: Tokens,
    },
    /// The available funds were insufficient.
    InsufficientFunds {
        /// The reported balance.
        balance: Tokens,
    },
    /// The transaction was reported as too old.
    TxTooOld {
        /// The reported permitted age window in nanoseconds.
        allowed_window_nanos: u64,
    },
    /// The transaction timestamp was reported as being in the future.
    TxCreatedInFuture,
    /// The transaction was reported as a duplicate.
    TxDuplicate {
        /// The original transaction's block index.
        duplicate_of: u64,
    },
}

/// A transfer's block index or reported error.
pub type TransferResult = Result<u64, TransferError>;

/// A requested range of ledger blocks.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct GetBlocksArgs {
    /// The requested starting block index.
    pub start: u64,
    /// The requested block count.
    pub length: u64,
}

/// An error reported by an encoded-block archive.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum QueryArchiveError {
    /// The archive did not accept the starting block index.
    BadFirstBlockIndex {
        /// The requested index.
        requested_index: u64,
        /// The reported first valid index.
        first_valid_index: u64,
    },
    /// Another archive error.
    Other {
        /// The reported error code.
        error_code: u64,
        /// The reported diagnostic text.
        error_message: String,
    },
}

/// Encoded blocks or an error reported by an archive.
pub type QueryEncodedBlocksResult = Result<Vec<Vec<u8>>, QueryArchiveError>;

/// An archived block range and the function that serves it.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ArchivedEncodedBlocksRange {
    /// The archive query function reference.
    pub callback: QueryEncodedBlocksFn,
    /// The reported starting block index.
    pub start: u64,
    /// The reported block count.
    pub length: u64,
}

/// Encoded ledger blocks with archive references and chain metadata.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct QueryEncodedBlocksResponse {
    /// Optional opaque certificate bytes.
    pub certificate: Option<Vec<u8>>,
    /// The returned encoded blocks.
    pub blocks: Vec<Vec<u8>>,
    /// The reported chain length.
    pub chain_length: u64,
    /// The reported first block index.
    pub first_block_index: u64,
    /// The returned archived ranges.
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
