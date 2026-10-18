//! External management HTTP, history, snapshot and status types with upstream identities.
//!
//! The exact 0.11.0 dependency retains required Candid formats, including pricing
//! and replication fields. Consumers own bounds, cycles, transforms, calls and
//! retry policy. This feature brings transitive Candid without selecting the
//! Platform codec, host parser, Agent or executor features.
//! Snapshot DTOs do not execute operations or decide restore safety, authority,
//! storage allocation, response admission or effect/recovery policy.

pub use ic_management_canister_types::{
    CanisterIdRecord, CanisterInfoArgs, CanisterInfoResult, CanisterStatusResult,
    CanisterStatusType, CanisterTimer, ChangeDetails, ChunkHash, CodeDeploymentMode,
    DefiniteCanisterSettings, HttpHeader, HttpMethod, HttpRequestArgs, HttpRequestResult,
    LoadCanisterSnapshotArgs, MemoryMetrics, OnLowWasmMemoryHookStatus, QueryStats,
    ReadCanisterSnapshotDataArgs, ReadCanisterSnapshotDataResult, ReadCanisterSnapshotMetadataArgs,
    ReadCanisterSnapshotMetadataResult, Snapshot, SnapshotDataKind, SnapshotDataOffset,
    SnapshotMetadataGlobal, SnapshotSource, StatusVisibility, TakeCanisterSnapshotArgs,
    TransformArgs, TransformContext, TransformFunc, UploadCanisterSnapshotDataArgs,
    UploadCanisterSnapshotMetadataArgs, UploadCanisterSnapshotMetadataResult,
};
