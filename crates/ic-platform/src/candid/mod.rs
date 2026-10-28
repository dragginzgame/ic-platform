//! Caller-bounded Candid reply decoding, independent of PocketIC or transport.
//!
//! Callers select finite work limits, authenticate sources and retain original
//! bytes. Ordinary single-result decoding permits Candid's bounded skipping of
//! additional values; exact-single-result decoding rejects them.
//! All decoders select Candid's compact diagnostics while retaining typed causes.
//! Compact diagnostics can include small inputs; callers own error disclosure.
//!
//! These wire/work quotas are not a Wasm recursion-depth guarantee. The selected
//! upstream decoder has no supported Wasm depth ceiling: a small, deeply nested
//! external reply can trap rather than return [`DecodeError`]. See
//! [the nesting issue](https://github.com/dragginzgame/ic-platform/issues/73).
//! Low-level [`contract`] APIs retain caller-owned configuration and budgeting.

use candid::{
    CandidType,
    de::{DecoderConfig, IDLDeserialize},
};
use serde::Deserialize;
use std::{error::Error, fmt};

pub mod decoder;
pub mod reserved;
pub mod sequence;

/// External Candid contract types, encoders and configured decoding.
///
/// Derive through `contract::CandidType` with
/// `#[candid_path("ic_platform::candid::contract")]`. Serde derives need
/// `#[serde(crate = "ic_platform::candid::contract::serde")]` when Serde is
/// reached only through this boundary. No upstream crate alias is required.
///
/// `DecoderConfig` and `decode_args_with_config` retain upstream configuration;
/// they do not select the parent module's finite-limit helper policy.
/// Service declaration macros and printer machinery require `candid-service`.
pub mod contract {
    pub use crate::principal::ExternalPrincipal as Principal;
    pub use candid::de::DecoderConfig;
    pub use candid::utils::{ArgumentDecoder, ArgumentEncoder, encode_args_ref};
    pub use candid::{
        CandidType, Decode, Encode, Error, Int, Nat, Reserved, TypeEnv, decode_args_with_config,
        encode_args, encode_one, idl_hash,
    };
    #[cfg(feature = "candid-service")]
    pub use candid::{candid_method, export_service};
    pub use serde;

    /// Service generation machinery, enabled only by `candid-service`.
    #[cfg(feature = "candid-service")]
    pub mod pretty;

    /// Trait machinery required by the qualified upstream Candid derive.
    /// This is an explicit upstream-coupled contract, not an independent codec.
    pub mod types {
        pub mod internal;
        pub mod subtype;
        pub use candid::types::{
            CandidType, Compound, Field, FieldDoc, FuncMode, Function, Label, Serializer, Type,
            TypeDoc, TypeId, TypeInner,
        };
    }
}

#[cfg(test)]
mod tests;

/// Caller-selected wire, decoding, skipping, type-table and header ceilings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecodeLimits {
    /// Maximum original reply bytes admitted before decoder construction.
    pub reply_bytes: usize,
    /// Maximum Candid decoding work; zero permits no charged work.
    pub decoding_quota: usize,
    /// Maximum work spent skipping values not requested by the caller.
    pub skipping_quota: usize,
    /// Maximum number of type-table entries in the wire header.
    pub type_entries: usize,
    /// Maximum wire bytes occupied by the Candid header.
    pub header_bytes: usize,
}

/// Wire admission or Candid decoding failure, retaining the original decoder cause.
#[derive(Debug)]
pub enum DecodeError {
    /// Original wire reply exceeds the selected byte ceiling.
    ReplyTooLarge {
        /// Observed original reply length.
        actual: usize,
        /// Caller-selected maximum reply length.
        maximum: usize,
    },
    /// Candid shape, work quota, truncation or exact-result admission failed.
    Candid(candid::Error),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReplyTooLarge { actual, maximum } => {
                write!(f, "Candid reply is {actual} bytes, maximum is {maximum}")
            }
            Self::Candid(error) => write!(f, "Candid reply decoding failed: {error}"),
        }
    }
}

impl Error for DecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ReplyTooLarge { .. } => None,
            Self::Candid(error) => Some(error),
        }
    }
}

fn config(bytes: &[u8], limits: DecodeLimits) -> Result<DecoderConfig, DecodeError> {
    if bytes.len() > limits.reply_bytes {
        return Err(DecodeError::ReplyTooLarge {
            actual: bytes.len(),
            maximum: limits.reply_bytes,
        });
    }
    let mut config = DecoderConfig::new();
    config
        .set_decoding_quota(limits.decoding_quota)
        .set_skipping_quota(limits.skipping_quota)
        .set_max_type_len(limits.type_entries)
        .set_max_header_len(limits.header_bytes)
        .set_full_error_message(false);
    Ok(config)
}

/// Decode one typed result with caller-bounded skipping of other Candid values.
///
/// # Errors
/// Refuses oversized replies and preserves Candid shape/quota failures.
pub fn decode_one<'a, T: Deserialize<'a> + CandidType>(
    bytes: &'a [u8],
    limits: DecodeLimits,
) -> Result<T, DecodeError> {
    candid::decode_one_with_config(bytes, &config(bytes, limits)?).map_err(DecodeError::Candid)
}

/// Decode top-level reply arguments with caller-bounded decoding and skipping.
///
/// `(T, U)` requests two arguments, while `(T,)` requests one. As with
/// [`decode_one`], additional values may be skipped within the selected quota.
///
/// # Errors
/// Refuses wire/work ceilings, malformed arguments and trailing bytes.
pub fn decode_args<'a, T: candid::utils::ArgumentDecoder<'a>>(
    bytes: &'a [u8],
    limits: DecodeLimits,
) -> Result<T, DecodeError> {
    candid::decode_args_with_config(bytes, &config(bytes, limits)?).map_err(DecodeError::Candid)
}

/// Decode exactly one typed Candid result, rejecting missing or additional values or bytes.
///
/// # Errors
/// Refuses wire/work ceilings, wrong result shape/arity and trailing bytes.
pub fn decode_exactly_one<'a, T: Deserialize<'a> + CandidType>(
    bytes: &'a [u8],
    limits: DecodeLimits,
) -> Result<T, DecodeError> {
    let config = config(bytes, limits)?;
    let mut decoder =
        IDLDeserialize::new_with_config(bytes, &config).map_err(DecodeError::Candid)?;
    if decoder.is_done() {
        return Err(DecodeError::Candid(candid::Error::msg(
            "expected exactly one Candid result",
        )));
    }
    let value = decoder.get_value::<T>().map_err(DecodeError::Candid)?;
    if !decoder.is_done() {
        return Err(DecodeError::Candid(candid::Error::msg(
            "expected exactly one Candid result",
        )));
    }
    decoder.done().map_err(DecodeError::Candid)?;
    Ok(value)
}
