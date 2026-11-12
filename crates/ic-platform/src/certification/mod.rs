//! Upstream IC certificates, hash trees and witnesses with registry identities.
//!
//! Consumers select decoding budgets, trust roots, authentication calls,
//! freshness and application leaf meanings. The optional `certification-cbor`
//! capability supplies byte-bounded encoding/decoding through `cbor`.
//! Deserialization is not authentication. The native `verify` module composes
//! strict decoding and Agent authentication only when both capabilities are selected.

#[cfg(feature = "certification-cbor")]
pub mod cbor;

pub use ic_certification::{
    AsHashTree, Certificate, Delegation, Hash, HashTree, Label, LookupResult, RbTree,
    SubtreeLookupResult, empty, fork, fork_hash, labeled, labeled_hash, leaf, pruned,
};

#[cfg(all(
    feature = "agent",
    feature = "certification-cbor",
    not(target_family = "wasm")
))]
pub mod verify;
