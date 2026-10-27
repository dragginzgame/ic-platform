//! External management lifecycle, settings, signing, HTTP, history and snapshot contracts.
//!
//! The exact 0.11.0 dependency retains required Candid formats, including pricing
//! and replication fields. Consumers own bounds, cycles, transforms, calls and
//! retry policy. This feature brings transitive Candid without selecting the
//! Platform codec, host parser, Agent or executor features.
//! Snapshot DTOs do not execute operations or decide restore safety, authority,
//! storage allocation, response admission or effect/recovery policy.

pub use ic_management_canister_types::{
    CanisterIdRecord, CanisterInfoArgs, CanisterInfoResult, CanisterInstallMode, CanisterSettings,
    CanisterStatusResult, CanisterStatusType, CanisterTimer, Change, ChangeDetails, ChangeOrigin,
    ChunkHash, CodeDeploymentMode, CodeDeploymentRecord, ControllersChangeRecord, CreationRecord,
    DefiniteCanisterSettings, DeleteCanisterSnapshotArgs, EcdsaCurve, EcdsaKeyId,
    EcdsaPublicKeyArgs, EcdsaPublicKeyResult, EnvironmentVariable, FromCanisterRecord,
    FromUserRecord, HttpHeader, HttpMethod, HttpRequestArgs, HttpRequestResult,
    InstallChunkedCodeArgs, LoadCanisterSnapshotArgs, LogVisibility, MemoryMetrics,
    OnLowWasmMemoryHookStatus, QueryStats, ReadCanisterSnapshotDataArgs,
    ReadCanisterSnapshotDataResult, ReadCanisterSnapshotMetadataArgs,
    ReadCanisterSnapshotMetadataResult, SignWithEcdsaArgs, SignWithEcdsaResult, Snapshot,
    SnapshotDataKind, SnapshotDataOffset, SnapshotMetadataGlobal, SnapshotSource, StatusVisibility,
    TakeCanisterSnapshotArgs, TransformArgs, TransformContext, TransformFunc, UninstallCodeArgs,
    UpdateSettingsArgs, UploadCanisterSnapshotDataArgs, UploadCanisterSnapshotMetadataArgs,
    UploadCanisterSnapshotMetadataResult, UploadChunkArgs,
};

// Settings and install requests retain the complete upstream type. Consumers
// select permitted actions and preserve exact encoded bytes across retries.
