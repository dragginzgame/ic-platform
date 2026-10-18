//! A standalone consumer of the explicit certification surface.
use ic_platform::certification::{
    AsHashTree, Certificate, Delegation, Hash, HashTree, Label, LookupResult, RbTree,
    SubtreeLookupResult, empty, fork, fork_hash, labeled, labeled_hash, leaf, pruned,
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
    // Independent SHA-256 vector: labelled domain, label, then the leaf digest.
    assert_eq!(
        tree.digest(),
        [
            0xb3, 0x79, 0x22, 0xaa, 0xcb, 0x6f, 0x56, 0x70, 0x85, 0x23, 0x2a, 0x2f, 0x3d, 0x2f,
            0xd6, 0x68, 0x7a, 0x93, 0x9c, 0x5f, 0x79, 0xba, 0xc0, 0x67, 0x35, 0x5d, 0x37, 0xfe,
            0xc7, 0x1e, 0xfe, 0x7c,
        ]
    );
    assert_eq!(
        tree.digest(),
        labeled_hash(b"receipt", &leaf(b"payload".to_vec()).digest())
    );
    assert!(matches!(
        tree.lookup_subtree([b"receipt"]),
        SubtreeLookupResult::Found(_)
    ));
    assert!(matches!(
        tree.lookup_subtree([b"absent"]),
        SubtreeLookupResult::Absent
    ));
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
        hidden.lookup_subtree([b"receipt"]),
        SubtreeLookupResult::Unknown
    ));
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

    // Auth's store uses nested mutable trees and witnesses with this exact identity.
    let seed: Hash = [1; 32];
    let message: Hash = [2; 32];
    let mut messages: RbTree<Hash, Vec<u8>> = RbTree::new();
    messages.insert(message, Vec::new());
    let mut store: RbTree<Hash, RbTree<Hash, Vec<u8>>> = RbTree::new();
    store.insert(seed, messages);
    let root = store.root_hash();
    assert_eq!(
        root,
        labeled(seed, labeled(message, leaf(Vec::new()))).digest()
    );
    assert_eq!(store.as_hash_tree().digest(), root);
    let witness = store.nested_witness(&seed, |messages| messages.witness(&message));
    assert_eq!(witness.digest(), root);
    assert_eq!(
        witness.lookup_path([&seed, &message]),
        LookupResult::Found(&[])
    );
    store.modify(&seed, |messages| messages.insert([3; 32], vec![4]));
    assert_ne!(store.root_hash(), root);
    store.modify(&seed, |messages| messages.delete(&[3; 32]));
    assert_eq!(store.root_hash(), root);
    store.delete(&seed);
    assert_eq!(store.root_hash(), empty().digest());
}

#[test]
fn certificate_and_witness_operations_preserve_upstream_semantics() {
    roundtrip();
}
