//! IC certificate boundaries, separate from identity, execution and protocol codecs.
//!
//! Opt-in capabilities decode untrusted certificates and authenticate direct-root
//! or singly delegated certificates with explicit trust, authority and time inputs.
//! Defaults are empty.
#![forbid(unsafe_code)]

#[cfg(feature = "certificate-decode")]
pub mod certificate;
#[cfg(feature = "ecdsa")]
pub mod ecdsa;
#[cfg(feature = "ed25519")]
pub mod ed25519;
/// Upstream BLS signing types for explicitly selected fixtures.
#[cfg(feature = "certificate-fixture-signing")]
pub mod fixture_signing {
    pub use ic_verify_bls_signature::PrivateKey;
}
#[cfg(all(feature = "ingress-packets", not(target_family = "wasm")))]
pub mod ingress;
#[cfg(all(feature = "native-read", not(target_family = "wasm")))]
pub mod native;
