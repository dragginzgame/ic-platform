//! Bounded Principal identities, canonical text, and Serde representations.
//!
//! Bytes remain opaque; callers own identity policy and authority decisions.

use data_encoding::BASE32_NOPAD;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::{fmt, str::FromStr};

#[cfg(feature = "principal-self-authenticating")]
mod self_authenticating;

/// An opaque IC identity containing at most 29 significant bytes.
///
/// Ordering compares length first, then bytes. Binary Serde includes only the
/// significant bytes; human-readable Serde uses canonical Principal text.
/// Deserialization accepts string or byte callbacks, but not numeric sequences.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Principal {
    len: u8,
    bytes: [u8; 29],
}

/// A failed Principal byte or text admission.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
pub enum PrincipalError {
    /// A binary identity exceeds 29 bytes.
    #[error("Bytes is longer than 29 bytes.")]
    BytesTooLong(),
    /// Text does not contain a valid unpadded Base32 payload.
    #[error("Text must be in valid Base32 encoding.")]
    InvalidBase32(),
    /// Decoded text is shorter than its four-byte checksum.
    #[error("Text is too short.")]
    TextTooShort(),
    /// Text exceeds its admission bound or encodes too many payload bytes.
    #[error("Text is too long.")]
    TextTooLong(),
    /// The supplied checksum differs from the checksum of the identity bytes.
    #[error("CRC32 check sequence doesn't match with calculated from Principal bytes.")]
    CheckSequenceNotMatch(),
    /// The identity is valid but its text has noncanonical dash positions.
    #[error("Text should be separated by - (dash) every 5 characters: expected \"{0}\"")]
    AbnormalGrouped(Principal),
}

impl Principal {
    /// The maximum significant binary length.
    pub const MAX_LENGTH_IN_BYTES: usize = 29;

    /// The maximum admitted UTF-8 byte length of Principal text.
    pub const MAX_LENGTH_IN_TEXT: usize = 63;

    /// Returns the zero-byte management canister identity.
    pub const fn management_canister() -> Self {
        Self::from_slice(&[])
    }

    /// Returns the one-byte anonymous identity, whose byte is 4.
    pub const fn anonymous() -> Self {
        Self::from_slice(&[4])
    }

    /// Copies significant bytes into a Principal.
    ///
    /// # Panics
    ///
    /// Panics if `slice` contains more than 29 bytes.
    pub const fn from_slice(slice: &[u8]) -> Self {
        match Self::try_from_slice(slice) {
            Ok(principal) => principal,
            Err(_) => panic!("a Principal cannot contain more than 29 bytes"),
        }
    }

    /// Copies bytes, returning [`PrincipalError::BytesTooLong`] above 29 bytes.
    pub const fn try_from_slice(slice: &[u8]) -> Result<Self, PrincipalError> {
        if slice.len() > Self::MAX_LENGTH_IN_BYTES {
            return Err(PrincipalError::BytesTooLong());
        }
        let mut principal = Self {
            len: slice.len() as u8,
            bytes: [0; Self::MAX_LENGTH_IN_BYTES],
        };
        let mut index = 0;
        while index < slice.len() {
            principal.bytes[index] = slice[index];
            index += 1;
        }
        Ok(principal)
    }

    /// Parses bounded, checksummed Base32 text with canonical dash grouping.
    ///
    /// ASCII case differences are accepted. Inputs longer than 63 bytes are
    /// refused before decoding. Whitespace and padding are not accepted.
    pub fn from_text<S: AsRef<str>>(text: S) -> Result<Self, PrincipalError> {
        let text = text.as_ref();
        if text.len() > Self::MAX_LENGTH_IN_TEXT {
            return Err(PrincipalError::TextTooLong());
        }

        let mut symbols = [0_u8; Self::MAX_LENGTH_IN_TEXT];
        let mut symbol_count = 0;
        for byte in text.bytes() {
            if byte != b'-' {
                symbols[symbol_count] = byte.to_ascii_uppercase();
                symbol_count += 1;
            }
        }
        let decoded_len = BASE32_NOPAD
            .decode_len(symbol_count)
            .map_err(|_| PrincipalError::InvalidBase32())?;
        // At most 63 Base32 digits fit the text bound: floor(63 * 5 / 8) = 39.
        // Keeping the full decoded bound lets us distinguish an oversized
        // payload from invalid Base32, before examining its checksum.
        let mut decoded = [0_u8; 39];
        let used = BASE32_NOPAD
            .decode_mut(&symbols[..symbol_count], &mut decoded[..decoded_len])
            .map_err(|_| PrincipalError::InvalidBase32())?;
        if used < 4 {
            return Err(PrincipalError::TextTooShort());
        }
        let payload = &decoded[4..used];
        if payload.len() > Self::MAX_LENGTH_IN_BYTES {
            return Err(PrincipalError::TextTooLong());
        }
        if decoded[..4] != crc32fast::hash(payload).to_be_bytes() {
            return Err(PrincipalError::CheckSequenceNotMatch());
        }

        let principal = Self::from_slice(payload);
        let mut canonical = [0_u8; Self::MAX_LENGTH_IN_TEXT];
        if !principal
            .text_into(&mut canonical)
            .eq_ignore_ascii_case(text)
        {
            return Err(PrincipalError::AbnormalGrouped(principal));
        }
        Ok(principal)
    }

    /// Returns lowercase canonical text with a dash after every five digits.
    pub fn to_text(&self) -> String {
        let mut text = [0_u8; Self::MAX_LENGTH_IN_TEXT];
        self.text_into(&mut text).to_owned()
    }

    /// Borrows only the significant bytes.
    pub const fn as_slice(&self) -> &[u8] {
        self.bytes.split_at(self.len as usize).0
    }

    /// Returns the significant byte length; zero identifies the management canister.
    #[expect(
        clippy::len_without_is_empty,
        reason = "A zero-length Principal identifies the management canister"
    )]
    pub const fn len(&self) -> u8 {
        self.len
    }

    /// Borrows the fixed storage, whose tail after [`Self::len`] is zero.
    pub const fn as_fixed_bytes(&self) -> &[u8; 29] {
        &self.bytes
    }

    fn text_into<'a>(&self, output: &'a mut [u8; Self::MAX_LENGTH_IN_TEXT]) -> &'a str {
        let mut checked = [0_u8; 4 + Self::MAX_LENGTH_IN_BYTES];
        checked[..4].copy_from_slice(&crc32fast::hash(self.as_slice()).to_be_bytes());
        let checked_len = 4 + usize::from(self.len);
        checked[4..checked_len].copy_from_slice(self.as_slice());

        let encoded_len = BASE32_NOPAD.encode_len(checked_len);
        let mut encoded = [0_u8; 53];
        BASE32_NOPAD.encode_mut(&checked[..checked_len], &mut encoded[..encoded_len]);
        let mut written = 0;
        for (index, byte) in encoded[..encoded_len].iter().enumerate() {
            if index != 0 && index % 5 == 0 {
                output[written] = b'-';
                written += 1;
            }
            output[written] = byte.to_ascii_lowercase();
            written += 1;
        }
        // The supplier emits ASCII Base32 digits; inserted dashes are ASCII too.
        std::str::from_utf8(&output[..written]).expect("Base32 text is ASCII")
    }
}

impl fmt::Display for Principal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut text = [0_u8; Self::MAX_LENGTH_IN_TEXT];
        formatter.write_str(self.text_into(&mut text))
    }
}

impl FromStr for Principal {
    type Err = PrincipalError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::from_text(text)
    }
}

impl AsRef<[u8]> for Principal {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl TryFrom<&str> for Principal {
    type Error = PrincipalError;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        Self::from_text(text)
    }
}

impl TryFrom<Vec<u8>> for Principal {
    type Error = PrincipalError;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from_slice(&bytes)
    }
}

impl TryFrom<&Vec<u8>> for Principal {
    type Error = PrincipalError;

    fn try_from(bytes: &Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from_slice(bytes)
    }
}

impl TryFrom<&[u8]> for Principal {
    type Error = PrincipalError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::try_from_slice(bytes)
    }
}

impl Serialize for Principal {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            let mut text = [0_u8; Self::MAX_LENGTH_IN_TEXT];
            serializer.serialize_str(self.text_into(&mut text))
        } else {
            serializer.serialize_bytes(self.as_slice())
        }
    }
}

struct PrincipalVisitor;

impl de::Visitor<'_> for PrincipalVisitor {
    type Value = Principal;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("bytes or string")
    }

    fn visit_str<E: de::Error>(self, text: &str) -> Result<Principal, E> {
        Principal::from_text(text).map_err(E::custom)
    }
    fn visit_bytes<E: de::Error>(self, bytes: &[u8]) -> Result<Principal, E> {
        Principal::try_from_slice(bytes).map_err(E::custom)
    }
}

impl<'de> Deserialize<'de> for Principal {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            deserializer.deserialize_str(PrincipalVisitor)
        } else {
            deserializer.deserialize_bytes(PrincipalVisitor)
        }
    }
}

#[cfg(test)]
mod tests;
