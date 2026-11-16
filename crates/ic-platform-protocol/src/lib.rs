//! Bounded external protocol formats, independent of transport and execution.
//!
//! Capabilities are explicit; default features are empty. The initial
//! `cloud-engine` capability encodes requests only. `cloud-engine-response` adds
//! bounded selected response decoding and canonical identities. `http-request`
//! encodes a narrow management HTTP GET argument without a codec dependency.
//! This package does not replace the complete legacy Platform surface.

#![forbid(unsafe_code)]

#[cfg(feature = "caffeine")]
pub mod caffeine;

#[cfg(feature = "cloud-engine")]
pub mod cloud_engine;

#[cfg(any(
    feature = "cloud-engine-response",
    feature = "management-chunks",
    feature = "management-snapshots",
    feature = "caffeine"
))]
pub mod candid;

#[cfg(feature = "management-snapshots")]
pub mod management_snapshots;

#[cfg(feature = "management-chunks")]
pub mod management_chunks;

#[cfg(feature = "registry-settings")]
pub mod registry_settings;

#[cfg(feature = "http-request")]
pub mod http_request;

#[cfg(feature = "registry")]
pub mod registry;

#[cfg(feature = "registry-placement")]
pub mod placement;

#[cfg(feature = "raw-rand")]
pub mod raw_rand;
