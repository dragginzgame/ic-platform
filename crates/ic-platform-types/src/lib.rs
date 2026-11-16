//! Canonical owned IC Principal, Account and Subaccount identities.
//!
//! Byte/text admission, ordering and Serde belong here. Self-authentication is
//! opt-in and delegates SHA-224 to its upstream supplier. Consumers must select
//! one Cargo source/version cohort when exchanging these public Rust types.
//! Identity validity establishes neither authorization nor authenticated evidence.

#![forbid(unsafe_code)]

pub mod account;
pub mod principal;
pub use account::{Account, AccountError, DEFAULT_SUBACCOUNT, Subaccount};
pub use principal::{Principal, PrincipalError};
