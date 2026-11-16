//! Upstream Dalek identities for native and Wasm Ed25519 verification.
//!
//! Use `Scalar::from_canonical_bytes` before charging verification work, reject
//! weak keys and use `VerifyingKey::verify_strict`. Ordinary Ed25519 over a digest
//! is distinct from Ed25519ph. Callers own exact message bytes, enrollment and
//! budgets. These primitives do not impose those application policies.
//! Signing is available for consumer-owned fixtures; no RNG is selected here.

pub use curve25519_dalek::scalar::Scalar;
pub use ed25519_dalek::{Signature, SignatureError, Signer, SigningKey, VerifyingKey};
