//! Canonical owned IC identities, versioned together with the Platform workspace.
//!
//! Bytes, bounded text, Serde and account semantics belong here. External Principal
//! conversions and unchanged upstream self-authentication require explicit features.

pub mod account;
pub mod principal;
pub use account::{Account, AccountError, Subaccount};
pub use principal::{Principal, PrincipalError};
