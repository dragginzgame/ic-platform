//! Native/Wasm secp256k1 ECDSA primitives with original K256 identities.
//!
//! No RNG, runtime or Candid feature is selected. Consumers own prehash/domain
//! construction, high-S rejection, explicit normalization and key custody.
//! This is independent of the native IC public-derivation `secp256k1` surface.

pub use k256::ecdsa::signature::Error as SignatureError;
pub use k256::ecdsa::signature::hazmat::{PrehashSigner, PrehashVerifier};
pub use k256::ecdsa::{Signature, SigningKey, VerifyingKey};
