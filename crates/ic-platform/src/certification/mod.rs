//! Explicit upstream IC certificate and hash-tree identities and operations.
//!
//! This capability is independent of Candid and the executor. Serde support is
//! retained for interoperability; it does not provide strict bounded decoding.
//! Host owns its CBOR decoder. Consumers own trust roots, signature verification,
//! resource budgets, freshness, witness admission and application leaf meanings.
//! Constructing or decoding a certificate does not authenticate it.

pub use ic_certification::hash_tree::{SubtreeLookupResult, labeled_hash};
pub use ic_certification::{
    Certificate, Delegation, HashTree, Label, LookupResult, empty, fork, fork_hash, labeled, leaf,
    pruned,
};
