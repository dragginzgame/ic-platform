//! Bounded host gzip codecs maintained by IC Tooling.
//!
//! Consumers select compression levels and input/output limits. IC Tooling owns
//! the codec implementation; Platform exposes no compression backend types and
//! adds no admission policy, filesystem access or publication effects.

pub use ic_host_artifacts::artifact::{GzipError, decode_gzip, encode_gzip};
