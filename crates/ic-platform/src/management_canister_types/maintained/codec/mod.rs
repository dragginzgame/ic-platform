//! Complete explicit wire schemas for the selected management declarations.
use super::{
    CanisterId, CanisterInfoResult, CanisterInstallMode, CanisterSettings, CanisterStatusResult,
    CanisterStatusType, Change, ChangeDetails, ChunkHash, CodeDeploymentMode, CodeDeploymentRecord,
    ControllersChangeRecord, CreationRecord, DefiniteCanisterSettings, EcdsaCurve, EcdsaKeyId,
    EcdsaPublicKeyArgs, EnvironmentVariable, FromCanisterRecord, HttpHeader, HttpMethod,
    HttpRequestArgs, HttpRequestResult, InstallChunkedCodeArgs, LoadSnapshotRecord, LogVisibility,
    MemoryMetrics, Nat, Principal, QueryStats, RenameCanisterRecord, RenameToRecord, Reserved,
    SignWithEcdsaArgs, Snapshot, SnapshotId, SnapshotSource, SnapshotVisibility, StatusVisibility,
    TransformArgs, TransformContext, UpgradeFlags, UploadChunkArgs, WasmMemoryPersistence,
};
crate::candid_record!(EnvironmentVariable {834174833 => value: String, 1224700491 => name: String});
crate::candid_record!(CanisterSettings {238856128 => freezing_threshold: Option<Nat>, 241629288 => wasm_memory_threshold: Option<Nat>, 356749323 => environment_variables: Option<Vec<EnvironmentVariable>>, 570880087 => controllers: Option<Vec<Principal>>, 793974846 => minimum_incoming_canister_call_cycles: Option<Nat>, 1095112320 => reserved_cycles_limit: Option<Nat>, 2033331437 => log_visibility: Option<LogVisibility>, 2503910424 => log_memory_limit: Option<Nat>, 2709750061 => snapshot_visibility: Option<SnapshotVisibility>, 3380736376 => wasm_memory_limit: Option<Nat>, 3469368575 => status_visibility: Option<StatusVisibility>, 3844961758 => memory_allocation: Option<Nat>, 4174053672 => compute_allocation: Option<Nat>});
crate::candid_record!(DefiniteCanisterSettings {238856128 => freezing_threshold: Nat, 241629288 => wasm_memory_threshold: Nat, 356749323 => environment_variables: Vec<EnvironmentVariable>, 570880087 => controllers: Vec<Principal>, 793974846 => minimum_incoming_canister_call_cycles: Nat, 1095112320 => reserved_cycles_limit: Nat, 2033331437 => log_visibility: LogVisibility, 2503910424 => log_memory_limit: Nat, 2709750061 => snapshot_visibility: SnapshotVisibility, 3380736376 => wasm_memory_limit: Nat, 3469368575 => status_visibility: StatusVisibility, 3844961758 => memory_allocation: Nat, 4174053672 => compute_allocation: Nat});
crate::candid_record!(UploadChunkArgs {1170392685 => chunk: Vec<u8>, 1313628723 => canister_id: CanisterId});
crate::candid_variant!(CanisterInstallMode {235052488 => Reinstall, 1664201884 => Upgrade(Option<UpgradeFlags>), 2375734555 => Install});
crate::candid_record!(UpgradeFlags {793490780 => wasm_memory_persistence: Option<WasmMemoryPersistence>, 1539756096 => skip_pre_upgrade: Option<bool>});
crate::candid_record!(InstallChunkedCodeArgs {4849238 => arg: Vec<u8>, 1176624806 => wasm_module_hash: Vec<u8>, 1214305123 => mode: CanisterInstallMode, 1847393007 => chunk_hashes_list: Vec<ChunkHash>, 2807510037 => target_canister: CanisterId, 3052241285 => store_canister: Option<CanisterId>, 3599109322 => sender_canister_version: Option<u64>});
crate::candid_record!(CanisterStatusResult {58798021 => memory_metrics: MemoryMetrics, 100394802 => status: CanisterStatusType, 1054895615 => memory_size: Nat, 1614520508 => ready_for_migration: bool, 2096228120 => version: u64, 2190693645 => cycles: Nat, 2336062691 => settings: DefiniteCanisterSettings, 2642624104 => query_stats: QueryStats, 2733945392 => idle_cycles_burned_per_day: Nat, 2928387969 => module_hash: Option<Vec<u8>>, 4090107140 => reserved_cycles: Nat});
crate::candid_variant!(CanisterStatusType {1130484237 => Stopped, 2990082932 => Stopping, 3949555199 => Running});
crate::candid_record!(MemoryMetrics {4264996 => wasm_binary_size: Nat, 81310498 => log_memory_store_size: Nat, 197078956 => wasm_chunk_store_size: Nat, 1646009124 => canister_history_size: Nat, 1710030651 => stable_memory_size: Nat, 1777917585 => snapshots_size: Nat, 3598099748 => wasm_memory_size: Nat, 3776075235 => global_memory_size: Nat, 4108871332 => custom_sections_size: Nat});
crate::candid_record!(QueryStats {813120577 => response_payload_bytes_total: Nat, 1563534595 => num_instructions_total: Nat, 2783848481 => num_calls_total: Nat, 4195608591 => request_payload_bytes_total: Nat});
crate::candid_record!(CanisterInfoResult {570880087 => controllers: Vec<Principal>, 2928387969 => module_hash: Option<Vec<u8>>, 3067116063 => recent_changes: Vec<Change>, 3862312591 => total_num_changes: u64});
crate::candid_record!(FromCanisterRecord {783802816 => canister_version: Option<u64>, 1313628723 => canister_id: Principal});
crate::candid_variant!(CodeDeploymentMode {235052488 => Reinstall, 1664201884 => Upgrade, 2375734555 => Install});
crate::candid_record!(LoadSnapshotRecord {783802816 => canister_version: u64, 842117339 => source: SnapshotSource, 2242915486 => from_canister_id: Option<Principal>, 3569147778 => taken_at_timestamp: u64, 4045790262 => snapshot_id: SnapshotId});
crate::candid_record!(RenameCanisterRecord {754389724 => rename_to: RenameToRecord, 1313628723 => canister_id: Principal, 1920899912 => requested_by: Principal, 3862312591 => total_num_changes: u64});
crate::candid_variant!(ChangeDetails {737642527 => Creation(CreationRecord), 1168034903 => CodeDeployment(CodeDeploymentRecord), 1239220413 => LoadSnapshot(LoadSnapshotRecord), 2838302280 => RenameCanister(RenameCanisterRecord), 2881846168 => ControllersChange(ControllersChangeRecord), 4081328912 => CodeUninstall});
crate::candid_record!(HttpRequestArgs {5843823 => url: String, 156956385 => method: HttpMethod, 309734248 => max_response_bytes: Option<u64>, 1092319906 => body: Option<Vec<u8>>, 1167273324 => transform: Option<TransformContext>, 1661489734 => headers: Vec<HttpHeader>, 2677914640 => is_replicated: Option<bool>, 4182041247 => pricing_version: Option<u32>});
crate::candid_record!(HttpRequestResult {100394802 => status: Nat, 1092319906 => body: Vec<u8>, 1661489734 => headers: Vec<HttpHeader>});
crate::candid_variant!(HttpMethod {5144726 => GET, 5595855 => PUT, 1158359328 => HEAD, 1247577184 => POST, 1792589867 => DELETE, 3176597192 => PATCH});
crate::candid_record!(HttpHeader {834174833 => value: String, 1224700491 => name: String});
crate::candid_record!(TransformArgs {427267567 => context: Vec<u8>, 2718740097 => response: HttpRequestResult});
crate::candid_record!(EcdsaKeyId {1224700491 => name: String, 1314409647 => curve: EcdsaCurve});
crate::candid_record!(EcdsaPublicKeyArgs {1072395707 => key_id: EcdsaKeyId, 1313628723 => canister_id: Option<CanisterId>, 1445762093 => derivation_path: Vec<Vec<u8>>});
crate::candid_record!(SignWithEcdsaArgs {1072395707 => key_id: EcdsaKeyId, 1445762093 => derivation_path: Vec<Vec<u8>>, 3146564742 => message_hash: Vec<u8>});
crate::candid_record!(Snapshot {23515 => id: SnapshotId, 1926775388 => total_size: u64, 3569147778 => taken_at_timestamp: u64});
crate::candid_variant!(SnapshotSource {833703953 => MetadataUpload(Reserved), 2359638948 => TakenFromCanister(Reserved)});
