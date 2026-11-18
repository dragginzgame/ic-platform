//! Independent creation reply replay and resource refusals on wasm32.
#![forbid(unsafe_code)]
use ic_platform_protocol::{candid::Limits, ledger, management};

fn main() {
    let limits = Limits {
        bytes: 4096,
        header: 1024,
        types: 32,
        work: 16384,
        allocation: 16384,
        vector: 4096,
        skipping: 16384,
    };
    let bytes = include_bytes!("../../fixtures/management-metrics.bin");
    let metrics = management::decode_canister_metrics(bytes, limits).unwrap();
    assert_eq!(
        metrics.cycles_consumed.instructions.0.to_string(),
        "340282366920938463463374607431768211456"
    );
    for end in 0..bytes.len() {
        assert!(management::decode_canister_metrics(&bytes[..end], limits).is_err());
    }

    let bytes = include_bytes!("../../fixtures/ledger-created.bin");
    let limits = Limits {
        bytes: bytes.len(),
        header: 8192,
        types: 128,
        work: 262144,
        allocation: 262144,
        vector: 1024,
        skipping: 8192,
    };
    let ledger::CreateResult::Ok(value) = ledger::decode_created(bytes, limits).unwrap() else {
        panic!()
    };
    assert_eq!(
        value.block_id.0.to_string(),
        "340282366920938463463374607431768211456"
    );
    assert_eq!(value.canister_id.as_slice(), &[9; 29]);
    for end in 0..bytes.len() {
        assert!(ledger::decode_created(&bytes[..end], limits).is_err());
    }
    for select in 0..5 {
        let mut bounded = limits;
        match select {
            0 => bounded.bytes -= 1,
            1 => bounded.header = 0,
            2 => bounded.types = 0,
            3 => bounded.work = 0,
            _ => bounded.allocation = 0,
        };
        assert!(ledger::decode_created(bytes, bounded).is_err());
    }
    retained_root_frames(
        &management::CanisterArgs {
            canister_id: value.canister_id,
        },
        limits,
    );
    let args = management::InstallCodeArgs {
        mode: management::InstallMode::Upgrade(None),
        canister_id: value.canister_id,
        wasm_module: vec![0, 97, 115, 109],
        arg: vec![1, 2, 3],
        sender_canister_version: Some(u64::MAX),
    };
    let encoded = args.encode(4096).unwrap();
    assert_eq!(args.encode(encoded.len()).unwrap(), encoded);
    assert!(args.encode(encoded.len() - 1).is_err());
}

fn retained_root_frames(
    target: &ic_platform_protocol::management_snapshots::CanisterArgs,
    limits: Limits,
) {
    let target = target.canister_id;
    for present in [false, true] {
        let settings = present.then(|| management::CanisterSettings {
            controllers: Some(vec![target]),
            ..Default::default()
        });
        let provisional = management::ProvisionalCreateArgs {
            settings: settings.clone(),
            specified_id: None,
            amount: Some(0u64.into()),
            sender_canister_version: None,
        };
        let expected: &[u8] = if present {
            include_bytes!("../../fixtures/root-provisional-present.bin")
        } else {
            include_bytes!("../../fixtures/root-provisional-absent.bin")
        };
        assert_eq!(provisional.encode(expected.len()).unwrap(), expected);
        assert!(provisional.encode(expected.len() - 1).is_err());
        let create = management::CreateCanisterArgs {
            settings,
            sender_canister_version: Some(u64::MAX),
        };
        let expected: &[u8] = if present {
            include_bytes!("../../fixtures/root-create-present.bin")
        } else {
            include_bytes!("../../fixtures/root-create-absent.bin")
        };
        assert_eq!(create.encode(expected.len()).unwrap(), expected);
        assert!(create.encode(expected.len() - 1).is_err());
        let install = management::InstallChunkedCodeArgs {
            mode: management::InstallMode::Upgrade(None),
            target_canister: target,
            store_canister: present.then_some(target),
            chunk_hashes_list: vec![
                ic_platform_protocol::management_chunks::ChunkHash { hash: vec![1; 32] },
                ic_platform_protocol::management_chunks::ChunkHash { hash: vec![2; 32] },
            ],
            wasm_module_hash: vec![3; 32],
            arg: vec![0, 255, 128],
            sender_canister_version: present.then_some(u64::MAX),
        };
        let expected: &[u8] = if present {
            include_bytes!("../../fixtures/root-chunked-present.bin")
        } else {
            include_bytes!("../../fixtures/root-chunked-absent.bin")
        };
        assert_eq!(install.encode(expected.len()).unwrap(), expected);
        assert!(install.encode(expected.len() - 1).is_err());
    }
    let cmc = ledger::CmcCreationArgs {
        settings: Some(management::CreationSettings {
            controllers: Some(vec![target]),
            ..Default::default()
        }),
        subnet_selection: None,
    };
    assert_eq!(
        cmc.encode(4096).unwrap(),
        include_bytes!("../../fixtures/root-cmc.bin")
    );
    let update = management::UpdateSettingsArgs {
        canister_id: target,
        settings: management::CanisterSettings {
            controllers: Some(vec![target]),
            minimum_incoming_canister_call_cycles: Some(1u64.into()),
            log_memory_limit: Some(2u64.into()),
            status_visibility: Some(management::StatusVisibility::Public),
            ..Default::default()
        },
        sender_canister_version: None,
    };
    assert_eq!(
        update.encode(4096).unwrap(),
        include_bytes!("../../fixtures/root-update.bin")
    );
    let status = include_bytes!("../../fixtures/root-status.bin");
    assert_eq!(
        management::decode_canister_status(status, limits)
            .unwrap()
            .version,
        u64::MAX
    );
    for end in 0..status.len() {
        assert!(management::decode_canister_status(&status[..end], limits).is_err());
    }
    let mut trailing = status.to_vec();
    trailing.push(0);
    assert!(management::decode_canister_status(&trailing, limits).is_err());
    let mut cap = limits;
    cap.bytes = status.len() - 1;
    assert!(management::decode_canister_status(status, cap).is_err());
}
