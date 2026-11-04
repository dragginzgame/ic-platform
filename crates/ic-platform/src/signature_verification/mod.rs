//! Upstream canister-signature verification with checked admission and evidence.
//!
//! Cryptographic verification and key formats delegate upstream. Platform keeps
//! DER framing, identity-key refusal and authenticated signing-time extraction.
//! Callers retain trust roots, byte/depth limits, freshness and authority policy.

#[cfg(test)]
mod tests;

use crate::bls::PublicKey;
use crate::canister_sig_creation::{admission, extract_raw_root_pk_from_der};
use ic_certification::{Certificate, HashTreeNode, LookupResult};
use serde::Deserialize;

// Project only the certificate bytes. The full envelope/tree was already
// authenticated by upstream; no hash-tree verification is implemented here.
#[derive(Deserialize)]
struct SigningCertificate {
    #[serde(with = "serde_bytes")]
    certificate: Vec<u8>,
}

fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    serde_cbor::from_slice(bytes).map_err(|error| error.to_string())
}

fn admit_key(bytes: &[u8]) -> Result<(), String> {
    match PublicKey::deserialize(bytes) {
        Ok(key) if !key.is_identity() => Ok(()),
        _ => Err("invalid BLS signature".into()),
    }
}

fn verified_certificate(
    message: &[u8],
    signature_cbor: &[u8],
    public_key_der: &[u8],
    root: &[u8],
) -> Result<Certificate, String> {
    admission::canister_sig_der(public_key_der)
        .map_err(|error| format!("failed to parse canister sig public key: {error}"))?;
    ic_signature_verification::verify_canister_sig(message, signature_cbor, public_key_der, root)?;
    // Registry BLS 0.6.0 still accepts identity keys. Reject both trust-root and
    // delegated keys; all point parsing/identity operations remain upstream.
    admit_key(root)?;
    let envelope: SigningCertificate = decode(signature_cbor)?;
    let certificate: Certificate = decode(&envelope.certificate)?;
    if let Some(delegation) = &certificate.delegation {
        let delegated: Certificate = decode(&delegation.certificate)?;
        let path = [
            b"subnet".as_slice(),
            delegation.subnet_id.as_slice(),
            b"public_key".as_slice(),
        ];
        let LookupResult::Found(der) = delegated.tree.lookup_path(path) else {
            return Err("subnet public key not found".into());
        };
        admit_key(&extract_raw_root_pk_from_der(der)?)?;
    }
    Ok(certificate)
}

/// Verify a canister signature through upstream, retaining DER and key admission.
///
/// # Errors
/// Returns upstream verification diagnostics or a checked admission refusal.
pub fn verify_canister_sig(
    message: &[u8],
    signature_cbor: &[u8],
    public_key_der: &[u8],
    ic_root_public_key_raw: &[u8],
) -> Result<(), String> {
    verified_certificate(
        message,
        signature_cbor,
        public_key_der,
        ic_root_public_key_raw,
    )
    .map(|_| ())
}

/// Failure to authenticate a canister signature or extract its signing time.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CanisterSigTimeError {
    /// An unchanged upstream verifier rejected the signature; diagnostics are opaque.
    Verification(String),
    /// Time is absent, ambiguous, or not a complete unsigned 64-bit LEB128 value.
    InvalidCertificateTime,
}

impl std::fmt::Display for CanisterSigTimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Verification(message) => f.write_str(message),
            Self::InvalidCertificateTime => f.write_str("invalid signing certificate time"),
        }
    }
}
impl std::error::Error for CanisterSigTimeError {}

/// Verify a canister signature and return its authenticated signing time in ns.
///
/// Calls the upstream verifier, then decodes its authenticated signing certificate.
/// This requires a second CBOR decode because upstream exposes no certificate
/// evidence. The delegation certificate's time is never returned. Verification
/// failure always prevents time extraction.
///
/// Time must be a unique visible top-level leaf and a complete u64 LEB128 value
/// of at most ten bytes. Non-minimal encodings within that bound remain accepted,
/// matching Auth's existing reader. Absent, unknown, non-leaf, duplicate visible
/// time labels, overflow, truncation and trailing bytes are rejected.
///
/// Callers retain byte/depth admission, DER pre-validation, trusted inputs and
/// all freshness/future-skew policy. Owned DER framing admission precedes the
/// unchanged imported public-key parser; this is not a bounded proof decoder.
/// Verification authenticates the message and time, not application authority.
pub fn verify_canister_sig_with_time(
    message: &[u8],
    signature_cbor: &[u8],
    public_key_der: &[u8],
    ic_root_public_key_raw: &[u8],
) -> Result<u64, CanisterSigTimeError> {
    let certificate = verified_certificate(
        message,
        signature_cbor,
        public_key_der,
        ic_root_public_key_raw,
    )
    .map_err(CanisterSigTimeError::Verification)?;
    let LookupResult::Found(bytes) = certificate.tree.lookup_path([b"time".as_slice()]) else {
        return Err(CanisterSigTimeError::InvalidCertificateTime);
    };
    // Upstream lookup assumes sorted labels. Refuse visible duplicates instead
    // of choosing one timestamp from a malformed, but authenticated, tree.
    if visible_time_labels(certificate.tree.as_ref()) != 1 {
        return Err(CanisterSigTimeError::InvalidCertificateTime);
    }
    read_time(bytes)
}

fn visible_time_labels(node: &HashTreeNode) -> usize {
    match node {
        HashTreeNode::Fork(children) => {
            // Only uniqueness matters; cap counting, not traversal or admission.
            (visible_time_labels(&children.0) + visible_time_labels(&children.1)).min(2)
        }
        HashTreeNode::Labeled(label, _) => usize::from(label.as_bytes() == b"time"),
        _ => 0,
    }
}

fn read_time(bytes: &[u8]) -> Result<u64, CanisterSigTimeError> {
    if bytes.len() > 10 {
        return Err(CanisterSigTimeError::InvalidCertificateTime);
    }
    let mut value = 0_u64;
    for (index, &byte) in bytes.iter().enumerate() {
        if index == 9 && byte > 1 {
            break;
        }
        value |= u64::from(byte & 0x7f) << (index * 7);
        if byte & 0x80 == 0 {
            return if index + 1 == bytes.len() {
                Ok(value)
            } else {
                Err(CanisterSigTimeError::InvalidCertificateTime)
            };
        }
    }
    Err(CanisterSigTimeError::InvalidCertificateTime)
}
