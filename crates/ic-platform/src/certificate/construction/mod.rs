//! Explicit untrusted certificate construction for issuers and independent fixtures.
//! Construction never supplies authenticated evidence or selects a trust root.
#[cfg(test)]
mod tests;

use serde::Serialize;
use std::io::{self, Write};

pub use ic_certification::{
    Certificate, Delegation, HashTree, LookupResult, empty, fork, labeled, leaf, pruned,
};

/// Construction refusal with no partial output.
#[derive(Debug, thiserror::Error)]
pub enum ConstructionError {
    /// Complete encoded output exceeds the caller's allowance.
    #[error("encoded certificate exceeds output allowance")]
    OutputLimit,
    /// Upstream serialization failed.
    #[error("certificate serialization failed: {0}")]
    Encoding(#[source] serde_cbor::Error),
    /// Canister key cannot fit the supported short-form DER envelope.
    #[error("canister key exceeds the short-form DER boundary")]
    InvalidKey,
}

struct Bounded {
    bytes: Vec<u8>,
    maximum: usize,
    refused: bool,
}
impl Write for Bounded {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.maximum.saturating_sub(self.bytes.len()) {
            self.refused = true;
            return Err(io::Error::other("output limit"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn encode(value: &impl Serialize, maximum: usize) -> Result<Vec<u8>, ConstructionError> {
    let mut output = Bounded {
        bytes: Vec::new(),
        maximum,
        refused: false,
    };
    let result = {
        let mut serializer =
            serde_cbor::Serializer::new(serde_cbor::ser::IoWrite::new(&mut output));
        serializer
            .self_describe()
            .and_then(|()| value.serialize(&mut serializer))
    };
    if output.refused {
        return Err(ConstructionError::OutputLimit);
    }
    result.map_err(ConstructionError::Encoding)?;
    Ok(output.bytes)
}

/// Encode a tagged certificate without authenticating or normalizing its contents.
///
/// # Errors
/// Refuses output beyond `maximum` or an upstream serialization failure.
pub fn encode_certificate(
    value: &Certificate,
    maximum: usize,
) -> Result<Vec<u8>, ConstructionError> {
    encode(value, maximum)
}

#[derive(Serialize)]
struct Signature<'a> {
    #[serde(with = "serde_bytes")]
    certificate: &'a [u8],
    tree: &'a HashTree,
}

/// Encode an opaque certificate and witness as a tagged canister signature.
///
/// # Errors
/// Refuses output beyond `maximum` or an upstream serialization failure.
pub fn encode_canister_signature(
    certificate: &[u8],
    tree: &HashTree,
    maximum: usize,
) -> Result<Vec<u8>, ConstructionError> {
    encode(&Signature { certificate, tree }, maximum)
}

/// Encode a canister key using canonical short-form IC SubjectPublicKeyInfo framing.
///
/// # Errors
/// Refuses a seed/issuer combination outside the verifier's 20–129 byte boundary.
pub fn canister_key_der(
    issuer: candidate_types::Principal,
    seed: &[u8],
) -> Result<Vec<u8>, ConstructionError> {
    let payload = 1usize
        .checked_add(issuer.as_slice().len())
        .and_then(|n| n.checked_add(seed.len()))
        .ok_or(ConstructionError::InvalidKey)?;
    if payload > 110 {
        return Err(ConstructionError::InvalidKey);
    }
    let mut bytes = vec![
        0x30,
        u8::try_from(payload + 17).map_err(|_| ConstructionError::InvalidKey)?,
        0x30,
        0x0c,
        0x06,
        0x0a,
        0x2b,
        0x06,
        0x01,
        0x04,
        0x01,
        0x83,
        0xb8,
        0x43,
        0x01,
        0x02,
        0x03,
        u8::try_from(payload + 1).map_err(|_| ConstructionError::InvalidKey)?,
        0,
    ];
    bytes.push(u8::try_from(issuer.as_slice().len()).map_err(|_| ConstructionError::InvalidKey)?);
    bytes.extend_from_slice(issuer.as_slice());
    bytes.extend_from_slice(seed);
    Ok(bytes)
}
