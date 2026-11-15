//! Bounded external protocol formats, independent of transport and execution.
//!
//! Capabilities are explicit; default features are empty. The initial
//! `cloud-engine` capability encodes requests only. `cloud-engine-response` adds
//! bounded selected response decoding and canonical identities. This unpublished
//! package is not yet a qualified replacement for Platform.

#![forbid(unsafe_code)]

#[cfg(feature = "cloud-engine")]
pub mod cloud_engine;

#[cfg(feature = "cloud-engine-response")]
pub mod candid;

#[cfg(feature = "registry-settings")]
pub mod registry_settings;
