//! Untrusted canister-signature envelopes; authentication is a separate obligation.

#[cfg(test)]
mod tests;

use super::CertificateDecodeLimits;

/// An envelope refusal; successful decoding does not authenticate a signature.
#[derive(Debug, thiserror::Error)]
pub enum CanisterSignatureDecodeError {
    /// Complete input exceeds the caller's bound, before parsing.
    #[error("canister signature has {actual} bytes, exceeding {maximum}")]
    InputLimit {
        /// Supplied encoded length.
        actual: usize,
        /// Inclusive caller-selected limit.
        maximum: usize,
    },
    /// The required CBOR self-describing tag is missing.
    #[error("canister signature requires CBOR tag 55799")]
    MissingTag,
    /// Invalid outer envelope or witness-tree grammar.
    #[error("invalid canister signature encoding: {0}")]
    InvalidEncoding(#[source] serde_cbor::Error),
}

/// Immutable unverified envelope tied to the exact original signature bytes.
///
/// The embedded certificate remains opaque. Neither its grammar, signatures,
/// authority, time nor the witness/message/key relationship is checked here.
///
/// ```compile_fail
/// use ic_platform::certificate::DecodedCanisterSignature;
/// fn mutate(value: &mut DecodedCanisterSignature<'_>) {
///     value.certificate_bytes().clear();
/// }
/// ```
#[derive(Debug)]
pub struct DecodedCanisterSignature<'a> {
    original: &'a [u8],
    certificate: Vec<u8>,
    witness: ic_certification::HashTree,
}
impl<'a> DecodedCanisterSignature<'a> {
    /// The original complete signature encoding, without re-encoding.
    #[must_use]
    pub const fn original_bytes(&self) -> &'a [u8] {
        self.original
    }

    /// Exact unverified embedded certificate payload, copied from its CBOR blob.
    #[must_use]
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }

    /// Unverified witness tree. A successful lookup is not authentication.
    #[must_use]
    pub const fn unverified_witness(&self) -> &ic_certification::HashTree {
        &self.witness
    }
}

/// Decode one tagged canister-signature envelope after whole-input admission.
///
/// Requires `certificate` as a CBOR byte string and `tree` as an IC hash tree.
/// Rejects duplicate/unknown fields, malformed tree nodes and trailing data.
/// The byte allowance is not a heap/work quota; the parser retains its recursion
/// limit. Embedded certificate grammar and witness ordering are not checked.
///
/// # Errors
/// Returns input-limit, missing-tag or encoding refusal without partial output.
pub fn decode_canister_signature(
    bytes: &[u8],
    limits: CertificateDecodeLimits,
) -> Result<DecodedCanisterSignature<'_>, CanisterSignatureDecodeError> {
    if bytes.len() > limits.bytes {
        return Err(CanisterSignatureDecodeError::InputLimit {
            actual: bytes.len(),
            maximum: limits.bytes,
        });
    }
    if !bytes.starts_with(&[0xd9, 0xd9, 0xf7]) {
        return Err(CanisterSignatureDecodeError::MissingTag);
    }
    let (certificate, witness) = super::wire::decode_canister_signature(bytes)
        .map_err(CanisterSignatureDecodeError::InvalidEncoding)?;
    Ok(DecodedCanisterSignature {
        original: bytes,
        certificate,
        witness,
    })
}
