//! Controlled dependencies for external Candid protocols and IC runtime integration.
//!
//! [`Principal`] is always available. `candid` selects bounded decoding;
//! `executor` selects the shared CDK executor without a Candid dependency.
//! Consumers retain protocol schemas, resource limits and lifecycle policy.

pub use ic_principal::Principal;

#[cfg(feature = "candid")]
pub mod candid;
#[cfg(feature = "certification")]
pub mod certification;
#[cfg(feature = "executor")]
pub mod executor;
