//! Canonical IC runtime, independently released from crypto and certificate adapters.
//! Select `ic0`, `executor`, `runtime` or `cdk` explicitly; defaults are empty.
//! Runtime calls accept raw bytes. Codecs, storage and scheduling remain consumer-owned.

// This link-only crate carries the executor's Cargo singleton identity and has
// deliberately no callable API. Keep its use conditional on executor selection.
#[cfg(feature = "executor")]
use ic_platform_runtime_guard as _;

#[cfg(feature = "runtime")]
pub use ic_platform_types::{Principal, PrincipalError};
#[cfg(feature = "runtime")]
pub mod call;
#[cfg(feature = "runtime")]
pub mod cdk;
#[cfg(feature = "executor")]
pub mod executor;
#[cfg(feature = "ic0")]
pub mod ic0;
