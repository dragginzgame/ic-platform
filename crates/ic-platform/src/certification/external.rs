//! Explicit native Agent conversions. These copy labels and rebuild tree boxes;
//! they preserve node order and bytes and perform no hashing or authentication.

use super::upstream::{Certificate, Delegation, HashTree, hash_tree::HashTreeNode};

impl From<Certificate> for ic_certification::Certificate {
    fn from(value: Certificate) -> Self {
        Self {
            tree: value.tree.into(),
            signature: value.signature,
            delegation: value.delegation.map(|d| ic_certification::Delegation {
                subnet_id: d.subnet_id,
                certificate: d.certificate,
            }),
        }
    }
}

impl From<ic_certification::Certificate> for Certificate {
    fn from(value: ic_certification::Certificate) -> Self {
        Self {
            tree: value.tree.into(),
            signature: value.signature,
            delegation: value.delegation.map(|d| Delegation {
                subnet_id: d.subnet_id,
                certificate: d.certificate,
            }),
        }
    }
}

impl From<HashTree> for ic_certification::HashTree {
    fn from(value: HashTree) -> Self {
        fn convert(node: HashTreeNode<Vec<u8>>) -> ic_certification::HashTree {
            use ic_certification::{empty, fork, labeled, leaf, pruned};
            match node {
                HashTreeNode::Empty() => empty(),
                HashTreeNode::Fork(children) => {
                    let (left, right) = *children;
                    fork(convert(left), convert(right))
                }
                HashTreeNode::Labeled(label, child) => labeled(label.as_bytes(), convert(*child)),
                HashTreeNode::Leaf(bytes) => leaf(bytes),
                HashTreeNode::Pruned(hash) => pruned(hash),
            }
        }
        convert(value.into())
    }
}

impl From<ic_certification::HashTree> for HashTree {
    fn from(value: ic_certification::HashTree) -> Self {
        fn convert(node: ic_certification::HashTreeNode) -> HashTree {
            use super::upstream::{empty, fork, labeled, leaf, pruned};
            use ic_certification::HashTreeNode;
            match node {
                HashTreeNode::Empty() => empty(),
                HashTreeNode::Fork(children) => {
                    let (left, right) = *children;
                    fork(convert(left), convert(right))
                }
                HashTreeNode::Labeled(label, child) => labeled(label.as_bytes(), convert(*child)),
                HashTreeNode::Leaf(bytes) => leaf(bytes),
                HashTreeNode::Pruned(hash) => pruned(hash),
            }
        }
        convert(value.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certification::{empty, fork, labeled, leaf, pruned};

    #[test]
    fn agent_conversion_preserves_every_node_and_certificate_bytes() {
        let tree = fork(
            empty(),
            fork(labeled([0, 255], leaf(vec![1, 0, 255])), pruned([42; 32])),
        );
        for delegation in [
            None,
            Some(Delegation {
                subnet_id: vec![0, 255],
                certificate: vec![1, 2, 0, 255],
            }),
        ] {
            let owned = Certificate {
                tree: tree.clone(),
                signature: vec![3, 0, 255],
                delegation,
            };
            let bytes = serde_cbor::to_vec(&owned).unwrap();
            let external: ic_certification::Certificate = owned.clone().into();
            assert_eq!(external, serde_cbor::from_slice(&bytes).unwrap());
            assert_eq!(serde_cbor::to_vec(&external).unwrap(), bytes);
            assert_eq!(external.tree.digest(), owned.tree.digest());
            assert_eq!(Certificate::from(external), owned);
        }
    }

    #[test]
    fn owned_and_external_decoders_agree_on_malformed_nodes() {
        // Bad tag, wrong arity, malformed pruned digest, wrong label/leaf type.
        for bytes in [
            &[0x81, 0x05][..],
            &[0x82, 0x00, 0x00],
            &[0x82, 0x04, 0x41, 0x00],
            &[0x83, 0x02, 0x01, 0x81, 0x00],
            &[0x82, 0x03, 0x01],
        ] {
            assert!(serde_cbor::from_slice::<HashTree>(bytes).is_err());
            assert!(serde_cbor::from_slice::<ic_certification::HashTree>(bytes).is_err());
        }
    }
}
