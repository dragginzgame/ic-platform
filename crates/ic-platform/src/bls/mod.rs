//! Explicit upstream BLS12-381 key, signature and verification identities.
//!
//! Upstream defaults are disabled and `alloc` is selected, matching the reviewed
//! IC verifier and Auth fixtures. No RNG feature or entropy source is selected;
//! fixtures construct keys from caller-supplied bytes. Key custody, entropy,
//! message/domain selection and trust policy remain consumer-owned. This feature
//! selects neither Candid, canister-signature creation nor an executor.

pub use ic_verify_bls_signature::{
    InvalidPrivateKey, InvalidPublicKey, InvalidSignature, PrivateKey, PublicKey, Signature,
    verify_bls_signature,
};
