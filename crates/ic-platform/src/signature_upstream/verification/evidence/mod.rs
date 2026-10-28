//! Evidence extraction using the unchanged imported verification helpers.

#[cfg(test)]
mod tests;

use super::{
    CanisterSigPublicKey, HashTreeNode, LookupResult, check_certified_data_and_get_certificate,
    check_sig_path, parse_signature_cbor, verify_certificate,
};

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
/// Calls the same upstream parsing, certified-data, seed/message, delegation and
/// BLS helpers as `verify_canister_sig`, in the same order. The signature envelope
/// and signing certificate are each decoded once. The delegation certificate's
/// time is never returned. No time is returned if any verification step fails.
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
    let verify = || -> Result<_, String> {
        let signature = parse_signature_cbor(signature_cbor)?;
        let public_key = CanisterSigPublicKey::try_from(public_key_der)
            .map_err(|e| format!("failed to parse canister sig public key: {e}"))?;
        let certificate =
            check_certified_data_and_get_certificate(&signature, &public_key.canister_id)?;
        check_sig_path(&signature, &public_key, message)?;
        verify_certificate(&certificate, public_key.canister_id, ic_root_public_key_raw)?;
        Ok(certificate)
    };
    let certificate = verify().map_err(CanisterSigTimeError::Verification)?;
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
