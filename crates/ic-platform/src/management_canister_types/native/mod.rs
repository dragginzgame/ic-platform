//! Native management request and response declarations.
//!
//! These records preserve external fields; snapshot handling, limits and
//! orchestration remain with consumers and the IC.

use super::maintained::{CanisterId, ChunkHash, SnapshotId, SnapshotSource};
use crate::candid_codec::Nat;
use serde::{Deserialize, Serialize};

mod client;
mod codec;

pub use client::{
    CanisterLogRecord, FetchCanisterLogsResult, InstallCodeArgs,
    ProvisionalCreateCanisterWithCyclesArgs,
};

/// A request to take a canister snapshot.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct TakeCanisterSnapshotArgs {
    /// The canister identified by the request.
    pub canister_id: CanisterId,
    /// An optional existing snapshot identifier to replace.
    pub replace_snapshot: Option<SnapshotId>,
    /// The optional request to uninstall the canister's code.
    pub uninstall_code: Option<bool>,
    /// The sender's optional canister-version value.
    pub sender_canister_version: Option<u64>,
}

/// A request to load an identified snapshot into a canister.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct LoadCanisterSnapshotArgs {
    /// The destination canister.
    pub canister_id: CanisterId,
    /// The snapshot identifier preserved as ordinary sequence bytes.
    pub snapshot_id: SnapshotId,
    /// The sender's optional canister-version value.
    pub sender_canister_version: Option<u64>,
}

/// Identifies the snapshot whose metadata is requested.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReadCanisterSnapshotMetadataArgs {
    /// The canister holding the snapshot.
    pub canister_id: CanisterId,
    /// The requested snapshot identifier.
    pub snapshot_id: SnapshotId,
}

/// Snapshot metadata reported by the IC.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReadCanisterSnapshotMetadataResult {
    /// The optional reported origin of the snapshot.
    pub source: Option<SnapshotSource>,
    /// The recorded snapshot timestamp.
    pub taken_at_timestamp: u64,
    /// The reported Wasm module size.
    pub wasm_module_size: u64,
    /// Reported global values, retaining absent entries.
    pub globals: Vec<Option<SnapshotMetadataGlobal>>,
    /// The reported Wasm memory size.
    pub wasm_memory_size: u64,
    /// The reported stable-memory size.
    pub stable_memory_size: u64,
    /// The reported stored Wasm chunk hashes.
    pub wasm_chunk_store: StoredChunksResult,
    /// The recorded canister version.
    pub canister_version: u64,
    /// Certified data using the response's byte-buffer Serde representation.
    #[serde(with = "serde_bytes")]
    pub certified_data: Vec<u8>,
    /// The optional recorded global-timer state.
    pub global_timer: Option<CanisterTimer>,
    /// The optional recorded low-memory-hook state.
    pub on_low_wasm_memory_hook_status: Option<OnLowWasmMemoryHookStatus>,
}

/// A global value in snapshot metadata.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SnapshotMetadataGlobal {
    /// A signed 32-bit integer.
    #[serde(rename = "i32")]
    I32(i32),
    /// A signed 64-bit integer.
    #[serde(rename = "i64")]
    I64(i64),
    /// A single-precision floating value.
    #[serde(rename = "f32")]
    F32(f32),
    /// A double-precision floating value.
    #[serde(rename = "f64")]
    F64(f64),
    /// A vector global represented by the existing unbounded natural number.
    #[serde(rename = "v128")]
    V128(Nat),
}

/// A reported global-timer state, without scheduling behavior.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CanisterTimer {
    /// No timer is active in the reported state.
    #[serde(rename = "inactive")]
    Inactive,
    /// The reported active timer value.
    #[serde(rename = "active")]
    Active(u64),
}

/// The reported state of a canister's low-Wasm-memory hook.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OnLowWasmMemoryHookStatus {
    /// The activation condition has not been satisfied.
    #[serde(rename = "condition_not_satisfied")]
    ConditionNotSatisfied,
    /// The hook is reported as ready.
    #[serde(rename = "ready")]
    Ready,
    /// The hook is reported as executed.
    #[serde(rename = "executed")]
    Executed,
}

/// Selects a portion of an identified snapshot to read.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReadCanisterSnapshotDataArgs {
    /// The canister holding the snapshot.
    pub canister_id: CanisterId,
    /// The snapshot identifier.
    pub snapshot_id: SnapshotId,
    /// The requested data region or chunk.
    pub kind: SnapshotDataKind,
}

/// A snapshot region or stored chunk selected for reading.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SnapshotDataKind {
    /// A range of the Wasm module.
    #[serde(rename = "wasm_module")]
    WasmModule {
        /// The supplied range offset.
        offset: u64,
        /// The supplied range size.
        size: u64,
    },
    /// A range of Wasm memory.
    #[serde(rename = "wasm_memory")]
    WasmMemory {
        /// The supplied range offset.
        offset: u64,
        /// The supplied range size.
        size: u64,
    },
    /// A range of stable memory.
    #[serde(rename = "stable_memory")]
    StableMemory {
        /// The supplied range offset.
        offset: u64,
        /// The supplied range size.
        size: u64,
    },
    /// A Wasm chunk identified by opaque hash bytes.
    #[serde(rename = "wasm_chunk")]
    WasmChunk {
        /// The chunk hash using byte-buffer Serde.
        #[serde(with = "serde_bytes")]
        hash: Vec<u8>,
    },
}

/// The bytes returned for a snapshot-data read.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReadCanisterSnapshotDataResult {
    /// The returned chunk using byte-buffer Serde.
    #[serde(with = "serde_bytes")]
    pub chunk: Vec<u8>,
}

/// Metadata supplied when uploading a snapshot.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UploadCanisterSnapshotMetadataArgs {
    /// The canister receiving the uploaded snapshot.
    pub canister_id: CanisterId,
    /// An optional existing snapshot identifier to replace.
    pub replace_snapshot: Option<SnapshotId>,
    /// The supplied Wasm module size.
    pub wasm_module_size: u64,
    /// The supplied global values.
    pub globals: Vec<SnapshotMetadataGlobal>,
    /// The supplied Wasm memory size.
    pub wasm_memory_size: u64,
    /// The supplied stable-memory size.
    pub stable_memory_size: u64,
    /// Certified data using the upload's ordinary sequence Serde representation.
    pub certified_data: Vec<u8>,
    /// The optional supplied global-timer state.
    pub global_timer: Option<CanisterTimer>,
    /// The optional supplied low-memory-hook state.
    pub on_low_wasm_memory_hook_status: Option<OnLowWasmMemoryHookStatus>,
}

/// The identifier returned after snapshot metadata is uploaded.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UploadCanisterSnapshotMetadataResult {
    /// The assigned snapshot identifier.
    pub snapshot_id: SnapshotId,
}

/// A snapshot-data upload request.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UploadCanisterSnapshotDataArgs {
    /// The canister receiving the data.
    pub canister_id: CanisterId,
    /// The snapshot identifier.
    pub snapshot_id: SnapshotId,
    /// The destination region or chunk selection.
    pub kind: SnapshotDataOffset,
    /// The uploaded bytes using ordinary sequence Serde.
    pub chunk: Vec<u8>,
}

/// A destination within snapshot data being uploaded.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SnapshotDataOffset {
    /// A position in the Wasm module.
    #[serde(rename = "wasm_module")]
    WasmModule {
        /// The supplied destination offset.
        offset: u64,
    },
    /// A position in Wasm memory.
    #[serde(rename = "wasm_memory")]
    WasmMemory {
        /// The supplied destination offset.
        offset: u64,
    },
    /// A position in stable memory.
    #[serde(rename = "stable_memory")]
    StableMemory {
        /// The supplied destination offset.
        offset: u64,
    },
    /// A complete Wasm chunk.
    #[serde(rename = "wasm_chunk")]
    WasmChunk,
}

/// Chunk hashes reported for the snapshot's Wasm chunk store.
pub type StoredChunksResult = Vec<ChunkHash>;

#[cfg(test)]
mod tests;
