//! Independent upstream Candid/management schemas for selected Host operations.
#![forbid(unsafe_code)]
#![cfg(all(feature = "ledger", not(target_family = "wasm")))]
use candid::{CandidType, Nat, Principal};
use ic_management_canister_types as management_schema;
use ic_platform_protocol::{candid::Limits, ledger, management};
use serde::Deserialize;
fn limits() -> Limits {
    Limits {
        bytes: 2_100_000,
        header: 8192,
        types: 128,
        work: 8_000_000,
        allocation: 4_000_000,
        vector: 2_100_000,
        skipping: 262_144,
    }
}
fn target() -> candidate_types::Principal {
    candidate_types::Principal::from_slice(&[9; 29])
}
fn request<T: CandidType + for<'a> Deserialize<'a>>(
    encode: impl Fn(usize) -> Result<Vec<u8>, management::EncodeError>,
) -> T {
    let bytes = encode(2_100_000).unwrap();
    assert_eq!(encode(bytes.len()).unwrap(), bytes);
    assert_eq!(
        encode(bytes.len() - 1),
        Err(management::EncodeError::OutputLimit)
    );
    candid::decode_one(&bytes).unwrap()
}
#[test]
fn management_creation_settings_install_and_exact_bounds() {
    let create = management::ProvisionalCreateArgs {
        amount: Some("340282366920938463463374607431768211456".parse().unwrap()),
        settings: Some(management::CanisterSettings {
            controllers: Some(vec![target()]),
            ..Default::default()
        }),
        specified_id: Some(target()),
        sender_canister_version: Some(u64::MAX),
    };
    let decoded: management_schema::ProvisionalCreateCanisterWithCyclesArgs =
        request(|max| create.encode(max));
    assert_eq!(
        decoded.amount.unwrap().0.to_string(),
        "340282366920938463463374607431768211456"
    );
    assert_eq!(decoded.sender_canister_version, Some(u64::MAX));
    assert_eq!(
        decoded.specified_id.unwrap().as_slice(),
        target().as_slice()
    );
    assert_eq!(
        decoded.settings.unwrap().controllers.unwrap()[0].as_slice(),
        target().as_slice()
    );
    let update = management::UpdateSettingsArgs {
        canister_id: target(),
        settings: management::CanisterSettings {
            controllers: Some(vec![]),
            compute_allocation: Some(0.into()),
            log_visibility: Some(management::LogVisibility::AllowedViewers(vec![target()])),
            environment_variables: Some(vec![management::EnvironmentVariable {
                name: "key".into(),
                value: "value".into(),
            }]),
            ..Default::default()
        },
        sender_canister_version: None,
    };
    let decoded: management_schema::UpdateSettingsArgs = request(|max| update.encode(max));
    assert_eq!(decoded.settings.controllers, Some(vec![]));
    assert_eq!(decoded.settings.compute_allocation, Some(Nat::from(0_u64)));
    assert_eq!(decoded.sender_canister_version, None);
    assert_eq!(
        decoded.settings.environment_variables.unwrap()[0].value,
        "value"
    );
    assert!(matches!(
        decoded.settings.log_visibility,
        Some(management_schema::LogVisibility::AllowedViewers(_))
    ));
    for (mode, expected) in [
        (
            management::InstallMode::Install,
            management_schema::CanisterInstallMode::Install,
        ),
        (
            management::InstallMode::Reinstall,
            management_schema::CanisterInstallMode::Reinstall,
        ),
        (
            management::InstallMode::Upgrade(None),
            management_schema::CanisterInstallMode::Upgrade(None),
        ),
        (
            management::InstallMode::Upgrade(Some(management::UpgradeFlags {
                skip_pre_upgrade: Some(false),
                wasm_memory_persistence: Some(management::MemoryPersistence::Keep),
            })),
            management_schema::CanisterInstallMode::Upgrade(Some(
                management_schema::UpgradeFlags {
                    skip_pre_upgrade: Some(false),
                    wasm_memory_persistence: Some(management_schema::WasmMemoryPersistence::Keep),
                },
            )),
        ),
    ] {
        let args = management::InstallCodeArgs {
            mode,
            canister_id: target(),
            wasm_module: vec![0xA5; 1024 * 1024],
            arg: vec![0, 255, 128],
            sender_canister_version: None,
        };
        let decoded: management_schema::InstallCodeArgs = request(|max| args.encode(max));
        assert_eq!(decoded.mode, expected);
        assert_eq!(decoded.wasm_module, args.wasm_module);
        assert_eq!(decoded.arg, args.arg);
        assert_eq!(decoded.sender_canister_version, None);
    }
    let reply = candid::encode_one(management_schema::CanisterIdRecord {
        canister_id: Principal::from_slice(target().as_slice()),
    })
    .unwrap();
    assert_eq!(
        management::decode_created(&reply, limits())
            .unwrap()
            .canister_id,
        target()
    );
    for n in 0..reply.len() {
        assert!(management::decode_created(&reply[..n], limits()).is_err());
    }
    assert!(management::decode_empty_response(&candid::encode_one(()).unwrap(), limits()).is_err());
    assert!(management::decode_empty_response(b"DIDL\0\0", limits()).is_ok());
}
#[derive(CandidType, Deserialize)]
struct Cmc {
    settings: Option<management_schema::CanisterSettings>,
    subnet_selection: Option<Selection>,
}
#[derive(CandidType, Deserialize)]
enum Selection {
    Subnet { subnet: Principal },
    Filter { subnet_type: Option<String> },
}
#[derive(CandidType, Deserialize)]
struct Create {
    from_subaccount: Option<Vec<u8>>,
    created_at_time: Option<u64>,
    amount: Nat,
    creation_args: Option<Cmc>,
}
#[test]
fn ledger_creation_preserves_subnet_options_and_wide_amounts() {
    for subaccount in [None, Some([0; 32]), Some([255; 32])] {
        for timestamp in [None, Some(u64::MAX)] {
            for subnet in [
                None,
                Some(ledger::SubnetSelection::Subnet(ledger::ExplicitSubnet {
                    subnet: target(),
                })),
                Some(ledger::SubnetSelection::Filter(ledger::SubnetFilter {
                    subnet_type: Some("application".into()),
                })),
            ] {
                let args = ledger::CreateCanisterArgs {
                    from_subaccount: subaccount,
                    created_at_time: timestamp,
                    amount: "340282366920938463463374607431768211456".parse().unwrap(),
                    creation_args: Some(ledger::CmcCreationArgs {
                        subnet_selection: subnet,
                        settings: Some(management::CreationSettings {
                            controllers: Some(vec![target()]),
                            freezing_threshold: Some(42.into()),
                            ..Default::default()
                        }),
                    }),
                };
                let value: Create = request(|max| args.encode(max));
                assert_eq!(value.from_subaccount, subaccount.map(|s| s.to_vec()));
                assert_eq!(value.created_at_time, timestamp);
                assert_eq!(
                    value.amount.0.to_string(),
                    "340282366920938463463374607431768211456"
                );
                let creation = value.creation_args.unwrap();
                assert_eq!(
                    creation.settings.unwrap().freezing_threshold,
                    Some(Nat::from(42_u64))
                );
                match creation.subnet_selection {
                    None => assert!(args.creation_args.unwrap().subnet_selection.is_none()),
                    Some(Selection::Subnet { subnet }) => {
                        assert_eq!(subnet.as_slice(), target().as_slice())
                    }
                    Some(Selection::Filter { subnet_type }) => {
                        assert_eq!(subnet_type.as_deref(), Some("application"))
                    }
                }
            }
        }
    }
    let args = ledger::CreateCanisterArgs {
        from_subaccount: None,
        created_at_time: None,
        amount: 0.into(),
        creation_args: None,
    };
    let value: Create = request(|max| args.encode(max));
    assert!(value.creation_args.is_none());
}
#[derive(CandidType)]
struct Created {
    block_id: Nat,
    canister_id: Principal,
}
#[derive(CandidType)]
enum Failure {
    InsufficientFunds {
        balance: Nat,
    },
    TooOld,
    CreatedInFuture {
        ledger_time: u64,
    },
    TemporarilyUnavailable,
    Duplicate {
        duplicate_of: Nat,
        canister_id: Option<Principal>,
    },
    FailedToCreate {
        fee_block: Option<Nat>,
        refund_block: Option<Nat>,
        error: String,
    },
    GenericError {
        message: String,
        error_code: Nat,
    },
}
#[test]
fn complete_ledger_outcomes_preserve_duplicates_and_refuse_malformed_frames() {
    let large: Nat = "340282366920938463463374607431768211456".parse().unwrap();
    let success: Result<Created, Failure> = Ok(Created {
        block_id: large.clone(),
        canister_id: Principal::from_slice(target().as_slice()),
    });
    let bytes = candid::encode_one(success).unwrap();
    if std::env::var_os("REGENERATE_LEDGER_FIXTURE").is_some() {
        std::fs::write(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../engine-view-wasm/fixtures/ledger-created.bin"
            ),
            &bytes,
        )
        .unwrap();
    }
    assert_eq!(
        bytes.as_slice(),
        include_bytes!("../../engine-view-wasm/fixtures/ledger-created.bin")
    );
    let ledger::CreateResult::Ok(value) = ledger::decode_created(&bytes, limits()).unwrap() else {
        panic!()
    };
    assert_eq!(value.block_id.0.to_string(), large.0.to_string());
    assert_eq!(value.canister_id, target());
    let errors = [
        Failure::InsufficientFunds {
            balance: large.clone(),
        },
        Failure::TooOld,
        Failure::CreatedInFuture {
            ledger_time: u64::MAX,
        },
        Failure::TemporarilyUnavailable,
        Failure::Duplicate {
            duplicate_of: large.clone(),
            canister_id: None,
        },
        Failure::Duplicate {
            duplicate_of: large.clone(),
            canister_id: Some(Principal::from_slice(target().as_slice())),
        },
        Failure::FailedToCreate {
            fee_block: Some(large.clone()),
            refund_block: None,
            error: "failed".into(),
        },
        Failure::GenericError {
            message: "unknown".into(),
            error_code: large,
        },
    ];
    for (index, error) in errors.into_iter().enumerate() {
        let value: Result<Created, Failure> = Err(error);
        let bytes = candid::encode_one(value).unwrap();
        let decoded = ledger::decode_created(
            &bytes,
            Limits {
                bytes: bytes.len(),
                ..limits()
            },
        )
        .unwrap();
        match (&decoded, index) {
            (ledger::CreateResult::Err(ledger::CreateError::InsufficientFunds(v)), 0) => {
                assert_eq!(
                    v.balance.0.to_string(),
                    "340282366920938463463374607431768211456"
                )
            }
            (ledger::CreateResult::Err(ledger::CreateError::TooOld), 1) => {}
            (ledger::CreateResult::Err(ledger::CreateError::CreatedInFuture(v)), 2) => {
                assert_eq!(v.ledger_time, u64::MAX)
            }
            (ledger::CreateResult::Err(ledger::CreateError::TemporarilyUnavailable), 3) => {}
            (ledger::CreateResult::Err(ledger::CreateError::Duplicate(_)), 4 | 5) => {}
            (ledger::CreateResult::Err(ledger::CreateError::FailedToCreate(v)), 6) => {
                assert_eq!(v.error, "failed");
                assert!(v.refund_block.is_none());
                assert_eq!(
                    v.fee_block.as_ref().unwrap().0.to_string(),
                    "340282366920938463463374607431768211456"
                );
            }
            (ledger::CreateResult::Err(ledger::CreateError::GenericError(v)), 7) => {
                assert_eq!(v.message, "unknown");
                assert_eq!(
                    v.error_code.0.to_string(),
                    "340282366920938463463374607431768211456"
                );
            }
            _ => panic!("changed ledger error"),
        }
        if let ledger::CreateResult::Err(ledger::CreateError::Duplicate(value)) = decoded {
            assert_eq!(
                value.canister_id,
                if index == 4 { None } else { Some(target()) }
            );
            assert_eq!(
                value.duplicate_of.0.to_string(),
                "340282366920938463463374607431768211456"
            );
        }
        assert!(
            ledger::decode_created(
                &bytes,
                Limits {
                    bytes: bytes.len() - 1,
                    ..limits()
                }
            )
            .is_err()
        );
        for n in 0..bytes.len() {
            assert!(ledger::decode_created(&bytes[..n], limits()).is_err());
        }
        let mut trailing = bytes;
        trailing.push(0);
        assert!(ledger::decode_created(&trailing, limits()).is_err());
    }
    #[derive(CandidType)]
    enum Unknown {
        NewError,
    }
    let unknown: Result<Created, Unknown> = Err(Unknown::NewError);
    assert!(ledger::decode_created(&candid::encode_one(unknown).unwrap(), limits()).is_err());
}
#[derive(CandidType, Deserialize)]
struct IcrcAccount {
    owner: Principal,
    subaccount: Option<Vec<u8>>,
}
#[test]
fn canonical_accounts_amounts_and_icp_identity() {
    let mut identities = vec![];
    for subaccount in [None, Some([0; 32]), Some([255; 32])] {
        let account = candidate_types::Account {
            owner: candidate_types::Principal::management_canister(),
            subaccount,
        };
        let decoded: IcrcAccount = request(|max| ledger::encode_balance_account(&account, max));
        assert_eq!(decoded.owner, Principal::management_canister());
        assert_eq!(decoded.subaccount, subaccount.map(|s| s.to_vec()));
        identities.push(ledger::icp_account_identifier(&account));
    }
    assert_eq!(identities[0], identities[1]);
    assert_ne!(identities[0], identities[2]);
    let literal = identities[0]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    assert_eq!(
        literal,
        "2d0e897f7e862d2b57d9bc9ea5c65f9a24ac6c074575f47898314b8d6cb0929d"
    );
    let bytes = candid::encode_one(
        "340282366920938463463374607431768211456"
            .parse::<Nat>()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        ledger::decode_amount(&bytes, limits())
            .unwrap()
            .0
            .to_string(),
        "340282366920938463463374607431768211456"
    );
    assert!(
        ledger::decode_amount(
            &bytes,
            Limits {
                allocation: 0,
                ..limits()
            }
        )
        .is_err()
    );
    assert_eq!(ledger::encode_fee_args(6).unwrap(), b"DIDL\0\0");
    assert!(ledger::encode_fee_args(5).is_err());
    assert!(ledger::decode_amount(b"DIDL\0\0", limits()).is_err());
}

#[test]
fn unselected_fields_share_work_and_skip_budgets() {
    #[derive(CandidType)]
    struct FutureCreated {
        block_id: Nat,
        canister_id: Principal,
        extra: Vec<Vec<u8>>,
    }
    let value: Result<FutureCreated, Failure> = Ok(FutureCreated {
        block_id: 0_u64.into(),
        canister_id: Principal::management_canister(),
        extra: vec![vec![5; 64]; 4],
    });
    let bytes = candid::encode_one(value).unwrap();
    assert!(ledger::decode_created(&bytes, limits()).is_ok());
    assert!(
        ledger::decode_created(
            &bytes,
            Limits {
                skipping: 0,
                ..limits()
            }
        )
        .is_err()
    );
    assert!(
        ledger::decode_created(
            &bytes,
            Limits {
                work: 0,
                ..limits()
            }
        )
        .is_err()
    );
    assert!(ledger::decode_created(&bytes[..bytes.len() - 1], limits()).is_err());
}

#[test]
fn retained_management_requests_match_legacy_bytes() {
    fn frozen(name: &str, bytes: &[u8]) {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../engine-view-wasm/fixtures")
            .join(format!("root-{name}.bin"));
        assert_eq!(bytes, std::fs::read(path).unwrap(), "retained {name}");
    }
    let cmc = ledger::CmcCreationArgs {
        settings: Some(management::CreationSettings {
            controllers: Some(vec![target()]),
            ..Default::default()
        }),
        subnet_selection: None,
    };
    frozen("cmc", &cmc.encode(4096).unwrap());
    for present in [false, true] {
        let settings = present.then(|| management::CanisterSettings {
            controllers: Some(vec![target()]),
            ..Default::default()
        });
        let label = if present { "present" } else { "absent" };
        let provisional = management::ProvisionalCreateArgs {
            settings: settings.clone(),
            specified_id: None,
            amount: Some(0u64.into()),
            sender_canister_version: None,
        };
        frozen(
            &format!("provisional-{label}"),
            &provisional.encode(4096).unwrap(),
        );
        let create = management::CreateCanisterArgs {
            settings,
            sender_canister_version: Some(u64::MAX),
        };
        frozen(&format!("create-{label}"), &create.encode(4096).unwrap());
        let args = management::InstallChunkedCodeArgs {
            mode: management::InstallMode::Upgrade(None),
            target_canister: target(),
            store_canister: present.then_some(target()),
            chunk_hashes_list: vec![
                ic_platform_protocol::management_chunks::ChunkHash { hash: vec![1; 32] },
                ic_platform_protocol::management_chunks::ChunkHash { hash: vec![2; 32] },
            ],
            wasm_module_hash: vec![3; 32],
            arg: vec![0, 255, 128],
            sender_canister_version: present.then_some(u64::MAX),
        };
        let decoded: management_schema::InstallChunkedCodeArgs = request(|n| args.encode(n));
        assert_eq!(decoded.wasm_module_hash, args.wasm_module_hash);
        assert_eq!(decoded.chunk_hashes_list[0].hash, vec![1; 32]);
        frozen(
            if present {
                "chunked-present"
            } else {
                "chunked-absent"
            },
            &args.encode(4096).unwrap(),
        );
    }
    let update = management::UpdateSettingsArgs {
        canister_id: target(),
        settings: management::CanisterSettings {
            controllers: Some(vec![target()]),
            minimum_incoming_canister_call_cycles: Some(1u64.into()),
            log_memory_limit: Some(2u64.into()),
            status_visibility: Some(management::StatusVisibility::Public),
            ..Default::default()
        },
        sender_canister_version: None,
    };
    frozen("update", &update.encode(4096).unwrap());
}

#[derive(CandidType)]
struct ExecutionCharges {
    instructions: Nat,
    other: Vec<bool>,
}
#[derive(CandidType)]
struct Metrics {
    cycles_consumed: ExecutionCharges,
    unrelated: String,
}
#[test]
fn execution_charges_keep_arbitrary_naturals_and_bound_skipped_values() {
    let limits = Limits {
        bytes: 4096,
        header: 1024,
        types: 32,
        work: 16384,
        allocation: 16384,
        vector: 4096,
        skipping: 16384,
    };
    let value = Metrics {
        cycles_consumed: ExecutionCharges {
            instructions: "340282366920938463463374607431768211456".parse().unwrap(),
            other: vec![true, false],
        },
        unrelated: "skipped metric".into(),
    };
    let bytes = candid::encode_one(value).unwrap();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../engine-view-wasm/fixtures/management-metrics.bin");
    if std::env::var_os("REGENERATE_METRICS_CORPUS").is_some() {
        std::fs::write(&path, &bytes).unwrap();
    }
    assert_eq!(std::fs::read(path).unwrap(), bytes);
    let got = management::decode_canister_metrics(&bytes, limits).unwrap();
    assert_eq!(
        got.cycles_consumed.instructions.0.to_string(),
        "340282366920938463463374607431768211456"
    );
    assert!(
        got.cycles_consumed
            .instructions
            .0
            .to_string()
            .parse::<u128>()
            .is_err()
    );
    for n in 0..bytes.len() {
        assert!(management::decode_canister_metrics(&bytes[..n], limits).is_err());
    }
    let mut invalid_skipped = bytes.clone();
    let text = invalid_skipped
        .windows(b"skipped metric".len())
        .position(|w| w == b"skipped metric")
        .unwrap();
    invalid_skipped[text] = 255;
    assert!(management::decode_canister_metrics(&invalid_skipped, limits).is_err());
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(management::decode_canister_metrics(&trailing, limits).is_err());
    for dimension in 0..7 {
        let set = |n| {
            let mut l = limits;
            match dimension {
                0 => l.bytes = n,
                1 => l.header = n,
                2 => l.types = n,
                3 => l.work = n,
                4 => l.allocation = n,
                5 => l.vector = n,
                _ => l.skipping = n,
            };
            l
        };
        let (mut low, mut high) = (0, 16384);
        while low < high {
            let mid = low + (high - low) / 2;
            if management::decode_canister_metrics(&bytes, set(mid)).is_ok() {
                high = mid
            } else {
                low = mid + 1
            }
        }
        management::decode_canister_metrics(&bytes, set(low)).unwrap();
        if low > 0 {
            assert!(management::decode_canister_metrics(&bytes, set(low - 1)).is_err());
        }
    }
    assert!(
        management::decode_canister_metrics(&candid::encode_args(()).unwrap(), limits).is_err()
    );
    assert!(
        management::decode_canister_metrics(&candid::encode_args((1u64, 2u64)).unwrap(), limits)
            .is_err()
    );
    assert!(
        management::decode_canister_metrics(&candid::encode_one(1u64).unwrap(), limits).is_err()
    );
}
