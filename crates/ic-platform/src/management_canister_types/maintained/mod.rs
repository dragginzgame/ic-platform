//! Shared management request, response and event declarations.
//!
//! These types carry protocol fields without applying canister or caller policy.

use crate::candid_codec::{Nat, Reserved};
use crate::principal::Principal;
use serde::{Deserialize, Serialize};

mod codec;
mod transform_func;

pub use transform_func::TransformFunc;

/// A principal identifying a canister.
pub type CanisterId = Principal;

/// An opaque snapshot identifier.
pub type SnapshotId = Vec<u8>;

crate::candid_record! {
    /// A request identifying one canister.
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
    pub struct CanisterIdRecord {
        /// The canister addressed by the request.
        1313628723 => pub canister_id: CanisterId,
    }
}

crate::candid_record! {
    /// An opaque hash identifying a stored chunk.
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
    pub struct ChunkHash {
        /// The hash bytes.
        #[serde(with = "serde_bytes")]
        1158164430 => pub hash: Vec<u8>,
    }
}

crate::candid_variant! {
    /// The requested visibility of canister logs.
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default)]
    pub enum LogVisibility {
        /// Access for the canister's controllers.
        #[default]
        #[serde(rename = "controllers")]
        570880087 => Controllers,
        /// Public access.
        #[serde(rename = "public")]
        1727234217 => Public,
        /// Access for the listed principals.
        #[serde(rename = "allowed_viewers")]
        2235716810 => AllowedViewers(Vec<Principal>),
    }
}

crate::candid_variant! {
    /// The requested visibility of canister snapshots.
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default)]
    pub enum SnapshotVisibility {
        /// Access for the canister's controllers.
        #[default]
        #[serde(rename = "controllers")]
        570880087 => Controllers,
        /// Public access.
        #[serde(rename = "public")]
        1727234217 => Public,
        /// Access for the listed principals.
        #[serde(rename = "allowed_viewers")]
        2235716810 => AllowedViewers(Vec<Principal>),
    }
}

crate::candid_variant! {
    /// The requested visibility of canister status.
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default)]
    pub enum StatusVisibility {
        /// Access for the canister's controllers.
        #[default]
        #[serde(rename = "controllers")]
        570880087 => Controllers,
        /// Public access.
        #[serde(rename = "public")]
        1727234217 => Public,
        /// Access for the listed principals.
        #[serde(rename = "allowed_viewers")]
        2235716810 => AllowedViewers(Vec<Principal>),
    }
}

/// A named environment value.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default)]
pub struct EnvironmentVariable {
    /// The variable name.
    pub name: String,
    /// The variable value.
    pub value: String,
}

/// Optional canister settings supplied in a request.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default)]
pub struct CanisterSettings {
    /// The optional controller list.
    pub controllers: Option<Vec<Principal>>,
    /// The optional compute allocation.
    pub compute_allocation: Option<Nat>,
    /// The optional memory allocation.
    pub memory_allocation: Option<Nat>,
    /// The optional freezing threshold.
    pub freezing_threshold: Option<Nat>,
    /// The optional reserved-cycle limit.
    pub reserved_cycles_limit: Option<Nat>,
    /// The optional minimum cycles for incoming canister calls.
    pub minimum_incoming_canister_call_cycles: Option<Nat>,
    /// The optional log visibility.
    pub log_visibility: Option<LogVisibility>,
    /// The optional log-memory limit.
    pub log_memory_limit: Option<Nat>,
    /// The optional snapshot visibility.
    pub snapshot_visibility: Option<SnapshotVisibility>,
    /// The optional status visibility.
    pub status_visibility: Option<StatusVisibility>,
    /// The optional Wasm-memory limit.
    pub wasm_memory_limit: Option<Nat>,
    /// The optional Wasm-memory threshold.
    pub wasm_memory_threshold: Option<Nat>,
    /// The optional environment-variable list.
    pub environment_variables: Option<Vec<EnvironmentVariable>>,
}

/// Complete canister settings returned by the IC.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default)]
pub struct DefiniteCanisterSettings {
    /// The controller list.
    pub controllers: Vec<Principal>,
    /// The compute allocation.
    pub compute_allocation: Nat,
    /// The memory allocation.
    pub memory_allocation: Nat,
    /// The freezing threshold.
    pub freezing_threshold: Nat,
    /// The reserved-cycle limit.
    pub reserved_cycles_limit: Nat,
    /// The minimum cycles for incoming canister calls.
    pub minimum_incoming_canister_call_cycles: Nat,
    /// The log visibility.
    pub log_visibility: LogVisibility,
    /// The log-memory limit.
    pub log_memory_limit: Nat,
    /// The snapshot visibility.
    pub snapshot_visibility: SnapshotVisibility,
    /// The status visibility.
    pub status_visibility: StatusVisibility,
    /// The Wasm-memory limit.
    pub wasm_memory_limit: Nat,
    /// The Wasm-memory threshold.
    pub wasm_memory_threshold: Nat,
    /// The environment-variable list.
    pub environment_variables: Vec<EnvironmentVariable>,
}

crate::candid_record! {
    /// Parameters for creating a canister.
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default)]
    pub struct CreateCanisterArgs {
        /// Optional initial settings.
        2336062691 => pub settings: Option<CanisterSettings>,
        /// The sender's optional canister-version value.
        3599109322 => pub sender_canister_version: Option<u64>,
    }
}

crate::candid_record! {
    /// Settings to apply to an existing canister.
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
    pub struct UpdateSettingsArgs {
        /// The canister to update.
        1313628723 => pub canister_id: CanisterId,
        /// The supplied settings.
        2336062691 => pub settings: CanisterSettings,
        /// The sender's optional canister-version value.
        3599109322 => pub sender_canister_version: Option<u64>,
    }
}

/// A chunk supplied for a canister's chunk store.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct UploadChunkArgs {
    /// The canister receiving the chunk.
    pub canister_id: CanisterId,
    /// The uploaded chunk bytes.
    #[serde(with = "serde_bytes")]
    pub chunk: Vec<u8>,
}

/// The requested code-installation mode.
#[derive(
    Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default,
)]
pub enum CanisterInstallMode {
    /// Install code into a canister.
    #[default]
    #[serde(rename = "install")]
    Install,
    /// Reinstall the canister's code.
    #[serde(rename = "reinstall")]
    Reinstall,
    /// Upgrade code with optional upgrade flags.
    #[serde(rename = "upgrade")]
    Upgrade(Option<UpgradeFlags>),
}

/// Optional flags accompanying a code upgrade.
#[derive(
    Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default,
)]
pub struct UpgradeFlags {
    /// The optional request to skip the pre-upgrade hook.
    pub skip_pre_upgrade: Option<bool>,
    /// The optional Wasm-memory persistence choice.
    pub wasm_memory_persistence: Option<WasmMemoryPersistence>,
}

crate::candid_variant! {
    /// The Wasm-memory persistence choice for an upgrade.
    #[derive(
        Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default,
    )]
    pub enum WasmMemoryPersistence {
        /// Retain Wasm memory.
        #[serde(rename = "keep")]
        1191628933 => Keep,
        /// Replace Wasm memory.
        #[default]
        #[serde(rename = "replace")]
        2871543860 => Replace,
    }
}

/// Parameters for installing code assembled from stored chunks.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct InstallChunkedCodeArgs {
    /// The requested installation mode.
    pub mode: CanisterInstallMode,
    /// The canister receiving the code.
    pub target_canister: CanisterId,
    /// The optional canister holding the chunks.
    pub store_canister: Option<CanisterId>,
    /// Chunk hashes in the supplied assembly order.
    pub chunk_hashes_list: Vec<ChunkHash>,
    /// The supplied module hash bytes.
    #[serde(with = "serde_bytes")]
    pub wasm_module_hash: Vec<u8>,
    /// The supplied initialization argument bytes.
    #[serde(with = "serde_bytes")]
    pub arg: Vec<u8>,
    /// The sender's optional canister-version value.
    pub sender_canister_version: Option<u64>,
}

crate::candid_record! {
    /// A request to uninstall a canister's code.
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
    pub struct UninstallCodeArgs {
        /// The canister whose code is to be uninstalled.
        1313628723 => pub canister_id: CanisterId,
        /// The sender's optional canister-version value.
        3599109322 => pub sender_canister_version: Option<u64>,
    }
}

/// The status and resource information reported for a canister.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct CanisterStatusResult {
    /// The reported execution status.
    pub status: CanisterStatusType,
    /// Whether the canister is reported ready for migration.
    pub ready_for_migration: bool,
    /// The reported canister version.
    pub version: u64,
    /// The reported settings.
    pub settings: DefiniteCanisterSettings,
    /// The optional installed module hash, using ordinary sequence Serde.
    pub module_hash: Option<Vec<u8>>,
    /// The reported total memory size.
    pub memory_size: Nat,
    /// The reported memory breakdown.
    pub memory_metrics: MemoryMetrics,
    /// The reported cycle balance.
    pub cycles: Nat,
    /// The reported reserved cycles.
    pub reserved_cycles: Nat,
    /// The reported idle-cycle consumption per day.
    pub idle_cycles_burned_per_day: Nat,
    /// The reported query statistics.
    pub query_stats: QueryStats,
}

/// A canister's reported execution status.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum CanisterStatusType {
    /// The canister is running.
    #[serde(rename = "running")]
    Running,
    /// The canister is stopping.
    #[serde(rename = "stopping")]
    Stopping,
    /// The canister is stopped.
    #[serde(rename = "stopped")]
    Stopped,
}

/// Reported memory sizes by category.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct MemoryMetrics {
    /// Wasm memory size.
    pub wasm_memory_size: Nat,
    /// Stable memory size.
    pub stable_memory_size: Nat,
    /// Global memory size.
    pub global_memory_size: Nat,
    /// Wasm binary size.
    pub wasm_binary_size: Nat,
    /// Custom-section size.
    pub custom_sections_size: Nat,
    /// Canister-history size.
    pub canister_history_size: Nat,
    /// Wasm chunk-store size.
    pub wasm_chunk_store_size: Nat,
    /// Snapshot size.
    pub snapshots_size: Nat,
    /// Log memory-store size.
    pub log_memory_store_size: Nat,
}

/// Reported cumulative query statistics.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct QueryStats {
    /// Total query calls.
    pub num_calls_total: Nat,
    /// Total query instructions.
    pub num_instructions_total: Nat,
    /// Total request payload bytes.
    pub request_payload_bytes_total: Nat,
    /// Total response payload bytes.
    pub response_payload_bytes_total: Nat,
}

crate::candid_record! {
    /// A request for canister information and recent changes.
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
    pub struct CanisterInfoArgs {
        /// The canister to inspect.
        1313628723 => pub canister_id: Principal,
        /// The optional requested change count.
        2098503289 => pub num_requested_changes: Option<u64>,
    }
}

/// Canister information and change history returned by the IC.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct CanisterInfoResult {
    /// The reported total number of changes.
    pub total_num_changes: u64,
    /// The returned recent changes.
    pub recent_changes: Vec<Change>,
    /// The optional module hash, using ordinary sequence Serde.
    pub module_hash: Option<Vec<u8>>,
    /// The reported controllers.
    pub controllers: Vec<Principal>,
}

crate::candid_record! {
    /// The user associated with a canister change.
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
    pub struct FromUserRecord {
        /// The initiating user principal.
        1869947023 => pub user_id: Principal,
    }
}

/// The canister associated with a canister change.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct FromCanisterRecord {
    /// The initiating canister principal.
    pub canister_id: Principal,
    /// The optional initiating canister version.
    pub canister_version: Option<u64>,
}

crate::candid_variant! {
    /// The recorded origin of a canister change.
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
    pub enum ChangeOrigin {
        /// A change initiated by a user.
        #[serde(rename = "from_user")]
        572139648 => FromUser(FromUserRecord),
        /// A change initiated by a canister.
        #[serde(rename = "from_canister")]
        3061905116 => FromCanister(FromCanisterRecord),
    }
}

crate::candid_record! {
    /// Details recorded when a canister is created.
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
    pub struct CreationRecord {
        /// The recorded initial controllers.
        570880087 => pub controllers: Vec<Principal>,
        /// The optional environment-variable hash, using ordinary sequence Serde.
        3117986754 => pub environment_variables_hash: Option<Vec<u8>>,
    }
}

/// The mode recorded for a code deployment.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum CodeDeploymentMode {
    /// An installation.
    #[serde(rename = "install")]
    Install,
    /// A reinstallation.
    #[serde(rename = "reinstall")]
    Reinstall,
    /// An upgrade.
    #[serde(rename = "upgrade")]
    Upgrade,
}

crate::candid_record! {
    /// Details recorded for a code deployment.
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
    pub struct CodeDeploymentRecord {
        /// The recorded deployment mode.
        1214305123 => pub mode: CodeDeploymentMode,
        /// The recorded module hash bytes.
        #[serde(with = "serde_bytes")]
        2928387969 => pub module_hash: Vec<u8>,
    }
}

/// Details recorded when a snapshot is loaded.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct LoadSnapshotRecord {
    /// The optional source canister principal.
    pub from_canister_id: Option<Principal>,
    /// The loaded snapshot identifier.
    pub snapshot_id: SnapshotId,
    /// The recorded canister version.
    pub canister_version: u64,
    /// The recorded snapshot timestamp.
    pub taken_at_timestamp: u64,
    /// The recorded snapshot source.
    pub source: SnapshotSource,
}

crate::candid_record! {
    /// The destination information recorded for a canister rename.
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
    pub struct RenameToRecord {
        /// The destination canister principal.
        1313628723 => pub canister_id: Principal,
        /// The recorded destination version.
        2096228120 => pub version: u64,
        /// The recorded destination change count.
        3862312591 => pub total_num_changes: u64,
    }
}

/// Details recorded for a canister rename.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct RenameCanisterRecord {
    /// The canister principal recorded before the rename.
    pub canister_id: Principal,
    /// The recorded change count.
    pub total_num_changes: u64,
    /// The recorded rename destination.
    pub rename_to: RenameToRecord,
    /// The principal requesting the rename.
    pub requested_by: Principal,
}

crate::candid_record! {
    /// Controllers recorded by a controller-change event.
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
    pub struct ControllersChangeRecord {
        /// The recorded controller list.
        570880087 => pub controllers: Vec<Principal>,
    }
}

/// The details attached to a recorded canister change.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum ChangeDetails {
    /// Canister creation details.
    #[serde(rename = "creation")]
    Creation(CreationRecord),
    /// Code was uninstalled.
    #[serde(rename = "code_uninstall")]
    CodeUninstall,
    /// Code deployment details.
    #[serde(rename = "code_deployment")]
    CodeDeployment(CodeDeploymentRecord),
    /// Snapshot loading details.
    #[serde(rename = "load_snapshot")]
    LoadSnapshot(LoadSnapshotRecord),
    /// Controller change details.
    #[serde(rename = "controllers_change")]
    ControllersChange(ControllersChangeRecord),
    /// Canister rename details.
    #[serde(rename = "rename_canister")]
    RenameCanister(RenameCanisterRecord),
}

crate::candid_record! {
    /// One recorded canister change.
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
    pub struct Change {
        /// The recorded timestamp in nanoseconds.
        268680022 => pub timestamp_nanos: u64,
        /// The recorded canister version.
        783802816 => pub canister_version: u64,
        /// The recorded change origin.
        1258363366 => pub origin: ChangeOrigin,
        /// Optional change details.
        2874596546 => pub details: Option<ChangeDetails>,
    }
}

/// Parameters supplied for a management HTTP request.
#[derive(Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct HttpRequestArgs {
    /// The requested URL.
    pub url: String,
    /// The optional response-size limit supplied by the caller.
    pub max_response_bytes: Option<u64>,
    /// The requested HTTP method.
    pub method: HttpMethod,
    /// The supplied request headers.
    pub headers: Vec<HttpHeader>,
    /// The optional request body, using ordinary sequence Serde.
    pub body: Option<Vec<u8>>,
    /// The optional response-transform function and context.
    pub transform: Option<TransformContext>,
    /// The optional replication choice.
    pub is_replicated: Option<bool>,
    /// The optional pricing version.
    pub pricing_version: Option<u32>,
}

/// A response returned for a management HTTP request.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default)]
pub struct HttpRequestResult {
    /// The reported response status.
    pub status: Nat,
    /// The returned response headers.
    pub headers: Vec<HttpHeader>,
    /// The returned response body bytes.
    #[serde(with = "serde_bytes")]
    pub body: Vec<u8>,
}

/// An HTTP method supported by the request protocol.
#[derive(
    Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default,
)]
pub enum HttpMethod {
    /// The GET method.
    #[default]
    #[serde(rename = "get")]
    GET,
    /// The HEAD method.
    #[serde(rename = "head")]
    HEAD,
    /// The POST method.
    #[serde(rename = "post")]
    POST,
    /// The PUT method.
    #[serde(rename = "put")]
    PUT,
    /// The DELETE method.
    #[serde(rename = "delete")]
    DELETE,
    /// The PATCH method.
    #[serde(rename = "patch")]
    PATCH,
}

/// One HTTP header name and value.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default)]
pub struct HttpHeader {
    /// The header name.
    pub name: String,
    /// The header value.
    pub value: String,
}

crate::candid_record! {
    /// A response-transform function with caller-supplied context bytes.
    #[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
    pub struct TransformContext {
        /// The response-transform function reference.
        340962072 => pub function: TransformFunc,
        /// The opaque context bytes passed to the transform.
        #[serde(with = "serde_bytes")]
        427267567 => pub context: Vec<u8>,
    }
}

/// The response and context supplied to an HTTP transform function.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransformArgs {
    /// The response supplied for transformation.
    pub response: HttpRequestResult,
    /// The opaque caller context bytes.
    #[serde(with = "serde_bytes")]
    pub context: Vec<u8>,
}

/// An ECDSA key identifier carried by management requests.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default)]
pub struct EcdsaKeyId {
    /// The requested curve identifier.
    pub curve: EcdsaCurve,
    /// The requested key name.
    pub name: String,
}

crate::candid_variant! {
    /// A protocol identifier for an ECDSA curve.
    #[derive(
        Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default,
    )]
    pub enum EcdsaCurve {
        /// The secp256k1 curve identifier.
        #[default]
        #[serde(rename = "secp256k1")]
        492384026 => Secp256k1,
        /// The secp256r1 curve identifier.
        #[serde(rename = "secp256r1")]
        492385587 => Secp256r1,
    }
}

impl From<EcdsaCurve> for u32 {
    fn from(curve: EcdsaCurve) -> Self {
        match curve {
            EcdsaCurve::Secp256k1 => 0,
            EcdsaCurve::Secp256r1 => 1,
        }
    }
}

/// Parameters requesting ECDSA public-key bytes from the management service.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default)]
pub struct EcdsaPublicKeyArgs {
    /// The optional canister principal supplied for the request.
    pub canister_id: Option<CanisterId>,
    /// The supplied derivation path, using ordinary nested sequence Serde.
    pub derivation_path: Vec<Vec<u8>>,
    /// The requested key identifier.
    pub key_id: EcdsaKeyId,
}

crate::candid_record! {
    /// Opaque public-key material returned by the management service.
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default)]
    pub struct EcdsaPublicKeyResult {
        /// The returned public-key bytes.
        #[serde(with = "serde_bytes")]
        881350601 => pub public_key: Vec<u8>,
        /// The returned chain-code bytes.
        #[serde(with = "serde_bytes")]
        3110171755 => pub chain_code: Vec<u8>,
    }
}

/// Parameters requesting an ECDSA signature from the management service.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default)]
pub struct SignWithEcdsaArgs {
    /// The supplied message-hash bytes.
    #[serde(with = "serde_bytes")]
    pub message_hash: Vec<u8>,
    /// The supplied derivation path, using ordinary nested sequence Serde.
    pub derivation_path: Vec<Vec<u8>>,
    /// The requested key identifier.
    pub key_id: EcdsaKeyId,
}

crate::candid_record! {
    /// Opaque signature bytes returned by the management service.
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default)]
    pub struct SignWithEcdsaResult {
        /// The returned signature bytes.
        #[serde(with = "serde_bytes")]
        359375608 => pub signature: Vec<u8>,
    }
}

/// A snapshot identifier and its reported metadata.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default)]
pub struct Snapshot {
    /// The snapshot identifier.
    pub id: SnapshotId,
    /// The reported snapshot timestamp.
    pub taken_at_timestamp: u64,
    /// The reported total snapshot size.
    pub total_size: u64,
}

crate::candid_record! {
    /// A request to delete an identified canister snapshot.
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
    pub struct DeleteCanisterSnapshotArgs {
        /// The canister holding the snapshot.
        1313628723 => pub canister_id: CanisterId,
        /// The snapshot identifier to delete.
        4045790262 => pub snapshot_id: SnapshotId,
    }
}

/// The recorded origin of a snapshot.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum SnapshotSource {
    /// A snapshot taken from a canister, with a reserved protocol payload.
    #[serde(rename = "taken_from_canister")]
    TakenFromCanister(Reserved),
    /// A snapshot supplied through metadata upload, with a reserved payload.
    #[serde(rename = "metadata_upload")]
    MetadataUpload(Reserved),
}
