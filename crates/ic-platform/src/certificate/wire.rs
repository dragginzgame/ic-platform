//! Strict wire visitors adapted from ic-platform c24e3a59583eabcf5b6bb87987269c13a209a51f.
//! Derived there from ic-host-tools 9ed16c7a480ad80ae6ff606bd17de3350a2b0b97.
//! Copyright 2026 ic-host-tools contributors. Toko Studios Network Licence 1.0.
//! Scope and provenance: docs/qualification/platform-certificate.md.

use ic_certification::{Certificate, Delegation, HashTree, empty, fork, labeled, leaf, pruned};
use serde::{
    Deserialize, Deserializer,
    de::{self, IgnoredAny, SeqAccess, Visitor},
};
use std::fmt;

pub(super) fn decode(bytes: &[u8]) -> Result<Certificate, serde_cbor::Error> {
    let wire: CertificateWire = serde_cbor::from_slice(bytes)?;
    Ok(Certificate {
        tree: wire.tree.0,
        signature: wire.signature.0,
        delegation: wire.delegation.map(|value| Delegation {
            subnet_id: value.subnet_id.0,
            certificate: value.certificate.0,
        }),
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CertificateWire {
    tree: OwnedTree,
    signature: ByteString,
    #[serde(default)]
    delegation: Option<DelegationWire>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DelegationWire {
    subnet_id: ByteString,
    certificate: ByteString,
}

// Shared internally with strict protocol-content admission; never a public codec.
struct ByteString(Vec<u8>);
struct ByteVisitor;
impl<'de> Deserialize<'de> for ByteString {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        // deserialize_bytes permits integer arrays in some codecs; any + a
        // byte-only visitor admits the actual CBOR major type instead.
        d.deserialize_any(ByteVisitor)
    }
}
impl Visitor<'_> for ByteVisitor {
    type Value = ByteString;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CBOR byte string")
    }
    fn visit_bytes<E: de::Error>(self, bytes: &[u8]) -> Result<Self::Value, E> {
        Ok(ByteString(bytes.to_vec()))
    }
    fn visit_byte_buf<E: de::Error>(self, bytes: Vec<u8>) -> Result<Self::Value, E> {
        Ok(ByteString(bytes))
    }
}

struct OwnedTree(HashTree);
struct TreeVisitor;
impl<'de> Deserialize<'de> for OwnedTree {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        // Require an actual array instead of codec-specific sequence coercions.
        d.deserialize_any(TreeVisitor)
    }
}
impl<'de> Visitor<'de> for TreeVisitor {
    type Value = OwnedTree;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("IC hash-tree array")
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let tag: u64 = required(&mut seq)?;
        let tree = match tag {
            0 => empty(),
            1 => fork(
                required::<_, OwnedTree>(&mut seq)?.0,
                required::<_, OwnedTree>(&mut seq)?.0,
            ),
            2 => labeled(
                required::<_, ByteString>(&mut seq)?.0,
                required::<_, OwnedTree>(&mut seq)?.0,
            ),
            3 => leaf(required::<_, ByteString>(&mut seq)?.0),
            4 => pruned(
                <[u8; 32]>::try_from(required::<_, ByteString>(&mut seq)?.0)
                    .map_err(|_| de::Error::custom("pruned IC digest must have 32 bytes"))?,
            ),
            _ => return Err(de::Error::custom("unknown IC hash-tree tag")),
        };
        if seq.next_element::<IgnoredAny>()?.is_some() {
            return Err(de::Error::custom("extra IC hash-tree child"));
        }
        Ok(OwnedTree(tree))
    }
}
fn required<'de, A: SeqAccess<'de>, T: Deserialize<'de>>(seq: &mut A) -> Result<T, A::Error> {
    seq.next_element()?
        .ok_or_else(|| de::Error::custom("missing IC hash-tree element"))
}
