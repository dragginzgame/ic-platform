use super::maintained as owned;
#[cfg(not(target_family = "wasm"))]
use candid::CandidType;

// The registry schema supplier is selected only in the native graph.
#[cfg(not(target_family = "wasm"))]
#[test]
fn selected_schemas_match_the_pinned_external_contract() {
    macro_rules! compare {
        ($($ty:ident),+ $(,)?) => {
            $(assert_eq!(owned::$ty::_ty(), ic_management_canister_types::$ty::_ty(), stringify!($ty));)+
        };
    }
    compare!(
        CanisterId,
        CanisterIdRecord,
        CanisterInfoArgs,
        CanisterInfoResult,
        CanisterInstallMode,
        CanisterSettings,
        CanisterStatusResult,
        CanisterStatusType,
        Change,
        ChangeDetails,
        ChangeOrigin,
        ChunkHash,
        CodeDeploymentMode,
        CodeDeploymentRecord,
        ControllersChangeRecord,
        CreateCanisterArgs,
        CreationRecord,
        DefiniteCanisterSettings,
        DeleteCanisterSnapshotArgs,
        EcdsaCurve,
        EcdsaKeyId,
        EcdsaPublicKeyArgs,
        EcdsaPublicKeyResult,
        EnvironmentVariable,
        FromCanisterRecord,
        FromUserRecord,
        HttpHeader,
        HttpMethod,
        HttpRequestArgs,
        HttpRequestResult,
        InstallChunkedCodeArgs,
        LoadSnapshotRecord,
        LogVisibility,
        MemoryMetrics,
        QueryStats,
        RenameCanisterRecord,
        RenameToRecord,
        SignWithEcdsaArgs,
        SignWithEcdsaResult,
        Snapshot,
        SnapshotId,
        SnapshotSource,
        SnapshotVisibility,
        StatusVisibility,
        TransformArgs,
        TransformContext,
        TransformFunc,
        UninstallCodeArgs,
        UpdateSettingsArgs,
        UpgradeFlags,
        UploadChunkArgs,
        WasmMemoryPersistence
    );
}

#[cfg(not(target_family = "wasm"))]
#[test]
fn create_canister_preserves_pinned_bytes_and_complete_numeric_ranges() {
    assert_eq!(
        std::any::TypeId::of::<super::CreateCanisterArgs>(),
        std::any::TypeId::of::<ic_management_canister_types::CreateCanisterArgs>()
    );
    let large: candid::Nat = "340282366920938463463374607431768211456".parse().unwrap();
    let settings = owned::CanisterSettings {
        controllers: Some(vec![candid::Principal::anonymous()]),
        reserved_cycles_limit: Some(large),
        environment_variables: Some(vec![owned::EnvironmentVariable {
            name: "fixture".into(),
            value: "preserve\0bytes".into(),
        }]),
        ..Default::default()
    };
    for settings in [
        None,
        Some(owned::CanisterSettings::default()),
        Some(settings),
    ] {
        let upstream_settings: Option<ic_management_canister_types::CanisterSettings> =
            candid::decode_one(&candid::encode_one(&settings).unwrap()).unwrap();
        for version in [None, Some(0), Some(u64::MAX)] {
            let request = owned::CreateCanisterArgs {
                settings: settings.clone(),
                sender_canister_version: version,
            };
            let upstream = ic_management_canister_types::CreateCanisterArgs {
                settings: upstream_settings.clone(),
                sender_canister_version: version,
            };
            let expected = candid::encode_one(&upstream).unwrap();
            assert_eq!(candid::encode_one(&request).unwrap(), expected);
            assert_eq!(
                candid::decode_one::<owned::CreateCanisterArgs>(&expected).unwrap(),
                request
            );
        }
    }
}

fn hex(text: &str) -> Vec<u8> {
    text.trim()
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn owned_records_preserve_independent_history_status_and_snapshot_bytes() {
    macro_rules! roundtrip {
        ($ty:ty, $fixture:literal) => {{
            let bytes = hex(include_str!(concat!("../../../tests/fixtures/", $fixture)));
            let value: $ty = candid::decode_one(&bytes).unwrap();
            assert_eq!(candid::encode_one(value).unwrap(), bytes);
        }};
    }
    roundtrip!(owned::CanisterInfoArgs, "management-info-args.hex");
    roundtrip!(owned::CanisterInfoResult, "management-info-result.hex");
    roundtrip!(
        Vec<owned::CanisterStatusResult>,
        "management-snapshot-status.hex"
    );
    roundtrip!(Vec<owned::Snapshot>, "management-snapshot-list.hex");
    roundtrip!(
        owned::DeleteCanisterSnapshotArgs,
        "management-snapshot-delete.hex"
    );
}

#[test]
fn ecdsa_cost_selectors_preserve_upstream_values() {
    assert_eq!(u32::from(owned::EcdsaCurve::Secp256k1), 0);
    assert_eq!(u32::from(owned::EcdsaCurve::Secp256r1), 1);
}
