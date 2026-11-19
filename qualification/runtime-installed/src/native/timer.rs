//! Installed timer invocation, callback ownership and committed observations.

use candid::Principal;
use ic_management_canister_types::CanisterStatusType;
use ic_testkit::pocket_ic::PocketIcBuilder;
use std::time::Duration;

pub(super) fn run() -> Result<(), Box<dyn std::error::Error>> {
    let wasm = std::fs::read(
        "target/runtime-rollback/wasm32-unknown-unknown/release/runtime_rollback.wasm",
    )?;
    tokio::runtime::Builder::new_current_thread().enable_all().build()?.block_on(async {
        let owned = PocketIcBuilder::new().with_application_subnet().with_server_url(std::env::var("POCKET_IC_SERVER_URL")?.parse()?).with_max_request_time_ms(Some(30_000)).build_async().await;
        let pic = &owned;
        for (mode, expected) in [(0, [1,1,2]), (1, [1,1,3]), (2, [1,0,1]), (3, [0,0,0]), (4, [0,0,0])] {
            let id = pic.create_canister().await;
            pic.add_cycles(id, 1_u128 << 70).await;
            pic.install_canister(id, wasm.clone(), vec![], None).await;
            let version = pic.update_call(id, Principal::anonymous(), "timer_arm", vec![mode]).await.unwrap();
            pic.advance_time(Duration::from_secs(2)).await;
            for _ in 0..20 { pic.tick().await; }
            let observed = pic.query_call(id, Principal::anonymous(), "timer_observe", vec![]).await.unwrap();
            assert_eq!(&observed[..3], &expected, "timer mode {mode}");
            assert!(u64::from_le_bytes(observed[3..].try_into().unwrap()) >= u64::from_le_bytes(version.as_slice().try_into().unwrap()));
            let stage = [3, 2, 2, 1, 1][usize::from(mode)];
            assert_eq!(pic.query_call(id, Principal::anonymous(), "timer_record", vec![]).await.unwrap(), [1, mode, stage]);
            // A timer task never consumes an unrelated update's reply authority.
            assert_eq!(pic.update_call(id, Principal::anonymous(), "timer_peer", vec![]).await.unwrap(), vec![42]);
            pic.stop_canister(id, None).await.unwrap();
            assert_eq!(pic.canister_status(id, None).await.unwrap().status, CanisterStatusType::Stopped);
            pic.upgrade_canister(id, wasm.clone(), vec![], None).await.unwrap();
            pic.start_canister(id, None).await.unwrap();
            let after = pic.query_call(id, Principal::anonymous(), "timer_observe", vec![]).await.unwrap();
            assert_eq!(pic.query_call(id, Principal::anonymous(), "timer_record", vec![]).await.unwrap(), [2, mode, stage]);
            assert_eq!(&after[..3], &[0,0,0]);
            assert!(u64::from_le_bytes(after[3..].try_into().unwrap()) > u64::from_le_bytes(version.try_into().unwrap()));
        }
        println!("installed timer completion, cleanup, retained uncertainty, disarming and upgrade version passed");
        owned.drop().await;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
}
