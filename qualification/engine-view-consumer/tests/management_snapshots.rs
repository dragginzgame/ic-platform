//! Original upstream management schemas independently check the owned wire mappings.
#![forbid(unsafe_code)]
#![cfg(all(feature = "management-snapshots", not(target_family = "wasm")))]
use candid::{CandidType, Principal};
use ic_management_canister_types as oracle;
use ic_platform_protocol::{
    candid::{Error, Limits},
    management_snapshots as wire,
};
use serde::Deserialize;

fn limits() -> Limits {
    Limits {
        bytes: 2 * 1024 * 1024,
        header: 65536,
        types: 64,
        work: 8 * 1024 * 1024,
        allocation: 2 * 1024 * 1024,
        vector: 1024 * 1024,
        skipping: 65536,
    }
}
fn target() -> candidate_types::Principal {
    candidate_types::Principal::from_slice(&[255; 29])
}
fn verify_request<T: CandidType + for<'a> Deserialize<'a>>(
    encode: impl Fn(usize) -> Result<Vec<u8>, wire::EncodeError>,
) -> T {
    let bytes = encode(2 * 1024 * 1024).unwrap();
    assert_eq!(encode(bytes.len()).unwrap(), bytes);
    assert_eq!(encode(bytes.len() - 1), Err(wire::EncodeError::OutputLimit));
    candid::decode_one(&bytes).unwrap()
}
#[test]
fn management_request_arity_identity_option_presence_and_limits() {
    let value: oracle::CanisterIdRecord = verify_request(|max| {
        wire::CanisterArgs {
            canister_id: target(),
        }
        .encode(max)
    });
    assert_eq!(value.canister_id.as_slice(), target().as_slice());
    for replacement in [None, Some(vec![]), Some(vec![1; 256])] {
        for uninstall in [None, Some(false), Some(true)] {
            let args = wire::TakeSnapshotArgs {
                canister_id: target(),
                replace_snapshot: replacement.clone(),
                uninstall_code: uninstall,
                sender_canister_version: Some(u64::MAX),
            };
            let parsed: oracle::TakeCanisterSnapshotArgs = verify_request(|max| args.encode(max));
            assert_eq!(parsed.replace_snapshot, replacement);
            assert_eq!(parsed.uninstall_code, uninstall);
            assert_eq!(parsed.sender_canister_version, Some(u64::MAX));
        }
    }
    let args = wire::SnapshotArgs {
        canister_id: target(),
        snapshot_id: vec![9; 256],
    };
    let parsed: oracle::ReadCanisterSnapshotMetadataArgs = verify_request(|max| args.encode(max));
    assert_eq!(parsed.snapshot_id, args.snapshot_id);
    let args = wire::LoadSnapshotArgs {
        canister_id: target(),
        snapshot_id: vec![9; 256],
        sender_canister_version: None,
    };
    let parsed: oracle::LoadCanisterSnapshotArgs = verify_request(|max| args.encode(max));
    assert_eq!(parsed.snapshot_id, args.snapshot_id);
    assert_eq!(parsed.sender_canister_version, None);
}
#[test]
fn data_regions_full_chunks_and_twenty_mebibyte_transfer() {
    for kind in [
        wire::ReadKind::WasmModule(wire::Extent {
            offset: u64::MAX,
            size: u64::MAX,
        }),
        wire::ReadKind::WasmMemory(wire::Extent { offset: 1, size: 2 }),
        wire::ReadKind::StableMemory(wire::Extent { offset: 3, size: 4 }),
        wire::ReadKind::WasmChunk(wire::SnapshotChunk { hash: vec![5; 32] }),
    ] {
        let args = wire::ReadDataArgs {
            canister_id: target(),
            snapshot_id: vec![1],
            kind: kind.clone(),
        };
        let parsed: oracle::ReadCanisterSnapshotDataArgs = verify_request(|max| args.encode(max));
        match (kind, parsed.kind) {
            (
                wire::ReadKind::WasmModule(v),
                oracle::SnapshotDataKind::WasmModule { offset, size },
            )
            | (
                wire::ReadKind::WasmMemory(v),
                oracle::SnapshotDataKind::WasmMemory { offset, size },
            )
            | (
                wire::ReadKind::StableMemory(v),
                oracle::SnapshotDataKind::StableMemory { offset, size },
            ) => {
                assert_eq!(v.offset, offset);
                assert_eq!(v.size, size);
            }
            (wire::ReadKind::WasmChunk(v), oracle::SnapshotDataKind::WasmChunk { hash }) => {
                assert_eq!(v.hash, hash)
            }
            _ => panic!("wrong region"),
        }
    }
    for kind in [
        wire::WriteKind::WasmModule(wire::Offset { offset: u64::MAX }),
        wire::WriteKind::WasmMemory(wire::Offset { offset: 2 }),
        wire::WriteKind::StableMemory(wire::Offset { offset: 3 }),
        wire::WriteKind::WasmChunk,
    ] {
        let args = wire::UploadDataArgs {
            canister_id: target(),
            snapshot_id: vec![3],
            kind,
            chunk: vec![],
        };
        let parsed: oracle::UploadCanisterSnapshotDataArgs = verify_request(|max| args.encode(max));
        assert!(parsed.chunk.is_empty());
        assert_eq!(parsed.snapshot_id, [3]);
    }
    for ordinal in 0..20u64 {
        let data = vec![ordinal as u8; 1024 * 1024];
        let reply = candid::encode_one(oracle::ReadCanisterSnapshotDataResult {
            chunk: data.clone(),
        })
        .unwrap();
        assert_eq!(
            wire::decode_snapshot_data(&reply, limits()).unwrap().chunk,
            data
        );
        let args = wire::UploadDataArgs {
            canister_id: target(),
            snapshot_id: vec![8],
            kind: wire::WriteKind::StableMemory(wire::Offset {
                offset: ordinal * 1024 * 1024,
            }),
            chunk: data,
        };
        let parsed: oracle::UploadCanisterSnapshotDataArgs = verify_request(|max| args.encode(max));
        assert_eq!(parsed.chunk, args.chunk);
        assert!(
            matches!(parsed.kind,oracle::SnapshotDataOffset::StableMemory{offset} if offset==ordinal*1024*1024)
        );
    }
}
fn metadata() -> oracle::ReadCanisterSnapshotMetadataResult {
    oracle::ReadCanisterSnapshotMetadataResult {
        source: Some(oracle::SnapshotSource::TakenFromCanister(candid::Reserved)),
        taken_at_timestamp: u64::MAX,
        wasm_module_size: u64::MAX,
        globals: vec![
            Some(oracle::SnapshotMetadataGlobal::I32(i32::MIN)),
            Some(oracle::SnapshotMetadataGlobal::I64(i64::MIN)),
            Some(oracle::SnapshotMetadataGlobal::F32(f32::from_bits(
                0xffc12345,
            ))),
            Some(oracle::SnapshotMetadataGlobal::F64(f64::from_bits(
                0xfff8123456789abc,
            ))),
            Some(oracle::SnapshotMetadataGlobal::V128(u128::MAX.into())),
            None,
        ],
        wasm_memory_size: 20 * 1024 * 1024,
        stable_memory_size: u64::MAX,
        wasm_chunk_store: vec![oracle::ChunkHash { hash: vec![9; 32] }],
        canister_version: u64::MAX,
        certified_data: vec![3; 32],
        global_timer: Some(oracle::CanisterTimer::Active(u64::MAX)),
        on_low_wasm_memory_hook_status: Some(oracle::OnLowWasmMemoryHookStatus::Ready),
    }
}
#[test]
fn metadata_retains_float_bits_globals_provenance_and_upload_shape() {
    let bytes = candid::encode_one(metadata()).unwrap();
    let value = wire::decode_snapshot_metadata(&bytes, limits()).unwrap();
    assert_eq!(
        value.globals,
        vec![
            Some(wire::Global::I32(i32::MIN)),
            Some(wire::Global::I64(i64::MIN)),
            Some(wire::Global::F32(wire::Float32Bits(0xffc12345))),
            Some(wire::Global::F64(wire::Float64Bits(0xfff8123456789abc))),
            Some(wire::Global::V128(u128::MAX)),
            None
        ]
    );
    assert_eq!(
        value.source,
        Some(wire::SnapshotSource::TakenFromCanister(wire::Reserved))
    );
    let args = wire::UploadMetadataArgs {
        canister_id: target(),
        replace_snapshot: Some(vec![]),
        globals: value.globals.into_iter().flatten().collect(),
        certified_data: value.certified_data,
        global_timer: value.global_timer,
        on_low_wasm_memory_hook_status: value.on_low_wasm_memory_hook_status,
        wasm_module_size: value.wasm_module_size,
        stable_memory_size: value.stable_memory_size,
        wasm_memory_size: value.wasm_memory_size,
    };
    let parsed: oracle::UploadCanisterSnapshotMetadataArgs = verify_request(|max| args.encode(max));
    assert_eq!(parsed.globals.len(), 5);
    assert!(
        matches!(parsed.globals[2],oracle::SnapshotMetadataGlobal::F32(v) if v.to_bits()==0xffc12345)
    );
    assert!(
        matches!(parsed.globals[3],oracle::SnapshotMetadataGlobal::F64(v) if v.to_bits()==0xfff8123456789abc)
    );
    assert_eq!(parsed.replace_snapshot, Some(vec![]));
    assert_eq!(parsed.wasm_memory_size, 20 * 1024 * 1024);
    for end in 0..bytes.len() {
        assert!(wire::decode_snapshot_metadata(&bytes[..end], limits()).is_err());
    }
    let mut trailing = bytes;
    trailing.push(0);
    assert_eq!(
        wire::decode_snapshot_metadata(&trailing, limits()),
        Err(Error::Trailing)
    );
}
#[derive(CandidType, Deserialize)]
enum Status {
    #[serde(rename = "stopped")]
    Stopped,
}
#[derive(CandidType)]
struct Settings {
    controllers: Vec<Principal>,
    discarded: String,
}
#[derive(CandidType)]
struct StatusReply {
    version: u64,
    status: Status,
    settings: Settings,
    module_hash: Option<Vec<u8>>,
}
#[test]
fn selected_status_skips_discarded_values_and_caps_fields() {
    for (count, hash, accepted) in [(10, 32, true), (11, 32, false), (10, 33, false)] {
        let bytes = candid::encode_one(StatusReply {
            version: u64::MAX,
            status: Status::Stopped,
            settings: Settings {
                controllers: vec![Principal::anonymous(); count],
                discarded: "x".repeat(32768),
            },
            module_hash: Some(vec![3; hash]),
        })
        .unwrap();
        let value = wire::decode_canister_status(&bytes, limits());
        if accepted {
            assert_eq!(value.unwrap().settings.controllers.len(), 10);
        } else {
            assert_eq!(value, Err(Error::Limit));
        }
        let mut no_skip = limits();
        no_skip.skipping = 0;
        assert_eq!(
            wire::decode_canister_status(&bytes, no_skip),
            Err(Error::Limit)
        );
    }
}
#[test]
fn independent_snapshot_and_allocation_fields_and_vector_limits() {
    for (count, id_length, accepted) in [(1024, 256, true), (1025, 1, false), (1, 257, false)] {
        let snapshots = vec![
            oracle::Snapshot {
                id: vec![7; id_length],
                taken_at_timestamp: u64::MAX,
                total_size: u64::MAX
            };
            count
        ];
        let bytes = candid::encode_one(&snapshots).unwrap();
        let result = wire::decode_list_snapshots(&bytes, limits());
        if accepted {
            let values = result.unwrap();
            assert_eq!(values.len(), count);
            assert_eq!(values[0].id, snapshots[0].id);
            assert_eq!(values[0].taken_at_timestamp, u64::MAX);
        } else {
            assert_eq!(result, Err(Error::Limit));
        }
    }
    let single = oracle::Snapshot {
        id: vec![1],
        taken_at_timestamp: u64::MAX,
        total_size: 0,
    };
    let bytes = candid::encode_one(single).unwrap();
    assert_eq!(
        wire::decode_take_snapshot(&bytes, limits()).unwrap().id,
        [1]
    );
    for length in [0, 256, 257] {
        let bytes = candid::encode_one(oracle::UploadCanisterSnapshotMetadataResult {
            snapshot_id: vec![1; length],
        })
        .unwrap();
        assert_eq!(
            wire::decode_snapshot_allocation(&bytes, limits()).is_ok(),
            length <= 256
        );
    }
}
#[test]
fn metadata_field_caps_refuse_before_materializing_values() {
    for select in 0..4 {
        let mut value = metadata();
        match select {
            0 => value.globals = vec![None; 4097],
            1 => value.certified_data = vec![0; 33],
            2 => value.wasm_chunk_store = vec![oracle::ChunkHash { hash: vec![] }; 1025],
            _ => value.wasm_chunk_store[0].hash = vec![0; 33],
        }
        let bytes = candid::encode_one(value).unwrap();
        assert_eq!(
            wire::decode_snapshot_metadata(&bytes, limits()),
            Err(Error::Limit)
        );
    }
    let bytes = candid::encode_one(metadata()).unwrap();
    for select in 0..6 {
        let mut bound = limits();
        match select {
            0 => bound.bytes = bytes.len() - 1,
            1 => bound.header = 0,
            2 => bound.types = 0,
            3 => bound.work = 0,
            4 => bound.allocation = 0,
            _ => bound.vector = 0,
        }
        assert_eq!(
            wire::decode_snapshot_metadata(&bytes, bound),
            Err(Error::Limit)
        );
    }
}

#[test]
fn frozen_metadata_corpus_and_empty_result_arity() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../engine-view-wasm/fixtures/snapshot-metadata.bin");
    let bytes = candid::encode_one(metadata()).unwrap();
    if std::env::var_os("REGENERATE_SNAPSHOT_CORPUS").is_some() {
        std::fs::write(&path, &bytes).unwrap();
    }
    assert_eq!(std::fs::read(path).unwrap(), bytes);
    assert_eq!(
        wire::decode_empty_response(&candid::encode_args(()).unwrap(), limits()),
        Ok(())
    );
    assert_eq!(
        wire::decode_empty_response(&candid::encode_one(()).unwrap(), limits()),
        Err(Error::Arity)
    );
    assert_eq!(
        wire::decode_empty_response(b"DIDL\x01\x6d\x7b\x00", limits()),
        Ok(())
    );
    assert_eq!(
        wire::decode_empty_response(b"DIDL\x01\x6d\x01\x00", limits()),
        Err(Error::Malformed)
    );
    assert_eq!(
        wire::decode_empty_response(b"DIDL\x00\x00\x00", limits()),
        Err(Error::Trailing)
    );
}

#[derive(CandidType)]
struct PromotedMetadata<S: CandidType, G: CandidType> {
    source: S,
    globals: Vec<G>,
    canister_version: u64,
    certified_data: Vec<u8>,
    global_timer: (),
    on_low_wasm_memory_hook_status: candid::Reserved,
    wasm_module_size: u64,
    stable_memory_size: u64,
    wasm_chunk_store: Vec<oracle::ChunkHash>,
    taken_at_timestamp: u64,
    wasm_memory_size: u64,
}
fn promoted<S: CandidType, G: CandidType>(source: S, globals: Vec<G>) -> Vec<u8> {
    candid::encode_one(PromotedMetadata {
        source,
        globals,
        canister_version: 0,
        certified_data: vec![],
        global_timer: (),
        on_low_wasm_memory_hook_status: candid::Reserved,
        wasm_module_size: 0,
        stable_memory_size: 0,
        wasm_chunk_store: vec![],
        taken_at_timestamp: 0,
        wasm_memory_size: 0,
    })
    .unwrap()
}
#[derive(CandidType, Deserialize)]
enum WrongSource {
    #[serde(rename = "taken_from_canister")]
    Taken(u8),
}
#[derive(CandidType)]
enum FutureSource {
    Future(candid::Reserved),
}
#[test]
fn known_optional_promotion_never_hides_unknown_values_or_provenance() {
    let bytes = promoted(
        oracle::SnapshotSource::MetadataUpload(candid::Reserved),
        vec![oracle::SnapshotMetadataGlobal::I32(i32::MAX)],
    );
    let value = wire::decode_snapshot_metadata(&bytes, limits()).unwrap();
    assert_eq!(
        value.source,
        Some(wire::SnapshotSource::MetadataUpload(wire::Reserved))
    );
    assert_eq!(value.globals, [Some(wire::Global::I32(i32::MAX))]);
    assert!(value.global_timer.is_none() && value.on_low_wasm_memory_hook_status.is_none());
    let bytes = promoted((), vec![candid::Reserved]);
    let value = wire::decode_snapshot_metadata(&bytes, limits()).unwrap();
    assert!(value.source.is_none());
    assert_eq!(value.globals, [None]);
    for bytes in [
        promoted(WrongSource::Taken(0), vec![()]),
        promoted(FutureSource::Future(candid::Reserved), vec![()]),
        promoted((), vec![true]),
    ] {
        assert_eq!(
            wire::decode_snapshot_metadata(&bytes, limits()),
            Err(Error::Type)
        );
    }
    let mut value = metadata();
    value.globals = vec![Some(oracle::SnapshotMetadataGlobal::V128(
        "340282366920938463463374607431768211456".parse().unwrap(),
    ))];
    assert_eq!(
        wire::decode_snapshot_metadata(&candid::encode_one(value).unwrap(), limits()),
        Err(Error::Limit)
    );
    // Duplicate and reversed selected labels are rejected during table admission.
    for labels in [[1, 1], [1, 0]] {
        let mut malformed = b"DIDL\x01\x6c\x02".to_vec();
        for label in labels {
            malformed.extend([label, 0x78]);
        }
        malformed.extend([1, 0]);
        malformed.extend([0; 16]);
        assert_eq!(
            wire::decode_snapshot_metadata(&malformed, limits()),
            Err(Error::Malformed)
        );
    }
}

#[derive(CandidType)]
struct InventoryExtra {
    id: Vec<u8>,
    total_size: u64,
    taken_at_timestamp: u64,
    extra: Vec<u8>,
}
#[test]
fn inventory_count_limit_does_not_become_a_nested_skip_limit() {
    let bytes = candid::encode_one(vec![InventoryExtra {
        id: vec![1],
        total_size: 0,
        taken_at_timestamp: 0,
        extra: vec![0; 1025],
    }])
    .unwrap();
    assert_eq!(
        wire::decode_list_snapshots(&bytes, limits()).unwrap().len(),
        1
    );
    let mut restricted = limits();
    restricted.vector = 1024;
    assert_eq!(
        wire::decode_list_snapshots(&bytes, restricted),
        Err(Error::Limit)
    );
}

#[test]
fn required_status_version_is_lossless_and_refuses_missing_or_wrong_type() {
    #[derive(CandidType)]
    struct WithoutVersion {
        status: Status,
        settings: Settings,
        module_hash: Option<Vec<u8>>,
    }
    #[derive(CandidType)]
    struct WrongVersion {
        status: Status,
        settings: Settings,
        module_hash: Option<Vec<u8>>,
        version: String,
    }
    let settings = || Settings {
        controllers: vec![],
        discarded: String::new(),
    };
    let missing = candid::encode_one(WithoutVersion {
        status: Status::Stopped,
        settings: settings(),
        module_hash: None,
    })
    .unwrap();
    assert!(wire::decode_canister_status(&missing, limits()).is_err());
    let wrong = candid::encode_one(WrongVersion {
        status: Status::Stopped,
        settings: settings(),
        module_hash: None,
        version: "1".into(),
    })
    .unwrap();
    assert!(wire::decode_canister_status(&wrong, limits()).is_err());
    for version in [0, u64::MAX] {
        let bytes = candid::encode_one(StatusReply {
            status: Status::Stopped,
            settings: settings(),
            module_hash: None,
            version,
        })
        .unwrap();
        assert_eq!(
            wire::decode_canister_status(&bytes, limits())
                .unwrap()
                .version,
            version
        );
        if version == u64::MAX {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../engine-view-wasm/fixtures/root-status.bin");
            if std::env::var_os("REGENERATE_ROOT_STATUS").is_some() {
                std::fs::write(&path, &bytes).unwrap();
            }
            assert_eq!(std::fs::read(path).unwrap(), bytes);
        }
        for end in 0..bytes.len() {
            assert!(wire::decode_canister_status(&bytes[..end], limits()).is_err());
        }
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert_eq!(
            wire::decode_canister_status(&trailing, limits()),
            Err(Error::Trailing)
        );
        let mut cap = limits();
        cap.bytes = bytes.len();
        assert!(wire::decode_canister_status(&bytes, cap).is_ok());
        cap.bytes -= 1;
        assert_eq!(wire::decode_canister_status(&bytes, cap), Err(Error::Limit));
    }
}
