//! Explicit upstream canister-signature public-key formats and hashing.
//!
//! These retain upstream DER/raw encodings, error strings and domain separators.
//! Upstream 1.3.1 brings Candid for Principal and raw ic0 for its internal store;
//! this module does not expose that store or publish certification roots. Short
//! DER input can panic in the upstream parser: preserve consumer pre-admission,
//! canonical re-encoding and seed/issuer checks. Hashing does not authenticate.

pub use ic_canister_sig_creation::{
    CanisterSigPublicKey, IC_ROOT_PK_DER, IC_ROOT_PK_DER_PREFIX, extract_raw_root_pk_from_der,
    hash_bytes, hash_with_domain,
};
