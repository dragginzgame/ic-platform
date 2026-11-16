use super::*;
use ic_certification::{HashTree, HashTreeNode, empty, fork, labeled, leaf, pruned};
use serde_cbor::Value;

fn wire(node: &HashTreeNode) -> Value {
    let integer = |n| Value::Integer(n);
    Value::Array(match node {
        HashTreeNode::Empty() => vec![integer(0)],
        HashTreeNode::Fork(pair) => vec![integer(1), wire(&pair.0), wire(&pair.1)],
        HashTreeNode::Labeled(label, child) => vec![
            integer(2),
            Value::Bytes(label.as_bytes().to_vec()),
            wire(child),
        ],
        HashTreeNode::Leaf(bytes) => vec![integer(3), Value::Bytes(bytes.clone())],
        HashTreeNode::Pruned(hash) => vec![integer(4), Value::Bytes(hash.to_vec())],
    })
}
fn certificate(tree: &HashTree, key: u8) -> Vec<u8> {
    let mut message = b"\x0dic-state-root".to_vec();
    message.extend_from_slice(&tree.digest());
    envelope(
        &serde_cbor::to_vec(&wire(tree.as_ref())).unwrap(),
        &signer(key).sign(&message).serialize(),
    )
}
fn attach(mut outer: Vec<u8>, authority: &[u8], subnet: &[u8]) -> Vec<u8> {
    outer[0] = 0xa3;
    outer.extend_from_slice(b"\x6adelegation\xa2\x69subnet_id");
    outer.extend_from_slice(&serde_cbor::to_vec(&Value::Bytes(subnet.to_vec())).unwrap());
    outer.extend_from_slice(b"\x6bcertificate");
    outer.extend_from_slice(&serde_cbor::to_vec(&Value::Bytes(authority.to_vec())).unwrap());
    outer
}
fn time(value: u8) -> HashTree {
    labeled(b"time".to_vec(), leaf(vec![value]))
}
fn legacy(ranges: &[u8], key: &[u8], timestamp: u8) -> HashTree {
    fork(
        labeled(
            b"subnet".to_vec(),
            labeled(
                vec![1],
                fork(
                    labeled(b"canister_ranges".to_vec(), leaf(ranges.to_vec())),
                    labeled(b"public_key".to_vec(), leaf(key.to_vec())),
                ),
            ),
        ),
        time(timestamp),
    )
}
fn delegated(authority: &HashTree) -> Vec<u8> {
    attach(certificate(&time(10), 2), &certificate(authority, 1), &[1])
}
fn policy() -> TimePolicy {
    TimePolicy {
        certificate: window(),
        delegation: TimeWindow {
            reference_ns: 5,
            max_age_ns: 1,
            max_future_skew_ns: 1,
        },
    }
}
fn caps() -> VerificationLimits {
    VerificationLimits {
        tree_nodes: 100,
        tree_depth: 16,
        delegation_bytes: 4096,
        canister_ranges: 2,
    }
}
fn check<'a>(
    bytes: &'a [u8],
    target: &[u8],
    policy: TimePolicy,
    caps: VerificationLimits,
) -> Result<AuthenticatedCertificate<'a>, VerifyError> {
    let decoded =
        decode_certificate(bytes, CertificateDecodeLimits { bytes: bytes.len() }).unwrap();
    authenticate_certificate(
        decoded,
        &root(1),
        Principal::try_from_slice(target).unwrap(),
        policy,
        caps,
    )
}
const RANGE: &[u8] = b"\x81\x82\x41\x02\x41\x06";

#[test]
fn delegated_evidence_preserves_both_time_contexts_and_exact_bytes() {
    let bytes = delegated(&legacy(RANGE, root(2).as_der(), 5));
    for target in [2, 4, 6] {
        let evidence = check(&bytes, &[target], policy(), caps()).unwrap();
        assert_eq!(evidence.original_bytes(), bytes);
        assert_eq!(evidence.original_bytes().as_ptr(), bytes.as_ptr());
        assert_eq!(evidence.root_key_der(), root(1).as_der());
        assert_eq!(evidence.effective_canister().as_slice(), &[target]);
        assert_eq!(evidence.time_window(), window());
        assert_eq!(evidence.time_ns(), 10);
        assert_eq!(
            evidence.delegation(),
            Some((Principal::from_slice(&[1]), policy().delegation, 5))
        );
        assert_eq!(evidence.into_unverified().original_bytes(), bytes);
    }
    for target in [1, 7] {
        assert_eq!(
            check(&bytes, &[target], policy(), caps()).unwrap_err(),
            VerifyError::NotAuthorized
        );
    }
}

#[test]
fn windows_are_independent_and_inclusive() {
    for timestamp in [3, 4, 5, 6, 7] {
        let bytes = delegated(&legacy(RANGE, root(2).as_der(), timestamp));
        let result = check(&bytes, &[4], policy(), caps());
        if (4..=6).contains(&timestamp) {
            assert!(result.is_ok());
        } else {
            assert_eq!(result.unwrap_err(), VerifyError::TimeOutsideWindow);
        }
    }
    let bytes = delegated(&legacy(RANGE, root(2).as_der(), 5));
    let mut p = policy();
    p.certificate = p.delegation;
    assert_eq!(
        check(&bytes, &[4], p, caps()).unwrap_err(),
        VerifyError::TimeOutsideWindow
    );
}

#[test]
fn both_signatures_and_checked_subnet_keys_are_required() {
    let tree = legacy(RANGE, root(2).as_der(), 5);
    let bad_root = attach(certificate(&time(10), 2), &certificate(&tree, 2), &[1]);
    assert_eq!(
        check(&bad_root, &[4], policy(), caps()).unwrap_err(),
        VerifyError::Signature
    );
    let wrong_subnet = delegated(&legacy(RANGE, root(1).as_der(), 5));
    assert_eq!(
        check(&wrong_subnet, &[4], policy(), caps()).unwrap_err(),
        VerifyError::Signature
    );
    for key in [
        vec![0; 133],
        {
            // Canonical G2 x=2 is on-curve but outside the subgroup; independently
            // classified by the external subgroup test using upstream point admission.
            let mut k = *root(2).as_der();
            k[37..].fill(0);
            k[37] = 0x80;
            k[132] = 2;
            k.to_vec()
        },
        {
            let mut k = *root(2).as_der();
            k[37..].fill(0);
            k[37] = 0xc0;
            k.to_vec()
        },
    ] {
        let bytes = delegated(&legacy(RANGE, &key, 5));
        assert_eq!(
            check(&bytes, &[4], policy(), caps()).unwrap_err(),
            VerifyError::InvalidRootKey
        );
    }
}

#[test]
fn nested_malformed_and_oversized_delegations_refuse() {
    let authority = certificate(&legacy(RANGE, root(2).as_der(), 5), 1);
    let nested = attach(authority.clone(), &[], &[1]);
    let bytes = attach(certificate(&time(10), 2), &nested, &[1]);
    assert_eq!(
        check(&bytes, &[4], policy(), caps()).unwrap_err(),
        VerifyError::InvalidDelegation
    );
    let bytes = attach(certificate(&time(10), 2), &authority, &[1; 30]);
    assert_eq!(
        check(&bytes, &[4], policy(), caps()).unwrap_err(),
        VerifyError::InvalidDelegation
    );
    let bytes = attach(certificate(&time(10), 2), &[0], &[1]);
    assert_eq!(
        check(&bytes, &[4], policy(), caps()).unwrap_err(),
        VerifyError::InvalidDelegationEncoding
    );
    let bytes = attach(certificate(&time(10), 2), &authority, &[1]);
    let mut cap = caps();
    cap.delegation_bytes = authority.len() - 1;
    assert_eq!(
        check(&bytes, &[4], policy(), cap).unwrap_err(),
        VerifyError::DelegationLimit
    );
    cap.delegation_bytes += 1;
    assert!(check(&bytes, &[4], policy(), cap).is_ok());
}

#[test]
fn shared_tree_and_range_allowances_have_exact_boundaries() {
    // Outer time = 2 nodes; legacy authority = 10 nodes; maximum depth = 6.
    let bytes = delegated(&legacy(RANGE, root(2).as_der(), 5));
    let mut cap = caps();
    cap.tree_nodes = 12;
    cap.tree_depth = 6;
    cap.canister_ranges = 1;
    assert!(check(&bytes, &[4], policy(), cap).is_ok());
    cap.tree_nodes -= 1;
    assert_eq!(
        check(&bytes, &[4], policy(), cap).unwrap_err(),
        VerifyError::TreeLimit
    );
    cap.tree_nodes += 1;
    cap.tree_depth -= 1;
    assert_eq!(
        check(&bytes, &[4], policy(), cap).unwrap_err(),
        VerifyError::TreeLimit
    );
    cap.tree_depth += 1;
    cap.canister_ranges = 0;
    assert_eq!(
        check(&bytes, &[4], policy(), cap).unwrap_err(),
        VerifyError::RangeLimit
    );
}

#[test]
fn entire_range_leaf_is_checked_after_a_match() {
    for ranges in [
        b"\x82\x82\x41\x02\x41\x06\x82\x41\x09\x41\x08".as_slice(), // reversed
        b"\x82\x82\x41\x02\x41\x06\x82\x41\x05\x41\x08",            // overlap
        b"\x82\x82\x41\x02\x41\x06\x82\x61x\x41\x08",               // text
        b"\x81\x83\x41\x02\x41\x06\x40",                            // extra tuple item
        b"\x81\x82\x41\x02\x41\x06\x00",                            // trailing CBOR
    ] {
        let bytes = delegated(&legacy(ranges, root(2).as_der(), 5));
        assert_eq!(
            check(&bytes, &[4], policy(), caps()).unwrap_err(),
            VerifyError::InvalidRanges
        );
    }
    let bytes = delegated(&legacy(b"\x80", root(2).as_der(), 5));
    assert_eq!(
        check(&bytes, &[4], policy(), caps()).unwrap_err(),
        VerifyError::NotAuthorized
    );
}

#[test]
fn bytewise_order_and_indefinite_lists_preserve_range_semantics() {
    // [01 ff, 02] includes [01 ff] and [02], despite opposite length-first order.
    let bytes = delegated(&legacy(
        b"\x9f\x9f\x42\x01\xff\x41\x02\xff\xff",
        root(2).as_der(),
        5,
    ));
    assert!(check(&bytes, &[1, 255], policy(), caps()).is_ok());
    assert!(check(&bytes, &[2], policy(), caps()).is_ok());
    assert_eq!(
        check(&bytes, &[1], policy(), caps()).unwrap_err(),
        VerifyError::NotAuthorized
    );
    let bytes = delegated(&legacy(
        b"\x81\x82\x41\x02\x42\x01\xff",
        root(2).as_der(),
        5,
    ));
    assert_eq!(
        check(&bytes, &[2], policy(), caps()).unwrap_err(),
        VerifyError::InvalidRanges
    );
}

fn sharded(shards: HashTree) -> HashTree {
    fork(
        labeled(b"canister_ranges".to_vec(), labeled(vec![1], shards)),
        fork(
            labeled(
                b"subnet".to_vec(),
                labeled(
                    vec![1],
                    labeled(b"public_key".to_vec(), leaf(root(2).as_der().to_vec())),
                ),
            ),
            time(5),
        ),
    )
}
#[test]
fn selected_shard_cannot_fall_back_or_disagree_with_its_label() {
    let good = labeled(vec![2], leaf(RANGE.to_vec()));
    let bytes = delegated(&sharded(good.clone()));
    assert!(check(&bytes, &[4], policy(), caps()).is_ok());
    assert_eq!(
        check(&bytes, &[1], policy(), caps()).unwrap_err(),
        VerifyError::NotAuthorized
    );
    for child in [pruned([0; 32]), empty()] {
        let bytes = delegated(&sharded(fork(good.clone(), labeled(vec![3], child))));
        assert_eq!(
            check(&bytes, &[4], policy(), caps()).unwrap_err(),
            VerifyError::MissingDelegationPath
        );
    }
    let bytes = delegated(&sharded(labeled(vec![1], leaf(RANGE.to_vec()))));
    assert_eq!(
        check(&bytes, &[4], policy(), caps()).unwrap_err(),
        VerifyError::InvalidRanges
    );
    let bytes = delegated(&sharded(fork(
        good,
        labeled(vec![3], leaf(b"\x81\x82\x41\x03\x41\x03".to_vec())),
    )));
    assert_eq!(
        check(&bytes, &[4], policy(), caps()).unwrap_err(),
        VerifyError::NotAuthorized
    );
}

#[test]
fn unavailable_authority_and_ambiguous_delegation_trees_refuse() {
    for authority in [time(5), fork(pruned([0; 32]), time(5))] {
        let bytes = delegated(&authority);
        assert_eq!(
            check(&bytes, &[4], policy(), caps()).unwrap_err(),
            VerifyError::MissingDelegationPath
        );
    }
    let bytes = delegated(&fork(time(5), time(5)));
    assert_eq!(
        check(&bytes, &[4], policy(), caps()).unwrap_err(),
        VerifyError::InvalidTree
    );
}

#[test]
fn range_principal_lengths_and_streaming_limits_are_enforced() {
    let endpoint = vec![7; 29];
    let pair = Value::Array(vec![
        Value::Bytes(endpoint.clone()),
        Value::Bytes(endpoint.clone()),
    ]);
    let ranges = serde_cbor::to_vec(&Value::Array(vec![pair])).unwrap();
    let bytes = delegated(&legacy(&ranges, root(2).as_der(), 5));
    assert!(check(&bytes, &endpoint, policy(), caps()).is_ok());
    let ranges = serde_cbor::to_vec(&Value::Array(vec![Value::Array(vec![
        Value::Bytes(vec![7; 30]),
        Value::Bytes(vec![7; 30]),
    ])]))
    .unwrap();
    let bytes = delegated(&legacy(&ranges, root(2).as_der(), 5));
    assert_eq!(
        check(&bytes, &[4], policy(), caps()).unwrap_err(),
        VerifyError::InvalidRanges
    );
    // Indefinite outer list has no size hint: enforce the cap after a match.
    let ranges = b"\x9f\x82\x41\x02\x41\x06\x82\x41\x08\x41\x09\xff";
    let bytes = delegated(&legacy(ranges, root(2).as_der(), 5));
    let mut cap = caps();
    cap.canister_ranges = 1;
    assert_eq!(
        check(&bytes, &[4], policy(), cap).unwrap_err(),
        VerifyError::RangeLimit
    );
    cap.canister_ranges = 2;
    assert!(check(&bytes, &[4], policy(), cap).is_ok());
}

#[test]
fn selected_subnet_time_uses_subnet_authority_and_signing_timestamp() {
    // Subnet read-state need not expose canister ranges. This tree reveals only
    // its certified subnet key and delegation time; no canister authority is claimed.
    let authority = fork(
        labeled(
            b"subnet".to_vec(),
            labeled(
                vec![1],
                labeled(b"public_key".to_vec(), leaf(root(2).as_der().to_vec())),
            ),
        ),
        time(5),
    );
    let bytes = delegated(&authority);
    let decode =
        || decode_certificate(&bytes, CertificateDecodeLimits { bytes: bytes.len() }).unwrap();
    let subnet = Principal::from_slice(&[1]);
    let mut cap = caps();
    cap.canister_ranges = 0;
    let evidence =
        authenticate_subnet_time(decode(), &root(1), subnet, None, policy(), cap).unwrap();
    assert_eq!(evidence.time_ns(), 10);
    assert_eq!(evidence.delegation(), Some((policy().delegation, 5)));
    assert_eq!(evidence.subnet(), subnet);
    assert_eq!(
        authenticate_subnet_time(
            decode(),
            &root(1),
            Principal::from_slice(&[2]),
            None,
            policy(),
            cap
        )
        .unwrap_err(),
        VerifyError::SubnetMismatch
    );
    // The same input still cannot serve as canister authority without ranges.
    assert_eq!(
        check(&bytes, &[4], policy(), cap).unwrap_err(),
        VerifyError::MissingDelegationPath
    );
}
