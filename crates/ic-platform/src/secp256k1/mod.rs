//! Native offline secp256k1 public-key derivation from exactly upstream 0.3.0.
//!
//! Consumers retain network/key selection, path validation and authority policy.
//! No private-key owner, runtime calls or replacement cryptography is introduced.
//! Upstream unconditionally selects RNG dependencies; this surface is host-only.

pub use ic_secp256k1::{KeyDecodingError, MasterPublicKeyId, PocketIcMasterPublicKeyId, PublicKey};
