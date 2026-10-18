//! Controlled dependencies for external Candid protocols and IC runtime integration.
//!
//! [`Principal`] is always available. `candid` selects bounded decoding;
//! `executor` selects the shared CDK executor without a Candid dependency.
//! `candid-parser` selects native host parsing and binding generation only.
//! `agent` selects native IC transport and identities, with consumer-owned policy.
//! `ic0` selects qualified raw system calls without Candid or an executor.
//! `management-canister-types` selects external management HTTP/history/snapshot/status DTOs.
//! `icrc-ledger-types` selects external ledger DTOs without storage integration.
//! Consumers retain protocol schemas, resource limits and lifecycle policy.

pub use ic_principal::{Principal, PrincipalError};

#[cfg(all(feature = "agent", not(target_family = "wasm")))]
pub mod agent;

#[cfg(feature = "candid")]
pub mod candid;
#[cfg(all(feature = "candid-parser", not(target_family = "wasm")))]
pub mod candid_parser;
#[cfg(feature = "certification")]
pub mod certification;
#[cfg(feature = "executor")]
pub mod executor;
#[cfg(feature = "ic0")]
pub mod ic0;
#[cfg(feature = "icrc-ledger-types")]
pub mod icrc_ledger_types;
#[cfg(feature = "management-canister-types")]
pub mod management_canister_types;
