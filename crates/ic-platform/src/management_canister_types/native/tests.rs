use super::*;
use crate::candid_codec::{
    Encode,
    decoder::{Decode, decode_one},
    encode_one,
};

#[test]
fn all_native_snapshot_schemas_match_registry_0110() {
    macro_rules! compare {
        ($($name:ident),+ $(,)?) => { $(assert_eq!(
            encode_one(&Vec::<$name>::new(), 65536).unwrap(),
            candid::encode_one(Vec::<ic_management_canister_types::$name>::new()).unwrap(),
            stringify!($name)
        );)+ };
    }
    compare!(
        TakeCanisterSnapshotArgs,
        LoadCanisterSnapshotArgs,
        ReadCanisterSnapshotMetadataArgs,
        ReadCanisterSnapshotMetadataResult,
        SnapshotMetadataGlobal,
        CanisterTimer,
        OnLowWasmMemoryHookStatus,
        ReadCanisterSnapshotDataArgs,
        SnapshotDataKind,
        ReadCanisterSnapshotDataResult,
        UploadCanisterSnapshotMetadataArgs,
        UploadCanisterSnapshotMetadataResult,
        UploadCanisterSnapshotDataArgs,
        SnapshotDataOffset,
        StoredChunksResult,
        InstallCodeArgs,
        ProvisionalCreateCanisterWithCyclesArgs,
        CanisterLogRecord,
        FetchCanisterLogsResult
    );
}

fn check<
    O: Encode + for<'a> Decode<'a> + serde::Serialize,
    R: candid::CandidType + serde::Serialize,
>(
    value: R,
) {
    let bytes = candid::encode_one(&value).unwrap();
    let owned: O = decode_one(&bytes, crate::tests::reference::limits()).unwrap();
    assert_eq!(encode_one(&owned, 65536).unwrap(), bytes);
    assert_eq!(
        serde_cbor::to_vec(&owned).unwrap(),
        serde_cbor::to_vec(&value).unwrap()
    );
}

#[test]
fn native_globals_and_timer_payloads_preserve_external_bits_and_serde() {
    use ic_management_canister_types as upstream;
    for value in [
        upstream::SnapshotMetadataGlobal::I32(i32::MIN),
        upstream::SnapshotMetadataGlobal::I64(i64::MAX),
        upstream::SnapshotMetadataGlobal::F32(-0.0),
        upstream::SnapshotMetadataGlobal::F64(f64::INFINITY),
        upstream::SnapshotMetadataGlobal::F64(f64::from_bits(0x7ff8_0000_0000_0042)),
        upstream::SnapshotMetadataGlobal::V128(
            "340282366920938463463374607431768211456".parse().unwrap(),
        ),
    ] {
        check::<SnapshotMetadataGlobal, _>(value);
    }
    check::<CanisterTimer, _>(upstream::CanisterTimer::Inactive);
    check::<CanisterTimer, _>(upstream::CanisterTimer::Active(u64::MAX));
    for value in [
        upstream::OnLowWasmMemoryHookStatus::ConditionNotSatisfied,
        upstream::OnLowWasmMemoryHookStatus::Ready,
        upstream::OnLowWasmMemoryHookStatus::Executed,
    ] {
        check::<OnLowWasmMemoryHookStatus, _>(value);
    }
}

#[test]
fn native_client_values_preserve_registry_bytes_and_serde() {
    use ic_management_canister_types as upstream;
    for mode in [
        upstream::CanisterInstallMode::Install,
        upstream::CanisterInstallMode::Reinstall,
        upstream::CanisterInstallMode::Upgrade(None),
    ] {
        check::<InstallCodeArgs, _>(upstream::InstallCodeArgs {
            mode,
            canister_id: candid::Principal::anonymous(),
            wasm_module: vec![0, 97, 115, 109, 255],
            arg: vec![0, 255, 17],
            sender_canister_version: Some(u64::MAX),
        });
    }
    check::<ProvisionalCreateCanisterWithCyclesArgs, _>(
        upstream::ProvisionalCreateCanisterWithCyclesArgs::default(),
    );
    check::<ProvisionalCreateCanisterWithCyclesArgs, _>(
        upstream::ProvisionalCreateCanisterWithCyclesArgs {
            amount: Some(u128::MAX.into()),
            settings: Some(upstream::CanisterSettings {
                controllers: Some(vec![candid::Principal::anonymous()]),
                reserved_cycles_limit: Some(u128::MAX.into()),
                ..Default::default()
            }),
            specified_id: Some(candid::Principal::management_canister()),
            sender_canister_version: Some(u64::MAX),
        },
    );
    let log = upstream::CanisterLogRecord {
        idx: u64::MAX,
        timestamp_nanos: u64::MAX,
        content: vec![0, 255, 17],
    };
    check::<CanisterLogRecord, _>(log.clone());
    for canister_log_records in [vec![], vec![log]] {
        check::<FetchCanisterLogsResult, _>(upstream::FetchCanisterLogsResult {
            canister_log_records,
        });
    }
}
