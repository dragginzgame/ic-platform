//! Maintained packaging of upstream BLS12-381 keys, signatures and verification.
//!
//! The optimized upstream allocation path is retained, matching the reviewed
//! IC verifier and Auth fixtures. No production RNG or entropy source is selected;
//! fixtures construct keys from caller-supplied bytes. Key custody, entropy,
//! message/domain selection and trust policy remain consumer-owned. This feature
//! selects neither Candid, canister-signature creation nor an executor.

#[rustfmt::skip]
#[cfg_attr(doc, expect(rustdoc::bare_urls, reason = "Preserve the original private curve design notes."))]
#[allow(deprecated, reason = "Digest 0.10 permits generic-array versions both before and after deprecation; an expectation fails with older consumer locks.")]
#[expect(dead_code, unused_imports, reason = "Retain complete upstream arithmetic and original tests behind the private module.")]
#[expect(clippy::needless_lifetimes, clippy::wrong_self_convention, clippy::needless_borrows_for_generic_args, clippy::manual_is_multiple_of, clippy::manual_div_ceil, clippy::needless_range_loop, clippy::needless_borrow, reason = "Keep imported arithmetic bodies byte-reconstructible, including explicit borrows and division spelling.")]
#[cfg_attr(test, expect(clippy::clone_on_copy, clippy::bool_assert_comparison, clippy::op_ref, reason = "Retain original upstream test bodies and assertions."))]
mod curve;
#[rustfmt::skip]
mod upstream;

pub use upstream::{
    InvalidPrivateKey, InvalidPublicKey, InvalidSignature, PrivateKey, PublicKey, Signature,
    verify_bls_signature,
};
