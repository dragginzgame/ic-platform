//! Platform-only consumer of external management DTOs and Candid formats.
use ic_platform::Principal;
use ic_platform::candid::{DecodeError, DecodeLimits, contract, decode_exactly_one};
use ic_platform::management_canister_types::{
    CanisterIdRecord, CanisterInfoArgs, CanisterInfoResult, CanisterStatusResult,
    CanisterStatusType, CanisterTimer, ChangeDetails, ChunkHash, CodeDeploymentMode,
    DefiniteCanisterSettings, DeleteCanisterSnapshotArgs, HttpHeader, HttpMethod, HttpRequestArgs,
    HttpRequestResult, LoadCanisterSnapshotArgs, MemoryMetrics, OnLowWasmMemoryHookStatus,
    QueryStats, ReadCanisterSnapshotDataArgs, ReadCanisterSnapshotDataResult,
    ReadCanisterSnapshotMetadataArgs, ReadCanisterSnapshotMetadataResult, Snapshot,
    SnapshotDataKind, SnapshotDataOffset, SnapshotMetadataGlobal, SnapshotSource, StatusVisibility,
    TakeCanisterSnapshotArgs, TransformArgs, TransformContext, TransformFunc,
    UploadCanisterSnapshotDataArgs, UploadCanisterSnapshotMetadataArgs,
    UploadCanisterSnapshotMetadataResult,
};

/// Preserve the probe's request bytes, transform references and caller admission.
pub fn roundtrip() {
    let limits = DecodeLimits {
        reply_bytes: 4096,
        decoding_quota: 1_000_000,
        skipping_quota: 10_000,
        type_entries: 64,
        header_bytes: 2048,
    };
    let mut request = HttpRequestArgs {
        url: "https://perf-context.invalid/0".into(),
        max_response_bytes: Some(256),
        method: HttpMethod::GET,
        headers: vec![],
        body: None,
        transform: None,
        is_replicated: None,
        pricing_version: Some(1),
    };
    // Original 0.11.0 owner, before Platform adaptation: the existing Canic probe
    // request. Captured independently under target/management-upstream/.
    let original_hex = concat!(
        "4449444c0e6c08efd6e40271e1edeb4a01e8d6d8930102a2f5ed880403ecdaccac0405c6",
        "a4a198060a90f8f6fc090c9fc593ca0f0d6b069681ba027fcfc5d5027fa0d2aca8047fe0",
        "88f2d2047fab80e3d6067fc88ddcea0b7f6e786e046d7b6e066c0298d6caa20107efabde",
        "cb01046a0108010901016c02efabdecb010481ddb2900a096c03b2ceef2f7da2f5ed8804",
        "04c6a4a198060a6d0b6c02f1fee18d0371cbe4fdc704716e7e6e7901001e68747470733a",
        "2f2f706572662d636f6e746578742e696e76616c69642f30000100010000000000000000",
        "00000101000000",
    );
    let original: Vec<u8> = original_hex
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect();
    assert_eq!(contract::encode_one(&request).unwrap(), original);
    assert_eq!(
        decode_exactly_one::<HttpRequestArgs>(&original, limits).unwrap(),
        request
    );
    request.headers.push(HttpHeader {
        name: "accept".into(),
        value: "application/octet-stream".into(),
    });
    request.body = Some(vec![0, 128, 255]);
    request.is_replicated = Some(false);
    request.transform = Some(TransformContext {
        function: TransformFunc::new(Principal::anonymous(), "transform".into()),
        context: vec![255, 0],
    });
    for method in [
        HttpMethod::GET,
        HttpMethod::HEAD,
        HttpMethod::POST,
        HttpMethod::PUT,
        HttpMethod::DELETE,
        HttpMethod::PATCH,
    ] {
        request.method = method;
        let bytes = contract::encode_one(&request).unwrap();
        assert_eq!(
            decode_exactly_one::<HttpRequestArgs>(&bytes, limits).unwrap(),
            request
        );
    }
    let result = HttpRequestResult {
        status: contract::Nat::from(200_u16),
        headers: request.headers,
        body: vec![0, 128, 255],
    };
    let bytes = contract::encode_one(&result).unwrap();
    assert_eq!(
        decode_exactly_one::<HttpRequestResult>(&bytes, limits).unwrap(),
        result
    );
    assert!(matches!(
        decode_exactly_one::<HttpRequestResult>(
            &bytes,
            DecodeLimits {
                reply_bytes: bytes.len() - 1,
                ..limits
            }
        ),
        Err(DecodeError::ReplyTooLarge { .. })
    ));
    let transform = TransformArgs {
        response: result,
        context: vec![255, 0],
    };
    let bytes = contract::encode_one(&transform).unwrap();
    assert_eq!(
        decode_exactly_one::<TransformArgs>(&bytes, limits).unwrap(),
        transform
    );
}

#[test]
fn management_http_wire_and_transform_roundtrips() {
    roundtrip();
}

fn wire_hex(text: &str) -> Vec<u8> {
    let (pairs, remainder) = text.trim().as_bytes().as_chunks::<2>();
    assert!(remainder.is_empty());
    pairs
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

/// Preserve Blob's bounded history request and the original owner's change records.
pub fn history_roundtrip() {
    let limits = DecodeLimits {
        reply_bytes: 65_536,
        decoding_quota: 2_000_000,
        skipping_quota: 1024,
        type_entries: 128,
        header_bytes: 4096,
    };
    // Encoded once by upstream 0.11.0, without Platform or copied DTO schemas.
    let request_wire = wire_hex(include_str!("fixtures/management-info-args.hex"));
    let request = CanisterInfoArgs {
        canister_id: Principal::anonymous(),
        num_requested_changes: Some(20),
    };
    assert_eq!(contract::encode_one(&request).unwrap(), request_wire);
    assert_eq!(
        decode_exactly_one::<CanisterInfoArgs>(&request_wire, limits).unwrap(),
        request
    );
    let history_wire = wire_hex(include_str!("fixtures/management-info-result.hex"));
    let history: CanisterInfoResult = decode_exactly_one(&history_wire, limits).unwrap();
    assert_eq!(contract::encode_one(&history).unwrap(), history_wire);
    assert_eq!(history.total_num_changes, 9);
    assert_eq!(history.recent_changes.len(), 9);
    assert_eq!(history.controllers, [Principal::anonymous()]);
    assert_eq!(history.module_hash, Some(vec![0xab; 32]));
    for (i, change) in history.recent_changes.iter().enumerate() {
        assert_eq!(change.canister_version, i as u64 + 1);
        assert_eq!(change.timestamp_nanos, i as u64 + 100);
    }
    let changes = &history.recent_changes;
    assert!(matches!(
        changes[0].details,
        Some(ChangeDetails::Creation(_))
    ));
    assert!(matches!(
        changes[1].details,
        Some(ChangeDetails::CodeUninstall)
    ));
    for (index, mode) in [
        CodeDeploymentMode::Install,
        CodeDeploymentMode::Reinstall,
        CodeDeploymentMode::Upgrade,
    ]
    .into_iter()
    .enumerate()
    {
        match &changes[index + 2].details {
            Some(ChangeDetails::CodeDeployment(deployment)) => {
                assert_eq!(deployment.mode, mode);
                assert_eq!(deployment.module_hash, [0xab; 32]);
            }
            other => panic!("expected deployment, got {other:?}"),
        }
    }
    assert!(matches!(
        changes[5].details,
        Some(ChangeDetails::ControllersChange(_))
    ));
    assert!(matches!(
        changes[6].details,
        Some(ChangeDetails::LoadSnapshot(_))
    ));
    assert!(matches!(
        changes[7].details,
        Some(ChangeDetails::RenameCanister(_))
    ));
    assert!(changes[8].details.is_none());
    assert!(
        decode_exactly_one::<CanisterInfoResult>(&history_wire[..history_wire.len() - 1], limits)
            .is_err()
    );
    let extra = contract::encode_args((&history, 0_u8)).unwrap();
    assert!(decode_exactly_one::<CanisterInfoResult>(&extra, limits).is_err());
    assert!(matches!(
        decode_exactly_one::<CanisterInfoResult>(
            &history_wire,
            DecodeLimits {
                reply_bytes: history_wire.len() - 1,
                ..limits
            }
        ),
        Err(DecodeError::ReplyTooLarge { .. })
    ));
}

#[test]
fn management_history_preserves_wire_and_change_variants() {
    history_roundtrip();
}

fn snapshot_value<T>(hex: &str, skipping_quota: usize) -> T
where
    T: contract::CandidType + for<'de> contract::serde::Deserialize<'de>,
{
    let bytes = wire_hex(hex);
    let limits = DecodeLimits {
        reply_bytes: 65_536,
        decoding_quota: 16_000_000,
        skipping_quota,
        type_entries: 256,
        header_bytes: 16_384,
    };
    let value: T = decode_exactly_one(&bytes, limits)
        .unwrap_or_else(|error| panic!("{}: {error}", std::any::type_name::<T>()));
    assert_eq!(contract::encode_one(&value).unwrap(), bytes);
    assert!(decode_exactly_one::<T>(&bytes[..bytes.len() - 1], limits).is_err());
    let extra = contract::encode_args((&value, 0_u8)).unwrap();
    assert!(decode_exactly_one::<T>(&extra, limits).is_err());
    assert!(matches!(
        decode_exactly_one::<T>(
            &bytes,
            DecodeLimits {
                reply_bytes: bytes.len() - 1,
                ..limits
            }
        ),
        Err(DecodeError::ReplyTooLarge { .. })
    ));
    value
}

/// Preserve external snapshot/status identities and original-owner wire vectors.
pub fn snapshot_status_roundtrip() {
    macro_rules! original {
        ($ty:ty, $name:literal) => {
            original!($ty, $name, 0)
        };
        ($ty:ty, $name:literal, $skipping:literal) => {
            snapshot_value::<$ty>(
                include_str!(concat!("fixtures/management-snapshot-", $name, ".hex")),
                $skipping,
            )
        };
    }
    let target = original!(CanisterIdRecord, "target");
    assert_eq!(target.canister_id, Principal::anonymous());
    let take = original!(TakeCanisterSnapshotArgs, "take");
    assert_eq!(take.replace_snapshot, Some(vec![0, 255, 17]));
    assert_eq!(take.uninstall_code, Some(false));
    assert_eq!(take.sender_canister_version, Some(u64::MAX));
    let load = original!(LoadCanisterSnapshotArgs, "load");
    assert_eq!(load.snapshot_id, [0, 255, 17]);
    let delete = original!(DeleteCanisterSnapshotArgs, "delete");
    assert_eq!(delete.canister_id, target.canister_id);
    assert_eq!(delete.snapshot_id, load.snapshot_id);
    let empty_delete = original!(DeleteCanisterSnapshotArgs, "delete-empty");
    assert_eq!(empty_delete.canister_id, target.canister_id);
    assert!(empty_delete.snapshot_id.is_empty());
    let snapshots = original!(Vec<Snapshot>, "list");
    assert_eq!(snapshots[0].taken_at_timestamp, u64::MAX);
    let read = original!(ReadCanisterSnapshotMetadataArgs, "read-metadata");
    assert_eq!(read.snapshot_id, load.snapshot_id);
    // Upstream Reserved source payloads consume skipping work even in a full DTO.
    // Backup's stricter zero-skipping policy must retain this refusal.
    let metadata = original!(Vec<ReadCanisterSnapshotMetadataResult>, "metadata", 1024);
    let bytes = contract::encode_one(&metadata).unwrap();
    assert!(matches!(
        decode_exactly_one::<Vec<ReadCanisterSnapshotMetadataResult>>(
            &bytes,
            DecodeLimits {
                reply_bytes: 65_536,
                decoding_quota: 16_000_000,
                skipping_quota: 0,
                type_entries: 256,
                header_bytes: 16_384,
            }
        ),
        Err(DecodeError::Candid(_))
    ));
    assert!(matches!(
        metadata[0].source,
        Some(SnapshotSource::TakenFromCanister(_))
    ));
    assert!(matches!(
        metadata[1].source,
        Some(SnapshotSource::MetadataUpload(_))
    ));
    assert!(metadata[2].source.is_none());
    let chunks: &[ChunkHash] = &metadata[0].wasm_chunk_store;
    assert_eq!(chunks[0].hash, [0xab; 32]);
    let globals: &[Option<SnapshotMetadataGlobal>] = &metadata[0].globals;
    assert!(matches!(
        globals[0],
        Some(SnapshotMetadataGlobal::I32(i32::MIN))
    ));
    assert!(matches!(
        globals[1],
        Some(SnapshotMetadataGlobal::I64(i64::MAX))
    ));
    assert!(
        matches!(globals[2], Some(SnapshotMetadataGlobal::F32(value)) if value.to_bits() == (-0.0_f32).to_bits())
    );
    assert!(matches!(
        globals[3],
        Some(SnapshotMetadataGlobal::F64(1.25))
    ));
    assert!(
        matches!(&globals[4], Some(SnapshotMetadataGlobal::V128(value)) if *value == u128::MAX)
    );
    assert!(globals[5].is_none());
    assert!(matches!(
        metadata[0].global_timer,
        Some(CanisterTimer::Inactive)
    ));
    assert!(matches!(
        metadata[1].global_timer,
        Some(CanisterTimer::Active(u64::MAX))
    ));
    assert!(metadata[2].global_timer.is_none());
    assert!(matches!(
        metadata[0].on_low_wasm_memory_hook_status,
        Some(OnLowWasmMemoryHookStatus::ConditionNotSatisfied)
    ));
    assert!(matches!(
        metadata[1].on_low_wasm_memory_hook_status,
        Some(OnLowWasmMemoryHookStatus::Ready)
    ));
    assert!(matches!(
        metadata[2].on_low_wasm_memory_hook_status,
        Some(OnLowWasmMemoryHookStatus::Executed)
    ));

    let reads = original!(Vec<ReadCanisterSnapshotDataArgs>, "read-data");
    assert!(matches!(
        reads[0].kind,
        SnapshotDataKind::WasmModule { offset: 0, size: 8 }
    ));
    assert!(matches!(
        reads[1].kind,
        SnapshotDataKind::WasmMemory { offset: 3, size: 4 }
    ));
    assert!(matches!(
        reads[2].kind,
        SnapshotDataKind::StableMemory { offset: 5, size: 6 }
    ));
    assert!(matches!(&reads[3].kind, SnapshotDataKind::WasmChunk { hash } if *hash == [0xab; 32]));
    let data = original!(ReadCanisterSnapshotDataResult, "data");
    assert_eq!(data.chunk, [0, 128, 255]);
    let upload = original!(UploadCanisterSnapshotMetadataArgs, "upload-metadata");
    assert_eq!(upload.globals.len(), 5);
    assert_eq!(upload.replace_snapshot, take.replace_snapshot);
    let uploaded = original!(UploadCanisterSnapshotMetadataResult, "uploaded");
    assert_eq!(uploaded.snapshot_id, load.snapshot_id);
    let writes = original!(Vec<UploadCanisterSnapshotDataArgs>, "upload-data");
    assert!(matches!(
        writes[0].kind,
        SnapshotDataOffset::WasmModule { offset: 0 }
    ));
    assert!(matches!(
        writes[1].kind,
        SnapshotDataOffset::WasmMemory { offset: 3 }
    ));
    assert!(matches!(
        writes[2].kind,
        SnapshotDataOffset::StableMemory { offset: 5 }
    ));
    assert!(matches!(writes[3].kind, SnapshotDataOffset::WasmChunk));
    assert!(writes.iter().all(|write| write.chunk == data.chunk));

    let statuses = original!(Vec<CanisterStatusResult>, "status");
    for (status, expected) in statuses.iter().zip([
        CanisterStatusType::Running,
        CanisterStatusType::Stopping,
        CanisterStatusType::Stopped,
    ]) {
        assert_eq!(status.status, expected);
        assert_eq!(status.cycles, contract::Nat::from(u128::MAX));
        let settings: &DefiniteCanisterSettings = &status.settings;
        assert_eq!(settings.controllers, [Principal::anonymous()]);
        let memory: &MemoryMetrics = &status.memory_metrics;
        assert_eq!(memory.snapshots_size, contract::Nat::from(8_u64));
        let stats: &QueryStats = &status.query_stats;
        assert_eq!(stats.num_calls_total, contract::Nat::from(1_u64));
    }
    assert!(matches!(
        statuses[0].settings.status_visibility,
        StatusVisibility::Controllers
    ));
    assert!(matches!(
        statuses[1].settings.status_visibility,
        StatusVisibility::Public
    ));
    assert!(
        matches!(&statuses[2].settings.status_visibility, StatusVisibility::AllowedViewers(viewers) if *viewers == [Principal::anonymous()])
    );
}

#[test]
fn management_snapshot_status_preserves_wire_and_variants() {
    snapshot_status_roundtrip();
}
