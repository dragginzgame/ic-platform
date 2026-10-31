//! Stable owned Principal identity; required external IC identities remain explicit.
//!
//! ```compile_fail
//! ic_platform::candid::contract::encode_one(ic_platform::Principal::anonymous()).unwrap();
//! ```

#[cfg(feature = "principal-external")]
pub use ic_platform_types::principal::{ExternalPrincipal, ExternalPrincipalError};
pub use ic_platform_types::principal::{Principal, PrincipalError};
