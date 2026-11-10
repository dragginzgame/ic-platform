# Shared management compatibility contract

This is a coordinator-reviewed public API specification for isolated authors. It
records names, type/field/variant ordering, traits and serialization attributes,
not implementation bodies, supplier tests or documentation. Do not read the old
module, its codec/tests, supplier sources, history or diffs.

Implementation scope: replace only
`crates/ic-platform/src/management_canister_types/maintained/mod.rs`. Retain
private `mod codec; mod transform_func;` and explicit public TransformFunc
reexport. Do not read or edit those existing owned modules or parent exports.
Imports are canonical `crate::principal::Principal`,
`crate::candid_codec::{Nat, Reserved}`, and Serde.
Keep the existing compiled module unchanged until the coordinator releases its
hold. A fresh unreferenced draft may be authored in the normal checkout at
`crates/ic-platform/src/management_canister_types/replacement/mod.rs`; no module
declaration may select it before baseline capture. Move it into the assigned
maintained path only after release. Write a design implementation record. Do not run Cargo; scoped
rustfmt with skip_children is allowed. Use concise newly written field docs.

All declarations are passive: do not validate, clamp, normalize, allocate
extra state or implement network, crypto, storage or timer policy. Nat is an
unbounded owned BigUint carrier (including values above u128); Reserved is the
existing unit struct. Do not add trait implementations absent from the table.
`bytes` means Serde bytes via serde_bytes; other byte vectors retain ordinary
sequence Serde, including optional vectors and nested byte paths. All listed
struct fields are public. Fields stay in listed order; enum order is listed
order. Empty/default records use ordinary derived field defaults; marked enum
variants determine Default. No hidden validation or field serde defaults.

Aliases: CanisterId = Principal; SnapshotId = Vec<u8>.

## CanisterIdRecord

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone.

- canister_id: CanisterId.

## ChunkHash

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone.

- hash: Vec<u8> [bytes].

## LogVisibility

Kind: enum. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default.

- Controllers: unit [Serde name controllers] [default].
- Public: unit [Serde name public].
- AllowedViewers: Vec<Principal> [Serde name allowed_viewers].

## SnapshotVisibility

Kind: enum. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default.

- Controllers: unit [Serde name controllers] [default].
- Public: unit [Serde name public].
- AllowedViewers: Vec<Principal> [Serde name allowed_viewers].

## StatusVisibility

Kind: enum. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default.

- Controllers: unit [Serde name controllers] [default].
- Public: unit [Serde name public].
- AllowedViewers: Vec<Principal> [Serde name allowed_viewers].

## EnvironmentVariable

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default.

- name: String.
- value: String.

## CanisterSettings

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default.

- controllers: Option<Vec<Principal>>.
- compute_allocation: Option<Nat>.
- memory_allocation: Option<Nat>.
- freezing_threshold: Option<Nat>.
- reserved_cycles_limit: Option<Nat>.
- minimum_incoming_canister_call_cycles: Option<Nat>.
- log_visibility: Option<LogVisibility>.
- log_memory_limit: Option<Nat>.
- snapshot_visibility: Option<SnapshotVisibility>.
- status_visibility: Option<StatusVisibility>.
- wasm_memory_limit: Option<Nat>.
- wasm_memory_threshold: Option<Nat>.
- environment_variables: Option<Vec<EnvironmentVariable>>.

## DefiniteCanisterSettings

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default.

- controllers: Vec<Principal>.
- compute_allocation: Nat.
- memory_allocation: Nat.
- freezing_threshold: Nat.
- reserved_cycles_limit: Nat.
- minimum_incoming_canister_call_cycles: Nat.
- log_visibility: LogVisibility.
- log_memory_limit: Nat.
- snapshot_visibility: SnapshotVisibility.
- status_visibility: StatusVisibility.
- wasm_memory_limit: Nat.
- wasm_memory_threshold: Nat.
- environment_variables: Vec<EnvironmentVariable>.

## CreateCanisterArgs

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default.

- settings: Option<CanisterSettings>.
- sender_canister_version: Option<u64>.

## UpdateSettingsArgs

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone.

- canister_id: CanisterId.
- settings: CanisterSettings.
- sender_canister_version: Option<u64>.

## UploadChunkArgs

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone.

- canister_id: CanisterId.
- chunk: Vec<u8> [bytes].

## CanisterInstallMode

Kind: enum. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default.

- Install: unit [Serde name install] [default].
- Reinstall: unit [Serde name reinstall].
- Upgrade: Option<UpgradeFlags> [Serde name upgrade].

## UpgradeFlags

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default.

- skip_pre_upgrade: Option<bool>.
- wasm_memory_persistence: Option<WasmMemoryPersistence>.

## WasmMemoryPersistence

Kind: enum. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default.

- Keep: unit [Serde name keep].
- Replace: unit [Serde name replace] [default].

## InstallChunkedCodeArgs

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone.

- mode: CanisterInstallMode.
- target_canister: CanisterId.
- store_canister: Option<CanisterId>.
- chunk_hashes_list: Vec<ChunkHash>.
- wasm_module_hash: Vec<u8> [bytes].
- arg: Vec<u8> [bytes].
- sender_canister_version: Option<u64>.

## UninstallCodeArgs

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone.

- canister_id: CanisterId.
- sender_canister_version: Option<u64>.

## CanisterStatusResult

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone.

- status: CanisterStatusType.
- ready_for_migration: bool.
- version: u64.
- settings: DefiniteCanisterSettings.
- module_hash: Option<Vec<u8>>.
- memory_size: Nat.
- memory_metrics: MemoryMetrics.
- cycles: Nat.
- reserved_cycles: Nat.
- idle_cycles_burned_per_day: Nat.
- query_stats: QueryStats.

## CanisterStatusType

Kind: enum. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy.

- Running: unit [Serde name running].
- Stopping: unit [Serde name stopping].
- Stopped: unit [Serde name stopped].

## MemoryMetrics

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone.

- wasm_memory_size: Nat.
- stable_memory_size: Nat.
- global_memory_size: Nat.
- wasm_binary_size: Nat.
- custom_sections_size: Nat.
- canister_history_size: Nat.
- wasm_chunk_store_size: Nat.
- snapshots_size: Nat.
- log_memory_store_size: Nat.

## QueryStats

Kind: struct. Traits: Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize.

- num_calls_total: Nat.
- num_instructions_total: Nat.
- request_payload_bytes_total: Nat.
- response_payload_bytes_total: Nat.

## CanisterInfoArgs

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone.

- canister_id: Principal.
- num_requested_changes: Option<u64>.

## CanisterInfoResult

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone.

- total_num_changes: u64.
- recent_changes: Vec<Change>.
- module_hash: Option<Vec<u8>>.
- controllers: Vec<Principal>.

## FromUserRecord

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone.

- user_id: Principal.

## FromCanisterRecord

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone.

- canister_id: Principal.
- canister_version: Option<u64>.

## ChangeOrigin

Kind: enum. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone.

- FromUser: FromUserRecord [Serde name from_user].
- FromCanister: FromCanisterRecord [Serde name from_canister].

## CreationRecord

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone.

- controllers: Vec<Principal>.
- environment_variables_hash: Option<Vec<u8>>.

## CodeDeploymentMode

Kind: enum. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy.

- Install: unit [Serde name install].
- Reinstall: unit [Serde name reinstall].
- Upgrade: unit [Serde name upgrade].

## CodeDeploymentRecord

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone.

- mode: CodeDeploymentMode.
- module_hash: Vec<u8> [bytes].

## LoadSnapshotRecord

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone.

- from_canister_id: Option<Principal>.
- snapshot_id: SnapshotId.
- canister_version: u64.
- taken_at_timestamp: u64.
- source: SnapshotSource.

## RenameToRecord

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone.

- canister_id: Principal.
- version: u64.
- total_num_changes: u64.

## RenameCanisterRecord

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone.

- canister_id: Principal.
- total_num_changes: u64.
- rename_to: RenameToRecord.
- requested_by: Principal.

## ControllersChangeRecord

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone.

- controllers: Vec<Principal>.

## ChangeDetails

Kind: enum. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone.

- Creation: CreationRecord [Serde name creation].
- CodeUninstall: unit [Serde name code_uninstall].
- CodeDeployment: CodeDeploymentRecord [Serde name code_deployment].
- LoadSnapshot: LoadSnapshotRecord [Serde name load_snapshot].
- ControllersChange: ControllersChangeRecord [Serde name controllers_change].
- RenameCanister: RenameCanisterRecord [Serde name rename_canister].

## Change

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone.

- timestamp_nanos: u64.
- canister_version: u64.
- origin: ChangeOrigin.
- details: Option<ChangeDetails>.

## HttpRequestArgs

Kind: struct. Traits: Deserialize, Debug, PartialEq, Eq, Clone, Default.

- url: String.
- max_response_bytes: Option<u64>.
- method: HttpMethod.
- headers: Vec<HttpHeader>.
- body: Option<Vec<u8>>.
- transform: Option<TransformContext>.
- is_replicated: Option<bool>.
- pricing_version: Option<u32>.

## HttpRequestResult

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default.

- status: Nat.
- headers: Vec<HttpHeader>.
- body: Vec<u8> [bytes].

## HttpMethod

Kind: enum. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default.

- GET: unit [Serde name get] [default].
- HEAD: unit [Serde name head].
- POST: unit [Serde name post].
- PUT: unit [Serde name put].
- DELETE: unit [Serde name delete].
- PATCH: unit [Serde name patch].

## HttpHeader

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default.

- name: String.
- value: String.

## TransformContext

Kind: struct. Traits: Clone, Debug, Deserialize, PartialEq, Eq.

- function: TransformFunc.
- context: Vec<u8> [bytes].

## TransformArgs

Kind: struct. Traits: Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize.

- response: HttpRequestResult.
- context: Vec<u8> [bytes].

## EcdsaKeyId

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default.

- curve: EcdsaCurve.
- name: String.

## EcdsaCurve

Kind: enum. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default.

- Secp256k1: unit [Serde name secp256k1] [default].
- Secp256r1: unit [Serde name secp256r1].

## EcdsaPublicKeyArgs

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default.

- canister_id: Option<CanisterId>.
- derivation_path: Vec<Vec<u8>>.
- key_id: EcdsaKeyId.

## EcdsaPublicKeyResult

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default.

- public_key: Vec<u8> [bytes].
- chain_code: Vec<u8> [bytes].

## SignWithEcdsaArgs

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default.

- message_hash: Vec<u8> [bytes].
- derivation_path: Vec<Vec<u8>>.
- key_id: EcdsaKeyId.

## SignWithEcdsaResult

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default.

- signature: Vec<u8> [bytes].

## Snapshot

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default.

- id: SnapshotId.
- taken_at_timestamp: u64.
- total_size: u64.

## DeleteCanisterSnapshotArgs

Kind: struct. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone.

- canister_id: CanisterId.
- snapshot_id: SnapshotId.

## SnapshotSource

Kind: enum. Traits: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone.

- TakenFromCanister: Reserved [Serde name taken_from_canister].
- MetadataUpload: Reserved [Serde name metadata_upload].

## Sole conversion and retained adapter

The public conversion From<EcdsaCurve> for u32 maps Secp256k1 to 0 and
Secp256r1 to 1. It selects a protocol cost code; it does not implement ECDSA.

TransformFunc remains the existing owned public tuple wrapper around
`candid_codec::references::Func { principal: Principal, method: String }`. It has
new(Principal, String), From<Func>, Clone, Debug, Eq, PartialEq, Deserialize,
Encode and Decode. Its function schema is one TransformArgs argument returning
one HttpRequestResult with query mode. Existing constructor/codec behavior must
remain untouched. HttpRequestArgs and TransformContext deliberately lack
Serialize/Ord/Hash; do not broaden them. TransformArgs has Serialize/Deserialize
and Hash but no Ord.

Supplemental public input fact for tests: Func's Serde deserialization accepts
an owned byte buffer containing marker 5, unsigned LEB128 UTF-8 method byte
length, method bytes and the remaining raw Principal bytes (0–29). A fixed
short-method fixture suffices; no source inspection or general framing helper
is required. This is distinct from its typed Candid function encoding.

## Public consumer access and qualification

Only these 44 names are exported at `ic_platform::management_canister_types`:
`CanisterIdRecord`, `CanisterInfoArgs`, `CanisterInfoResult`, `CanisterInstallMode`, `CanisterSettings`, `CanisterStatusResult`, `CanisterStatusType`, `Change`, `ChangeDetails`, `ChangeOrigin`, `ChunkHash`, `CodeDeploymentMode`, `CodeDeploymentRecord`, `ControllersChangeRecord`, `CreateCanisterArgs`, `CreationRecord`, `DefiniteCanisterSettings`, `DeleteCanisterSnapshotArgs`, `EcdsaCurve`, `EcdsaKeyId`, `EcdsaPublicKeyArgs`, `EcdsaPublicKeyResult`, `EnvironmentVariable`, `FromCanisterRecord`, `FromUserRecord`, `HttpHeader`, `HttpMethod`, `HttpRequestArgs`, `HttpRequestResult`, `InstallChunkedCodeArgs`, `LogVisibility`, `MemoryMetrics`, `QueryStats`, `SignWithEcdsaArgs`, `SignWithEcdsaResult`, `Snapshot`, `SnapshotSource`, `StatusVisibility`, `TransformArgs`, `TransformContext`, `TransformFunc`, `UninstallCodeArgs`, `UpdateSettingsArgs`, `UploadChunkArgs`.

Other listed types are reachable as nested field/variant payloads, not by root
name. Public tests can construct them using inferred Default or deserialization
of containing records; do not widen exports for tests. Preserve all 52 selected
schemas (49 declarations, two aliases and TransformFunc). Native extensions
remain unchanged. Parent owns the existing registry schema/value oracles.

Public tests should use this contract only: own JSON/CBOR/Candid observations,
trait/default/ordering/Serde checks and complete enum/numeric/option coverage.
Use a finite deterministic transcript prefixed MANAGEMENT_SHARED_OBSERVATIONS=
for parent old/new capture; parent freezes the original transcript as a fixture.
No imported expressive tests or source scans. Do not add a fingerprint of the
new implementation as a replacement for behavior checks.

Available codec API: encode_one(&value, maximum_bytes),
decoder::decode_one::<T>(&bytes, Limits). Limits has bytes, header, types, work,
allocation usize fields. Decode is lifetime-generic. Nat::from(u64/u128) and
decimal FromStr work. Owned Principal supports anonymous(), management_canister()
and from_slice(&[u8]) for at most 29 bytes. Caller budgets stay explicit.
