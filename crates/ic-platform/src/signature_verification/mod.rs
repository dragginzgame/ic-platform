//! Authentication of IC canister signatures and their signing-certificate time.
//!
//! Certified-tree hashing and curve primitives remain external implementations;
//! the BLS boundary owns checked admission and verification composition. Callers
//! supply their trust root and own freshness, input budgets and authorization.

use crate::canister_sig_creation::{
    CanisterSigPublicKey, extract_raw_root_pk_from_der, hash_bytes,
};
use crate::principal::Principal;
use ic_certification::{
    Certificate, Delegation, HashTree, HashTreeNode, LookupResult, SubtreeLookupResult,
};
use serde::{Deserialize, de::DeserializeOwned};

const SELF_DESCRIBED_CBOR: &[u8] = b"\xd9\xd9\xf7";

/// Failure to authenticate a signature or extract its authenticated time.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum CanisterSigTimeError {
    /// Signature or certificate authentication failed; its diagnostic is opaque.
    #[error("{0}")]
    Verification(String),
    /// The authenticated signing certificate has no single complete u64 time.
    #[error("invalid signing certificate time")]
    InvalidCertificateTime,
}

#[derive(Deserialize)]
struct SignatureEnvelope {
    #[serde(with = "serde_bytes")]
    certificate: Vec<u8>,
    tree: HashTree,
}

// These wire projections use Serde's duplicate-field checks while preserving
// named, packed-field and sequence forms of the external certificate protocol.
#[derive(Deserialize)]
struct CertificateEnvelope {
    tree: HashTree,
    #[serde(with = "serde_bytes")]
    signature: Vec<u8>,
    delegation: Option<DelegationEnvelope>,
}

#[derive(Deserialize)]
struct DelegationEnvelope {
    #[serde(with = "serde_bytes")]
    subnet_id: Vec<u8>,
    #[serde(with = "serde_bytes")]
    certificate: Vec<u8>,
}

/// Verifies the witness, issuing-canister binding and certificate trust chain.
///
/// Does not enforce freshness or select a network trust root.
pub fn verify_canister_sig(
    message: &[u8],
    signature_cbor: &[u8],
    public_key_der: &[u8],
    ic_root_public_key_raw: &[u8],
) -> Result<(), String> {
    authenticate(
        message,
        signature_cbor,
        public_key_der,
        RootKey::Raw(ic_root_public_key_raw),
    )
    .map(|_| ())
}

/// Verifies a canister signature and returns its authenticated signing time.
///
/// The time is the signing certificate's unsigned LEB128 value, in nanoseconds
/// since the Unix epoch. Freshness and acceptance windows belong to the caller.
pub fn verify_canister_sig_with_time(
    message: &[u8],
    signature_cbor: &[u8],
    public_key_der: &[u8],
    ic_root_public_key_raw: &[u8],
) -> Result<u64, CanisterSigTimeError> {
    let certificate = authenticate(
        message,
        signature_cbor,
        public_key_der,
        RootKey::Raw(ic_root_public_key_raw),
    )
    .map_err(CanisterSigTimeError::Verification)?;
    signing_time(&certificate.tree).ok_or(CanisterSigTimeError::InvalidCertificateTime)
}

/// Verify with caller-owned pairing preparation for the selected IC trust root.
///
/// Preparation caches only immutable key data. Every certificate, delegation and
/// witness is authenticated on every call. The caller selects and retains the
/// root; this function neither selects trust nor caches certificate acceptance.
pub fn verify_canister_sig_with_prepared_root(
    message: &[u8],
    signature_cbor: &[u8],
    public_key_der: &[u8],
    root: &crate::bls::PreparedPublicKey,
) -> Result<(), String> {
    authenticate(
        message,
        signature_cbor,
        public_key_der,
        RootKey::Prepared(root),
    )
    .map(|_| ())
}

/// Authenticate with a prepared root and extract the signing certificate's time.
///
/// Has the same checks as [`verify_canister_sig_with_time`]; freshness remains
/// caller-owned. Preparation does not bypass identity-key refusal.
pub fn verify_canister_sig_with_time_and_prepared_root(
    message: &[u8],
    signature_cbor: &[u8],
    public_key_der: &[u8],
    root: &crate::bls::PreparedPublicKey,
) -> Result<u64, CanisterSigTimeError> {
    let certificate = authenticate(
        message,
        signature_cbor,
        public_key_der,
        RootKey::Prepared(root),
    )
    .map_err(CanisterSigTimeError::Verification)?;
    signing_time(&certificate.tree).ok_or(CanisterSigTimeError::InvalidCertificateTime)
}

#[derive(Clone, Copy)]
enum RootKey<'a> {
    Raw(&'a [u8]),
    Prepared(&'a crate::bls::PreparedPublicKey),
}

fn authenticate(
    message: &[u8],
    signature_cbor: &[u8],
    public_key_der: &[u8],
    root_key: RootKey<'_>,
) -> Result<Certificate, String> {
    let key = CanisterSigPublicKey::try_from(public_key_der)?;
    let envelope: SignatureEnvelope = decode_tagged(signature_cbor, "signature")?;
    let seed_hash = hash_bytes(&key.seed);
    let message_hash = hash_bytes(message);
    let witness_leaf = required_leaf(&envelope.tree, &[b"sig", &seed_hash, &message_hash])?;
    if !witness_leaf.is_empty() {
        return Err("signature witness leaf is not empty".into());
    }

    let certificate = decode_certificate(&envelope.certificate)?;
    let certified_data = required_leaf(
        &certificate.tree,
        &[b"canister", key.canister_id.as_slice(), b"certified_data"],
    )?;
    if certified_data != envelope.tree.digest() {
        return Err("signature witness does not match certified data".into());
    }

    match &certificate.delegation {
        None => authenticate_certificate(&certificate, root_key)?,
        Some(delegation) => {
            let authority = decode_certificate(&delegation.certificate)?;
            if authority.delegation.is_some() {
                return Err("nested certificate delegation is unsupported".into());
            }
            authenticate_certificate(&authority, root_key)?;
            authorize_issuer(&authority.tree, &delegation.subnet_id, &key.canister_id)?;
            let subnet_key_der = required_leaf(
                &authority.tree,
                &[b"subnet", &delegation.subnet_id, b"public_key"],
            )?;
            let subnet_key = extract_raw_root_pk_from_der(subnet_key_der)?;
            authenticate_certificate(&certificate, RootKey::Raw(&subnet_key))?;
        }
    }
    Ok(certificate)
}

fn decode_tagged<T: DeserializeOwned>(bytes: &[u8], context: &str) -> Result<T, String> {
    if !bytes.starts_with(SELF_DESCRIBED_CBOR) {
        return Err(format!("{context} CBOR doesn't have a self-describing tag"));
    }
    serde_cbor::from_slice(bytes).map_err(|error| format!("invalid certificate CBOR: {error}"))
}

fn decode_certificate(bytes: &[u8]) -> Result<Certificate, String> {
    let decoded: CertificateEnvelope = decode_tagged(bytes, "certificate")?;
    Ok(Certificate {
        tree: decoded.tree,
        signature: decoded.signature,
        delegation: decoded.delegation.map(|delegation| Delegation {
            subnet_id: delegation.subnet_id,
            certificate: delegation.certificate,
        }),
    })
}

fn required_leaf<'tree>(tree: &'tree HashTree, path: &[&[u8]]) -> Result<&'tree [u8], String> {
    match tree.lookup_path(path.iter().copied()) {
        LookupResult::Found(value) => Ok(value),
        _ => Err("required certified leaf is unavailable".into()),
    }
}

fn authenticate_certificate(certificate: &Certificate, key: RootKey<'_>) -> Result<(), String> {
    // The BLS boundary owns checked point/subgroup admission and identity refusal
    // for both root and delegated keys; no second point decode is needed here.
    let mut signed = [0; 46];
    signed[..14].copy_from_slice(b"\x0dic-state-root");
    signed[14..].copy_from_slice(&certificate.tree.digest());
    match key {
        RootKey::Raw(bytes) => {
            crate::bls::verify_bls_signature(&certificate.signature, &signed, bytes)
        }
        RootKey::Prepared(key) => crate::bls::Signature::deserialize(&certificate.signature)
            .map_err(|_| ())
            .and_then(|signature| key.verify(&signed, &signature)),
    }
    .map_err(|()| "invalid BLS signature".into())
}

fn authorize_issuer(tree: &HashTree, subnet_id: &[u8], issuer: &Principal) -> Result<(), String> {
    let sharded_path: [&[u8]; 2] = [b"canister_ranges", subnet_id];
    match tree.lookup_subtree(sharded_path) {
        SubtreeLookupResult::Found(shards) => {
            let bytes = selected_shard(shards.as_ref(), issuer.as_slice())
                .ok_or("no applicable certified canister-range shard")?;
            authorize_ranges(bytes, issuer)
        }
        SubtreeLookupResult::Absent | SubtreeLookupResult::Unknown => {
            let bytes = required_leaf(tree, &[b"subnet", subnet_id, b"canister_ranges"])?;
            authorize_ranges(bytes, issuer)
        }
    }
}

fn authorize_ranges(bytes: &[u8], issuer: &Principal) -> Result<(), String> {
    let ranges: Vec<(Principal, Principal)> = serde_cbor::from_slice(bytes)
        .map_err(|error| format!("invalid certified canister ranges: {error}"))?;
    if ranges
        .iter()
        .any(|(start, end)| start <= issuer && issuer <= end)
    {
        Ok(())
    } else {
        Err("signing canister id not in canister_ranges".into())
    }
}

// Shard routing uses byte order, independently of the Principal ordering used
// inside the selected range leaf. A visible but unusable shard must not expose
// an older legacy authorization path as a fallback.
fn selected_shard<'tree>(root: &'tree HashTreeNode, issuer: &[u8]) -> Option<&'tree [u8]> {
    let mut pending = vec![root];
    let mut selected: Option<(&[u8], &HashTreeNode)> = None;
    while let Some(node) = pending.pop() {
        match node {
            HashTreeNode::Fork(children) => {
                pending.push(&children.0);
                pending.push(&children.1);
            }
            HashTreeNode::Labeled(label, child)
                if label.as_bytes() <= issuer
                    && selected.is_none_or(|(current, _)| current < label.as_bytes()) =>
            {
                selected = Some((label.as_bytes(), child));
            }
            _ => {}
        }
    }
    match selected?.1 {
        HashTreeNode::Leaf(bytes) => Some(bytes),
        _ => None,
    }
}

fn signing_time(tree: &HashTree) -> Option<u64> {
    // Preserve the external lookup's ordering and visibility rules. Counting
    // labels below adds duplicate refusal; it does not replace tree admission.
    let LookupResult::Found(bytes) = tree.lookup_path([b"time".as_slice()]) else {
        return None;
    };
    let mut pending = vec![tree.as_ref()];
    let mut seen = false;
    while let Some(node) = pending.pop() {
        match node {
            HashTreeNode::Fork(children) => {
                pending.push(&children.0);
                pending.push(&children.1);
            }
            HashTreeNode::Labeled(label, _) if label.as_bytes() == b"time" => {
                if seen {
                    return None;
                }
                seen = true;
            }
            _ => {}
        }
    }
    if seen { decode_time(bytes) } else { None }
}

fn decode_time(bytes: &[u8]) -> Option<u64> {
    if bytes.is_empty() || bytes.len() > 10 {
        return None;
    }
    let mut value = 0_u64;
    for (index, &byte) in bytes.iter().enumerate() {
        if index == 9 && byte > 1 {
            return None;
        }
        value |= u64::from(byte & 0x7f) << (index * 7);
        if byte & 0x80 == 0 {
            return (index + 1 == bytes.len()).then_some(value);
        }
    }
    None
}

#[cfg(test)]
mod tests;
