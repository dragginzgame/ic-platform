#![forbid(unsafe_code)]

//! Independent compile/execution fixture for the public response projection.

use candidate_types::Principal;
use ic_platform_protocol::cloud_engine::EngineView;

/// Exercise a public type crossing without byte conversion or a wrapper identity.
#[must_use]
pub fn selected_subnet(view: &EngineView) -> Option<Principal> {
    view.subnet_id
}

#[cfg(test)]
mod tests;

/// Independently declared schemas for native qualification only.
#[cfg(not(target_family = "wasm"))]
#[doc(hidden)]
pub mod fixtures;
