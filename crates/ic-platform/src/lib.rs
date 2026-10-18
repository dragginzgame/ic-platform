//! Controlled dependencies for external Candid protocols and IC runtime integration.
//!
//! No capabilities are enabled by default. `candid` selects bounded decoding;
//! `executor` selects the shared CDK executor without a Candid dependency.
//! Consumers retain protocol schemas, resource limits and lifecycle policy.

#[cfg(feature = "candid")]
pub mod candid;
#[cfg(feature = "executor")]
pub mod executor;
