//! Standard Ed25519 primitives with original Dalek type identities.
//!
//! Available on native and Wasm without RNG, Candid or runtime integration.
//! Consumers retain message/domain construction, canonical-scalar admission,
//! strict verification, key custody and application policy. No cryptography is copied.

pub use curve25519_dalek::scalar::Scalar;
pub use ed25519_dalek::{Signature, SignatureError, Signer, SigningKey, VerifyingKey};
