//! Existing native snapshot contracts maintained from DFINITY 0.11.0 (Apache-2.0).
//! See provenance.json; declarations preserve upstream fields, variants and Serde.
use super::maintained::{CanisterId, ChunkHash, SnapshotId, SnapshotSource};
use crate::candid_codec::Nat;
use serde::{Deserialize, Serialize};
mod client;
mod codec;
pub use client::{
    CanisterLogRecord, FetchCanisterLogsResult, InstallCodeArgs,
    ProvisionalCreateCanisterWithCyclesArgs,
};

/// # Take Canister Snapshot Args.
///
/// Argument type of [`take_canister_snapshot`](https://docs.internetcomputer.org/references/management-canister/#take_canister_snapshot).
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct TakeCanisterSnapshotArgs {
    /// Canister ID.
    pub canister_id: CanisterId,
    /// An optional snapshot ID to be replaced by the new snapshot.
    ///
    /// The snapshot identified by the specified ID will be deleted once a new snapshot has been successfully created.
    pub replace_snapshot: Option<SnapshotId>,
    /// If true, uninstall the canister code after taking the snapshot.
    pub uninstall_code: Option<bool>,
    /// Must match the canister's [`canister_version`](https://docs.internetcomputer.org/references/ic-interface-spec/canister-interface/#system-api-canister-version) value when specified.
    pub sender_canister_version: Option<u64>,
}

/// # Load Canister Snapshot Args.
///
/// Argument type of [`load_canister_snapshot`](https://docs.internetcomputer.org/references/management-canister/#load_canister_snapshot).
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct LoadCanisterSnapshotArgs {
    /// Canister ID.
    pub canister_id: CanisterId,
    /// ID of the snapshot to be loaded.
    pub snapshot_id: SnapshotId,
    /// Must match the canister's [`canister_version`](https://docs.internetcomputer.org/references/ic-interface-spec/canister-interface/#system-api-canister-version) value when specified.
    pub sender_canister_version: Option<u64>,
}

/// # Read Canister Snapshot Metadata Args.
///
/// Argument type of [`read_canister_snapshot_metadata`](https://docs.internetcomputer.org/references/management-canister/#read_canister_snapshot_metadata).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReadCanisterSnapshotMetadataArgs {
    /// Canister ID.
    pub canister_id: CanisterId,
    /// ID of the snapshot to be read.
    pub snapshot_id: SnapshotId,
}

/// # Read Canister Snapshot Metadata Result.
///
/// Result type of [`read_canister_snapshot_metadata`](https://docs.internetcomputer.org/references/management-canister/#read_canister_snapshot_metadata).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReadCanisterSnapshotMetadataResult {
    /// How the snapshot was created.
    pub source: Option<SnapshotSource>,
    /// The Unix nanosecond timestamp the snapshot was taken at.
    pub taken_at_timestamp: u64,
    /// The size of the Wasm module in bytes.
    pub wasm_module_size: u64,
    /// The exported WebAssembly global variables of the canister.
    pub globals: Vec<Option<SnapshotMetadataGlobal>>,
    /// The size of the Wasm heap memory in bytes.
    pub wasm_memory_size: u64,
    /// The size of the stable memory in bytes.
    pub stable_memory_size: u64,
    /// Hashes of the chunks in the Wasm chunk store.
    pub wasm_chunk_store: StoredChunksResult,
    /// The version of the canister at the time the snapshot was taken.
    pub canister_version: u64,
    /// The certified data of the canister.
    #[serde(with = "serde_bytes")]
    pub certified_data: Vec<u8>,
    /// The status of the canister's global timer.
    pub global_timer: Option<CanisterTimer>,
    /// The status of the on-low-Wasm-memory hook.
    pub on_low_wasm_memory_hook_status: Option<OnLowWasmMemoryHookStatus>,
}

/// # Snapshot Metadata Global.
///
/// An exported WebAssembly global variable of a canister snapshot.
///
/// See [`ReadCanisterSnapshotMetadataResult::globals`].
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SnapshotMetadataGlobal {
    /// A 32-bit integer.
    #[serde(rename = "i32")]
    I32(i32),
    /// A 64-bit integer.
    #[serde(rename = "i64")]
    I64(i64),
    /// A 32-bit floating point number.
    #[serde(rename = "f32")]
    F32(f32),
    /// A 64-bit floating point number.
    #[serde(rename = "f64")]
    F64(f64),
    /// A 128-bit integer.
    #[serde(rename = "v128")]
    V128(Nat),
}

/// # Canister Timer.
///
/// The status of a canister's global timer.
///
/// See [`ReadCanisterSnapshotMetadataResult::global_timer`].
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CanisterTimer {
    /// The global timer is not set.
    #[serde(rename = "inactive")]
    Inactive,
    /// The global timer is set to fire at the given Unix nanosecond timestamp.
    #[serde(rename = "active")]
    Active(u64),
}

/// # On Low Wasm Memory Hook Status.
///
/// The execution status of the on-low-Wasm-memory hook.
///
/// See [`ReadCanisterSnapshotMetadataResult::on_low_wasm_memory_hook_status`].
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OnLowWasmMemoryHookStatus {
    /// The hook's trigger condition is not satisfied.
    #[serde(rename = "condition_not_satisfied")]
    ConditionNotSatisfied,
    /// The hook is scheduled to run.
    #[serde(rename = "ready")]
    Ready,
    /// The hook has already been executed.
    #[serde(rename = "executed")]
    Executed,
}

/// # Read Canister Snapshot Data Args.
///
/// Argument type of [`read_canister_snapshot_data`](https://docs.internetcomputer.org/references/management-canister/#read_canister_snapshot_data).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReadCanisterSnapshotDataArgs {
    /// Canister ID.
    pub canister_id: CanisterId,
    /// ID of the snapshot.
    pub snapshot_id: SnapshotId,
    /// The region of the snapshot to read.
    pub kind: SnapshotDataKind,
}

/// # Snapshot Data Kind.
///
/// Identifies a region within a snapshot to read.
///
/// See [`ReadCanisterSnapshotDataArgs::kind`].
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SnapshotDataKind {
    /// Wasm module.
    #[serde(rename = "wasm_module")]
    WasmModule {
        /// Offset in bytes.
        offset: u64,
        /// Size of the data in bytes.
        size: u64,
    },
    /// Wasm memory.
    #[serde(rename = "wasm_memory")]
    WasmMemory {
        /// Offset in bytes.
        offset: u64,
        /// Size of the data in bytes.
        size: u64,
    },
    /// Stable memory.
    #[serde(rename = "stable_memory")]
    StableMemory {
        /// Offset in bytes.
        offset: u64,
        /// Size of the data in bytes.
        size: u64,
    },
    /// Chunk hash.
    #[serde(rename = "wasm_chunk")]
    WasmChunk {
        /// The hash of the chunk.
        #[serde(with = "serde_bytes")]
        hash: Vec<u8>,
    },
}

/// # Read Canister Snapshot Data Result.
///
/// Result type of [`read_canister_snapshot_data`](https://docs.internetcomputer.org/references/management-canister/#read_canister_snapshot_data).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReadCanisterSnapshotDataResult {
    /// The requested chunk of snapshot data.
    #[serde(with = "serde_bytes")]
    pub chunk: Vec<u8>,
}

/// # Upload Canister Snapshot Metadata Args.
///
/// Argument type of [`upload_canister_snapshot_metadata`](https://docs.internetcomputer.org/references/management-canister/#upload_canister_snapshot_metadata).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UploadCanisterSnapshotMetadataArgs {
    /// Canister ID.
    pub canister_id: CanisterId,
    /// An optional snapshot ID to be replaced by the new snapshot.
    ///
    /// The snapshot identified by the specified ID will be deleted once a new snapshot has been successfully created.
    pub replace_snapshot: Option<SnapshotId>,
    /// The size of the Wasm module.
    pub wasm_module_size: u64,
    /// The globals.
    pub globals: Vec<SnapshotMetadataGlobal>,
    /// The size of the Wasm memory.
    pub wasm_memory_size: u64,
    /// The size of the stable memory.
    pub stable_memory_size: u64,
    /// The certified data.
    pub certified_data: Vec<u8>,
    /// The status of the global timer.
    pub global_timer: Option<CanisterTimer>,
    /// The status of the low wasm memory hook.
    pub on_low_wasm_memory_hook_status: Option<OnLowWasmMemoryHookStatus>,
}

/// # Upload Canister Snapshot Metadata Result.
///
/// Result type of [`upload_canister_snapshot_metadata`](https://docs.internetcomputer.org/references/management-canister/#upload_canister_snapshot_metadata).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UploadCanisterSnapshotMetadataResult {
    /// The ID of the snapshot.
    pub snapshot_id: SnapshotId,
}

/// # Upload Canister Snapshot Data Args.
///
/// Argument type of [`upload_canister_snapshot_data`](https://docs.internetcomputer.org/references/management-canister/#upload_canister_snapshot_data).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UploadCanisterSnapshotDataArgs {
    /// Canister ID.
    pub canister_id: CanisterId,
    /// ID of the snapshot.
    pub snapshot_id: SnapshotId,
    /// The region of the snapshot to write.
    pub kind: SnapshotDataOffset,
    /// The chunk of data to upload.
    pub chunk: Vec<u8>,
}

/// # Snapshot Data Offset.
///
/// Identifies a region within a snapshot to write.
///
/// See [`UploadCanisterSnapshotDataArgs::kind`].
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SnapshotDataOffset {
    /// Wasm module.
    #[serde(rename = "wasm_module")]
    WasmModule {
        /// Offset in bytes.
        offset: u64,
    },
    /// Wasm memory.
    #[serde(rename = "wasm_memory")]
    WasmMemory {
        /// Offset in bytes.
        offset: u64,
    },
    /// Stable memory.
    #[serde(rename = "stable_memory")]
    StableMemory {
        /// Offset in bytes.
        offset: u64,
    },
    /// Wasm chunk.
    #[serde(rename = "wasm_chunk")]
    WasmChunk,
}

/// # Stored Chunks Result
///
/// Result type of [`stored_chunks`](https://docs.internetcomputer.org/references/management-canister/#stored_chunks).
pub type StoredChunksResult = Vec<ChunkHash>;

#[cfg(test)]
mod tests;
