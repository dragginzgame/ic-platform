//! Native PocketIC client identities at the upstream maintenance boundary.
//!
//! Testkit owns server provisioning, compatibility admission, lifecycle and retry
//! policy. These exports start no server and add no replacement client behavior.
//! PocketIC's management DTOs retain its upstream version; they are not aliases
//! for Platform's independently selected management-canister DTO surface.

pub use pocket_ic::{
    CanisterLogRecord, CanisterSettings, CanisterStatusResult, CreateCanisterParams,
    CreateCanisterPlacement, ErrorCode, LATEST_SERVER_VERSION, PocketIc, PocketIcBuilder,
    RejectCode, RejectResponse, Snapshot, Time,
};

pub mod common;
pub mod nonblocking;

#[cfg(test)]
mod tests;
