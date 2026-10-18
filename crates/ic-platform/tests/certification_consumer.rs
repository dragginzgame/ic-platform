//! A standalone consumer of the explicit certification surface.
use ic_platform::certification::{
    Certificate, Delegation, HashTree, Label, LookupResult, empty, fork, fork_hash, labeled, leaf,
    pruned,
};

/// Exercise receipt lookup, witness hashes and certificate field identities.
pub fn roundtrip() {
    let payload = leaf(b"payload".to_vec());
    // SHA-256 of length-prefixed "ic-hashtree-leaf" followed by "payload".
    assert_eq!(
        payload.digest(),
        [
            0xb1, 0xf6, 0xcb, 0x58, 0x66, 0x8a, 0xed, 0xfb, 0x7d, 0x27, 0x6e, 0x78, 0xd3, 0x6a,
            0xbd, 0x98, 0xce, 0x2d, 0xe8, 0x66, 0x33, 0xce, 0x17, 0x10, 0xba, 0x54, 0x87, 0x8e,
            0xd1, 0x2c, 0x33, 0xe5
        ]
    );
    let label = Label::from_bytes(b"receipt");
    let tree: HashTree = labeled(label, payload);
    assert!(matches!(
        tree.lookup_path([b"receipt"]),
        LookupResult::Found(b"payload")
    ));
    assert!(matches!(
        tree.lookup_path([b"absent"]),
        LookupResult::Absent
    ));
    let hidden = pruned(tree.digest());
    assert!(matches!(
        hidden.lookup_path([b"receipt"]),
        LookupResult::Unknown
    ));
    let sibling = empty();
    let expected = fork_hash(&tree.digest(), &sibling.digest());
    assert_eq!(fork(tree.clone(), sibling.clone()).digest(), expected);
    assert_eq!(fork(hidden, sibling).digest(), expected);
    let certificate = Certificate {
        tree,
        signature: vec![1, 2, 3],
        delegation: Some(Delegation {
            subnet_id: vec![4],
            certificate: vec![5],
        }),
    };
    assert_eq!(certificate.signature, [1, 2, 3]);
    assert_eq!(certificate.delegation.unwrap().subnet_id, [4]);
}

#[test]
fn certificate_and_witness_operations_preserve_upstream_semantics() {
    roundtrip();
}
