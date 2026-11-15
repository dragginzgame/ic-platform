use ic_testkit::{
    pic::{PocketIcStartupConfig, server_assets},
    pocket_ic::{ErrorCode, PocketIcBuilder},
};
use std::{path::Path, time::Duration};

pub(super) fn run() -> Result<(), Box<dyn std::error::Error>> {
    let directory = Path::new(server_assets::DEFAULT_DIRECTORY);
    match std::env::args().nth(1).as_deref() {
        Some("setup") => {
            server_assets::setup(directory, || false)?;
            return Ok(());
        }
        Some("check") => {
            server_assets::check(directory)?;
            return Ok(());
        }
        None => {}
        _ => return Err("expected no arguments, setup or check".into()),
    }
    let server_binary = server_assets::check(directory)?;
    let wasm = std::fs::read(
        "target/runtime-rollback/wasm32-unknown-unknown/release/runtime_rollback.wasm",
    )?;
    tokio::runtime::Builder::new_current_thread().enable_all().build()?.block_on(async {
        let owned = PocketIcStartupConfig::Spawn {
            server_binary,
            timeout: Duration::from_secs(30),
            server_hard_ttl: Some(Duration::from_secs(120)),
            server_idle_ttl: None,
            server_output_files: None,
        }.build_instance(PocketIcBuilder::new().with_application_subnet()).await?;
        let pic = owned.instance();
        let canister = pic.create_canister().await;
        pic.add_cycles(canister, 10_000_000_000_000).await;
        pic.install_canister(canister, wasm.clone(), vec![], None).await;
        // PocketIC impersonates this explicit sender; no agent identity crosses
        // into the candidate's Wasm dependency graph.
        let caller = canister;
        for (mode, expected) in [(0, 0), (1, 1), (2, 1), (0, 1), (3, 1), (0, 1), (1, 2)] {
            let result = pic.update_call(canister, caller, "probe", vec![mode]).await;
            if mode >= 2 {
                let rejection = result.expect_err("trapping update must not deliver a reply");
                // Rust's wasm panic ends in `unreachable`, not `ic0.trap`.
                assert_eq!(rejection.error_code, ErrorCode::CanisterTrapped);
            } else {
                assert_eq!(result.expect("update must succeed"), vec![expected]);
            }
        }
        let peer = pic.create_canister().await;
        pic.add_cycles(peer, 10_000_000_000_000).await;
        pic.install_canister(peer, wasm, vec![], None).await;
        for (index, mode) in [0, 1, 2, 0].into_iter().enumerate() {
            let mut input = vec![mode];
            input.extend_from_slice(peer.as_slice());
            let result = pic.update_call(canister, caller, "await_probe", input).await;
            if mode == 0 {
                assert_eq!(result.expect("resumed handler must reply"), vec![u8::try_from(index + 1).unwrap()]);
            } else {
                let rejection = result.expect_err("callback trap must roll back its reply");
                assert_eq!(rejection.error_code, ErrorCode::CanisterTrapped);
            }
            let started = u8::try_from(index + 1).unwrap();
            let completed = if index == 3 { 2 } else { 1 };
            assert_eq!(pic.update_call(canister, caller, "observe", vec![]).await.unwrap(),
                vec![started, completed, started], "cleanup must drop exactly once and callback writes must roll back");
            assert_eq!(pic.update_call(peer, caller, "probe", vec![0]).await.unwrap(),
                vec![started], "caller rollback must not undo committed peer effects");
        }
        owned.close().await;
        println!("installed rollback passed: 7 immediate updates; 4 awaited calls with state/destructor/peer observations; callback traps and subsequent success");
        Ok::<(), Box<dyn std::error::Error>>(())
    })
}
