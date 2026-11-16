//! Installed synchronous exports without any executor or generated entrypoint.
use candid::Principal;
use ic_management_canister_types::{CanisterSettings, CanisterStatusType};
use ic_testkit::pocket_ic::{PocketIcBuilder, nonblocking::PocketIc};

async fn observe(pic: &PocketIc, id: Principal, sender: Principal, value: u8, controller: bool) {
    for replicated in [false, true] {
        let reply = if replicated {
            pic.update_call(id, sender, "observe_update", vec![]).await
        } else {
            pic.query_call(id, sender, "observe", vec![]).await
        }
        .unwrap();
        assert_eq!(reply.len(), 18 + sender.as_slice().len());
        assert_eq!(reply[0], value);
        assert_eq!(reply[1], u8::from(controller));
        assert_eq!(&reply[2..reply.len() - 16], sender.as_slice());
        let before =
            u64::from_le_bytes(reply[reply.len() - 16..reply.len() - 8].try_into().unwrap());
        let after = u64::from_le_bytes(reply[reply.len() - 8..].try_into().unwrap());
        assert!(
            before > 0 && after > before,
            "installed instructions must advance"
        );
    }
}

pub(super) fn run() -> Result<(), Box<dyn std::error::Error>> {
    let wasm =
        std::fs::read("target/runtime-raw/wasm32-unknown-unknown/release/runtime_exports.wasm")?;
    tokio::runtime::Builder::new_current_thread().enable_all().build()?.block_on(async {
        let owned = PocketIcBuilder::new().with_application_subnet().with_server_url(std::env::var("POCKET_IC_SERVER_URL")?.parse()?).with_max_request_time_ms(Some(30_000)).build_async().await;
        let pic = &owned;
        let original = Principal::anonymous();
        let replacement = Principal::from_slice(&[1,2,3]);
        let id = pic.create_canister().await;
        pic.add_cycles(id, 1u128 << 70).await;
        pic.install_canister(id, wasm.clone(), vec![], None).await;
        assert_eq!(pic.query_call(id, original, "stable_observe", vec![]).await.unwrap(), [2u64.to_le_bytes().as_slice(), &[0,0,0]].concat());
        assert_eq!(pic.update_call(id, original, "system_probe", vec![0]).await.unwrap(), 2u64.to_le_bytes());
        let before_burn = pic.cycle_balance(id).await;
        assert!(pic.update_call(id, original, "system_probe", vec![1]).await.is_err());
        let after_burn = pic.cycle_balance(id).await;
        assert!(before_burn.saturating_sub(after_burn) < (1u128 << 63), "trapped burn must roll back");
        for mode in 2..=3 { assert!(pic.update_call(id, original, "system_probe", vec![mode]).await.is_err(), "system mode {mode} must refuse"); }
        // IC empty stable reads access no bytes, including an out-of-range offset.
        assert!(pic.update_call(id, original, "system_probe", vec![4]).await.unwrap().is_empty());
        assert_eq!(pic.query_call(id, original, "stable_observe", vec![]).await.unwrap(), [2u64.to_le_bytes().as_slice(), &[0,128,255]].concat());
        assert_eq!(pic.update_call(id, original, "system_probe", vec![5]).await.unwrap(), ((1u128 << 64)+7).to_le_bytes());
        assert_eq!(pic.update_call(id, original, "system_probe", vec![7]).await.unwrap().len(),16);
        observe(pic, id, original, 7, true).await;
        observe(pic, id, replacement, 7, false).await;
        assert!(pic.update_call(id, original, "probe", vec![]).await.unwrap().is_empty());
        assert_eq!(pic.update_call(id, original, "probe", vec![0,0,128,255,7]).await.unwrap(), vec![0,128,255,7]);
        assert!(pic.update_call(id, replacement, "probe", vec![1]).await.is_err());
        assert!(pic.update_call(id, original, "probe", vec![1]).await.unwrap().is_empty());
        observe(pic, id, original, 8, true).await;
        for mode in 2..=8 {
            assert!(pic.update_call(id, original, "probe", vec![mode]).await.is_err(), "mode {mode} must trap");
            observe(pic, id, original, 8, true).await;
        }
        pic.update_canister_settings(id, None, CanisterSettings {controllers:Some(vec![replacement]), ..Default::default()}).await.unwrap();
        observe(pic, id, original, 8, false).await;
        observe(pic, id, replacement, 8, true).await;
        assert!(pic.update_call(id, original, "probe", vec![1]).await.is_err());
        assert!(pic.update_call(id, replacement, "probe", vec![1]).await.unwrap().is_empty());
        observe(pic, id, replacement, 9, true).await;
        pic.stop_canister(id, Some(replacement)).await.unwrap();
        assert_eq!(pic.canister_status(id, Some(replacement)).await.unwrap().status, CanisterStatusType::Stopped);
        pic.upgrade_canister(id, wasm, vec![], Some(replacement)).await.unwrap();
        pic.start_canister(id, Some(replacement)).await.unwrap();
        observe(pic, id, replacement, 9, true).await;
        assert_eq!(pic.query_call(id, replacement, "stable_observe", vec![]).await.unwrap(), [2u64.to_le_bytes().as_slice(), &[0,128,255]].concat());
        let exhausted = pic.create_canister().await;
        pic.add_cycles(exhausted, 1u128 << 70).await;
        pic.install_canister(exhausted, std::fs::read("target/runtime-raw/wasm32-unknown-unknown/release/runtime_exports.wasm")?, vec![], None).await;
        let burned = pic.update_call(exhausted, original, "system_probe", vec![6]).await.unwrap();
        assert!(u128::from_le_bytes(burned.try_into().unwrap()) > (1u128 << 64));
        println!("stable memory, full-width cycle burn, clipping, rollback and HTTP cost checks passed");
        println!("raw installed: offset/empty replies, query/update identity and counters, live controller changes, trap/reply rollback, stopped upgrade passed");
        owned.drop().await;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
}
