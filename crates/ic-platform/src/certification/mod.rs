//! Maintained IC certificate and hash-tree identities and operations.
//!
//! This capability is independent of Candid and the executor. Serde support is
//! retained for interoperability; it does not provide strict bounded decoding.
//! Host owns its CBOR decoder. Consumers own trust roots, signature verification,
//! resource budgets, freshness, witness admission and application leaf meanings.
//! Constructing or decoding a certificate does not authenticate it.

#[rustfmt::skip]
#[expect(dead_code, reason = "Retain complete upstream helpers behind the selected public surface.")]
pub(crate) mod upstream;

#[cfg(all(feature = "agent", not(target_family = "wasm")))]
mod external;

pub use upstream::hash_tree::{SubtreeLookupResult, labeled_hash};
pub use upstream::{
    AsHashTree, Certificate, Delegation, Hash, HashTree, Label, LookupResult, RbTree, empty, fork,
    fork_hash, labeled, leaf, pruned,
};
