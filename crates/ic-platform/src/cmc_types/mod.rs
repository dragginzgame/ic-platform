//! Cycles-minting service request and response declarations.

use crate::candid_codec::Nat;
use crate::principal::Principal;
use serde::Deserialize;

/// A notification identifying a ledger payment for a canister top-up.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct NotifyTopUpArg {
    /// The payment's ledger block index.
    pub block_index: u64,
    /// The canister receiving the top-up.
    pub canister_id: Principal,
}

/// An error reported for a cycles-minting notification.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum NotifyError {
    /// The service reports a refund.
    Refunded {
        /// The reported refund reason.
        reason: String,
        /// The optional refund block index.
        block_index: Option<u64>,
    },
    /// The notification is still being processed.
    Processing,
    /// The service reports a transaction as too old, with its protocol value.
    TransactionTooOld(u64),
    /// The service reports an invalid transaction.
    InvalidTransaction(String),
    /// Another service error.
    Other {
        /// The reported error code.
        error_code: u64,
        /// The reported diagnostic text.
        error_message: String,
    },
}

/// The cycle amount or error returned for a top-up notification.
pub type NotifyTopUpResult = Result<Nat, NotifyError>;

/// An ICP-to-XDR conversion rate and its timestamp.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct IcpXdrConversionRate {
    /// The reported timestamp in seconds.
    pub timestamp_seconds: u64,
    /// The reported ten-thousandths of an XDR per ICP.
    pub xdr_permyriad_per_icp: u64,
}

/// A reported conversion rate with opaque certification material.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct IcpXdrConversionRateResponse {
    /// The reported rate data.
    pub data: IcpXdrConversionRate,
    /// The supplied hash-tree bytes.
    pub hash_tree: Vec<u8>,
    /// The supplied certificate bytes.
    pub certificate: Vec<u8>,
}

/// A notification identifying a payment for minting ledger cycles.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct NotifyMintCyclesArg {
    /// The payment's ledger block index.
    pub block_index: u64,
    /// The optional destination subaccount bytes.
    pub to_subaccount: Option<Vec<u8>>,
    /// The optional deposit memo bytes.
    pub deposit_memo: Option<Vec<u8>>,
}

/// The ledger outcome reported after cycles are minted.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct NotifyMintCyclesSuccess {
    /// The reported ledger block index.
    pub block_index: Nat,
    /// The reported minted cycle amount.
    pub minted: Nat,
    /// The reported balance.
    pub balance: Nat,
}

/// The outcome of a mint-cycles notification.
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
