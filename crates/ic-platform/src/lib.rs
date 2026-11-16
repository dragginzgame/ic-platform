//! IC certificate boundaries, separate from identity, execution and protocol codecs.
//!
//! Opt-in capabilities decode untrusted certificates and authenticate direct-root
//! or singly delegated certificates with explicit trust, authority and time inputs.
//! Defaults are empty.
#![forbid(unsafe_code)]

#[cfg(feature = "certificate-decode")]
pub mod certificate;

#[cfg(feature = "ed25519")]
pub mod ed25519;

#[cfg(feature = "ecdsa")]
pub mod ecdsa;

/// Upstream BLS signing types for explicitly selected native fixtures.
#[cfg(all(feature = "certificate-fixture-signing", not(target_family = "wasm")))]
pub mod fixture_signing {
    pub use ic_verify_bls_signature::PrivateKey;
}

#[cfg(all(feature = "native-read", not(target_family = "wasm")))]
pub mod native;

#[cfg(all(feature = "ingress-packets", not(target_family = "wasm")))]
pub mod ingress;
