//! Explicit upstream IC canister-signature verification.
//!
//! The result remains `Result<(), String>` with upstream parsing and diagnostics.
//! Callers retain byte/depth admission, protected issuer/seed/root inputs,
//! freshness and authority policy. Short public-key DER can panic upstream;
//! preserve pre-validation rather than treating this as a bounded decoder.
//! Verification selects imported creation, certification and BLS without Candid,
//! raw IC calls, an executor or host tooling. Success authenticates the supplied
//! message, not application authority.

pub use crate::signature_upstream::verification::verify_canister_sig;
