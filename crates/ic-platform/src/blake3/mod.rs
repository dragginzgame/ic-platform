//! Upstream BLAKE3 for application hashes, without a wrapper or alternate algorithm.
//!
//! Consumers own domain separation and format policy. This does not replace
//! required IC hashes, signatures or existing signed/persisted hash contracts.
//! Only upstream `std` is selected; threading and memory mapping are not enabled.

pub use ::blake3::{Hash, Hasher, hash};
