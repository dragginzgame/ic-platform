//! Controlled dependencies for external Candid protocols and IC runtime integration.
//!
//! [`Principal`] is always available. `candid` selects bounded decoding;
//! `executor` selects the shared CDK executor without a Candid dependency.
//! `candid-parser` selects native host parsing and binding generation only.
//! Consumers retain protocol schemas, resource limits and lifecycle policy.

pub use ic_principal::Principal;

#[cfg(feature = "candid")]
pub mod candid;
#[cfg(all(feature = "candid-parser", not(target_family = "wasm")))]
pub mod candid_parser;
#[cfg(feature = "certification")]
pub mod certification;
#[cfg(feature = "executor")]
pub mod executor;
