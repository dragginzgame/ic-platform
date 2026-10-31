//! Canonical owned IC identities, released independently of Platform runtime adapters.
//!
//! Bytes, bounded text, Serde and account semantics belong here. External Principal
//! conversions and unchanged upstream self-authentication require explicit features.

pub mod account;
pub mod principal;
pub use account::{Account, AccountError, Subaccount};
pub use principal::{Principal, PrincipalError};
