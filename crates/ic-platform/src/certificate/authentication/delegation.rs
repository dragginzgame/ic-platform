//! One root-signed delegation and complete selected canister-range authority.
use super::{
    Authority, PreparedRootKey, Principal, TimeWindow, VerificationLimits, VerifyError,
    checked_time, forest, ranges, verify_signature,
};
use crate::certificate::{CertificateDecodeLimits, decode_certificate};
use ic_certification::{
    Certificate, Delegation, HashTree, HashTreeNode, LookupResult, SubtreeLookupResult,
};

pub(super) fn verify(
    delegation: &Delegation,
    root: &PreparedRootKey,
    scope: Authority,
    window: TimeWindow,
    limits: VerificationLimits,
    nodes: &mut usize,
    certificate: &Certificate,
) -> Result<(Principal, TimeWindow, u64), VerifyError> {
    let subnet = Principal::try_from_slice(&delegation.subnet_id)
        .map_err(|_| VerifyError::InvalidDelegation)?;
    if delegation.certificate.len() > limits.delegation_bytes {
        return Err(VerifyError::DelegationLimit);
    }
    let authority = decode_certificate(
        &delegation.certificate,
        CertificateDecodeLimits {
            bytes: limits.delegation_bytes,
        },
    )
    .map_err(|_| VerifyError::InvalidDelegationEncoding)?;
    if authority.certificate.delegation.is_some() {
        return Err(VerifyError::InvalidDelegation);
    }
    forest(
        authority.certificate.tree.as_ref(),
        nodes,
        limits.tree_depth,
        &mut None,
        &mut false,
    )?;
    verify_signature(&authority.certificate, root)?;
    let time_ns = checked_time(&authority.certificate.tree, window.bounds()?)?;
    match scope {
        Authority::Canister(target) => authorize(
            &authority.certificate.tree,
            subnet.as_slice(),
            target,
            limits.canister_ranges,
        )?,
        Authority::Subnet { requested, .. } => {
            if requested != subnet {
                return Err(VerifyError::SubnetMismatch);
            }
        }
    }
    let key = required_leaf(
        &authority.certificate.tree,
        &[b"subnet", subnet.as_slice(), b"public_key"],
    )?;
    let key = PreparedRootKey::from_der(key)?;
    verify_signature(certificate, &key)?;
    Ok((subnet, window, time_ns))
}

fn required_leaf<'a>(tree: &'a HashTree, path: &[&[u8]]) -> Result<&'a [u8], VerifyError> {
    match tree.lookup_path(path.iter().copied()) {
        LookupResult::Found(bytes) => Ok(bytes),
        _ => Err(VerifyError::MissingDelegationPath),
    }
}

fn authorize(
    tree: &HashTree,
    subnet: &[u8],
    target: Principal,
    maximum: usize,
) -> Result<(), VerifyError> {
    match tree.lookup_subtree([b"canister_ranges".as_slice(), subnet]) {
        SubtreeLookupResult::Found(shards) => {
            let (label, child) = selected_shard(shards.as_ref(), target.as_slice())
                .ok_or(VerifyError::NotAuthorized)?;
            Principal::try_from_slice(label).map_err(|_| VerifyError::InvalidRanges)?;
            let HashTreeNode::Leaf(bytes) = child else {
                return Err(VerifyError::MissingDelegationPath);
            };
            ranges::authorize(bytes, target, maximum, Some(label))
        }
        // A separately authenticated legacy leaf can prove authority even if the
        // newer namespace is hidden. Unknown is never itself evidence of absence.
        SubtreeLookupResult::Absent | SubtreeLookupResult::Unknown => {
            let bytes = required_leaf(tree, &[b"subnet", subnet, b"canister_ranges"])?;
            ranges::authorize(bytes, target, maximum, None)
        }
    }
}

// Trees have already passed ordered-forest admission and traversal bounds.
// Select the greatest visible shard start <= target; never fall back from a
// selected malformed, pruned or nonauthorizing shard to an earlier shard.
fn selected_shard<'a>(
    node: &'a HashTreeNode,
    target: &[u8],
) -> Option<(&'a [u8], &'a HashTreeNode)> {
    match node {
        HashTreeNode::Fork(children) => {
            selected_shard(&children.1, target).or_else(|| selected_shard(&children.0, target))
        }
        HashTreeNode::Labeled(label, child) if label.as_bytes() <= target => {
            Some((label.as_bytes(), child.as_ref()))
        }
        _ => None,
    }
}
