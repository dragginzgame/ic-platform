#![cfg(not(target_family = "wasm"))]
//! Native passive management DTO observations from the reviewed contract.

use ic_platform::Principal;
use ic_platform::candid_codec::decoder::{Decode, Limits, decode_one};
use ic_platform::candid_codec::{Encode, Nat, Reserved, encode_one};
use ic_platform::management_canister_types::{
    CanisterInstallMode, CanisterLogRecord, CanisterSettings, CanisterTimer, ChunkHash,
    FetchCanisterLogsResult, InstallCodeArgs, LoadCanisterSnapshotArgs, OnLowWasmMemoryHookStatus,
    ProvisionalCreateCanisterWithCyclesArgs, ReadCanisterSnapshotDataArgs,
    ReadCanisterSnapshotDataResult, ReadCanisterSnapshotMetadataArgs,
    ReadCanisterSnapshotMetadataResult, SnapshotDataKind, SnapshotDataOffset,
    SnapshotMetadataGlobal, SnapshotSource, TakeCanisterSnapshotArgs,
    UploadCanisterSnapshotDataArgs, UploadCanisterSnapshotMetadataArgs,
    UploadCanisterSnapshotMetadataResult,
};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

fn huge() -> Nat {
    "340282366920938463463374607431768211456".parse().unwrap()
}

fn limits(length: usize) -> Limits {
    Limits {
        bytes: length,
        header: 32_768,
        types: 4096,
        work: 1_000_000,
        allocation: 1_000_000,
    }
}

fn globals() -> Vec<SnapshotMetadataGlobal> {
    vec![
        SnapshotMetadataGlobal::I32(i32::MIN),
        SnapshotMetadataGlobal::I32(i32::MAX),
        SnapshotMetadataGlobal::I64(i64::MIN),
        SnapshotMetadataGlobal::I64(i64::MAX),
        SnapshotMetadataGlobal::F32(1.25),
        SnapshotMetadataGlobal::F64(-2.5),
        SnapshotMetadataGlobal::V128(huge()),
    ]
}

fn kinds() -> Vec<SnapshotDataKind> {
    vec![
        SnapshotDataKind::WasmModule {
            offset: u64::MAX,
            size: u64::MAX,
        },
        SnapshotDataKind::WasmMemory {
            offset: 0,
            size: u64::MAX,
        },
        SnapshotDataKind::StableMemory {
            offset: u64::MAX,
            size: 0,
        },
        SnapshotDataKind::WasmChunk {
            hash: vec![0, 255, 128],
        },
    ]
}

fn offsets() -> Vec<SnapshotDataOffset> {
    vec![
        SnapshotDataOffset::WasmModule { offset: u64::MAX },
        SnapshotDataOffset::WasmMemory { offset: 0 },
        SnapshotDataOffset::StableMemory { offset: u64::MAX },
        SnapshotDataOffset::WasmChunk,
    ]
}

fn read_metadata() -> ReadCanisterSnapshotMetadataResult {
    let mut globals: Vec<_> = globals().into_iter().map(Some).collect();
    globals.push(None);
    ReadCanisterSnapshotMetadataResult {
        source: Some(SnapshotSource::TakenFromCanister(Reserved)),
        taken_at_timestamp: u64::MAX,
        wasm_module_size: u64::MAX,
        globals,
        wasm_memory_size: u64::MAX,
        stable_memory_size: u64::MAX,
        wasm_chunk_store: vec![ChunkHash { hash: vec![0, 255] }],
        canister_version: u64::MAX,
        certified_data: vec![0, 255, 128],
        global_timer: Some(CanisterTimer::Active(u64::MAX)),
        on_low_wasm_memory_hook_status: Some(OnLowWasmMemoryHookStatus::Executed),
    }
}

fn upload_metadata() -> UploadCanisterSnapshotMetadataArgs {
    UploadCanisterSnapshotMetadataArgs {
        canister_id: Principal::anonymous(),
        replace_snapshot: Some(vec![0, 255]),
        wasm_module_size: u64::MAX,
        globals: globals(),
        wasm_memory_size: u64::MAX,
        stable_memory_size: u64::MAX,
        certified_data: vec![0, 255, 128],
        global_timer: Some(CanisterTimer::Inactive),
        on_low_wasm_memory_hook_status: Some(OnLowWasmMemoryHookStatus::Ready),
    }
}

fn install(mode: CanisterInstallMode) -> InstallCodeArgs {
    InstallCodeArgs {
        mode,
        canister_id: Principal::management_canister(),
        wasm_module: vec![0, 255],
        arg: vec![128, 1],
        sender_canister_version: Some(u64::MAX),
    }
}

fn cbor_field<T: Serialize>(value: &T, name: &str) -> serde_cbor::Value {
    let serde_cbor::Value::Map(mut fields) = serde_cbor::value::to_value(value).unwrap() else {
        panic!("expected ordinary map")
    };
    fields
        .remove(&serde_cbor::Value::Text(name.to_owned()))
        .expect("declared field")
}

#[test]
fn declared_order_hash_clone_traits_and_option_defaults_remain_available() {
    fn ordered<T: Serialize + for<'a> Deserialize<'a> + Debug + Eq + Ord + Hash + Clone>() {}
    ordered::<TakeCanisterSnapshotArgs>();
    ordered::<LoadCanisterSnapshotArgs>();
    ordered::<InstallCodeArgs>();
    ordered::<ProvisionalCreateCanisterWithCyclesArgs>();
    ordered::<CanisterLogRecord>();
    ordered::<FetchCanisterLogsResult>();
    let default = ProvisionalCreateCanisterWithCyclesArgs::default();
    assert_eq!(default.amount, None);
    assert_eq!(default.settings, None);
    assert_eq!(default.specified_id, None);
    assert_eq!(default.sender_canister_version, None);
    assert_eq!(
        serde_json::from_str::<ProvisionalCreateCanisterWithCyclesArgs>("{}").unwrap(),
        default
    );
    assert_eq!(
        serde_json::to_value(&default)
            .unwrap()
            .as_object()
            .unwrap()
            .len(),
        4
    );
    let first: TakeCanisterSnapshotArgs =
        serde_json::from_value(serde_json::json!({"canister_id": Principal::anonymous()})).unwrap();
    assert_eq!(first.replace_snapshot, None);
    assert_eq!(first.uninstall_code, None);
    assert_eq!(first.sender_canister_version, None);
    let mut later = first.clone();
    later.replace_snapshot = Some(Vec::new());
    assert!(first < later);
    let mut left = DefaultHasher::new();
    let mut right = DefaultHasher::new();
    first.hash(&mut left);
    first.clone().hash(&mut right);
    assert_eq!(left.finish(), right.finish());
    let first = CanisterLogRecord {
        idx: 0,
        timestamp_nanos: u64::MAX,
        content: vec![255],
    };
    let second = CanisterLogRecord {
        idx: 1,
        timestamp_nanos: 0,
        content: vec![],
    };
    assert!(first < second);
}

#[test]
fn read_and_upload_byte_serde_asymmetries_remain_exact() {
    assert_eq!(
        cbor_field(&read_metadata(), "certified_data"),
        serde_cbor::Value::Bytes(vec![0, 255, 128])
    );
    assert!(
        matches!(cbor_field(&upload_metadata(), "certified_data"), serde_cbor::Value::Array(values) if values.len() == 3)
    );
    let read = ReadCanisterSnapshotDataResult {
        chunk: vec![0, 255],
    };
    assert_eq!(
        cbor_field(&read, "chunk"),
        serde_cbor::Value::Bytes(vec![0, 255])
    );
    let upload = UploadCanisterSnapshotDataArgs {
        canister_id: Principal::anonymous(),
        snapshot_id: vec![0, 255],
        kind: SnapshotDataOffset::WasmChunk,
        chunk: vec![0, 255],
    };
    assert!(
        matches!(cbor_field(&upload, "chunk"), serde_cbor::Value::Array(values) if values.len() == 2)
    );
    assert!(
        matches!(cbor_field(&upload, "snapshot_id"), serde_cbor::Value::Array(values) if values.len() == 2)
    );
    let install = install(CanisterInstallMode::Install);
    assert!(
        matches!(cbor_field(&install, "wasm_module"), serde_cbor::Value::Array(values) if values.len() == 2)
    );
    assert_eq!(
        cbor_field(&install, "arg"),
        serde_cbor::Value::Bytes(vec![128, 1])
    );
    let log = CanisterLogRecord {
        idx: 0,
        timestamp_nanos: 0,
        content: vec![0, 255],
    };
    assert_eq!(
        cbor_field(&log, "content"),
        serde_cbor::Value::Bytes(vec![0, 255])
    );
    assert_eq!(
        cbor_field(&ChunkHash { hash: vec![0, 255] }, "hash"),
        serde_cbor::Value::Bytes(vec![0, 255])
    );
    let serde_cbor::Value::Map(fields) = cbor_field(
        &SnapshotDataKind::WasmChunk { hash: vec![0, 255] },
        "wasm_chunk",
    ) else {
        panic!("variant fields")
    };
    assert_eq!(
        fields.get(&serde_cbor::Value::Text("hash".to_owned())),
        Some(&serde_cbor::Value::Bytes(vec![0, 255]))
    );
}

#[test]
fn floating_global_bits_survive_candid_without_canonicalization() {
    for bits in [
        0,
        0x8000_0000,
        0x7f80_0000,
        0xff80_0000,
        0x7fc1_2345,
        0xffc0_0042,
    ] {
        let value = SnapshotMetadataGlobal::F32(f32::from_bits(bits));
        let bytes = encode_one(&value, 4096).unwrap();
        let decoded: SnapshotMetadataGlobal = decode_one(&bytes, limits(bytes.len())).unwrap();
        let SnapshotMetadataGlobal::F32(value) = decoded else {
            panic!("f32 variant")
        };
        assert_eq!(value.to_bits(), bits);
    }
    for bits in [
        0,
        0x8000_0000_0000_0000,
        0x7ff0_0000_0000_0000,
        0xfff0_0000_0000_0000,
        0x7ff8_1234_5678_9abc,
        0xfff8_0000_0000_0042,
    ] {
        let value = SnapshotMetadataGlobal::F64(f64::from_bits(bits));
        let bytes = encode_one(&value, 4096).unwrap();
        let decoded: SnapshotMetadataGlobal = decode_one(&bytes, limits(bytes.len())).unwrap();
        let SnapshotMetadataGlobal::F64(value) = decoded else {
            panic!("f64 variant")
        };
        assert_eq!(value.to_bits(), bits);
    }
}

#[test]
fn optional_metadata_fields_deserialize_absent_and_nat_is_unbounded() {
    let mut json = serde_json::to_value(read_metadata()).unwrap();
    for name in ["source", "global_timer", "on_low_wasm_memory_hook_status"] {
        json.as_object_mut().unwrap().remove(name);
    }
    let restored: ReadCanisterSnapshotMetadataResult = serde_json::from_value(json).unwrap();
    assert!(restored.source.is_none());
    assert!(restored.global_timer.is_none());
    assert!(restored.on_low_wasm_memory_hook_status.is_none());
    let value = SnapshotMetadataGlobal::V128(huge());
    let bytes = encode_one(&value, 4096).unwrap();
    let SnapshotMetadataGlobal::V128(restored) = decode_one(&bytes, limits(bytes.len())).unwrap()
    else {
        panic!("v128 magnitude")
    };
    assert_eq!(restored, huge());
    let mut json = serde_json::to_value(upload_metadata()).unwrap();
    for name in [
        "replace_snapshot",
        "global_timer",
        "on_low_wasm_memory_hook_status",
    ] {
        json.as_object_mut().unwrap().remove(name);
    }
    let restored: UploadCanisterSnapshotMetadataArgs = serde_json::from_value(json).unwrap();
    assert!(restored.replace_snapshot.is_none());
    assert!(restored.global_timer.is_none());
    assert!(restored.on_low_wasm_memory_hook_status.is_none());
    assert!(serde_json::from_str::<ReadCanisterSnapshotMetadataResult>("{}").is_err());
    assert!(serde_json::from_str::<SnapshotDataKind>("\"wasm_chunk\"").is_err());
}

// Serialize finite values and check complete owned-codec roundtrip without
// assuming PartialEq on DTOs that intentionally contain floating-point values.
fn observation<T>(label: &str, value: &T) -> (String, String, Vec<u8>, Vec<u8>)
where
    T: Serialize + Encode + for<'a> Decode<'a> + for<'a> Deserialize<'a>,
{
    let json = serde_json::to_string(value).unwrap();
    let json_decoded: T = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_string(&json_decoded).unwrap(), json);
    let cbor = serde_cbor::to_vec(value).unwrap();
    let cbor_decoded: T = serde_cbor::from_slice(&cbor).unwrap();
    assert_eq!(serde_cbor::to_vec(&cbor_decoded).unwrap(), cbor);
    let candid = encode_one(value, 65_536).unwrap();
    let decoded: T = decode_one(&candid, limits(candid.len())).unwrap();
    assert_eq!(encode_one(&decoded, 65_536).unwrap(), candid);
    assert_eq!(serde_json::to_string(&decoded).unwrap(), json);
    (label.to_owned(), json, cbor, candid)
}

#[test]
fn deterministic_native_management_observations() {
    let id = Principal::anonymous();
    let take = TakeCanisterSnapshotArgs {
        canister_id: id,
        replace_snapshot: Some(vec![0, 255]),
        uninstall_code: Some(false),
        sender_canister_version: Some(u64::MAX),
    };
    let load = LoadCanisterSnapshotArgs {
        canister_id: id,
        snapshot_id: vec![0, 255],
        sender_canister_version: Some(u64::MAX),
    };
    let mut second_source = read_metadata();
    second_source.source = Some(SnapshotSource::MetadataUpload(Reserved));
    let read_args: Vec<_> = kinds()
        .into_iter()
        .map(|kind| ReadCanisterSnapshotDataArgs {
            canister_id: id,
            snapshot_id: vec![0, 255],
            kind,
        })
        .collect();
    let upload_args: Vec<_> = offsets()
        .into_iter()
        .map(|kind| UploadCanisterSnapshotDataArgs {
            canister_id: id,
            snapshot_id: vec![0, 255],
            kind,
            chunk: vec![0, 255, 128],
        })
        .collect();
    let provisional = ProvisionalCreateCanisterWithCyclesArgs {
        amount: Some(huge()),
        settings: Some(CanisterSettings {
            controllers: Some(vec![id, Principal::management_canister()]),
            reserved_cycles_limit: Some(huge()),
            ..Default::default()
        }),
        specified_id: Some(Principal::management_canister()),
        sender_canister_version: Some(u64::MAX),
    };
    let logs = vec![
        CanisterLogRecord {
            idx: 0,
            timestamp_nanos: 0,
            content: vec![],
        },
        CanisterLogRecord {
            idx: u64::MAX,
            timestamp_nanos: u64::MAX,
            content: vec![0, 255, 128],
        },
    ];
    let observations = vec![
        observation("take", &take),
        observation("load", &load),
        observation(
            "read-metadata-args",
            &ReadCanisterSnapshotMetadataArgs {
                canister_id: id,
                snapshot_id: vec![0, 255],
            },
        ),
        observation(
            "read-metadata-results-both-sources",
            &vec![read_metadata(), second_source],
        ),
        observation("globals-all-variants", &globals()),
        observation(
            "timers-all-variants",
            &vec![
                CanisterTimer::Inactive,
                CanisterTimer::Active(0),
                CanisterTimer::Active(u64::MAX),
            ],
        ),
        observation(
            "hook-status-all-variants",
            &vec![
                OnLowWasmMemoryHookStatus::ConditionNotSatisfied,
                OnLowWasmMemoryHookStatus::Ready,
                OnLowWasmMemoryHookStatus::Executed,
            ],
        ),
        observation("read-data-args", &read_args),
        observation("data-kind-all-variants", &kinds()),
        observation(
            "read-data-result",
            &ReadCanisterSnapshotDataResult {
                chunk: vec![0, 255, 128],
            },
        ),
        observation("upload-metadata-args", &upload_metadata()),
        observation(
            "upload-metadata-result",
            &UploadCanisterSnapshotMetadataResult {
                snapshot_id: vec![0, 255],
            },
        ),
        observation("upload-data-args", &upload_args),
        observation("data-offset-all-variants", &offsets()),
        observation(
            "install-all-modes",
            &vec![
                install(CanisterInstallMode::Install),
                install(CanisterInstallMode::Reinstall),
                install(CanisterInstallMode::Upgrade(None)),
            ],
        ),
        observation(
            "provisional-default-and-present",
            &vec![
                ProvisionalCreateCanisterWithCyclesArgs::default(),
                provisional,
            ],
        ),
        observation("log-records", &logs),
        observation(
            "fetch-logs",
            &FetchCanisterLogsResult {
                canister_log_records: logs,
            },
        ),
    ];
    let transcript = serde_json::to_string(&observations).unwrap();
    assert!(
        transcript.len() < 30_000,
        "reviewable native fixture: {} bytes",
        transcript.len()
    );
    println!("MANAGEMENT_NATIVE_OBSERVATIONS={transcript}");
    assert_eq!(
        transcript,
        include_str!("fixtures/management-native-observations.json").trim_end(),
        "native management JSON, CBOR and Candid representations changed"
    );
}
