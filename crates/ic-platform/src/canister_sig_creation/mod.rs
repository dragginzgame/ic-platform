//! Explicit upstream canister-signature public-key formats and hashing.
//!
//! These retain upstream-authored DER/raw encodings, error strings and domain separators.
//! The imported 1.3.1 implementation omits the unexposed store and Candid import;
//! this module does not expose that store or publish certification roots. Short
//! DER input can panic in the upstream parser: preserve consumer pre-admission,
//! canonical re-encoding and seed/issuer checks. Hashing does not authenticate.

pub use crate::signature_upstream::creation::{
    CanisterSigPublicKey, IC_ROOT_PK_DER, IC_ROOT_PK_DER_PREFIX, extract_raw_root_pk_from_der,
    hash_bytes, hash_with_domain,
};
