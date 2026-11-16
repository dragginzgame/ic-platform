//! IC certificate boundaries, separate from identity, execution and protocol codecs.
//!
//! Opt-in capabilities decode untrusted certificates and authenticate direct-root
//! or singly delegated certificates with explicit trust, authority and time inputs.
//! Defaults are empty.
#![forbid(unsafe_code)]

#[cfg(feature = "certificate-decode")]
pub mod certificate;
