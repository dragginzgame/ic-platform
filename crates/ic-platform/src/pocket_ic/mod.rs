//! Maintained native PocketIC client behind the Platform boundary.
//!
//! Testkit owns server provisioning, compatibility admission, lifecycle and retry
//! policy. Importing these types starts no server. Network operations return the
//! owner's ClientError with sources and accepted-operation evidence. Management
//! DTOs share the selected native management-canister 0.11 identities.

pub use ic_platform_pocket_ic::{
    CanisterInstallMode, CanisterLogRecord, CanisterSettings, CanisterStatusResult, ClientError,
    CreateCanisterParams, CreateCanisterPlacement, ErrorCode, LATEST_SERVER_VERSION, Operation,
    PocketIc, PocketIcBuilder, RejectCode, RejectResponse, RequestContext, Snapshot, Time,
};

pub mod common;
pub mod nonblocking;

#[cfg(test)]
mod tests;
