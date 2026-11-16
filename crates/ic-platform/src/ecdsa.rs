//! Upstream K256 identities for native and Wasm secp256k1 ECDSA.
//!
//! Callers supply the exact prehash and retain their high-S refusal policy.
//! `Signature::normalize_s` returns a signature; compare with the original to
//! reject high-S without repairing received bytes. SEC1 parsing and arithmetic
//! remain upstream. No RNG, Agent or executor is selected by this capability.

pub use k256::ecdsa::signature::Error as SignatureError;
pub use k256::ecdsa::signature::hazmat::{PrehashSigner, PrehashVerifier};
pub use k256::ecdsa::{Signature, SigningKey, VerifyingKey};
