//! Maintained native PocketIC client behind the Platform boundary.
//!
//! Testkit owns server provisioning, compatibility admission, lifecycle and retry
//! policy. Importing these types starts no server. Existing operation signatures
//! and client behavior are retained by the owned implementation package.
//! PocketIC's management DTOs retain its upstream version; they are not aliases
//! for Platform's independently selected management-canister DTO surface.

pub use ic_platform_pocket_ic::{
    CanisterInstallMode, CanisterLogRecord, CanisterSettings, CanisterStatusResult,
    CreateCanisterParams, CreateCanisterPlacement, ErrorCode, LATEST_SERVER_VERSION, PocketIc,
    PocketIcBuilder, RejectCode, RejectResponse, Snapshot, Time,
};

pub mod common;
pub mod nonblocking;

#[cfg(test)]
mod tests;
