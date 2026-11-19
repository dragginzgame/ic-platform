//! Cycles-ledger creation and ICRC balance/fee formats, without transport.
//! Duplicate, refusal and uncertain observations never authorize another operation.

pub use crate::candid::{Nat, encode::EncodeError};
use crate::{
    candid::{
        self, Decode, Error, Limits, Reader, Schema,
        encode::{Encode, Writer},
        projection::{absent, projection},
    },
    management::CreationSettings,
};
use candidate_types::{Account, Principal};
use serde::{Deserialize, Serialize};

projection! {
    ///
    /// SubnetFilter
    /// Explicit optional subnet filter.
    ///

    both SubnetFilter {
        /// Absent or exact subnet type.
        3816945308 => subnet_type: Option<String> [strict_optional],
    }
}

projection! {
    ///
    /// ExplicitSubnet
    /// Explicit subnet selection.
    ///

    both ExplicitSubnet {
        /// Requested subnet.
        2584155869 => subnet: Principal [required],
    }
}

projection! {
    ///
    /// SubnetSelection
    /// Caller-selected subnet authority; no discovery.
    ///

    both enum SubnetSelection {
        /// A selected filter.
        3116995544 => Filter(SubnetFilter),
        /// An exact subnet.
        3466300157 => Subnet(ExplicitSubnet),
    }
}

projection! {
    ///
    /// CmcCreationArgs
    /// Explicit CMC creation settings.
    ///

    encoding CmcCreationArgs {
        /// Optional subnet selection.
        1337893162 => subnet_selection: Option<SubnetSelection>,
        /// Optional creation settings.
        2336062691 => settings: Option<CreationSettings>,
    }
}

projection! {
    ///
    /// CreateCanisterArgs
    /// One cycles-ledger create_canister argument.
    ///

    encoding CreateCanisterArgs {
        /// Optional exact 32-byte source subaccount.
        1835347746 => from_subaccount: Option<[u8; 32]>,
        /// Optional original timestamp.
        3258775938 => created_at_time: Option<u64>,
        /// Arbitrary cycle amount.
        3573748184 => amount: Nat,
        /// Optional CMC arguments.
        4132524221 => creation_args: Option<CmcCreationArgs>,
    }
}

projection! {
    ///
    /// Created
    /// Ledger-reported creation success.
    ///

     Created {
        /// Ledger block identity.
        430475981 => block_id: Nat [required],
        /// Created identity.
        1313628723 => canister_id: Principal [required],
    }
}

projection! {
    ///
    /// Duplicate
    /// Original transaction observation; not permission to allocate again.
    ///

     Duplicate {
        /// Original block.
        326934155 => duplicate_of: Nat [required],
        /// Optional originally created canister.
        1313628723 => canister_id: Option<Principal> [strict_optional],
    }
}

projection! {
    ///
    /// Future
    /// Ledger time refusal.
    ///

     Future {
        /// Reported ledger time.
        2709806499 => ledger_time: u64 [required],
    }
}

projection! {
    ///
    /// InsufficientFunds
    /// Ledger balance refusal.
    ///

     InsufficientFunds {
        /// Reported balance.
        596483356 => balance: Nat [required],
    }
}

projection! {
    ///
    /// FailedToCreate
    /// Failed creation with explicit refund/fee observations.
    ///

     FailedToCreate {
        /// Exact diagnostic.
        1932118984 => error: String [required],
        /// Optional refund block.
        2817723142 => refund_block: Option<Nat> [strict_optional],
        /// Optional fee block.
        3052565332 => fee_block: Option<Nat> [strict_optional],
    }
}

projection! {
    ///
    /// GenericError
    /// Unclassified ledger error.
    ///

     GenericError {
        /// Exact diagnostic.
        2584819143 => message: String [required],
        /// Arbitrary error code.
        3601615940 => error_code: Nat [required],
    }
}

projection! {
    ///
    /// CreateError
    /// All selected create_canister errors; unknown selected variants refuse.
    ///

    enum CreateError {
        /// Other error.
        260448849 => GenericError(GenericError),
        /// Temporary refusal.
        658180290 => TemporarilyUnavailable,
        /// Retained original outcome.
        1122632043 => Duplicate(Duplicate),
        /// Timestamp in future.
        2608432112 => CreatedInFuture(Future),
        /// Creation failed.
        3210455508 => FailedToCreate(FailedToCreate),
        /// Timestamp too old.
        3373249171 => TooOld,
        /// Balance refusal.
        4206284395 => InsufficientFunds(InsufficientFunds),
    }
}

projection! {
    ///
    /// CreateResult
    /// Complete ledger success/refusal outcome.
    ///

    enum CreateResult {
        /// Reported success.
        17724 => Ok(Created),
        /// Reported error, never implicit retry permission.
        3456837 => Err(CreateError),
    }
}

/// Decode one complete creation result with shared byte/work/storage budgets.
/// # Errors
/// Refuses malformed, unknown selected variants, trailing or over-budget input.
pub fn decode_created(bytes: &[u8], limits: Limits) -> Result<CreateResult, Error> {
    candid::decode(bytes, limits)
}

/// Decode exactly one arbitrary Nat balance or fee.
/// # Errors
/// Refuses wrong arity/types, malformed, trailing or over-budget input.
pub fn decode_amount(bytes: &[u8], limits: Limits) -> Result<Nat, Error> {
    candid::decode(bytes, limits)
}
impl Encode for Account {
    const SCHEMA: Schema = Schema::Record(&[
        (947296307, &<Principal as Encode>::SCHEMA),
        (1349681965, &<Option<[u8; 32]> as Encode>::SCHEMA),
    ]);
    fn write(&self, writer: &mut Writer<'_>) -> Result<(), EncodeError> {
        self.owner.write(writer)?;
        self.subaccount.write(writer)
    }
}

/// Encode one canonical ICRC account without erasing absent-versus-zero presence.
/// # Errors
/// Refuses complete output above the inclusive byte limit before allocation.
pub fn encode_balance_account(
    account: &Account,
    maximum_bytes: usize,
) -> Result<Vec<u8>, EncodeError> {
    crate::candid::encode::encode(account, maximum_bytes)
}

/// Encode exact zero arguments for icrc1_fee.
/// # Errors
/// Refuses the complete output above the inclusive limit.
pub fn encode_fee_args(maximum_bytes: usize) -> Result<Vec<u8>, EncodeError> {
    if maximum_bytes < 6 {
        Err(EncodeError::OutputLimit)
    } else {
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(6)
            .map_err(|_| EncodeError::Allocation)?;
        bytes.extend_from_slice(b"DIDL\0\0");
        Ok(bytes)
    }
}

/// ICP account identifier: CRC32 prefix and upstream SHA-224 domain digest.
/// An absent subaccount and an explicit zero subaccount have the same ICP identity.
#[must_use]
pub fn icp_account_identifier(account: &Account) -> [u8; 32] {
    use sha2::{Digest, Sha224};
    let mut hash = Sha224::new();
    hash.update(b"\x0Aaccount-id");
    hash.update(account.owner.as_slice());
    hash.update(account.effective_subaccount());
    let digest = hash.finalize();
    let mut result = [0; 32];
    result[..4].copy_from_slice(&crc32fast::hash(&digest).to_be_bytes());
    result[4..].copy_from_slice(&digest);
    result
}

/// Encode exact zero arguments for icrc1_decimals or icrc1_symbol.
/// # Errors
/// Refuses the complete output above the inclusive ceiling or allocation failure.
pub fn encode_metadata_args(maximum_bytes: usize) -> Result<Vec<u8>, EncodeError> {
    encode_fee_args(maximum_bytes)
}

/// Decode exactly one nat8 decimals observation; no balance scaling is performed.
/// # Errors
/// Refuses wrong arity/type, malformed, trailing or over-budget input.
pub fn decode_decimals(bytes: &[u8], limits: Limits) -> Result<u8, Error> {
    candid::decode(bytes, limits)
}

/// Decode exactly one symbol, charging its UTF-8 bytes before reserving storage.
/// The caller's byte, work and logical-allocation ceilings bound the text.
/// This observation is independent of a successful raw balance read.
/// # Errors
/// Refuses wrong arity/type, malformed UTF-8, trailing or over-budget input.
pub fn decode_symbol(bytes: &[u8], limits: Limits) -> Result<String, Error> {
    candid::decode(bytes, limits)
}
