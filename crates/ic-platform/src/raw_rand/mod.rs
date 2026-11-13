//! The management canister's `raw_rand` wire protocol, without general Candid.
//!
//! Select `raw-rand`; select Runtime separately for raw calls. This codec neither
//! generates randomness nor authenticates it. Callers retain destination, wait,
//! retries, callback/cancellation handling and seed admission (including any
//! all-zero refusal). An all-zero result is protocol-valid and accepted here.
//!
//! # Accepted reply encoding
//!
//! The producer profile is `DIDL`, one type-table entry `vec nat8`, one result
//! referencing that entry (index zero), and exactly 32 payload bytes. This is
//! the replica's `Encode!(&buffer)` shape for its 32-byte random buffer. Integer
//! constructor/primitive opcodes use their one-byte encodings. The four count,
//! index and length fields admit minimal or zero-extended LEB128 encodings of
//! up to nine bytes each. No allocation or general type traversal occurs.
//!
//! This deliberately is **not the complete Candid grammar**: unused/duplicate
//! type entries, alternative type indices, subtype encodings and extra results
//! are refused, even if a general Candid decoder could interpret them. Such
//! encodings are unnecessary for the management producer. Use the explicitly
//! selected `candid` capability for general external Candid protocols.

use core::fmt;

/// Management method name; destination and transport remain caller-owned.
pub const METHOD: &str = "raw_rand";

/// Maximum reply: magic, two opcodes, four nine-byte integers and 32 bytes.
pub const MAX_REPLY_BYTES: usize = 74;

/// Exact empty Candid argument tuple. Encoding cannot fail or allocate.
#[must_use]
pub const fn encode() -> &'static [u8; 6] {
    crate::candid_args::empty()
}

/// A framing refusal, separate from Runtime's structured call failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecodeError {
    /// The complete reply exceeds [`MAX_REPLY_BYTES`].
    TooLarge,
    /// A required framing or payload byte is absent.
    Truncated,
    /// The Candid magic is absent or incorrect.
    InvalidMagic,
    /// The type table is not the single supported `vec nat8` entry.
    TypeTable,
    /// The reply does not declare exactly one argument.
    ResultArity,
    /// The result does not reference the supported blob type at index zero.
    ResultType,
    /// The declared blob length is not 32.
    BlobLength,
    /// Bytes remain after the one complete blob.
    TrailingBytes,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::TooLarge => "raw_rand reply exceeds 74 bytes",
            Self::Truncated => "truncated raw_rand reply",
            Self::InvalidMagic => "invalid raw_rand Candid magic",
            Self::TypeTable => "raw_rand requires one vec nat8 type-table entry",
            Self::ResultArity => "raw_rand requires exactly one result",
            Self::ResultType => "raw_rand result must reference type zero",
            Self::BlobLength => "raw_rand requires a 32-byte blob",
            Self::TrailingBytes => "trailing bytes after raw_rand result",
        })
    }
}

impl core::error::Error for DecodeError {}

/// Decode exactly one 32-byte result under the documented producer profile.
///
/// # Errors
/// Refuses oversized, truncated, malformed or unsupported framing, wrong blob
/// length, extra results and trailing bytes. Work is bounded by 74 input bytes.
pub fn decode(bytes: &[u8]) -> Result<[u8; 32], DecodeError> {
    if bytes.len() > MAX_REPLY_BYTES {
        return Err(DecodeError::TooLarge);
    }
    let (magic, mut rest) = bytes.split_at_checked(4).ok_or(DecodeError::Truncated)?;
    if magic != b"DIDL" {
        return Err(DecodeError::InvalidMagic);
    }
    expect_leb(&mut rest, 1, DecodeError::TypeTable)?;
    let (opcodes, after_types) = rest.split_at_checked(2).ok_or(DecodeError::Truncated)?;
    if opcodes != b"\x6d\x7b" {
        return Err(DecodeError::TypeTable);
    }
    rest = after_types;
    expect_leb(&mut rest, 1, DecodeError::ResultArity)?;
    expect_leb(&mut rest, 0, DecodeError::ResultType)?;
    expect_leb(&mut rest, 32, DecodeError::BlobLength)?;
    let (random, trailing) = rest.split_at_checked(32).ok_or(DecodeError::Truncated)?;
    if !trailing.is_empty() {
        return Err(DecodeError::TrailingBytes);
    }
    random.try_into().map_err(|_| DecodeError::Truncated)
}

// Match only this protocol's small nonnegative framing constants. The signed
// type index is zero, whose LEB128 encoding also requires zero extension.
fn expect_leb(bytes: &mut &[u8], mut expected: u8, error: DecodeError) -> Result<(), DecodeError> {
    for _ in 0..9 {
        let (&byte, rest) = bytes.split_first().ok_or(DecodeError::Truncated)?;
        *bytes = rest;
        if byte & 0x7f != expected {
            return Err(error);
        }
        expected >>= 7;
        if byte & 0x80 == 0 {
            return Ok(());
        }
    }
    Err(error)
}

#[cfg(test)]
mod tests;
