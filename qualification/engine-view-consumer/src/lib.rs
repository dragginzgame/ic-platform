//! Independent compile/execution fixture for the public response projection.

use ic_platform_protocol::cloud_engine::EngineView;
use ic_platform_types::Principal;

/// Exercise a public type crossing without byte conversion or a wrapper identity.
#[must_use]
pub fn selected_subnet(view: &EngineView) -> Option<Principal> {
    view.subnet_id
}

#[cfg(test)]
mod tests;
