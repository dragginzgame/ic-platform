//! Explicit upstream IC canister-signature verification.
//!
//! The original result remains `Result<(), String>` with upstream diagnostics.
//! [`verify_canister_sig_with_time`] additionally returns authenticated signing
//! time, with separate verification and time-format failure categories.
//! Callers retain byte/depth admission, protected issuer/seed/root inputs,
//! freshness and authority policy. Short public-key DER can panic upstream;
//! preserve pre-validation rather than treating this as a bounded decoder.
//! Verification selects imported creation, certification and BLS without Candid,
//! raw IC calls, an executor or host tooling. Success authenticates the supplied
//! message, not application authority.

pub use crate::signature_upstream::verification::verify_canister_sig;

pub use crate::signature_upstream::verification::evidence::{
    CanisterSigTimeError, verify_canister_sig_with_time,
};
