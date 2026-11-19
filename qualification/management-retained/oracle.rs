use ic_platform::{
    Principal,
    candid_codec::arguments::{EncodeArgs, encode_args},
    management_canister_types as m,
};

fn save<T: EncodeArgs>(name: &str, value: &T) {
    std::fs::write(
        format!("qualification/engine-view-wasm/fixtures/root-{name}.bin"),
        encode_args(value, 2_100_000).unwrap(),
    )
    .unwrap();
}

fn main() {
    let p = Principal::from_slice(&[9; 29]);
    for present in [false, true] {
        let settings = present.then(|| m::CanisterSettings {
            controllers: Some(vec![p]),
            ..Default::default()
        });
        let label = if present { "present" } else { "absent" };
        save(
            &format!("provisional-{label}"),
            &(m::ProvisionalCreateCanisterWithCyclesArgs {
                settings: settings.clone(),
                specified_id: None,
                amount: Some(0u64.into()),
                sender_canister_version: None,
            },),
        );
        save(
            &format!("create-{label}"),
            &(m::CreateCanisterArgs {
                settings,
                sender_canister_version: Some(u64::MAX),
            },),
        );
    }
    save(
        "update",
        &(m::UpdateSettingsArgs {
            canister_id: p,
            settings: m::CanisterSettings {
                controllers: Some(vec![p]),
                minimum_incoming_canister_call_cycles: Some(1u64.into()),
                log_memory_limit: Some(2u64.into()),
                status_visibility: Some(m::StatusVisibility::Public),
                ..Default::default()
            },
            sender_canister_version: None,
        },),
    );
    for present in [false, true] {
        save(
            if present {
                "chunked-present"
            } else {
                "chunked-absent"
            },
            &(m::InstallChunkedCodeArgs {
                mode: m::CanisterInstallMode::Upgrade(None),
                target_canister: p,
                store_canister: present.then_some(p),
                chunk_hashes_list: vec![
                    m::ChunkHash { hash: vec![1; 32] },
                    m::ChunkHash { hash: vec![2; 32] },
                ],
                wasm_module_hash: vec![3; 32],
                arg: vec![0, 255, 128],
                sender_canister_version: present.then_some(u64::MAX),
            },),
        );
    }

    use ic_platform::cycles_ledger_types as l;
    save(
        "cmc",
        &(l::CmcCreateCanisterArgs {
            settings: Some(l::CanisterSettings {
                controllers: Some(vec![p]),
                compute_allocation: None,
                memory_allocation: None,
                freezing_threshold: None,
                reserved_cycles_limit: None,
                wasm_memory_limit: None,
                wasm_memory_threshold: None,
                log_visibility: None,
                environment_variables: None,
            }),
            subnet_selection: None,
        },),
    );
}
