//! ICP transfer and NNS CMC notification codecs for canister top-ups.
//!
//! Callers choose the destination, fee and timestamp and retain the original
//! transfer identity. Decoding an outcome supplies no spending or replay policy.

use super::{EncodeError, Future, GenericError, InsufficientFunds, Nat};
use crate::candid::{
    self, Decode, Error, Limits, Reader, Schema,
    encode::{Encode, Writer},
    projection::{absent, projection},
};
use ic_platform_types::{Account, Principal, Subaccount};
use serde::{Deserialize, Serialize};

/// ICP ledger method for an ICRC-1 transfer.
pub const ICRC1_TRANSFER_METHOD: &str = "icrc1_transfer";
/// Cycles Minting Canister method for notifying a top-up payment.
pub const NOTIFY_TOP_UP_METHOD: &str = "notify_top_up";
/// The CMC top-up operation memo as an eight-byte little-endian ICRC-1 blob.
pub const TOP_UP_MEMO: [u8; 8] = 0x5055_5054_u64.to_le_bytes();

///
/// Icrc1TransferArgs
/// An exact-fee, timestamped ICP transfer with an eight-byte memo.
///

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Icrc1TransferArgs {
    /// Optional source subaccount. Presence is preserved.
    pub from_subaccount: Option<Subaccount>,
    /// Exact destination account. For top-ups, use the CMC owner and beneficiary subaccount.
    pub to: Account,
    /// Amount in the ledger's smallest units.
    pub amount: Nat,
    /// Exact fee in the ledger's smallest units, encoded as present even when zero.
    pub fee: Nat,
    /// Eight-byte memo, encoded as present.
    pub memo: [u8; 8],
    /// Original ledger timestamp in nanoseconds, encoded as present.
    pub created_at_time: u64,
}

impl Icrc1TransferArgs {
    /// Encodes the same argument bytes for the same retained fields.
    ///
    /// # Errors
    /// Refuses output above the inclusive byte limit or allocation failure.
    pub fn encode(&self, maximum_bytes: usize) -> Result<Vec<u8>, EncodeError> {
        candid::encode::encode(self, maximum_bytes)
    }
}

impl Encode for Icrc1TransferArgs {
    const SCHEMA: Schema = Schema::Record(&[
        (25979, &<Account as Encode>::SCHEMA),
        (5094982, &<Option<Nat> as Encode>::SCHEMA),
        (1213809850, &<Option<[u8; 8]> as Encode>::SCHEMA),
        (1835347746, &<Option<Subaccount> as Encode>::SCHEMA),
        (3258775938, &<Option<u64> as Encode>::SCHEMA),
        (3573748184, &<Nat as Encode>::SCHEMA),
    ]);

    fn write(&self, writer: &mut Writer<'_>) -> Result<(), EncodeError> {
        self.to.write(writer)?;
        Some(&self.fee).write(writer)?;
        Some(&self.memo).write(writer)?;
        self.from_subaccount.write(writer)?;
        Some(self.created_at_time).write(writer)?;

        self.amount.write(writer)
    }
}

projection! {
    ///
    /// BadFee
    /// Transfer fee refusal with the ledger-reported fee.
    ///

    BadFee {
        /// Expected fee in the ledger's smallest units.
        3725446591 => expected_fee: Nat [required],
    }
}

projection! {
    ///
    /// BadBurn
    /// Burn amount refusal with the ledger-reported minimum.
    ///

    BadBurn {
        /// Minimum burn amount in the ledger's smallest units.
        2765068699 => min_burn_amount: Nat [required],
    }
}

projection! {
    ///
    /// TransferDuplicate
    /// The ledger's observation of the original transfer.
    ///

    TransferDuplicate {
        /// Original ledger block index.
        326934155 => duplicate_of: Nat [required],
    }
}

projection! {
    ///
    /// Icrc1TransferError
    /// A transfer refusal, duplicate, or unknown error variant.
    ///

    open Unknown enum Icrc1TransferError {
        /// Other ledger error with its code and diagnostic.
        260448849 => GenericError(GenericError),
        /// Ledger reports temporary unavailability.
        658180290 => TemporarilyUnavailable,
        /// Burn amount refusal.
        1093787796 => BadBurn(BadBurn),
        /// Original transfer block.
        1122632043 => Duplicate(TransferDuplicate),
        /// Fee mismatch.
        2142953889 => BadFee(BadFee),
        /// The timestamp is ahead of ledger time.
        2608432112 => CreatedInFuture(Future),
        /// The timestamp is too old.
        3373249171 => TooOld,
        /// Reported balance is insufficient.
        4206284395 => InsufficientFunds(InsufficientFunds),
    }
}

projection! {
    ///
    /// Icrc1TransferResult
    /// The ledger-reported transfer outcome.
    ///

    open Unknown enum Icrc1TransferResult {
        /// Successful transfer block index.
        17724 => Ok(Nat),
        /// Ledger-reported refusal or duplicate.
        3456837 => Err(Icrc1TransferError),
    }
}

projection! {
    ///
    /// NotifyTopUpArgs
    /// The original ICP payment block and the beneficiary canister.
    ///

    encoding NotifyTopUpArgs {
        /// Original ICP ledger block, using the CMC's nat64 wire width.
        624086880 => block_index: u64,
        /// Beneficiary used to derive the transfer destination subaccount.
        1313628723 => canister_id: Principal,
    }
}

projection! {
    ///
    /// Refunded
    /// CMC reports a failed top-up and refund outcome.
    ///

    Refunded {
        /// Refund block if present.
        624086880 => block_index: Option<u64> [strict_optional],
        /// Exact CMC diagnostic.
        4238151620 => reason: String [required],
    }
}

projection! {
    ///
    /// NotifyOther
    /// A CMC error outside the selected dedicated cases.
    ///

    NotifyOther {
        /// Exact CMC diagnostic.
        1389388560 => error_message: String [required],
        /// CMC error code.
        3601615940 => error_code: u64 [required],
    }
}

projection! {
    ///
    /// NotifyTopUpError
    /// CMC-reported processing, refusal, refund or unknown error variant.
    ///

    open Unknown enum NotifyTopUpError {
        /// Refund outcome and optional refund block.
        694863607 => Refunded(Refunded),
        /// The transaction does not satisfy the CMC payment protocol.
        1650214983 => InvalidTransaction(String),
        /// Other CMC error with its code and diagnostic.
        3382957744 => Other(NotifyOther),
        /// The CMC reports that the payment is still in progress.
        3454426611 => Processing,
        /// The oldest block index the CMC can process.
        4113796817 => TransactionTooOld(u64),
    }
}

projection! {
    ///
    /// NotifyTopUpResult
    /// The CMC-reported top-up outcome.
    ///

    open Unknown enum NotifyTopUpResult {
        /// Reported cycles credited to the beneficiary.
        17724 => Ok(Nat),
        /// CMC-reported processing, refusal or refund.
        3456837 => Err(NotifyTopUpError),
    }
}

/// Derives the CMC destination subaccount from the beneficiary's principal bytes.
#[must_use]
pub fn top_up_subaccount(beneficiary: &Principal) -> Subaccount {
    let bytes = beneficiary.as_slice();
    let mut account = [0; 32];
    account[0] = bytes.len() as u8;
    account[1..][..bytes.len()].copy_from_slice(bytes);

    account
}

/// Decodes one complete ICRC-1 transfer result.
///
/// Unknown variant payloads are traversed under the skipping budget and only the
/// selected label is retained. Keep the original reply bytes if needed.
///
/// # Errors
/// Refuses malformed, unsupported, over-budget or trailing input.
pub fn decode_icrc1_transfer(bytes: &[u8], limits: Limits) -> Result<Icrc1TransferResult, Error> {
    candid::decode(bytes, limits)
}

/// Decodes one complete CMC top-up notification result.
///
/// Unknown variant payloads are traversed under the skipping budget and only the
/// selected label is retained. Keep the original reply bytes if needed.
///
/// # Errors
/// Refuses malformed, unsupported, over-budget or trailing input.
pub fn decode_notify_top_up(bytes: &[u8], limits: Limits) -> Result<NotifyTopUpResult, Error> {
    candid::decode(bytes, limits)
}
