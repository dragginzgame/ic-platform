//! Canonical owned IC identities, released independently of Platform runtime adapters.
//!
//! Bytes, bounded text, Serde and account semantics belong here. Unchanged upstream
//! self-authentication requires its explicit feature.

pub mod account;
pub mod principal;
pub use account::{Account, AccountError, Subaccount};
pub use principal::{Principal, PrincipalError};
