//! Canonical identity ownership for the local Types cohort.
//!
//! This identity slice is unpublished and not yet a replacement for
//! published Types across the full production cohort. Self-authentication is
//! opt-in; actual production consumer migration remains pending.
//! Identity validity establishes neither authorization nor authenticated evidence.

pub mod account;
pub mod principal;
pub use account::{Account, AccountError, DEFAULT_SUBACCOUNT, Subaccount};
pub use principal::{Principal, PrincipalError};
