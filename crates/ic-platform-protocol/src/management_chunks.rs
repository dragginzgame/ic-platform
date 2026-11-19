//! Bounded management chunk-cache messages. Transport, hash comparison, cache
//! authority, expiry and retries remain caller-owned. Decode authenticates nothing.

#[cfg(feature = "management")]
use crate::candid::encode::{Encode, Writer};
use crate::candid::{
    self, Decode, Error, Limits, Reader, Schema,
    projection::{absent, projection},
};
use candidate_types::Principal;
use serde::{Deserialize, Serialize};

projection! {
    ///
    /// ChunkHash
    /// Exact returned hash bytes, without assuming their length or algorithm.
    ///

    both ("management") ChunkHash {
        /// Caller-owned comparison against its expected chunk digest.
        1_158_164_430 => hash: Vec<u8> [required],
    }
}

/// Complete output admission failure; no partial request escapes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncodeError {
    /// Complete frame exceeded the inclusive caller allowance or usize capacity.
    OutputLimit,
    /// Allocation of admitted output failed.
    Allocation,
}
impl std::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "management chunk encoding refusal: {self:?}")
    }
}
impl std::error::Error for EncodeError {}

/// Decode exactly one vector of hash records with shared framing, work,
/// allocation, vector and skipped-field budgets.
///
/// # Errors
/// Refuses incompatible, malformed, truncated, trailing or over-budget data.
pub fn decode_stored_chunks(bytes: &[u8], limits: Limits) -> Result<Vec<ChunkHash>, Error> {
    candid::decode(bytes, limits)
}

/// Decode exactly one uploaded chunk's hash record under explicit budgets.
///
/// # Errors
/// Refuses incompatible, malformed, truncated, trailing or over-budget data.
pub fn decode_upload_chunk(bytes: &[u8], limits: Limits) -> Result<ChunkHash, Error> {
    candid::decode(bytes, limits)
}

/// Encode `stored_chunks`' required canister_id record using canonical Types.
///
/// # Errors
/// Refuses a complete output beyond `maximum_bytes` before allocation.
pub fn encode_stored_chunks(
    canister: Principal,
    maximum_bytes: usize,
) -> Result<Vec<u8>, EncodeError> {
    encode(canister, None, maximum_bytes)
}

/// Encode `upload_chunk` with verbatim chunk bytes and required canister_id.
///
/// # Errors
/// Refuses a complete output beyond `maximum_bytes` before allocation.
pub fn encode_upload_chunk(
    canister: Principal,
    chunk: &[u8],
    maximum_bytes: usize,
) -> Result<Vec<u8>, EncodeError> {
    encode(canister, Some(chunk), maximum_bytes)
}

fn encode(
    canister: Principal,
    chunk: Option<&[u8]>,
    maximum: usize,
) -> Result<Vec<u8>, EncodeError> {
    // Independently authored fixed schemas; fields sorted by Candid label hash.
    const STORED: &[u8] = b"DIDL\x01\x6c\x01\xb3\xc4\xb1\xf2\x04\x68\x01\x00";
    const UPLOAD: &[u8] =
        b"DIDL\x02\x6d\x7b\x6c\x02\xed\x8c\x8b\xae\x04\x00\xb3\xc4\xb1\xf2\x04\x68\x01\x01";
    let header = if chunk.is_some() { UPLOAD } else { STORED };
    let mut length_bytes = [0u8; 10];
    let mut length_count = 0;
    if let Some(chunk) = chunk {
        let mut length = chunk.len();
        loop {
            let byte = (length & 127) as u8;
            length >>= 7;
            length_bytes[length_count] = if length == 0 { byte } else { byte | 128 };
            length_count += 1;
            if length == 0 {
                break;
            }
        }
    }
    let size = header
        .len()
        .checked_add(2 + canister.as_slice().len())
        .and_then(|n| n.checked_add(length_count))
        .and_then(|n| n.checked_add(chunk.map_or(0, <[u8]>::len)))
        .filter(|n| *n <= maximum)
        .ok_or(EncodeError::OutputLimit)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(size)
        .map_err(|_| EncodeError::Allocation)?;
    bytes.extend_from_slice(header);
    bytes.extend_from_slice(&length_bytes[..length_count]);
    if let Some(chunk) = chunk {
        bytes.extend_from_slice(chunk);
    }
    bytes.extend_from_slice(&[1, canister.as_slice().len() as u8]);
    bytes.extend_from_slice(canister.as_slice());
    debug_assert_eq!(bytes.len(), size);
    Ok(bytes)
}
