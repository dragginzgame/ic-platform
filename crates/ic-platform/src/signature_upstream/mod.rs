//! Upstream-authored cryptographic code, preserved without algorithm changes.
// Source/license preservation is checked by check-signature-source.sh.
#![allow(dead_code, missing_docs)]
#![allow(rustdoc::bare_urls)] // Preserve original upstream documentation bytes.
mod admission;
#[rustfmt::skip]
pub mod creation;
#[cfg(feature = "signature-verification")]
#[rustfmt::skip]
pub mod verification;
