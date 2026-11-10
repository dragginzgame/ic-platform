//! Ledger transfer records, opaque memos and public transfer diagnostics.

use super::account::{Account, Subaccount};
use crate::candid_codec::Nat;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use std::fmt;

/// An unbounded token amount.
pub type NumTokens = Nat;

/// An unbounded ledger block index.
pub type BlockIndex = Nat;

/// Parameters supplied for a ledger transfer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferArg {
    /// The optional source subaccount.
    #[serde(default)]
    pub from_subaccount: Option<Subaccount>,
    /// The destination account.
    pub to: Account,
    /// An optional caller-supplied fee.
    #[serde(default)]
    pub fee: Option<NumTokens>,
    /// An optional caller-supplied creation timestamp.
    #[serde(default)]
    pub created_at_time: Option<u64>,
    /// Optional opaque memo bytes.
    #[serde(default)]
    pub memo: Option<Memo>,
    /// The requested token amount.
    pub amount: NumTokens,
}

/// Opaque transfer memo bytes without an application size policy.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord, Default)]
#[serde(transparent)]
pub struct Memo(
    /// The memo's owned byte buffer.
    pub ByteBuf,
);

impl From<u64> for Memo {
    fn from(value: u64) -> Self {
        Self(ByteBuf::from(value.to_be_bytes().to_vec()))
    }
}

impl From<Vec<u8>> for Memo {
    fn from(value: Vec<u8>) -> Self {
        Self(ByteBuf::from(value))
    }
}

impl From<ByteBuf> for Memo {
    fn from(value: ByteBuf) -> Self {
        Self(value)
    }
}

impl From<Memo> for ByteBuf {
    fn from(value: Memo) -> Self {
        value.0
    }
}

/// A transfer refusal reported by a ledger.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferError {
    /// The supplied fee differs from the ledger's expected fee.
    BadFee {
        /// The fee reported by the ledger.
        expected_fee: NumTokens,
    },
    /// The burn amount is below the ledger's minimum.
    BadBurn {
        /// The minimum amount reported by the ledger.
        min_burn_amount: NumTokens,
    },
    /// The source account cannot cover the transaction.
    InsufficientFunds {
        /// The source balance reported by the ledger.
        balance: NumTokens,
    },
    /// The creation timestamp is too old for the ledger.
    TooOld,
    /// The creation timestamp is ahead of the ledger's accepted time.
    CreatedInFuture {
        /// The current ledger time reported with the refusal.
        ledger_time: u64,
    },
    /// The ledger is temporarily unable to process the request.
    TemporarilyUnavailable,
    /// The ledger recognized an earlier transaction with the same identity.
    Duplicate {
        /// The earlier transaction's block index.
        duplicate_of: BlockIndex,
    },
    /// A ledger-defined error code and message.
    GenericError {
        /// The unbounded error code.
        error_code: Nat,
        /// The message preserved verbatim.
        message: String,
    },
}

impl fmt::Display for TransferError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadFee { expected_fee } => {
                write!(formatter, "transfer fee should be {expected_fee}")
            }
            Self::BadBurn { min_burn_amount } => write!(
                formatter,
                "the minimum number of tokens to be burned is {min_burn_amount}"
            ),
            Self::InsufficientFunds { balance } => write!(
                formatter,
                "the debit account doesn't have enough funds to complete the transaction, current balance: {balance}"
            ),
            Self::TooOld => {
                formatter.write_str("transaction's created_at_time is too far in the past")
            }
            Self::CreatedInFuture { ledger_time } => write!(
                formatter,
                "transaction's created_at_time is in future, current ledger time is {ledger_time}"
            ),
            Self::TemporarilyUnavailable => {
                formatter.write_str("the ledger is temporarily unavailable")
            }
            Self::Duplicate { duplicate_of } => write!(
                formatter,
                "transaction is a duplicate of another transaction in block {duplicate_of}"
            ),
            Self::GenericError {
                error_code,
                message,
            } => write!(formatter, "{error_code} {message}"),
        }
    }
}
