//! Byte-bounded CBOR for standard IC certificates and canister signatures.
//!
//! Upstream Serde implementations retain certificate/tree grammar and recursion
//! admission. Input limits are checked before decoding; output limits are enforced
//! while serializing, before growing the buffer. These are byte limits, not a
//! bound on total decoder allocations. Decoding does not authenticate anything.
//! Callers own all budgets, witness construction, trust, freshness and storage.

use super::{Certificate, HashTree};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::io::{self, Write};
use thiserror::Error;

pub mod strict;

/// Byte-budget or upstream CBOR failure. No partial output is returned.
#[derive(Debug, Error)]
pub enum CborError {
    /// Encoded input exceeds the caller's byte limit; decoding has not started.
    #[error("CBOR input exceeds the byte limit")]
    InputTooLarge,
    /// Upstream CBOR/Serde admission failed, including trailing bytes.
    #[error("invalid CBOR: {0}")]
    Decode(#[source] serde_cbor::Error),
    /// The next encoded write would exceed the caller's output byte limit.
    #[error("CBOR output exceeds the byte limit")]
    OutputTooLarge,
    /// Upstream serialization failed for a reason other than output exhaustion.
    #[error("CBOR encoding failed: {0}")]
    Encode(#[source] serde_cbor::Error),
}

/// Standard IC canister-signature envelope, without any verification assertion.
#[derive(Debug, Deserialize)]
pub struct CanisterSignature {
    /// Original certificate bytes, preserved without re-encoding.
    #[serde(with = "serde_bytes")]
    pub certificate: Vec<u8>,
    /// Caller-constructed certified witness with upstream type identity.
    pub tree: HashTree,
}

/// Decode a certificate only after checking its complete encoded byte length.
/// Does not recursively decode delegation bytes, authenticate or check freshness.
pub fn decode_certificate(bytes: &[u8], max_bytes: usize) -> Result<Certificate, CborError> {
    decode(bytes, max_bytes)
}

/// Decode a canister-signature envelope after checking its encoded byte length.
/// Certificate and delegation decoding require their own explicit caller budgets.
/// Upstream Serde accepts named maps, packed maps and sequence representations.
pub fn decode_canister_signature(
    bytes: &[u8],
    max_bytes: usize,
) -> Result<CanisterSignature, CborError> {
    decode(bytes, max_bytes)
}

/// Encode a standard certificate with a self-describing CBOR tag, within the
/// caller's output byte budget. No partial bytes escape on failure.
pub fn encode_certificate(
    certificate: &Certificate,
    max_bytes: usize,
) -> Result<Vec<u8>, CborError> {
    encode(certificate, max_bytes)
}

/// Encode a standard self-describing IC canister-signature envelope, preserving
/// certificate bytes and witness order. Does not decode or authenticate them.
/// The output buffer refuses any write exceeding the caller's byte budget.
pub fn encode_canister_signature(
    certificate: &[u8],
    tree: &HashTree,
    max_bytes: usize,
) -> Result<Vec<u8>, CborError> {
    #[derive(Serialize)]
    struct Envelope<'a> {
        #[serde(with = "serde_bytes")]
        certificate: &'a [u8],
        tree: &'a HashTree,
    }
    encode(&Envelope { certificate, tree }, max_bytes)
}

pub(crate) fn decode<T: DeserializeOwned>(bytes: &[u8], max_bytes: usize) -> Result<T, CborError> {
    if bytes.len() > max_bytes {
        return Err(CborError::InputTooLarge);
    }
    serde_cbor::from_slice(bytes).map_err(CborError::Decode)
}

fn encode(value: &impl Serialize, max_bytes: usize) -> Result<Vec<u8>, CborError> {
    encode_framed(value, max_bytes, true)
}

pub(crate) fn encode_framed(
    value: &impl Serialize,
    max_bytes: usize,
    tagged: bool,
) -> Result<Vec<u8>, CborError> {
    let mut output = Output {
        bytes: Vec::new(),
        maximum: max_bytes,
    };
    let mut serializer = serde_cbor::Serializer::new(serde_cbor::ser::IoWrite::new(&mut output));
    (if tagged {
        serializer.self_describe()
    } else {
        Ok(())
    })
    .and_then(|()| value.serialize(&mut serializer))
    .map_err(|error| {
        // This private sink's only I/O failure is budget exhaustion.
        if error.is_io() {
            CborError::OutputTooLarge
        } else {
            CborError::Encode(error)
        }
    })?;
    Ok(output.bytes)
}

/// Encode the untagged standard certificate representation, preserving its bytes.
pub fn encode_certificate_untagged(
    certificate: &Certificate,
    max_bytes: usize,
) -> Result<Vec<u8>, CborError> {
    encode_framed(certificate, max_bytes, false)
}

/// Encode a standard hash tree without a self-described tag.
pub fn encode_hash_tree(tree: &HashTree, max_bytes: usize) -> Result<Vec<u8>, CborError> {
    encode_framed(tree, max_bytes, false)
}

// Geometric buffer growth stays within the selected byte cap; allocator/tree
// overhead is outside this byte budget. Failed writes leave the buffer intact.
struct Output {
    bytes: Vec<u8>,
    maximum: usize,
}
impl Write for Output {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.maximum - self.bytes.len() {
            return Err(io::ErrorKind::WriteZero.into());
        }
        let required = self.bytes.len() + bytes.len();
        if required > self.bytes.capacity() {
            let capacity = required
                .max(self.bytes.capacity().saturating_mul(2))
                .min(self.maximum);
            self.bytes.reserve_exact(capacity - self.bytes.len());
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests;
