//! Selected canister management DTOs, preserving external Candid bytes.
//!
//! All targets use owned definitions and the owned Candid codec. Native snapshot
//! transfer declarations retain their existing target scope. No transport, storage
//! or cryptographic implementation is owned here.

mod maintained;

pub use maintained::{
    CanisterIdRecord, CanisterInfoArgs, CanisterInfoResult, CanisterInstallMode, CanisterSettings,
    CanisterStatusResult, CanisterStatusType, Change, ChangeDetails, ChangeOrigin, ChunkHash,
    CodeDeploymentMode, CodeDeploymentRecord, ControllersChangeRecord, CreateCanisterArgs,
    CreationRecord, DefiniteCanisterSettings, DeleteCanisterSnapshotArgs, EcdsaCurve, EcdsaKeyId,
    EcdsaPublicKeyArgs, EcdsaPublicKeyResult, EnvironmentVariable, FromCanisterRecord,
    FromUserRecord, HttpHeader, HttpMethod, HttpRequestArgs, HttpRequestResult,
    InstallChunkedCodeArgs, LogVisibility, MemoryMetrics, QueryStats, SignWithEcdsaArgs,
    SignWithEcdsaResult, Snapshot, SnapshotSource, StatusVisibility, TransformArgs,
    TransformContext, TransformFunc, UninstallCodeArgs, UpdateSettingsArgs, UploadChunkArgs,
};

#[cfg(not(target_family = "wasm"))]
mod native;
#[cfg(not(target_family = "wasm"))]
pub use native::{
    CanisterTimer, LoadCanisterSnapshotArgs, OnLowWasmMemoryHookStatus,
    ReadCanisterSnapshotDataArgs, ReadCanisterSnapshotDataResult, ReadCanisterSnapshotMetadataArgs,
    ReadCanisterSnapshotMetadataResult, SnapshotDataKind, SnapshotDataOffset,
    SnapshotMetadataGlobal, TakeCanisterSnapshotArgs, UploadCanisterSnapshotDataArgs,
    UploadCanisterSnapshotMetadataArgs, UploadCanisterSnapshotMetadataResult,
};

#[cfg(test)]
mod tests;
