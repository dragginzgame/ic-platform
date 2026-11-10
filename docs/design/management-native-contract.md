# Native management DTO compatibility contract

Specification-only input for the authorized implementer and separate public-test
author. Do not read existing/historical target modules, their tests, supplier
implementations or imported prose. The coordinator reviewed the public interface.
Do not inspect the shared maintained declarations: use the public facts below.

Scope is `crates/ic-platform/src/management_canister_types/native/mod.rs` and
`native/client.rs`, plus an implementation record. Hold product writes until the
coordinator captures old-owner observations and releases the hold. Drafts can live
under `/tmp/management-native-draft`. Agents run scoped rustfmt only, no Cargo.
Do not edit the existing codec mappings/tests, parent exports, dependencies,
features, shared declarations, cryptography or execution/transport code.

These are passive types, not a snapshot implementation, timer, installer or log
service. Consumers own limits, trust, identity admission, retries and orchestration.
No validation or narrowing is added. No new wrappers, blanket re-exports or
additional capabilities. Fresh concise field docs should replace imported prose.

## Structure, imported API facts and common representation

The parent already gates the whole native module with
`cfg(not(target_family = "wasm"))`; do not widen its visibility or change exports.
Native mod.rs must retain `mod client; mod codec;` and explicitly pub-use
CanisterLogRecord, FetchCanisterLogsResult, InstallCodeArgs,
ProvisionalCreateCanisterWithCyclesArgs from client. Keep `#[cfg(test)] mod tests;`
for existing coordinator-owned oracle tests; do not read that file.

Available shared aliases/types via `super::maintained` in native/mod.rs:
- CanisterId = canonical owned Principal.
- SnapshotId = Vec<u8>, ordinary sequence Serde (not a byte adapter).
- ChunkHash is a struct with public hash: Vec<u8>, serde_bytes-adapted.
- SnapshotSource enum: TakenFromCanister(Reserved), MetadataUpload(Reserved),
  Serde names `taken_from_canister`, `metadata_upload` respectively.

Native/client.rs uses `super::super::maintained` for CanisterId,
CanisterInstallMode, CanisterSettings. CanisterInstallMode variants are Install,
Reinstall, Upgrade(Option<UpgradeFlags>), corresponding Serde names install,
reinstall, upgrade. Default is Install. CanisterSettings implements Default with
all optional fields None and has public controllers: Option<Vec<Principal>>,
reserved_cycles_limit: Option<Nat>; use default for other fields if constructing
test data. Those shared definitions are outside this rewrite.

`crate::candid_codec::{Nat, Reserved}` are existing owned types; Nat.0 is BigUint,
Nat::from(u64/u128) and decimal FromStr work. Reserved is a unit struct usable as
Reserved. Nat is unbounded even where a protocol label mentions v128. No new
limits or hashing. Ordinary Serde derives use declaration field/variant order;
preserve it exactly. All fields listed below are public. No skip/default/flatten
attributes except the Default trait explicitly listed; absent optional fields
still deserialize as None via ordinary Serde.

Trait set A: Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
Clone. Trait set B: Serialize, Deserialize, Debug, Clone ONLY. No additional
traits. All byte vectors use ordinary sequence Serde unless explicitly marked
`bytes` below, meaning `#[serde(with = "serde_bytes")]`. Do not normalize the
intentional asymmetries between read/upload fields.

## Snapshot declarations in native/mod.rs

TakeCanisterSnapshotArgs (A), fields:
canister_id: CanisterId, replace_snapshot: Option<SnapshotId>,
uninstall_code: Option<bool>, sender_canister_version: Option<u64>.

LoadCanisterSnapshotArgs (A), fields:
canister_id: CanisterId, snapshot_id: SnapshotId,
sender_canister_version: Option<u64>.

ReadCanisterSnapshotMetadataArgs (B), fields:
canister_id: CanisterId, snapshot_id: SnapshotId.

ReadCanisterSnapshotMetadataResult (B), fields:
source: Option<SnapshotSource>, taken_at_timestamp: u64, wasm_module_size: u64,
globals: Vec<Option<SnapshotMetadataGlobal>>, wasm_memory_size: u64,
stable_memory_size: u64, wasm_chunk_store: StoredChunksResult,
canister_version: u64, certified_data: Vec<u8> (bytes),
global_timer: Option<CanisterTimer>,
on_low_wasm_memory_hook_status: Option<OnLowWasmMemoryHookStatus>.

SnapshotMetadataGlobal (B), variants in order with exact Serde names:
I32(i32) -> i32; I64(i64) -> i64; F32(f32) -> f32; F64(f64) -> f64;
V128(Nat) -> v128. Preserve floating values including signed zero, infinity and
NaN payloads on the Candid boundary; no canonicalization/extra checks here.

CanisterTimer (B), variants: Inactive -> inactive, Active(u64) -> active.
This is a status DTO, not a scheduler dependency or runtime timer.

OnLowWasmMemoryHookStatus (B), variants:
ConditionNotSatisfied -> condition_not_satisfied, Ready -> ready, Executed -> executed.

ReadCanisterSnapshotDataArgs (B), fields:
canister_id: CanisterId, snapshot_id: SnapshotId, kind: SnapshotDataKind.

SnapshotDataKind (B), variants in order:
WasmModule { offset: u64, size: u64 } -> wasm_module;
WasmMemory { offset: u64, size: u64 } -> wasm_memory;
StableMemory { offset: u64, size: u64 } -> stable_memory;
WasmChunk { hash: Vec<u8> (bytes) } -> wasm_chunk.

ReadCanisterSnapshotDataResult (B), one field chunk: Vec<u8> (bytes).

UploadCanisterSnapshotMetadataArgs (B), fields:
canister_id: CanisterId, replace_snapshot: Option<SnapshotId>,
wasm_module_size: u64, globals: Vec<SnapshotMetadataGlobal>,
wasm_memory_size: u64, stable_memory_size: u64,
certified_data: Vec<u8> (ordinary sequence), global_timer: Option<CanisterTimer>,
on_low_wasm_memory_hook_status: Option<OnLowWasmMemoryHookStatus>.

UploadCanisterSnapshotMetadataResult (B), one field snapshot_id: SnapshotId.

UploadCanisterSnapshotDataArgs (B), fields:
canister_id: CanisterId, snapshot_id: SnapshotId, kind: SnapshotDataOffset,
chunk: Vec<u8> (ordinary sequence).

SnapshotDataOffset (B), variants in order:
WasmModule { offset: u64 } -> wasm_module;
WasmMemory { offset: u64 } -> wasm_memory;
StableMemory { offset: u64 } -> stable_memory;
WasmChunk (unit) -> wasm_chunk.

Retain module alias StoredChunksResult = Vec<ChunkHash>. It is a private-module
dependency, not an added parent export.

## Client declarations in native/client.rs

InstallCodeArgs (A), fields:
mode: CanisterInstallMode, canister_id: CanisterId,
wasm_module: Vec<u8> (ordinary sequence), arg: Vec<u8> (bytes),
sender_canister_version: Option<u64>.

ProvisionalCreateCanisterWithCyclesArgs (A plus Default), fields:
amount: Option<Nat>, settings: Option<CanisterSettings>,
specified_id: Option<CanisterId>, sender_canister_version: Option<u64>.
Default sets each field None; no network defaults or policy inferred.

CanisterLogRecord (A), fields:
idx: u64, timestamp_nanos: u64, content: Vec<u8> (bytes).

FetchCanisterLogsResult (A), one field:
canister_log_records: Vec<CanisterLogRecord>.

## Independent public tests and records

Test author writes `crates/ic-platform/tests/management_native_consumer.rs`,
guarded at file level `#![cfg(not(target_family = "wasm"))]`, plus
`docs/design/management-native-public-tests-implementation.md`. All 18 names
are public at ic_platform::management_canister_types; shared dependency names
ChunkHash, SnapshotSource, CanisterInstallMode, CanisterSettings are there too.
Principal is ic_platform::Principal, Nat/Reserved are candid_codec exports.
encode_one(&T, byte_limit) and decoder::decode_one::<T>(&bytes, Limits) are
available. Limits has usize fields bytes/header/types/work/allocation. Existing
serde_json/serde_cbor dev packages are available.

Use modest finite cases covering all variants and 18 type schemas; exact
read/upload Serde bytes-vs-sequence differences, option defaults, unbounded Nat,
u64/i32/i64 extrema, special float bit patterns, trait ranges/order/hash, and
complete decode/encode roundtrips without assuming Eq on float-bearing types.
For JSON transcript use finite floats only; test special float bits separately
through Candid and/or CBOR. Record JSON text, CBOR bytes and Candid bytes for
representative grouped cases as one `MANAGEMENT_NATIVE_OBSERVATIONS=` JSON line,
keeping fixture size below 30KB. No environment-selected expectations. Coordinator
captures old owner, freezes outputs and compares unchanged new execution. Parent
already has independent registry schema/value tests; do not read/duplicate them
or add imported test suites. No Cargo in author contexts.

Implementer writes only two modules and
`docs/design/management-native-implementation.md`. Parent owns retirement of the
two native provenance files and their old-source guard after replacement passes;
the shared maintained source/license/guard remain. Public exports and codec
mappings must be byte-identical throughout this task. Record admitted actions
and limitations, not legal clearance or absence of model-training exposure.
