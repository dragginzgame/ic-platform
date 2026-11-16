//! Certificate authentication with explicit roots, authority and time windows.

mod delegation;
mod ranges;
mod subnet;
pub use subnet::{
    AuthenticatedSubnetTime, authenticate_subnet_time, authenticate_subnet_time_from_certificate,
};

use super::DecodedCertificate;
use candidate_types::Principal;
use ic_certification::{HashTreeNode, LookupResult};
use ic_verify_bls_signature::{PublicKey, Signature};

// Exact IC BLS SubjectPublicKeyInfo framing; no general ASN.1 parser is needed.
const DER_PREFIX: &[u8; 37] = b"\x30\x81\x82\x30\x1d\x06\x0d\x2b\x06\x01\x04\x01\x82\xdc\x7c\x05\x03\x01\x02\x01\x06\x0c\x2b\x06\x01\x04\x01\x82\xdc\x7c\x05\x03\x02\x01\x03\x61\x00";

/// Caller-owned, checked BLS root key. Preparation does not establish trust.
#[derive(Clone, Debug)]
pub struct PreparedRootKey {
    key: PublicKey,
    der: [u8; 133],
}
impl PreparedRootKey {
    /// Admit exact IC DER framing, an upstream-checked subgroup point and a
    /// nonidentity key. The caller remains responsible for selecting a trusted root.
    ///
    /// # Errors
    /// Returns `InvalidRootKey` for malformed framing, points or the identity.
    pub fn from_der(bytes: &[u8]) -> Result<Self, VerifyError> {
        let der: [u8; 133] = bytes.try_into().map_err(|_| VerifyError::InvalidRootKey)?;
        if !der.starts_with(DER_PREFIX) {
            return Err(VerifyError::InvalidRootKey);
        }
        let key = PublicKey::deserialize(&der[37..]).map_err(|_| VerifyError::InvalidRootKey)?;
        if identity(&key.serialize()) {
            return Err(VerifyError::InvalidRootKey);
        }
        Ok(Self { key, der })
    }

    /// The exact admitted key envelope; no implicit network root is selected.
    #[must_use]
    pub const fn as_der(&self) -> &[u8; 133] {
        &self.der
    }
}

// Upstream checked decoding enforces canonical compressed encodings. This is an
// encoding-level identity refusal, not a curve operation or point decoder.
fn identity(bytes: &[u8]) -> bool {
    bytes[0] == 0xc0 && bytes[1..].iter().all(|byte| *byte == 0)
}

/// Inclusive historical time policy, independent of any clock or freshness default.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimeWindow {
    /// Explicit reference time in Unix nanoseconds.
    pub reference_ns: u64,
    /// Maximum permitted age relative to the reference.
    pub max_age_ns: u64,
    /// Maximum permitted future skew relative to the reference.
    pub max_future_skew_ns: u64,
}
impl TimeWindow {
    fn bounds(self) -> Result<(u64, u64), VerifyError> {
        Ok((
            self.reference_ns
                .checked_sub(self.max_age_ns)
                .ok_or(VerifyError::InvalidWindow)?,
            self.reference_ns
                .checked_add(self.max_future_skew_ns)
                .ok_or(VerifyError::InvalidWindow)?,
        ))
    }
}

/// Independent inclusive windows for signing and delegation certificates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimePolicy {
    /// Window for the outer signing certificate.
    pub certificate: TimeWindow,
    /// Window for the root-signed delegation, when present.
    pub delegation: TimeWindow,
}

/// Traversal bounds checked before hashing or lookup; not decoding allocation quotas.
#[derive(Clone, Copy, Debug)]
pub struct VerificationLimits {
    /// Combined visible nodes across both trees, including empty and pruned nodes.
    pub tree_nodes: usize,
    /// Maximum root-inclusive tree depth.
    pub tree_depth: usize,
    /// Embedded delegation certificate bytes, checked before its decoding.
    pub delegation_bytes: usize,
    /// Maximum pairs in the complete selected certified range leaf.
    pub canister_ranges: usize,
}

/// Authentication refusal. No partially authenticated evidence is returned.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum VerifyError {
    /// The root envelope or checked public key is invalid or an identity point.
    #[error("invalid certificate root key")]
    InvalidRootKey,
    /// This direct-root API never ignores or follows a delegation.
    #[error("delegated certificates require the delegated verification path")]
    DelegationUnsupported,
    /// Nested delegation or an invalid subnet principal.
    #[error("invalid certificate delegation")]
    InvalidDelegation,
    /// Embedded certificate exceeds the caller's byte allowance.
    #[error("delegation certificate exceeds byte allowance")]
    DelegationLimit,
    /// Embedded certificate fails strict decoding.
    #[error("invalid delegation certificate encoding")]
    InvalidDelegationEncoding,
    /// A required certified authority leaf is absent, pruned or not a leaf.
    #[error("required certified delegation path is unavailable")]
    MissingDelegationPath,
    /// The complete selected range list is malformed or inconsistent.
    #[error("invalid certified canister ranges")]
    InvalidRanges,
    /// The range-pair allowance was exhausted.
    #[error("certified canister ranges exceed allowance")]
    RangeLimit,
    /// No admitted certified range contains the effective canister.
    #[error("certificate is not authorized for the effective canister")]
    NotAuthorized,
    /// The requested subnet differs from the authenticated signing authority.
    #[error("certificate does not belong to the requested subnet")]
    SubnetMismatch,
    /// Direct-root subnet observations need an explicitly trusted root/subnet binding.
    #[error("direct-root certificate requires a trusted root subnet identity")]
    RootSubnetRequired,
    /// Inclusive window arithmetic overflowed or underflowed.
    #[error("invalid certificate time window")]
    InvalidWindow,
    /// Tree node or depth allowance was exhausted before hashing/lookup.
    #[error("certificate tree exceeds traversal allowance")]
    TreeLimit,
    /// Visible sibling labels or leaves are ambiguous or out of order.
    #[error("ambiguous certificate tree")]
    InvalidTree,
    /// A complete, unique u64 LEB128 time leaf is unavailable.
    #[error("invalid certificate time")]
    InvalidTime,
    /// Certificate time falls outside the inclusive supplied window.
    #[error("certificate time outside admitted window")]
    TimeOutsideWindow,
    /// The signature encoding or cryptographic verification failed.
    #[error("invalid certificate signature")]
    Signature,
}

/// Immutable authenticated evidence bound to the original bytes and checked context.
///
/// This proves neither application authorization nor a request's status or freshness
/// outside the supplied window. Direct roots have global authority; delegated
/// evidence checks the supplied target against certified ranges. There is no mutable certificate
/// accessor, public constructor or Deserialize implementation.
///
/// ```compile_fail
/// use ic_platform::certificate::AuthenticatedCertificate;
/// fn edit(evidence: &mut AuthenticatedCertificate<'_>) {
///     evidence.time_ns = 0;
/// }
/// ```
/// ```compile_fail
/// use ic_platform::certificate::{AuthenticatedCertificate, DecodedCertificate};
/// fn promote(decoded: DecodedCertificate<'_>) -> AuthenticatedCertificate<'_> {
///     decoded.into()
/// }
/// ```
/// ```compile_fail
/// use ic_platform::certificate::AuthenticatedCertificate;
/// fn edit(evidence: &mut AuthenticatedCertificate<'_>) {
///     *evidence.tree() = ic_certification::empty();
/// }
/// ```
/// ```compile_fail
/// use ic_platform::certificate::AuthenticatedCertificate;
/// fn deserialize(bytes: &[u8]) -> AuthenticatedCertificate<'_> {
///     serde_cbor::from_slice(bytes).unwrap()
/// }
/// ```
#[derive(Debug)]
pub struct AuthenticatedCertificate<'a> {
    decoded: DecodedCertificate<'a>,
    root_der: [u8; 133],
    target: Principal,
    window: TimeWindow,
    time_ns: u64,
    delegation: Option<(Principal, TimeWindow, u64)>,
}
impl<'a> AuthenticatedCertificate<'a> {
    /// Exact original encoded certificate bytes.
    #[must_use]
    pub const fn original_bytes(&self) -> &'a [u8] {
        self.decoded.original_bytes()
    }
    /// Immutable authenticated tree, with upstream lookup distinctions intact.
    #[must_use]
    pub const fn tree(&self) -> &ic_certification::HashTree {
        &self.decoded.certificate.tree
    }
    /// Exact root admitted for this verification, not a default network root.
    #[must_use]
    pub const fn root_key_der(&self) -> &[u8; 133] {
        &self.root_der
    }
    /// Target supplied for this verification and checked against delegated ranges.
    #[must_use]
    pub const fn effective_canister(&self) -> Principal {
        self.target
    }
    /// Explicit window checked against the authenticated timestamp.
    #[must_use]
    pub const fn time_window(&self) -> TimeWindow {
        self.window
    }
    /// Authenticated certificate time, in Unix nanoseconds.
    #[must_use]
    pub const fn time_ns(&self) -> u64 {
        self.time_ns
    }
    /// Authenticated subnet, delegation window and timestamp, if delegated.
    #[must_use]
    pub const fn delegation(&self) -> Option<(Principal, TimeWindow, u64)> {
        self.delegation
    }
    /// Surrender authenticated status and recover the immutable unverified wrapper.
    #[must_use]
    pub fn into_unverified(self) -> DecodedCertificate<'a> {
        self.decoded
    }
}

/// Authenticate a certificate signed directly by an explicitly supplied root.
///
/// Delegations always refuse. Caller owns trust-root admission, target selection,
/// the time window and application leaf/path acceptance. Unknown/pruned lookups
/// remain distinguishable from absence. No global evidence cache is used.
///
/// # Errors
/// Refuses delegation, invalid windows, exhausted traversal bounds, ambiguous
/// trees, invalid/out-of-window time and invalid signatures. No fallback applies.
pub fn authenticate_direct_certificate<'a>(
    decoded: DecodedCertificate<'a>,
    root: &PreparedRootKey,
    effective_canister: Principal,
    window: TimeWindow,
    limits: VerificationLimits,
) -> Result<AuthenticatedCertificate<'a>, VerifyError> {
    if decoded.certificate.delegation.is_some() {
        return Err(VerifyError::DelegationUnsupported);
    }
    authenticate_certificate(
        decoded,
        root,
        effective_canister,
        TimePolicy {
            certificate: window,
            delegation: window,
        },
        limits,
    )
}

/// Authenticate a direct-root or singly delegated certificate.
///
/// Both visible trees share the node allowance and are admitted before hashing.
/// Delegation authority requires a root signature, its own explicit time window,
/// a checked subnet key and the target in a complete validated range leaf.
/// No trust discovery, clock, retries or application acceptance is supplied.
///
/// # Errors
/// Refuses malformed/nested delegations, unavailable authority, range/time/key/
/// signature failures, invalid windows, ambiguous trees and exhausted bounds.
pub fn authenticate_certificate<'a>(
    decoded: DecodedCertificate<'a>,
    root: &PreparedRootKey,
    effective_canister: Principal,
    time: TimePolicy,
    limits: VerificationLimits,
) -> Result<AuthenticatedCertificate<'a>, VerifyError> {
    let verified = verify_certificate(
        &decoded.certificate,
        root,
        Authority::Canister(effective_canister),
        time,
        limits,
    )?;
    Ok(AuthenticatedCertificate {
        decoded,
        root_der: root.der,
        target: effective_canister,
        window: time.certificate,
        time_ns: verified.time_ns,
        delegation: verified.delegation,
    })
}

#[derive(Clone, Copy)]
enum Authority {
    Canister(Principal),
    Subnet {
        requested: Principal,
        root_subnet: Option<Principal>,
    },
}

struct VerifiedContext {
    time_ns: u64,
    delegation: Option<(Principal, TimeWindow, u64)>,
}

fn verify_certificate(
    certificate: &ic_certification::Certificate,
    root: &PreparedRootKey,
    authority: Authority,
    time: TimePolicy,
    limits: VerificationLimits,
) -> Result<VerifiedContext, VerifyError> {
    let bounds = time.certificate.bounds()?;
    time.delegation.bounds()?;
    let mut nodes = limits.tree_nodes;
    forest(
        certificate.tree.as_ref(),
        &mut nodes,
        limits.tree_depth,
        &mut None,
        &mut false,
    )?;
    let delegation = if let Some(value) = &certificate.delegation {
        Some(delegation::verify(
            value,
            root,
            authority,
            time.delegation,
            limits,
            &mut nodes,
            certificate,
        )?)
    } else {
        if let Authority::Subnet {
            requested,
            root_subnet,
        } = authority
        {
            let root_subnet = root_subnet.ok_or(VerifyError::RootSubnetRequired)?;
            if requested != root_subnet {
                return Err(VerifyError::SubnetMismatch);
            }
        }
        None
    };
    let time_ns = checked_time(&certificate.tree, bounds)?;
    if delegation.is_none() {
        verify_signature(certificate, root)?;
    }
    Ok(VerifiedContext {
        time_ns,
        delegation,
    })
}

fn verify_signature(
    certificate: &ic_certification::Certificate,
    root: &PreparedRootKey,
) -> Result<(), VerifyError> {
    let signature =
        Signature::deserialize(&certificate.signature).map_err(|_| VerifyError::Signature)?;
    if identity(&signature.serialize()) {
        return Err(VerifyError::Signature);
    }
    let mut message = [0; 46];
    message[..14].copy_from_slice(b"\x0dic-state-root");
    message[14..].copy_from_slice(&certificate.tree.digest());
    root.key
        .verify(&message, &signature)
        .map_err(|()| VerifyError::Signature)
}

fn checked_time(tree: &ic_certification::HashTree, bounds: (u64, u64)) -> Result<u64, VerifyError> {
    let LookupResult::Found(bytes) = tree.lookup_path([b"time".as_slice()]) else {
        return Err(VerifyError::InvalidTime);
    };
    if bytes.is_empty() || bytes.len() > 10 {
        return Err(VerifyError::InvalidTime);
    }
    let mut time = 0;
    for (index, &byte) in bytes.iter().enumerate() {
        if index == 9 && byte > 1 {
            return Err(VerifyError::InvalidTime);
        }
        time |= u64::from(byte & 127) << (index * 7);
        if byte & 128 == 0 {
            if index + 1 != bytes.len() {
                return Err(VerifyError::InvalidTime);
            }
            return if bounds.0 <= time && time <= bounds.1 {
                Ok(time)
            } else {
                Err(VerifyError::TimeOutsideWindow)
            };
        }
    }
    Err(VerifyError::InvalidTime)
}

// Ordered-forest admission adapted from the reviewed old Platform historical
// adapter at c24e3a59583eabcf5b6bb87987269c13a209a51f (Toko licence).
// This checks representation before upstream hashing; it implements no hashing.
fn forest<'a>(
    node: &'a HashTreeNode,
    nodes: &mut usize,
    depth: usize,
    previous: &mut Option<&'a [u8]>,
    value: &mut bool,
) -> Result<(), VerifyError> {
    *nodes = nodes.checked_sub(1).ok_or(VerifyError::TreeLimit)?;
    let next_depth = depth.checked_sub(1).ok_or(VerifyError::TreeLimit)?;
    match node {
        HashTreeNode::Fork(children) => {
            forest(&children.0, nodes, next_depth, previous, value)?;
            forest(&children.1, nodes, next_depth, previous, value)
        }
        HashTreeNode::Labeled(label, child) => {
            let label = label.as_bytes();
            if *value || previous.is_some_and(|old| old >= label) {
                return Err(VerifyError::InvalidTree);
            }
            *previous = Some(label);
            forest(child, nodes, next_depth, &mut None, &mut false)
        }
        HashTreeNode::Leaf(_) => {
            if *value || previous.is_some() {
                return Err(VerifyError::InvalidTree);
            }
            *value = true;
            Ok(())
        }
        HashTreeNode::Empty() | HashTreeNode::Pruned(_) => Ok(()),
    }
}

#[cfg(test)]
mod tests;
