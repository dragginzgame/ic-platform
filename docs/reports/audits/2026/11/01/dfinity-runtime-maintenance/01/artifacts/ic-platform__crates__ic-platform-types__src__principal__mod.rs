//! Platform-owned principal representation and non-cryptographic conversions.
//!
//! Adapted from ic_principal 0.1.5 under Apache-2.0; see NOTICE.md.
//! No Candid marker handling or generic range stepping is implemented.
//! `principal-external` adds registry Principal conversions; `principal-self-authenticating`
//! separately selects upstream public-key hashing. Required Candid DTOs use the
//! external Principal, converting at the boundary:
//!

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

/// An error happened while encoding, decoding or serializing a [`Principal`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, thiserror::Error)]
pub enum PrincipalError {
    /// More than 29 significant bytes.
    #[error("Bytes is longer than 29 bytes.")]
    BytesTooLong(),

    /// Invalid Base32 spelling or trailing bits.
    #[error("Text must be in valid Base32 encoding.")]
    InvalidBase32(),

    /// Missing complete checksum.
    #[error("Text is too short.")]
    TextTooShort(),

    /// Input exceeds 63 text bytes or decodes to more than 29 identity bytes.
    #[error("Text is too long.")]
    TextTooLong(),

    /// CRC32 checksum mismatch.
    #[error("CRC32 check sequence doesn't match with calculated from Principal bytes.")]
    CheckSequenceNotMatch(),

    /// Correct identity with noncanonical dash placement.
    #[error("Text should be separated by - (dash) every 5 characters: expected \"{0}\"")]
    AbnormalGrouped(Principal),
}

/// Generic ID on Internet Computer.
///
/// Principals are generic identifiers for canisters, users
/// and possibly other concepts in the future.
/// As far as most uses of the IC are concerned they are
/// opaque binary blobs with a length between 0 and 29 bytes,
/// and there is intentionally no mechanism to tell canister ids and user ids apart.
///
/// Note a principal is not necessarily tied with a public key-pair,
/// yet we need at least a key-pair of a related principal to sign
/// requests.
///
/// A Principal can be serialized to a byte array ([`Vec<u8>`]) or a text
/// representation, but the inner structure of the byte representation
/// is kept private.
///
/// Example of using a Principal object:
/// ```
/// # {
/// use ic_platform_types::Principal;
///
/// let text = "aaaaa-aa";  // The management canister ID.
/// let principal = Principal::from_text(text).expect("Could not decode the principal.");
/// assert!(principal.as_slice().is_empty());
/// assert_eq!(principal.to_text(), text);
///
/// # }
/// ```
///
/// Serialization is always available. It supports serializing
/// to a byte buffer for non-human readable serializer, and a string version for human
/// readable serializers.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Principal {
    /// Length.
    len: u8,

    /// The content buffer. When returning slices this should always be sized according to
    /// `len`.
    bytes: [u8; Self::MAX_LENGTH_IN_BYTES],
}

impl Principal {
    /// Maximum significant identity length.
    pub const MAX_LENGTH_IN_BYTES: usize = 29;
    /// Maximum canonical text length, including dashes and checksum.
    pub const MAX_LENGTH_IN_TEXT: usize = 63;
    #[allow(dead_code)]
    const CRC_LENGTH_IN_BYTES: usize = 4;

    const ANONYMOUS_TAG: u8 = 4;

    /// Construct a [`Principal`] of the IC management canister
    pub const fn management_canister() -> Self {
        Self {
            len: 0,
            bytes: [0; Self::MAX_LENGTH_IN_BYTES],
        }
    }

    /// Construct a self-authenticating ID from public key
    #[cfg(feature = "principal-self-authenticating")]
    pub fn self_authenticating<P: AsRef<[u8]>>(public_key: P) -> Self {
        ExternalPrincipal::self_authenticating(public_key.as_ref()).into()
    }

    /// Construct an anonymous ID.
    pub const fn anonymous() -> Self {
        let mut bytes = [0; Self::MAX_LENGTH_IN_BYTES];
        bytes[0] = Self::ANONYMOUS_TAG;
        Self { len: 1, bytes }
    }

    /// Returns `None` if the slice exceeds the max length.
    const fn from_slice_core(slice: &[u8]) -> Option<Self> {
        match slice.len() {
            len @ 0..=Self::MAX_LENGTH_IN_BYTES => {
                let mut bytes = [0; Self::MAX_LENGTH_IN_BYTES];
                let mut i = 0;
                while i < len {
                    bytes[i] = slice[i];
                    i += 1;
                }
                Some(Self {
                    len: len as u8,
                    bytes,
                })
            }
            _ => None,
        }
    }

    /// Construct a [`Principal`] from a slice of bytes.
    ///
    /// # Panics
    ///
    /// Panics if the slice is longer than 29 bytes.
    pub const fn from_slice(slice: &[u8]) -> Self {
        match Self::from_slice_core(slice) {
            Some(principal) => principal,
            _ => panic!("slice length exceeds capacity"),
        }
    }

    /// Construct a [`Principal`] from a slice of bytes.
    pub const fn try_from_slice(slice: &[u8]) -> Result<Self, PrincipalError> {
        match Self::from_slice_core(slice) {
            Some(principal) => Ok(principal),
            None => Err(PrincipalError::BytesTooLong()),
        }
    }

    /// Parse a [`Principal`] from text representation.
    pub fn from_text<S: AsRef<str>>(text: S) -> Result<Self, PrincipalError> {
        Self::parse_text(text.as_ref())
    }

    fn parse_text(input: &str) -> Result<Self, PrincipalError> {
        if input.len() > Self::MAX_LENGTH_IN_TEXT {
            return Err(PrincipalError::TextTooLong());
        }
        let mut encoded = [0u8; Self::MAX_LENGTH_IN_TEXT];
        let mut len = 0;
        for byte in input.bytes().filter(|&byte| byte != b'-') {
            encoded[len] = byte.to_ascii_uppercase();
            len += 1;
        }
        let encoding = data_encoding::BASE32_NOPAD;
        let size = encoding
            .decode_len(len)
            .map_err(|_| PrincipalError::InvalidBase32())?;
        let mut decoded = [0u8; 39]; // floor(63 * 5 / 8), before dash validation.
        let size = encoding
            .decode_mut(&encoded[..len], &mut decoded[..size])
            .map_err(|_| PrincipalError::InvalidBase32())?;
        if size < Self::CRC_LENGTH_IN_BYTES {
            return Err(PrincipalError::TextTooShort());
        }
        let payload = &decoded[Self::CRC_LENGTH_IN_BYTES..size];
        if payload.len() > Self::MAX_LENGTH_IN_BYTES {
            return Err(PrincipalError::TextTooLong());
        }
        if crc32fast::hash(payload).to_be_bytes() != decoded[..Self::CRC_LENGTH_IN_BYTES] {
            return Err(PrincipalError::CheckSequenceNotMatch());
        }
        let result = Self::from_slice(payload);
        let mut output = [0u8; Self::MAX_LENGTH_IN_TEXT];
        if !input.eq_ignore_ascii_case(result.write_text(&mut output)) {
            return Err(PrincipalError::AbnormalGrouped(result));
        }
        Ok(result)
    }

    /// Convert [`Principal`] to text representation.
    pub fn to_text(&self) -> String {
        self.write_text(&mut [0u8; Self::MAX_LENGTH_IN_TEXT])
            .to_owned()
    }

    /// Returns the [`Principal`]'s significant bytes as a slice: the first
    /// [`len`](Self::len) bytes of the backing array, without the trailing
    /// zero padding.
    ///
    /// Use [`as_fixed_bytes`](Self::as_fixed_bytes) to get the full,
    /// fixed-size backing array instead.
    #[inline]
    pub const fn as_slice(&self) -> &[u8] {
        self.bytes.split_at(self.len as usize).0
    }

    /// Returns the number of significant bytes in the [`Principal`], between
    /// `0` and [`MAX_LENGTH_IN_BYTES`](Self::MAX_LENGTH_IN_BYTES) inclusive.
    ///
    /// This equals the length of the slice returned by
    /// [`as_slice`](Self::as_slice).
    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub const fn len(&self) -> u8 {
        self.len
    }

    /// Returns a reference to the [`Principal`]'s fixed-size backing array.
    ///
    /// The array is always [`MAX_LENGTH_IN_BYTES`](Self::MAX_LENGTH_IN_BYTES)
    /// long; bytes from index [`len`](Self::len) onwards are always zero.
    /// Prefer [`as_slice`](Self::as_slice) for the significant bytes without
    /// padding.
    #[inline]
    pub const fn as_fixed_bytes(&self) -> &[u8; Self::MAX_LENGTH_IN_BYTES] {
        &self.bytes
    }
}

impl std::fmt::Display for Principal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.write_text(&mut [0u8; Self::MAX_LENGTH_IN_TEXT]))
    }
}

impl std::str::FromStr for Principal {
    type Err = PrincipalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Principal::from_text(s)
    }
}

impl TryFrom<&str> for Principal {
    type Error = PrincipalError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Principal::from_text(s)
    }
}

impl TryFrom<Vec<u8>> for Principal {
    type Error = PrincipalError;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(bytes.as_slice())
    }
}

impl TryFrom<&Vec<u8>> for Principal {
    type Error = PrincipalError;

    fn try_from(bytes: &Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(bytes.as_slice())
    }
}

impl TryFrom<&[u8]> for Principal {
    type Error = PrincipalError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::try_from_slice(bytes)
    }
}

impl AsRef<[u8]> for Principal {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

// Serialization
impl serde::Serialize for Principal {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            serializer.serialize_str(self.write_text(&mut [0; Self::MAX_LENGTH_IN_TEXT]))
        } else {
            serializer.serialize_bytes(self.as_slice())
        }
    }
}

// Deserialization
mod deserialize {
    use super::Principal;
    use std::convert::TryFrom;

    // Simple visitor for deserialization from bytes. We don't support other number types
    // as there's no need for it.
    pub(super) struct PrincipalVisitor;

    impl serde::de::Visitor<'_> for PrincipalVisitor {
        type Value = super::Principal;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("bytes or string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Principal::from_text(v).map_err(E::custom)
        }

        fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Principal::try_from(value).map_err(E::custom)
        }
        // Owned and borrowed binary bytes carry the same identity.
        fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Principal::try_from_slice(&v).map_err(E::custom)
        }
    }
}

impl<'de> serde::Deserialize<'de> for Principal {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Principal, D::Error> {
        if deserializer.is_human_readable() {
            deserializer.deserialize_str(deserialize::PrincipalVisitor)
        } else {
            deserializer.deserialize_bytes(deserialize::PrincipalVisitor)
        }
    }
}

/// Original registry identity required by external IC clients and DTOs.
/// Convert explicitly at the boundary; owned application payloads use [Principal].
#[cfg(feature = "principal-external")]
pub use ic_principal::Principal as ExternalPrincipal;
/// Original parse errors for the explicitly external principal identity.
#[cfg(feature = "principal-external")]
pub use ic_principal::PrincipalError as ExternalPrincipalError;

#[cfg(feature = "principal-external")]
impl From<ExternalPrincipal> for Principal {
    fn from(value: ExternalPrincipal) -> Self {
        Self::from_slice(value.as_slice())
    }
}
#[cfg(feature = "principal-external")]
impl From<Principal> for ExternalPrincipal {
    fn from(value: Principal) -> Self {
        Self::from_slice(value.as_slice())
    }
}
impl Principal {
    fn write_text<'a>(&self, output: &'a mut [u8; Self::MAX_LENGTH_IN_TEXT]) -> &'a str {
        let mut input = [0u8; 33];
        input[..4].copy_from_slice(&crc32fast::hash(self.as_slice()).to_be_bytes());
        input[4..4 + self.len as usize].copy_from_slice(self.as_slice());
        let encoding = data_encoding::BASE32_NOPAD;
        let mut encoded = [0u8; 53];
        let count = encoding.encode_len(4 + self.len as usize);
        encoding.encode_mut(&input[..4 + self.len as usize], &mut encoded[..count]);
        let mut written = 0;
        for (i, byte) in encoded[..count].iter().enumerate() {
            if i > 0 && i % 5 == 0 {
                output[written] = b'-';
                written += 1;
            }
            output[written] = byte.to_ascii_lowercase();
            written += 1;
        }
        std::str::from_utf8(&output[..written]).expect("Base32 is ASCII")
    }
}

#[cfg(test)]
mod tests;
