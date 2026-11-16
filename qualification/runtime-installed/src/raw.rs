//! Installed synchronous exports without any executor or generated entrypoint.
use candid::Principal;
use ic_management_canister_types::{CanisterSettings, CanisterStatusType};
use ic_testkit::{
    pic::PocketIcStartupConfig,
    pocket_ic::{PocketIcBuilder, nonblocking::PocketIc},
};
use std::{path::PathBuf, time::Duration};

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

pub(super) fn run(server_binary: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let wasm =
        std::fs::read("target/runtime-raw/wasm32-unknown-unknown/release/runtime_exports.wasm")?;
    tokio::runtime::Builder::new_current_thread().enable_all().build()?.block_on(async {
        let owned = PocketIcStartupConfig::Spawn {
            server_binary, timeout:Duration::from_secs(30), server_hard_ttl:Some(Duration::from_secs(120)),
            server_idle_ttl:None, server_output_files:None,
        }.build_instance(PocketIcBuilder::new().with_application_subnet()).await?;
        let pic = owned.instance();
        let original = Principal::anonymous();
        let replacement = Principal::from_slice(&[1,2,3]);
        let id = pic.create_canister().await;
        pic.add_cycles(id, 1u128 << 70).await;
        pic.install_canister(id, wasm.clone(), vec![], None).await;
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
        println!("raw installed: offset/empty replies, query/update identity and counters, live controller changes, trap/reply rollback, stopped upgrade passed");
        Ok::<(), Box<dyn std::error::Error>>(())
    })
}
