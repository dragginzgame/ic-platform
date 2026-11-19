mod accounting;
mod callbacks;
mod cross_subnet;
mod identity;
mod raw;
mod timer;
mod upgrade;

use candid::Principal;
use ic_testkit::pocket_ic::{ErrorCode, PocketIcBuilder};

pub(super) fn run() -> Result<(), Box<dyn std::error::Error>> {
    match std::env::args().nth(1).as_deref() {
        Some("raw") => return raw::run(),
        Some("timer") => return timer::run(),
        None => {}
        _ => return Err("expected no arguments, raw or timer".into()),
    }
    timer::run()?;
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(verify_installed())
}

async fn verify_installed() -> Result<(), Box<dyn std::error::Error>> {
    let wasm = std::fs::read(
        "target/runtime-rollback/wasm32-unknown-unknown/release/runtime_rollback.wasm",
    )?;
    let owned = PocketIcBuilder::new()
        .with_nns_subnet()
        .with_application_subnet()
        .with_application_subnet()
        .with_server_url(std::env::var("POCKET_IC_SERVER_URL")?.parse()?)
        .with_max_request_time_ms(Some(30_000))
        .build_async()
        .await;
    let pic = &owned;
    let canister = pic.create_canister().await;
    pic.add_cycles(canister, 1_u128 << 80).await;
    pic.install_canister(canister, wasm.clone(), vec![], None)
        .await;
    let random = pic
        .update_call(canister, Principal::anonymous(), "randomness", vec![])
        .await
        .unwrap();
    assert_eq!(random.len(), 32, "installed management raw_rand reply");
    identity::verify(pic, canister, [7; 32]).await;
    assert_eq!(
        pic.update_call(canister, Principal::anonymous(), "system_set", vec![11; 32])
            .await
            .unwrap(),
        vec![11; 32]
    );
    assert!(
        pic.update_call(
            canister,
            Principal::anonymous(),
            "system_encode_refusal",
            vec![]
        )
        .await
        .is_err()
    );
    identity::verify(pic, canister, [11; 32]).await;
    assert_eq!(
        pic.update_call(canister, Principal::anonymous(), "system_async", vec![])
            .await
            .unwrap(),
        vec![42]
    );
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
    let caller_subnet = pic.get_subnet(canister).await.unwrap();
    let peer = pic
        .create_canister_on_subnet(None, None, caller_subnet)
        .await;
    pic.add_cycles(peer, 10_000_000_000_000).await;
    pic.install_canister(peer, wasm.clone(), vec![], None).await;
    callbacks::verify(pic, canister, peer).await;
    let remote = cross_subnet::verify(pic, canister, peer, caller_subnet, &wasm).await?;
    upgrade::verify(pic, canister, peer, remote, wasm).await?;
    let observed = std::fs::read(
        "target/runtime-accounting/wasm32-unknown-unknown/release/runtime_rollback.wasm",
    )?;
    accounting::verify(pic, &observed).await;
    owned.drop().await;
    println!(
        "installed rollback passed: 7 immediate updates; 8 awaited calls with state/destructor/peer observations; callback/cleanup traps, dropped call interest, reply/rejection refunds, resource refusals, notify commit/rollback and subsequent success"
    );
    Ok(())
}
