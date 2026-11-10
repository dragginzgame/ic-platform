//! Specification-only public observations of passive shared management DTOs.

use ic_platform::Principal;
use ic_platform::candid_codec::decoder::{Decode, Limits, decode_one};
use ic_platform::candid_codec::{Encode, Nat, Reserved, encode_one};
use ic_platform::management_canister_types::{
    CanisterIdRecord, CanisterInfoArgs, CanisterInfoResult, CanisterInstallMode, CanisterSettings,
    CanisterStatusResult, CanisterStatusType, Change, ChangeDetails, ChangeOrigin, ChunkHash,
    CodeDeploymentMode, CodeDeploymentRecord, ControllersChangeRecord, CreateCanisterArgs,
    CreationRecord, DefiniteCanisterSettings, DeleteCanisterSnapshotArgs, EcdsaCurve, EcdsaKeyId,
    EcdsaPublicKeyArgs, EcdsaPublicKeyResult, EnvironmentVariable, FromCanisterRecord,
    FromUserRecord, HttpHeader, HttpMethod, HttpRequestArgs, HttpRequestResult,
    InstallChunkedCodeArgs, LogVisibility, MemoryMetrics, QueryStats, SignWithEcdsaArgs,
    SignWithEcdsaResult, Snapshot, SnapshotSource, StatusVisibility, TransformArgs,
    TransformContext, TransformFunc, UninstallCodeArgs, UpdateSettingsArgs, UploadChunkArgs,
};
use serde::de::{IntoDeserializer, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

fn huge() -> Nat {
    "340282366920938463463374607431768211456".parse().unwrap()
}
fn id() -> Principal {
    Principal::anonymous()
}
fn limits(length: usize) -> Limits {
    Limits {
        bytes: length,
        header: 65_536,
        types: 8192,
        work: 2_000_000,
        allocation: 2_000_000,
    }
}

fn settings() -> CanisterSettings {
    CanisterSettings {
        controllers: Some(vec![id(), Principal::management_canister()]),
        compute_allocation: Some(huge()),
        memory_allocation: Some(huge()),
        freezing_threshold: Some(huge()),
        reserved_cycles_limit: Some(huge()),
        minimum_incoming_canister_call_cycles: Some(huge()),
        log_visibility: Some(LogVisibility::AllowedViewers(vec![id()])),
        log_memory_limit: Some(huge()),
        snapshot_visibility: Some(
            serde_json::from_value(serde_json::json!({"allowed_viewers": [id()]})).unwrap(),
        ),
        status_visibility: Some(StatusVisibility::AllowedViewers(vec![id()])),
        wasm_memory_limit: Some(huge()),
        wasm_memory_threshold: Some(huge()),
        environment_variables: Some(vec![EnvironmentVariable {
            name: "\0name".to_owned(),
            value: "unvalidated\nλ".to_owned(),
        }]),
    }
}

fn definite() -> DefiniteCanisterSettings {
    DefiniteCanisterSettings {
        controllers: vec![id()],
        compute_allocation: huge(),
        memory_allocation: huge(),
        freezing_threshold: huge(),
        reserved_cycles_limit: huge(),
        minimum_incoming_canister_call_cycles: huge(),
        log_visibility: LogVisibility::Public,
        log_memory_limit: huge(),
        snapshot_visibility: serde_json::from_str("\"public\"").unwrap(),
        status_visibility: StatusVisibility::Public,
        wasm_memory_limit: huge(),
        wasm_memory_threshold: huge(),
        environment_variables: vec![EnvironmentVariable::default()],
    }
}

fn modes() -> Vec<CanisterInstallMode> {
    let mut keep = CanisterInstallMode::Upgrade(Some(Default::default()));
    if let CanisterInstallMode::Upgrade(Some(flags)) = &mut keep {
        flags.skip_pre_upgrade = Some(false);
        flags.wasm_memory_persistence = Some(serde_json::from_str("\"keep\"").unwrap());
    }
    let mut replace = CanisterInstallMode::Upgrade(Some(Default::default()));
    if let CanisterInstallMode::Upgrade(Some(flags)) = &mut replace {
        flags.skip_pre_upgrade = Some(true);
        flags.wasm_memory_persistence = Some(Default::default());
    }
    vec![
        CanisterInstallMode::Install,
        CanisterInstallMode::Reinstall,
        CanisterInstallMode::Upgrade(None),
        CanisterInstallMode::Upgrade(Some(Default::default())),
        keep,
        replace,
    ]
}

fn changes() -> Vec<Change> {
    let mut details = vec![
        None,
        Some(ChangeDetails::Creation(CreationRecord {
            controllers: vec![id()],
            environment_variables_hash: Some(vec![0, 255]),
        })),
        Some(ChangeDetails::CodeUninstall),
        Some(ChangeDetails::ControllersChange(ControllersChangeRecord {
            controllers: vec![],
        })),
    ];
    for mode in [
        CodeDeploymentMode::Install,
        CodeDeploymentMode::Reinstall,
        CodeDeploymentMode::Upgrade,
    ] {
        details.push(Some(ChangeDetails::CodeDeployment(CodeDeploymentRecord {
            mode,
            module_hash: vec![0, 255],
        })));
    }
    details.push(Some(
        serde_json::from_value(serde_json::json!({"load_snapshot": {
            "from_canister_id": id(), "snapshot_id": [0, 255], "canister_version": u64::MAX,
            "taken_at_timestamp": u64::MAX, "source": {"taken_from_canister": null}
        }}))
        .unwrap(),
    ));
    details.push(Some(serde_json::from_value(serde_json::json!({"rename_canister": {
        "canister_id": id(), "total_num_changes": u64::MAX,
        "rename_to": {"canister_id": Principal::management_canister(), "version": u64::MAX, "total_num_changes": u64::MAX},
        "requested_by": id()
    }})).unwrap()));
    details
        .into_iter()
        .enumerate()
        .map(|(index, details)| Change {
            timestamp_nanos: u64::MAX,
            canister_version: u64::MAX,
            origin: if index % 2 == 0 {
                ChangeOrigin::FromUser(FromUserRecord { user_id: id() })
            } else {
                ChangeOrigin::FromCanister(FromCanisterRecord {
                    canister_id: id(),
                    canister_version: Some(u64::MAX),
                })
            },
            details,
        })
        .collect()
}

fn status() -> CanisterStatusResult {
    CanisterStatusResult {
        status: CanisterStatusType::Running,
        ready_for_migration: true,
        version: u64::MAX,
        settings: definite(),
        module_hash: Some(vec![0, 255]),
        memory_size: huge(),
        memory_metrics: MemoryMetrics {
            wasm_memory_size: huge(),
            stable_memory_size: huge(),
            global_memory_size: huge(),
            wasm_binary_size: huge(),
            custom_sections_size: huge(),
            canister_history_size: huge(),
            wasm_chunk_store_size: huge(),
            snapshots_size: huge(),
            log_memory_store_size: huge(),
        },
        cycles: huge(),
        reserved_cycles: huge(),
        idle_cycles_burned_per_day: huge(),
        query_stats: QueryStats {
            num_calls_total: huge(),
            num_instructions_total: huge(),
            request_payload_bytes_total: huge(),
            response_payload_bytes_total: huge(),
        },
    }
}

fn response() -> HttpRequestResult {
    HttpRequestResult {
        status: huge(),
        headers: vec![HttpHeader {
            name: "\0header".to_owned(),
            value: "unvalidated\nλ".to_owned(),
        }],
        body: vec![0, 255, 128],
    }
}

fn transform() -> TransformContext {
    TransformContext {
        function: TransformFunc::new(Principal::management_canister(), "λ\0".to_owned()),
        context: vec![0, 255, 128],
    }
}

#[test]
fn declared_traits_defaults_and_ordering_remain_available() {
    fn ordered<T: Serialize + for<'a> Deserialize<'a> + Debug + Eq + Ord + Hash + Clone>() {}
    ordered::<CanisterSettings>();
    ordered::<DefiniteCanisterSettings>();
    ordered::<CanisterStatusResult>();
    ordered::<CanisterInfoResult>();
    ordered::<HttpRequestResult>();
    ordered::<EcdsaPublicKeyArgs>();
    ordered::<SignWithEcdsaArgs>();
    ordered::<Snapshot>();
    ordered::<InstallChunkedCodeArgs>();
    fn deserialize_only<T: for<'a> Deserialize<'a> + Debug + Eq + Clone>() {}
    deserialize_only::<HttpRequestArgs>();
    deserialize_only::<TransformContext>();
    deserialize_only::<TransformFunc>();
    fn hashable<T: Hash>() {}
    hashable::<TransformArgs>();
    let defaults = CanisterSettings::default();
    assert!(
        serde_json::to_value(&defaults)
            .unwrap()
            .as_object()
            .unwrap()
            .values()
            .all(serde_json::Value::is_null)
    );
    assert_eq!(
        serde_json::from_str::<CanisterSettings>("{}").unwrap(),
        defaults
    );
    assert_eq!(
        CreateCanisterArgs::default(),
        CreateCanisterArgs {
            settings: None,
            sender_canister_version: None
        }
    );
    assert_eq!(LogVisibility::default(), LogVisibility::Controllers);
    assert_eq!(StatusVisibility::default(), StatusVisibility::Controllers);
    assert_eq!(CanisterInstallMode::default(), CanisterInstallMode::Install);
    assert_eq!(HttpMethod::default(), HttpMethod::GET);
    assert_eq!(EcdsaCurve::default(), EcdsaCurve::Secp256k1);
    assert_eq!(u32::from(EcdsaCurve::Secp256k1), 0);
    assert_eq!(u32::from(EcdsaCurve::Secp256r1), 1);
    assert_eq!(
        EnvironmentVariable::default(),
        EnvironmentVariable {
            name: String::new(),
            value: String::new()
        }
    );
    assert_eq!(
        Snapshot::default(),
        Snapshot {
            id: vec![],
            taken_at_timestamp: 0,
            total_size: 0
        }
    );
    let default_request = HttpRequestArgs::default();
    assert_eq!(default_request.method, HttpMethod::GET);
    assert!(default_request.url.is_empty() && default_request.headers.is_empty());
    assert!(default_request.body.is_none() && default_request.transform.is_none());
    assert!(
        default_request.max_response_bytes.is_none()
            && default_request.is_replicated.is_none()
            && default_request.pricing_version.is_none()
    );
    assert!(LogVisibility::Controllers < LogVisibility::Public);
    assert!(LogVisibility::Public < LogVisibility::AllowedViewers(vec![]));
    assert!(CanisterStatusType::Running < CanisterStatusType::Stopping);
    assert!(CanisterStatusType::Stopping < CanisterStatusType::Stopped);
    assert!(
        CanisterIdRecord {
            canister_id: Principal::from_slice(&[255])
        } < CanisterIdRecord {
            canister_id: Principal::from_slice(&[0, 0])
        }
    );
    let mut left = DefaultHasher::new();
    let mut right = DefaultHasher::new();
    let original = settings();
    original.hash(&mut left);
    original.clone().hash(&mut right);
    assert_eq!(left.finish(), right.finish());
}

fn cbor_field<T: Serialize>(value: &T, name: &str) -> serde_cbor::Value {
    let serde_cbor::Value::Map(mut fields) = serde_cbor::value::to_value(value).unwrap() else {
        panic!("ordinary record map")
    };
    fields
        .remove(&serde_cbor::Value::Text(name.to_owned()))
        .unwrap()
}

#[test]
fn bytes_sequences_and_unvalidated_fields_keep_their_serde_shapes() {
    assert_eq!(
        cbor_field(
            &UploadChunkArgs {
                canister_id: id(),
                chunk: vec![0, 255]
            },
            "chunk"
        ),
        serde_cbor::Value::Bytes(vec![0, 255])
    );
    assert_eq!(
        cbor_field(&response(), "body"),
        serde_cbor::Value::Bytes(vec![0, 255, 128])
    );
    assert!(
        matches!(cbor_field(&status(), "module_hash"), serde_cbor::Value::Array(values) if values.len() == 2)
    );
    let args = EcdsaPublicKeyArgs {
        canister_id: Some(id()),
        derivation_path: vec![vec![], vec![0, 255]],
        key_id: EcdsaKeyId::default(),
    };
    let serde_cbor::Value::Array(paths) = cbor_field(&args, "derivation_path") else {
        panic!("path sequence")
    };
    assert!(matches!(&paths[1], serde_cbor::Value::Array(values) if values.len() == 2));
    assert_eq!(
        cbor_field(
            &EcdsaPublicKeyResult {
                public_key: vec![0, 255],
                chain_code: vec![]
            },
            "public_key"
        ),
        serde_cbor::Value::Bytes(vec![0, 255])
    );
    assert_eq!(
        cbor_field(
            &SignWithEcdsaResult {
                signature: vec![0, 255]
            },
            "signature"
        ),
        serde_cbor::Value::Bytes(vec![0, 255])
    );
    assert_eq!(
        cbor_field(
            &TransformArgs {
                response: response(),
                context: vec![0, 255]
            },
            "context"
        ),
        serde_cbor::Value::Bytes(vec![0, 255])
    );
    assert!(
        matches!(cbor_field(&Snapshot { id: vec![0, 255], taken_at_timestamp: 0, total_size: 0 }, "id"), serde_cbor::Value::Array(values) if values.len() == 2)
    );
    assert!(serde_json::from_str::<EnvironmentVariable>("{}").is_err());
    assert!(serde_json::from_str::<DefiniteCanisterSettings>("{}").is_err());
    assert!(serde_json::from_str::<HttpMethod>("\"GET\"").is_err());
}

struct OwnedBytes(Vec<u8>);
impl<'de> IntoDeserializer<'de, serde::de::value::Error> for OwnedBytes {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self {
        self
    }
}
impl<'de> Deserializer<'de> for OwnedBytes {
    type Error = serde::de::value::Error;
    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_byte_buf(self.0)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_newtype_struct(self)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf
        option unit unit_struct seq tuple tuple_struct map struct enum identifier ignored_any
    }
}

fn request_input() -> serde_json::Value {
    serde_json::json!({"url": "not a URL\0λ", "max_response_bytes": u64::MAX,
        "method": "patch", "headers": [{"name": "\0header", "value": "\nvalue"}],
        "body": [0,255,128], "transform": null, "is_replicated": false, "pricing_version": u32::MAX})
}

#[test]
fn deserialize_only_http_and_transform_remain_usable_without_serialize() {
    let mut request: HttpRequestArgs = serde_json::from_value(request_input()).unwrap();
    assert_eq!(request.method, HttpMethod::PATCH);
    assert_eq!(request.max_response_bytes, Some(u64::MAX));
    assert_eq!(request.body, Some(vec![0, 255, 128]));
    assert_eq!(request.is_replicated, Some(false));
    assert_eq!(request.pricing_version, Some(u32::MAX));
    let mut function = vec![5, 3];
    function.extend_from_slice("λ\0".as_bytes());
    function.extend_from_slice(Principal::management_canister().as_slice());
    let entries = [
        ("function", OwnedBytes(function)),
        ("context", OwnedBytes(vec![0, 255, 128])),
    ];
    let deserializer =
        serde::de::value::MapDeserializer::<_, serde::de::value::Error>::new(entries.into_iter());
    let context = TransformContext::deserialize(deserializer).unwrap();
    assert_eq!(context, transform());
    request.transform = Some(context);
    let bytes = encode_one(&request, 65_536).unwrap();
    assert_eq!(
        decode_one::<HttpRequestArgs>(&bytes, limits(bytes.len())).unwrap(),
        request
    );
    let bytes = encode_one(&transform(), 65_536).unwrap();
    assert_eq!(
        decode_one::<TransformContext>(&bytes, limits(bytes.len())).unwrap(),
        transform()
    );
    let sparse: HttpRequestArgs =
        serde_json::from_value(serde_json::json!({"url":"", "method":"get", "headers":[]}))
            .unwrap();
    assert!(
        sparse.body.is_none() && sparse.transform.is_none() && sparse.pricing_version.is_none()
    );
    assert!(serde_json::from_str::<HttpRequestArgs>("{}").is_err());
}

fn observation<T>(label: &str, value: &T) -> (String, String, Vec<u8>, Vec<u8>)
where
    T: Serialize + Encode + for<'a> Decode<'a> + for<'a> Deserialize<'a>,
{
    let json = serde_json::to_string(value).unwrap();
    let from_json: T = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_string(&from_json).unwrap(), json);
    let cbor = serde_cbor::to_vec(value).unwrap();
    let from_cbor: T = serde_cbor::from_slice(&cbor).unwrap();
    assert_eq!(serde_cbor::to_vec(&from_cbor).unwrap(), cbor);
    let candid = encode_one(value, 131_072).unwrap();
    let decoded: T = decode_one(&candid, limits(candid.len())).unwrap();
    assert_eq!(serde_json::to_string(&decoded).unwrap(), json);
    assert_eq!(encode_one(&decoded, 131_072).unwrap(), candid);
    (label.to_owned(), json, cbor, candid)
}

#[test]
fn deterministic_shared_management_observations() {
    let controllers = CanisterSettings {
        log_visibility: Some(Default::default()),
        snapshot_visibility: Some(Default::default()),
        status_visibility: Some(Default::default()),
        ..Default::default()
    };
    let public = CanisterSettings {
        log_visibility: Some(LogVisibility::Public),
        snapshot_visibility: Some(serde_json::from_str("\"public\"").unwrap()),
        status_visibility: Some(StatusVisibility::Public),
        ..Default::default()
    };
    let chunked = InstallChunkedCodeArgs {
        mode: modes().pop().unwrap(),
        target_canister: id(),
        store_canister: Some(Principal::management_canister()),
        chunk_hashes_list: vec![ChunkHash { hash: vec![0, 255] }],
        wasm_module_hash: vec![128, 1],
        arg: vec![0, 255],
        sender_canister_version: Some(u64::MAX),
    };
    let public_key = EcdsaPublicKeyArgs {
        canister_id: Some(id()),
        derivation_path: vec![vec![], vec![0, 255]],
        key_id: EcdsaKeyId {
            curve: EcdsaCurve::Secp256r1,
            name: "unvalidated\0λ".to_owned(),
        },
    };
    let serialized = vec![
        observation("canister-id", &CanisterIdRecord { canister_id: id() }),
        observation(
            "settings-all-visibility-variants",
            &vec![CanisterSettings::default(), controllers, public, settings()],
        ),
        observation(
            "definite-default-and-present",
            &vec![DefiniteCanisterSettings::default(), definite()],
        ),
        observation(
            "create-optional-fields",
            &vec![
                CreateCanisterArgs::default(),
                CreateCanisterArgs {
                    settings: Some(CanisterSettings::default()),
                    sender_canister_version: Some(u64::MAX),
                },
            ],
        ),
        observation(
            "update-settings",
            &UpdateSettingsArgs {
                canister_id: id(),
                settings: CanisterSettings::default(),
                sender_canister_version: Some(u64::MAX),
            },
        ),
        observation(
            "upload-chunk",
            &UploadChunkArgs {
                canister_id: id(),
                chunk: vec![0, 255],
            },
        ),
        observation("install-modes-and-nested-flags", &modes()),
        observation("install-chunked", &chunked),
        observation(
            "uninstall",
            &UninstallCodeArgs {
                canister_id: id(),
                sender_canister_version: Some(u64::MAX),
            },
        ),
        observation("status", &status()),
        observation(
            "status-types",
            &vec![
                CanisterStatusType::Running,
                CanisterStatusType::Stopping,
                CanisterStatusType::Stopped,
            ],
        ),
        observation(
            "info-args",
            &CanisterInfoArgs {
                canister_id: id(),
                num_requested_changes: Some(u64::MAX),
            },
        ),
        observation(
            "info-all-origins-and-details",
            &CanisterInfoResult {
                total_num_changes: u64::MAX,
                recent_changes: changes(),
                module_hash: Some(vec![0, 255]),
                controllers: vec![id()],
            },
        ),
        observation(
            "http-methods",
            &vec![
                HttpMethod::GET,
                HttpMethod::HEAD,
                HttpMethod::POST,
                HttpMethod::PUT,
                HttpMethod::DELETE,
                HttpMethod::PATCH,
            ],
        ),
        observation(
            "http-response-default-and-present",
            &vec![HttpRequestResult::default(), response()],
        ),
        observation(
            "transform-args",
            &TransformArgs {
                response: response(),
                context: vec![0, 255],
            },
        ),
        observation(
            "ecdsa-curves",
            &vec![EcdsaCurve::Secp256k1, EcdsaCurve::Secp256r1],
        ),
        observation(
            "ecdsa-public-key-args",
            &vec![EcdsaPublicKeyArgs::default(), public_key.clone()],
        ),
        observation(
            "ecdsa-public-key-result",
            &vec![
                EcdsaPublicKeyResult::default(),
                EcdsaPublicKeyResult {
                    public_key: vec![0, 255],
                    chain_code: vec![128, 1],
                },
            ],
        ),
        observation(
            "ecdsa-sign-args",
            &vec![
                SignWithEcdsaArgs::default(),
                SignWithEcdsaArgs {
                    message_hash: vec![0, 255],
                    derivation_path: public_key.derivation_path,
                    key_id: public_key.key_id,
                },
            ],
        ),
        observation(
            "ecdsa-sign-result",
            &vec![
                SignWithEcdsaResult::default(),
                SignWithEcdsaResult {
                    signature: vec![0, 255],
                },
            ],
        ),
        observation(
            "snapshots",
            &vec![
                Snapshot::default(),
                Snapshot {
                    id: vec![0, 255],
                    taken_at_timestamp: u64::MAX,
                    total_size: u64::MAX,
                },
            ],
        ),
        observation(
            "delete-snapshot",
            &DeleteCanisterSnapshotArgs {
                canister_id: id(),
                snapshot_id: vec![0, 255],
            },
        ),
        observation(
            "snapshot-sources",
            &vec![
                SnapshotSource::TakenFromCanister(Reserved),
                SnapshotSource::MetadataUpload(Reserved),
            ],
        ),
    ];
    let mut request: HttpRequestArgs = serde_json::from_value(request_input()).unwrap();
    request.transform = Some(transform());
    let deserialize_only = vec![
        ("http-request", encode_one(&request, 65_536).unwrap()),
        (
            "transform-context",
            encode_one(&transform(), 65_536).unwrap(),
        ),
        (
            "transform-function",
            encode_one(&transform().function, 65_536).unwrap(),
        ),
    ];
    let transcript = serde_json::to_string(&serde_json::json!({"serialized":serialized,"http_request_serde_input":request_input(),"deserialize_only_candid":deserialize_only})).unwrap();
    assert!(
        transcript.len() < 60_000,
        "keep grouped shared fixture finite: {} bytes",
        transcript.len()
    );
    assert_eq!(
        transcript,
        include_str!("fixtures/management-shared-observations.json").trim_end()
    );
    println!("MANAGEMENT_SHARED_OBSERVATIONS={transcript}");
}
