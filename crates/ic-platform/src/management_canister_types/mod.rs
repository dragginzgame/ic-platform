//! Selected canister management DTOs, preserving external Candid bytes.
//!
//! Wasm builds use Platform-maintained definitions from 0.11.0 for observed sibling
//! canister calls. Native builds retain the complete existing upstream DTO surface
//! and identities for Backup, Testkit and host tooling. No transport, policy or
//! cryptographic implementation is owned here. See maintained/provenance.json.

#[cfg(any(target_family = "wasm", test))]
mod maintained;

#[cfg(target_family = "wasm")]
pub use maintained::{
    CanisterIdRecord, CanisterInfoArgs, CanisterInfoResult, CanisterInstallMode, CanisterSettings,
    CanisterStatusResult, CanisterStatusType, Change, ChangeDetails, ChangeOrigin, ChunkHash,
    CodeDeploymentMode, CodeDeploymentRecord, ControllersChangeRecord, CreationRecord,
    DefiniteCanisterSettings, DeleteCanisterSnapshotArgs, EcdsaCurve, EcdsaKeyId,
    EcdsaPublicKeyArgs, EcdsaPublicKeyResult, EnvironmentVariable, FromCanisterRecord,
    FromUserRecord, HttpHeader, HttpMethod, HttpRequestArgs, HttpRequestResult,
    InstallChunkedCodeArgs, LogVisibility, MemoryMetrics, QueryStats, SignWithEcdsaArgs,
    SignWithEcdsaResult, Snapshot, SnapshotSource, StatusVisibility, TransformArgs,
    TransformContext, TransformFunc, UninstallCodeArgs, UpdateSettingsArgs, UploadChunkArgs,
};

#[cfg(not(target_family = "wasm"))]
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

#[cfg(test)]
mod tests;
