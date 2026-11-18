//! Exact occupied callback slots in a separately instrumented installed artifact.
use candid::Principal;
use ic_management_canister_types::CanisterStatusType;
use ic_testkit::pocket_ic::{ErrorCode, nonblocking::PocketIc};

async fn occupied(pic: &PocketIc, id: Principal) -> [u32; 2] {
    let bytes = pic
        .query_call(id, Principal::anonymous(), "registry_size", vec![])
        .await
        .unwrap();
    let bytes: [u8; 8] = bytes.try_into().unwrap();
    [
        u32::from_le_bytes(bytes[..4].try_into().unwrap()),
        u32::from_le_bytes(bytes[4..].try_into().unwrap()),
    ]
}

pub(super) async fn verify(pic: &PocketIc, wasm: &[u8]) {
    let id = pic.create_canister().await;
    pic.add_cycles(id, 1_u128 << 75).await;
    pic.install_canister(id, wasm.to_vec(), vec![], None).await;
    let peer = pic.create_canister().await;
    pic.add_cycles(peer, 1_u128 << 75).await;
    pic.install_canister(peer, wasm.to_vec(), vec![], None)
        .await;
    assert_eq!(occupied(pic, id).await, [0, 0]);
    // Normal replies, callback trap with successful cleanup, abandoned local
    // interest, and repeated cleanup failure. Count after all replies drain.
    for (mode, slots) in [
        (0, 0),
        (1, 0),
        (4, 0),
        (3, 1),
        (0, 1),
        (3, 2),
        (1, 2),
        (0, 2),
    ] {
        let mut args = vec![mode];
        args.extend_from_slice(peer.as_slice());
        let outcome = pic.update_call(id, peer, "await_probe", args).await;
        if matches!(mode, 1 | 3) {
            assert_eq!(outcome.unwrap_err().error_code, ErrorCode::CanisterTrapped);
        } else {
            outcome.unwrap();
        }
        for _ in 0..10 {
            pic.tick().await;
        }
        assert_eq!(
            occupied(pic, id).await,
            [slots, slots],
            "callback custody after mode {mode}"
        );
    }
    pic.stop_canister(id, None).await.unwrap();
    assert_eq!(
        pic.canister_status(id, None).await.unwrap().status,
        CanisterStatusType::Stopped
    );
    pic.upgrade_canister(id, wasm.to_vec(), vec![], None)
        .await
        .unwrap();
    pic.start_canister(id, None).await.unwrap();
    assert_eq!(occupied(pic, id).await, [0, 0]);

    let caller_subnet = pic.get_subnet(id).await.unwrap();
    let remote = pic
        .topology()
        .await
        .get_app_subnets()
        .into_iter()
        .find(|subnet| *subnet != caller_subnet)
        .unwrap();
    let mut calls = Vec::new();
    for (mode, sender) in [(0, Principal::anonymous()), (1, peer)] {
        let held = pic.create_canister_on_subnet(None, None, remote).await;
        pic.add_cycles(held, 1_u128 << 75).await;
        pic.install_canister(held, wasm.to_vec(), vec![], None)
            .await;
        let mut args = vec![mode];
        args.extend_from_slice(held.as_slice());
        let ingress = pic
            .submit_call(id, sender, "held_probe", args)
            .await
            .unwrap();
        calls.push((held, ingress));
    }
    for (held, _) in &calls {
        for _ in 0..20 {
            if pic
                .update_call(*held, peer, "upgrade_status", vec![])
                .await
                .unwrap()
                == [1, 0]
            {
                break;
            }
            pic.tick().await;
        }
        assert_eq!(
            pic.update_call(*held, peer, "upgrade_status", vec![])
                .await
                .unwrap(),
            [1, 0]
        );
    }
    assert_eq!(occupied(pic, id).await, [2, 2]);
    let (short, expired) = calls.remove(0);
    let (long, live) = calls.remove(0);
    pic.advance_time(std::time::Duration::from_secs(5)).await;
    pic.tick().await;
    assert_eq!(occupied(pic, id).await, [2, 2]);
    pic.advance_time(std::time::Duration::from_secs(55)).await;
    assert_eq!(pic.await_call(expired).await.unwrap(), [6]);
    assert_eq!(occupied(pic, id).await, [1, 1]);
    pic.update_call(short, peer, "release_upgrade", vec![])
        .await
        .unwrap();
    for _ in 0..20 {
        pic.tick().await;
    }
    assert_eq!(
        pic.update_call(short, peer, "upgrade_status", vec![])
            .await
            .unwrap(),
        [1, 1]
    );
    assert_eq!(occupied(pic, id).await, [1, 1]);
    assert_eq!(
        pic.update_call(long, peer, "upgrade_status", vec![])
            .await
            .unwrap(),
        [1, 0]
    );
    pic.update_call(long, peer, "release_upgrade", vec![])
        .await
        .unwrap();
    assert_eq!(pic.await_call(live).await.unwrap(), b"released");
    assert_eq!(occupied(pic, id).await, [0, 0]);
    println!(
        "callback registry: two trapped cleanups retained two slots; stopped upgrade cleared them; late completion preserved the independent live call"
    );
}
