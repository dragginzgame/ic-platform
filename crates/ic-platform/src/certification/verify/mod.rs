//! Native authentication of bounded certificate/witness evidence.
//! Callers select the Agent trust root, combined byte allowance and application leaf policy.
use crate::{
    Principal,
    agent::{Agent, AgentError, EffectiveId},
    certification::{
        Certificate, HashTree, LookupResult,
        cbor::strict::{DecodeError, DecodeLimits, decode_certificate, decode_hash_tree},
    },
};

/// Failure to authenticate and bind an exact certificate/witness pair.
#[derive(Debug, thiserror::Error)]
pub enum EvidenceError {
    /// Combined input size exceeded the explicit allowance.
    #[error("certificate evidence exceeds byte allowance")]
    Budget,
    /// A strict certificate/witness decoder rejected the bytes.
    #[error("invalid certificate evidence: {0}")]
    Decode(#[source] DecodeError),
    /// Agent authentication refused the certificate under its configured policy.
    #[error("certificate authentication failed: {0}")]
    Authentication(#[source] Box<AgentError>),
    /// The witness digest is not the exact canister's certified-data commitment.
    #[error("witness is not committed by the certificate")]
    WitnessMismatch,
}
/// Authenticate a certificate and bind its witness to the exact canister.
/// Does not choose a root key, retry, decode application leaves or retain evidence.
/// # Errors
/// Refuses combined-byte limits, malformed/trailing bytes, authentication and witness mismatch.
pub fn authenticate_hash_tree(
    agent: &Agent,
    canister: Principal,
    certificate: &[u8],
    witness: &[u8],
    maximum_bytes: usize,
) -> Result<HashTree, EvidenceError> {
    if certificate
        .len()
        .checked_add(witness.len())
        .is_none_or(|size| size > maximum_bytes)
    {
        return Err(EvidenceError::Budget);
    }
    let certificate = authenticate_certificate(agent, canister, certificate, maximum_bytes)?;
    let witness = decode_hash_tree(
        witness,
        DecodeLimits {
            bytes: maximum_bytes,
        },
    )
    .map_err(EvidenceError::Decode)?;
    verify_witness(&certificate, canister, &witness)?;
    Ok(witness)
}
/// Compare a witness digest to the exact canister's certified-data leaf.
/// This check alone does not authenticate a certificate.
/// # Errors
/// Refuses missing, unknown, non-leaf or unequal commitments.
pub fn verify_witness(
    certificate: &Certificate,
    canister: Principal,
    witness: &HashTree,
) -> Result<(), EvidenceError> {
    match certificate.tree.lookup_path([
        b"canister".as_slice(),
        canister.as_slice(),
        b"certified_data".as_slice(),
    ]) {
        LookupResult::Found(value) if value == witness.digest() => Ok(()),
        _ => Err(EvidenceError::WitnessMismatch),
    }
}

/// Authenticate strict certificate bytes under the caller’s configured Agent policy.
/// # Errors
/// Refuses byte limits, malformed/trailing CBOR and Agent authentication failures.
pub fn authenticate_certificate(
    agent: &Agent,
    canister: Principal,
    bytes: &[u8],
    maximum_bytes: usize,
) -> Result<Certificate, EvidenceError> {
    let certificate = decode_certificate(
        bytes,
        DecodeLimits {
            bytes: maximum_bytes,
        },
    )
    .map_err(EvidenceError::Decode)?;
    let effective = EffectiveId::Canister(
        canister
            .as_slice()
            .try_into()
            .expect("canonical principal fits Agent"),
    );
    agent
        .verify(&certificate, effective)
        .map_err(|source| EvidenceError::Authentication(Box::new(source)))?;
    Ok(certificate)
}

#[cfg(test)]
mod tests;
