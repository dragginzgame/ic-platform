//! Upstream IC certificates, hash trees and witnesses with registry identities.
//!
//! Consumers retain decoding budgets, trust roots, signature verification,
//! freshness and application leaf meanings. Deserialization is not authentication.

pub use ic_certification::{
    AsHashTree, Certificate, Delegation, Hash, HashTree, Label, LookupResult, RbTree,
    SubtreeLookupResult, empty, fork, fork_hash, labeled, labeled_hash, leaf, pruned,
};
