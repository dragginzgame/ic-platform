//! Passive external ICRC-10 discovery contract, without endpoint mounting or policy.
//!
//! Platform's Rust representation follows the named `name`/`url` record in
//! ICRC revision `1df0c6af6ecca4f4eb1956c67b29c0d13279f1cd`,
//! `ICRCs/ICRC-10/ICRC-10.did`. It is not an upstream Rust type identity.
//! Consumers select supported standards, preserve their names/URLs and own all
//! validation, admission and endpoint mounting. No discovery or consent is automatic.

/// One ICRC-10 discovery result, encoded as a named Candid record, not a tuple.
#[derive(Clone, Debug, Eq, PartialEq, candid::CandidType, serde::Deserialize)]
pub struct SupportedStandard {
    /// External standard name, without normalization or an allowed-name policy.
    pub name: String,
    /// External standard URL, preserved without parsing or validation.
    pub url: String,
}
