//! Selected Caffeine enrollment, callback and Cashier wire formats.
//!
//! Decoding authenticates nothing. Callers check root syntax/length/count,
//! certificate method/root equality and Cashier owner equality before effects.
//! Top-up success payloads remain opaque; these formats establish no settlement outcome.

use crate::candid::{
    self, Decode, Error, Limits, Reader, Schema,
    encode::{Encode, Writer},
    projection::{absent, projection},
};
pub use crate::candid::{Nat, encode::EncodeError};
use candidate_types::Principal;
use serde::{Deserialize, Serialize};

/// Arbitrary signed cycle balance, backed by the maintained upstream integer.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Int(pub num_bigint::BigInt);
impl std::fmt::Display for Int {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

projection! {
    ///
    /// CertificateReply
    /// Plain provider reply; this is not an IC certificate or acceptance proof.
    ///

    both CertificateReply {
        /// Caller must require its expected operation, normally `upload`.
        156_956_385 => method: String [required],
        /// Caller must compare the exact original canonical root text.
        278_101_296 => blob_hash: String [required],
    }
}

projection! {
    ///
    /// AccountRequest
    /// `account_info_get_v1` request, with canonical account identity.
    ///

    encoding AccountRequest {
        /// Exact account to inspect.
        2_707_029_165 => account: Principal,
    }
}

projection! {
    ///
    /// Gauges
    /// Provider-reported storage gauges; not availability or charge closure.
    ///

    Gauges {
        /// Reported chunk bytes.
        390_851_327 => chunk_size_bytes: u64 [required],
        /// Reported chunk count.
        2_550_751_805 => chunk_count: u64 [required],
        /// Reported blob count.
        2_562_436_397 => blob_count: u64 [required],
        /// Reported blob bytes.
        2_947_807_759 => blob_size_bytes: u64 [required],
    }
}

projection! {
    ///
    /// Account
    /// Selected Cashier account observation; caller checks the requested owner.
    ///

    Account {
        /// Exact returned owner.
        52_836_537 => account_owner_id: Principal [required],
        /// Arbitrary signed cycle balance, including negative debt.
        2_564_671_811 => cycle_balance: Int [required],
        /// Required selected gauges.
        2_695_045_597 => storage_usage: Gauges [required],
        /// Raw provider timestamp in nanoseconds.
        3_475_686_851 => last_charge_ts_ns: u64 [required],
    }
}
/// Validated, discarded known-error payload. Candid `reserved` projection
/// accepts any well-formed payload; all traversal and skipping budgets apply.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ErrorDetail;
impl Decode for ErrorDetail {
    const SCHEMA: Schema = Schema::Primitive(-16);
    fn read(reader: &mut Reader<'_, '_>, id: i64) -> Result<Self, Error> {
        reader.skip(id)?;
        Ok(Self)
    }
}

projection! {
    ///
    /// AccountError
    /// Known Cashier refusals. Unknown selected variants are refused.
    ///

    enum AccountError {
        /// No such account.
        566_069_724 => AccountNotFound,
        /// Provider operation failed.
        878_697_764 => OperationFailed(ErrorDetail),
        /// Insufficient account balance.
        1_653_746_489 => InsufficientBalance(ErrorDetail),
        /// Insufficient cycles.
        2_022_095_248 => InsufficientCycles(ErrorDetail),
        /// Provider refused authorization.
        2_102_411_630 => NotAuthorized(ErrorDetail),
        /// Provider refused arguments.
        3_028_638_836 => InvalidArgument(ErrorDetail),
        /// Account balance overflow.
        3_638_018_673 => AccountBalanceOverflow,
        /// No cycles were supplied.
        4_052_296_485 => TopUpWithoutCycles,
    }
}

projection! {
    ///
    /// AccountResult
    /// Exact selected result of `account_info_get_v1`.
    ///

    enum AccountResult {
        /// Account observation.
        17_724 => Ok(Account),
        /// Known provider refusal.
        3_456_837 => Err(AccountError),
    }
}

projection! {
    ///
    /// TopUpArgs
    /// `account_top_up_v1` argument. No reply or settlement interpretation.
    ///

    both TopUpArgs {
        /// Explicit absence versus arbitrary natural balance target.
        2_504_008_430 => target_balance: Option<Nat> [strict_optional],
        /// Explicit absence versus canonical account.
        2_707_029_165 => account: Option<Principal> [strict_optional],
    }
}
macro_rules! codec {
    ($decode:ident, $encode:ident, $ty:ty, $doc:literal) => {
        #[doc = $doc]
        /// # Errors
        /// Refuses incompatible, malformed, trailing or over-budget input.
        pub fn $decode(bytes: &[u8], limits: Limits) -> Result<$ty, Error> {
            candid::decode(bytes, limits)
        }

        #[doc = $doc]
        /// # Errors
        /// Refuses oversized output before allocation, or allocation failure.
        pub fn $encode(value: &$ty, maximum_bytes: usize) -> Result<Vec<u8>, EncodeError> {
            candid::encode::encode(value, maximum_bytes)
        }
    };
}

codec!(
    decode_root,
    encode_root,
    String,
    "Exactly one root text argument; syntax admission belongs to the caller."
);
codec!(
    decode_roots,
    encode_roots,
    Vec<Vec<u8>>,
    "Exactly one `vec blob` batch; no text-root fallback."
);
codec!(
    decode_liveness,
    encode_liveness,
    Vec<bool>,
    "Exactly one `vec bool` liveness result."
);
codec!(
    decode_certificate,
    encode_certificate,
    CertificateReply,
    "Exactly one plain provider certificate record."
);
/// Decode one Cashier result with explicit byte/header/type/work/storage/skip caps.
/// # Errors
/// Refuses incompatible, malformed, trailing or over-budget input.
pub fn decode_account(bytes: &[u8], limits: Limits) -> Result<AccountResult, Error> {
    candid::decode(bytes, limits)
}

/// Decode an exact zero-argument acknowledgement.
/// # Errors
/// Refuses nonzero arity, malformed/trailing input or exhausted budgets.
pub fn decode_empty(bytes: &[u8], limits: Limits) -> Result<(), Error> {
    candid::decode_empty(bytes, limits)
}

/// Encode an exact zero-argument acknowledgement.
/// # Errors
/// Refuses a ceiling shorter than the six-byte frame or allocation failure.
pub fn encode_empty(maximum_bytes: usize) -> Result<Vec<u8>, EncodeError> {
    const FRAME: &[u8] = b"DIDL\0\0";
    if maximum_bytes < FRAME.len() {
        return Err(EncodeError::OutputLimit);
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(FRAME.len())
        .map_err(|_| EncodeError::Allocation)?;
    bytes.extend_from_slice(FRAME);
    Ok(bytes)
}

/// Encode the enrollment probe's exact outer `opt record` argument. The outer
/// option and both record-field options preserve absence versus presence.
/// This is distinct from [`TopUpArgs::encode`], which emits the bare record.
/// # Errors
/// Refuses oversized output before allocation, or allocation failure.
pub fn encode_top_up(
    value: Option<&TopUpArgs>,
    maximum_bytes: usize,
) -> Result<Vec<u8>, EncodeError> {
    candid::encode::encode(&value, maximum_bytes)
}

impl TopUpArgs {
    /// Encode the bare top-up record (without the outer option).
    /// # Errors
    /// Refuses oversized output or allocation failure.
    pub fn encode(&self, maximum_bytes: usize) -> Result<Vec<u8>, EncodeError> {
        candid::encode::encode(self, maximum_bytes)
    }
}

/// Decode exactly one outer optional top-up record, retaining every option.
/// # Errors
/// Refuses incompatible, malformed, trailing or over-budget input.
pub fn decode_top_up(bytes: &[u8], limits: Limits) -> Result<Option<TopUpArgs>, Error> {
    candid::decode(bytes, limits)
}
/// Well-formed success payload, validated and discarded under traversal budgets.
/// This carries no account, acceptance, credit or settlement evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OpaqueTopUpSuccess;
impl Decode for OpaqueTopUpSuccess {
    const SCHEMA: Schema = Schema::Primitive(-16);
    fn read(reader: &mut Reader<'_, '_>, id: i64) -> Result<Self, Error> {
        reader.skip(id)?;
        Ok(Self)
    }
}

projection! {
    ///
    /// TopUpError
    /// Selected typed top-up refusals; unknown selected variants are refused.
    ///

    enum TopUpError {
        /// Returned unauthorized account identity; no authorization is inferred.
        2_102_411_630 => NotAuthorized(Principal),
        /// Account balance overflow.
        3_638_018_673 => AccountBalanceOverflow,
        /// Provider diagnostic text.
        3_758_147_371 => InternalError(String),
        /// No cycles supplied.
        4_052_296_485 => TopUpWithoutCycles,
    }
}

projection! {
    ///
    /// TopUpResult
    /// Top-up ABI outcome only, with an opaque validated success payload.
    ///

    enum TopUpResult {
        /// Well-formed success, without interpretation of its payload.
        17_724 => Ok(OpaqueTopUpSuccess),
        /// Known typed provider refusal.
        3_456_837 => Err(TopUpError),
    }
}

/// Decode exactly one top-up result under explicit traversal and storage limits.
/// # Errors
/// Refuses unknown selected errors, malformed selected or skipped payloads,
/// incorrect arity, trailing bytes and exhausted budgets.
pub fn decode_top_up_result(bytes: &[u8], limits: Limits) -> Result<TopUpResult, Error> {
    candid::decode(bytes, limits)
}
