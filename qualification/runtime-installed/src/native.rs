use candid::Principal;
use ic_management_canister_types::{CanisterIdRecord, CanisterStatusType};
use ic_testkit::{
    pic::{PocketIcStartupConfig, server_assets},
    pocket_ic::{ErrorCode, PocketIcBuilder, common::rest::RawEffectivePrincipal},
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
        }.build_instance(PocketIcBuilder::new().with_application_subnet().with_application_subnet()).await?;
        let pic = owned.instance();
        let canister = pic.create_canister().await;
        pic.add_cycles(canister, 1_u128 << 80).await;
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
        let caller_subnet = pic.get_subnet(canister).await.unwrap();
        let peer = pic.create_canister_on_subnet(None, None, caller_subnet).await;
        pic.add_cycles(peer, 10_000_000_000_000).await;
        pic.install_canister(peer, wasm.clone(), vec![], None).await;
        for (index, (mode, completed, dropped)) in [(0, 1, 1), (1, 1, 2), (2, 1, 3), (0, 2, 4), (3, 2, 4), (0, 3, 5), (4, 4, 6), (0, 5, 7)].into_iter().enumerate() {
            let mut input = vec![mode];
            input.extend_from_slice(peer.as_slice());
            let result = pic.update_call(canister, caller, "await_probe", input).await;
            let peer_count = u8::try_from(index + 1 + usize::from(index >= 6)).unwrap();
            if mode == 0 || mode == 4 {
                assert_eq!(result.expect("resumed handler must reply"), vec![peer_count]);
            } else {
                let rejection = result.expect_err("callback trap must roll back its reply");
                assert_eq!(rejection.error_code, ErrorCode::CanisterTrapped);
            }
            let started = u8::try_from(index + 1).unwrap();
            let mut expected = vec![started, completed, dropped];
            // Every prior handle is re-observed: later success must not turn
            // failed cleanup into a committed termination claim.
            expected.extend_from_slice(&[1, 2, 2, 1, 0, 1, 1, 1][..=index]);
            assert_eq!(pic.update_call(canister, caller, "observe", vec![]).await.unwrap(),
                expected, "committed counters and task lifecycle must match");
            assert_eq!(pic.update_call(peer, caller, "probe", vec![0]).await.unwrap(),
                vec![peer_count], "caller rollback must not undo committed peer effects");
        }
        for (mode, reply_refund, reject_refund) in [
            (0, 17_u128, 29_u128),
            (1, 17, 29),
            (2, 18_446_744_073_709_551_633, 36_893_488_147_419_103_261),
        ] {
            let mut refunds = reply_refund.to_le_bytes().to_vec();
            refunds.extend_from_slice(&reject_refund.to_le_bytes());
            refunds.push(5);
            let mut input = vec![mode];
            input.extend_from_slice(peer.as_slice());
            assert_eq!(pic.update_call(canister, caller, "refund_probe", input).await.unwrap(), refunds);
        }
        assert_eq!(pic.update_call(peer, caller, "probe", vec![0]).await.unwrap(), vec![9],
            "rejected peer update must roll back its increment");
        assert_eq!(pic.update_call(canister, caller, "observe", vec![]).await.unwrap(),
            vec![8, 5, 7, 1, 2, 2, 1, 0, 1, 1, 1], "refund callbacks must not alter retained task states");
        let mut input = vec![0];
        input.extend_from_slice(peer.as_slice());
        assert_eq!(pic.update_call(canister, caller, "resource_probe", input).await.unwrap(), vec![9]);
        assert_eq!(pic.update_call(peer, caller, "probe", vec![0]).await.unwrap(), vec![9],
            "resource refusals must not increment the peer");
        for (mode, expected) in [(0, 10), (1, 10), (2, 10), (0, 11)] {
            let mut input = vec![mode];
            input.extend_from_slice(peer.as_slice());
            let result = pic.update_call(canister, caller, "notify_probe", input).await;
            if mode == 1 {
                assert_eq!(result.expect_err("initial trap must roll back notification").error_code,
                    ErrorCode::CanisterTrapped);
            } else {
                assert_eq!(result.expect("notification receipt does not require remote success"), vec![expected]);
            }
            assert_eq!(pic.update_call(peer, caller, "probe", vec![0]).await.unwrap(), vec![expected]);
        }
        assert_eq!(pic.update_call(canister, caller, "observe", vec![]).await.unwrap(),
            vec![8, 5, 7, 1, 2, 2, 1, 0, 1, 1, 1], "notifications must not change prior task states");
        let query_error = pic.query_call(canister, caller, "refuse_update", vec![]).await
            .expect_err("query intentionally returns without a reply after asserting refusal");
        assert_eq!(query_error.error_code, ErrorCode::CanisterDidNotReply);
        assert_eq!(pic.update_call(canister, caller, "nested_probe", vec![0]).await.unwrap(), b"nested-refused");
        assert_eq!(pic.update_call(canister, caller, "probe", vec![1]).await.unwrap(), vec![3],
            "ordinary update must still work after context refusals");
        for capture in [1, 2] {
            for mode in [0, 1] {
                let mut input = vec![mode];
                input.extend_from_slice(peer.as_slice());
                let expected = if mode == 0 { vec![capture] } else { vec![capture, 2, 11] };
                assert_eq!(pic.update_call(canister, caller, "stale_probe", input).await.unwrap(), expected);
            }
        }
        let remote_subnet = pic.topology().await.get_app_subnets().into_iter()
            .find(|subnet| *subnet != caller_subnet).unwrap();
        let remote = pic.create_canister_on_subnet(None, None, remote_subnet).await;
        pic.install_canister(remote, wasm.clone(), vec![], None).await;
        let mut input = vec![0];
        input.extend_from_slice(remote.as_slice());
        let pending = pic.submit_call(canister, caller, "bounded_probe", input).await.unwrap();
        pic.tick().await;
        pic.advance_time(Duration::from_secs(60)).await;
        assert_eq!(pic.await_call(pending).await.unwrap(), vec![6]);
        let uncertain_peer = pic.update_call(remote, caller, "probe", vec![0]).await.unwrap();
        assert_eq!(uncertain_peer, vec![1], "SYS_UNKNOWN must coexist with a committed remote effect in this corpus");
        let mut input = vec![1];
        input.extend_from_slice(remote.as_slice());
        let subsequent = pic.update_call(canister, caller, "bounded_probe", input).await.unwrap();
        assert_eq!(subsequent, vec![uncertain_peer[0] + 1]);
        assert_eq!(pic.update_call(remote, caller, "probe", vec![0]).await.unwrap(), subsequent);
        println!("bounded wait passed: SYS_UNKNOWN; peer observation {uncertain_peer:?}; subsequent bounded reply {subsequent:?}");
        assert_eq!(pic.update_call(canister, caller, "observe", vec![]).await.unwrap(),
            vec![8, 5, 7, 1, 2, 2, 1, 0, 1, 1, 1], "failed-cleanup handle remains unresolved before upgrade");
        pic.stop_canister(canister, None).await.unwrap();
        assert_eq!(pic.canister_status(canister, None).await.unwrap().status, CanisterStatusType::Stopped);
        pic.upgrade_canister(canister, wasm.clone(), vec![], None).await.unwrap();
        pic.start_canister(canister, None).await.unwrap();
        assert_eq!(pic.update_call(canister, caller, "probe", vec![0]).await.unwrap(), vec![0]);
        assert_eq!(pic.update_call(canister, caller, "observe", vec![]).await.unwrap(), vec![0, 0, 0],
            "heap-only task observations must not survive this fixture's upgrade");
        assert_eq!(pic.update_call(peer, caller, "probe", vec![0]).await.unwrap(), vec![11]);
        assert_eq!(pic.update_call(remote, caller, "probe", vec![0]).await.unwrap(), vec![2],
            "caller upgrade must not undo remote effects");
        let mut input = vec![0];
        input.extend_from_slice(remote.as_slice());
        assert_eq!(pic.update_call(canister, caller, "await_probe", input).await.unwrap(), vec![3]);
        assert_eq!(pic.update_call(canister, caller, "observe", vec![]).await.unwrap(), vec![1, 1, 1, 1],
            "fresh executor task must complete and drop once after upgrade");
        assert_eq!(pic.update_call(remote, caller, "probe", vec![0]).await.unwrap(), vec![3]);
        println!("upgrade passed: heap observations reset; remote effects retained; fresh awaited task completed");
        let mut input = vec![0];
        input.extend_from_slice(peer.as_slice());
        let refusal = pic.update_call(canister, caller, "submission_probe", input).await.unwrap();
        assert_eq!(refusal.len(), 8);
        let accepted = u32::from_le_bytes(refusal[..4].try_into().unwrap());
        let code = u32::from_le_bytes(refusal[4..].try_into().unwrap());
        assert_eq!((accepted, code), (500, 2), "pinned PocketIC queue-refusal corpus changed");
        println!("system submission refusal: accepted={accepted}, raw code={code}");
        for _ in 0..10 { pic.tick().await; }
        let mut input = vec![0];
        input.extend_from_slice(peer.as_slice());
        assert_eq!(pic.update_call(canister, caller, "notify_probe", input).await.unwrap(), vec![12],
            "submission must succeed after the full queue drains");
        assert_eq!(pic.update_call(peer, caller, "probe", vec![0]).await.unwrap(), vec![12]);
        let mut input = vec![5];
        input.extend_from_slice(peer.as_slice());
        let no_reply = pic.update_call(canister, caller, "await_probe", input).await.expect_err("cleaned handler must not reply after orphan callback");
        assert_eq!(no_reply.error_code, ErrorCode::CanisterDidNotReply);
        for _ in 0..10 { pic.tick().await; }
        assert_eq!(pic.update_call(peer, caller, "probe", vec![0]).await.unwrap(), vec![14]);
        assert_eq!(pic.update_call(canister, caller, "observe", vec![]).await.unwrap(), vec![2, 1, 2, 1, 2],
            "late orphan response must not resume or destroy the cleaned task again");
        let mut input = vec![0];
        input.extend_from_slice(peer.as_slice());
        assert_eq!(pic.update_call(canister, caller, "await_probe", input).await.unwrap(), vec![15]);
        assert_eq!(pic.update_call(canister, caller, "observe", vec![]).await.unwrap(), vec![3, 2, 3, 1, 2, 1]);
        let accepting_peer = pic.create_canister_on_subnet(None, None, caller_subnet).await;
        let accepting_wasm = wat::parse_str(include_str!("../accept-cycles.wat"))?;
        pic.install_canister(accepting_peer, accepting_wasm, vec![], None).await;
        let mut input = vec![0];
        input.extend_from_slice(accepting_peer.as_slice());
        let mut expected = 5_u128.to_le_bytes().to_vec();
        expected.extend_from_slice(&12_u128.to_le_bytes());
        expected.extend_from_slice(&29_u128.to_le_bytes());
        assert_eq!(pic.update_call(canister, caller, "partial_refund_probe", input).await.unwrap(), expected);
        println!("partial refund passed: accepted 5 of 17, refunded 12; trapped acceptance refunded all 29");
        let mut input = vec![u8::try_from(remote.as_slice().len()).unwrap()];
        input.extend_from_slice(remote.as_slice());
        input.extend_from_slice(canister.as_slice());
        let in_flight = pic.submit_call(canister, caller, "upgrade_probe", input).await.unwrap();
        let mut started = false;
        for _ in 0..20 {
            if pic.update_call(remote, caller, "upgrade_status", vec![]).await.unwrap() == [1, 0] {
                started = true;
                break;
            }
        }
        assert!(started, "peer must be suspended before stop");
        assert!(pic.ingress_status(in_flight.clone()).await.is_none());
        println!("upgrade: requesting stop; waiting for outstanding callbacks");
        let stop_request = pic.submit_call_with_effective_principal(
            Principal::management_canister(),
            RawEffectivePrincipal::CanisterId(canister.as_slice().to_vec()),
            Principal::anonymous(), "stop_canister",
            candid::encode_one(CanisterIdRecord { canister_id: canister })?,
        ).await.unwrap();
        let mut stopping = false;
        for _ in 0..20 {
            if pic.canister_status(canister, None).await.unwrap().status == CanisterStatusType::Stopping {
                stopping = true;
                break;
            }
        }
        assert!(stopping, "stop must wait for the held call");
        assert!(pic.ingress_status(in_flight.clone()).await.is_none());
        assert!(pic.ingress_status(stop_request.clone()).await.is_none());
        println!("upgrade: Stopping observed; allowing the peer to finish");
        assert_eq!(pic.update_call(remote, caller, "release_upgrade", vec![]).await.unwrap(), b"released");
        pic.await_call(stop_request).await.unwrap();
        assert_eq!(pic.canister_status(canister, None).await.unwrap().status, CanisterStatusType::Stopped);
        assert_eq!(pic.ingress_status(in_flight).await.unwrap().unwrap(), b"released",
            "old handler must complete before upgrade");
        println!("upgrade: Stopped; old response drained; installing");
        pic.upgrade_canister(canister, wasm, vec![], None).await.unwrap();
        assert_eq!(pic.canister_status(canister, None).await.unwrap().status, CanisterStatusType::Stopped);
        pic.start_canister(canister, None).await.unwrap();
        assert_eq!(pic.canister_status(canister, None).await.unwrap().status, CanisterStatusType::Running);
        assert_eq!(pic.update_call(canister, caller, "observe", vec![]).await.unwrap(), vec![0, 0, 0]);
        let mut input = vec![0];
        input.extend_from_slice(peer.as_slice());
        assert_eq!(pic.update_call(canister, caller, "await_probe", input).await.unwrap(), vec![16]);
        assert_eq!(pic.update_call(canister, caller, "observe", vec![]).await.unwrap(), vec![1, 1, 1, 1]);
        println!("upgrade: Running; fresh callback completed after drained upgrade");
        owned.close().await;
        println!("installed rollback passed: 7 immediate updates; 8 awaited calls with state/destructor/peer observations; callback/cleanup traps, dropped call interest, reply/rejection refunds, resource refusals, notify commit/rollback and subsequent success");
        Ok::<(), Box<dyn std::error::Error>>(())
    })
}
