use super::identity;
use candid::Principal;
use ic_management_canister_types::CanisterStatusType;
use ic_testkit::pocket_ic::{ErrorCode, nonblocking::PocketIc};
use std::time::Duration;

pub(super) async fn verify(
    pic: &PocketIc,
    canister: Principal,
    peer: Principal,
    caller_subnet: Principal,
    wasm: &[u8],
) -> Result<Principal, Box<dyn std::error::Error>> {
    let caller = canister;
    let remote_subnet = pic
        .topology()
        .await
        .get_app_subnets()
        .into_iter()
        .find(|subnet| *subnet != caller_subnet)
        .unwrap();
    let remote = pic
        .create_canister_on_subnet(None, None, remote_subnet)
        .await;
    pic.install_canister(remote, wasm.to_vec(), vec![], None)
        .await;
    let mut input = vec![0];
    input.extend_from_slice(remote.as_slice());
    let pending = pic
        .submit_call(canister, caller, "bounded_probe", input)
        .await
        .unwrap();
    pic.tick().await;
    pic.advance_time(Duration::from_secs(60)).await;
    assert_eq!(pic.await_call(pending).await.unwrap(), vec![6]);
    let uncertain_peer = pic
        .update_call(remote, caller, "probe", vec![0])
        .await
        .unwrap();
    assert_eq!(
        uncertain_peer,
        vec![1],
        "SYS_UNKNOWN must coexist with a committed remote effect in this corpus"
    );
    let mut input = vec![1];
    input.extend_from_slice(remote.as_slice());
    let subsequent = pic
        .update_call(canister, caller, "bounded_probe", input)
        .await
        .unwrap();
    assert_eq!(subsequent, vec![uncertain_peer[0] + 1]);
    assert_eq!(
        pic.update_call(remote, caller, "probe", vec![0])
            .await
            .unwrap(),
        subsequent
    );
    println!(
        "bounded wait passed: SYS_UNKNOWN; peer observation {uncertain_peer:?}; subsequent bounded reply {subsequent:?}"
    );
    assert_eq!(
        pic.update_call(canister, caller, "observe", vec![])
            .await
            .unwrap(),
        vec![8, 5, 7, 1, 2, 2, 1, 0, 1, 1, 1],
        "failed-cleanup handle remains unresolved before upgrade"
    );
    pic.stop_canister(canister, None).await.unwrap();
    assert_eq!(
        pic.canister_status(canister, None).await.unwrap().status,
        CanisterStatusType::Stopped
    );
    pic.upgrade_canister(canister, wasm.to_vec(), vec![], None)
        .await
        .unwrap();
    pic.start_canister(canister, None).await.unwrap();
    identity::verify(pic, canister, [9; 32]).await;
    assert_eq!(
        pic.update_call(canister, caller, "probe", vec![0])
            .await
            .unwrap(),
        vec![0]
    );
    assert_eq!(
        pic.update_call(canister, caller, "observe", vec![])
            .await
            .unwrap(),
        vec![0, 0, 0],
        "heap-only task observations must not survive this fixture's upgrade"
    );
    assert_eq!(
        pic.update_call(peer, caller, "probe", vec![0])
            .await
            .unwrap(),
        vec![11]
    );
    assert_eq!(
        pic.update_call(remote, caller, "probe", vec![0])
            .await
            .unwrap(),
        vec![2],
        "caller upgrade must not undo remote effects"
    );
    let mut input = vec![0];
    input.extend_from_slice(remote.as_slice());
    assert_eq!(
        pic.update_call(canister, caller, "await_probe", input)
            .await
            .unwrap(),
        vec![3]
    );
    assert_eq!(
        pic.update_call(canister, caller, "observe", vec![])
            .await
            .unwrap(),
        vec![1, 1, 1, 1],
        "fresh executor task must complete and drop once after upgrade"
    );
    assert_eq!(
        pic.update_call(remote, caller, "probe", vec![0])
            .await
            .unwrap(),
        vec![3]
    );
    println!(
        "upgrade passed: heap observations reset; remote effects retained; fresh awaited task completed"
    );
    let mut input = vec![0];
    input.extend_from_slice(peer.as_slice());
    let refusal = pic
        .update_call(canister, caller, "submission_probe", input)
        .await
        .unwrap();
    assert_eq!(refusal.len(), 8);
    let accepted = u32::from_le_bytes(refusal[..4].try_into().unwrap());
    let code = u32::from_le_bytes(refusal[4..].try_into().unwrap());
    assert_eq!(
        (accepted, code),
        (500, 2),
        "pinned PocketIC queue-refusal corpus changed"
    );
    println!("system submission refusal: accepted={accepted}, raw code={code}");
    for _ in 0..10 {
        pic.tick().await;
    }
    let mut input = vec![0];
    input.extend_from_slice(peer.as_slice());
    assert_eq!(
        pic.update_call(canister, caller, "notify_probe", input)
            .await
            .unwrap(),
        vec![12],
        "submission must succeed after the full queue drains"
    );
    assert_eq!(
        pic.update_call(peer, caller, "probe", vec![0])
            .await
            .unwrap(),
        vec![12]
    );
    let mut input = vec![5];
    input.extend_from_slice(peer.as_slice());
    let no_reply = pic
        .update_call(canister, caller, "await_probe", input)
        .await
        .expect_err("cleaned handler must not reply after orphan callback");
    assert_eq!(no_reply.error_code, ErrorCode::CanisterDidNotReply);
    for _ in 0..10 {
        pic.tick().await;
    }
    assert_eq!(
        pic.update_call(peer, caller, "probe", vec![0])
            .await
            .unwrap(),
        vec![14]
    );
    assert_eq!(
        pic.update_call(canister, caller, "observe", vec![])
            .await
            .unwrap(),
        vec![2, 1, 2, 1, 2],
        "late orphan response must not resume or destroy the cleaned task again"
    );
    let mut input = vec![0];
    input.extend_from_slice(peer.as_slice());
    assert_eq!(
        pic.update_call(canister, caller, "await_probe", input)
            .await
            .unwrap(),
        vec![15]
    );
    assert_eq!(
        pic.update_call(canister, caller, "observe", vec![])
            .await
            .unwrap(),
        vec![3, 2, 3, 1, 2, 1]
    );
    let accepting_peer = pic
        .create_canister_on_subnet(None, None, caller_subnet)
        .await;
    let accepting_wasm = wat::parse_str(include_str!("../../accept-cycles.wat"))?;
    pic.install_canister(accepting_peer, accepting_wasm, vec![], None)
        .await;
    let mut input = vec![0];
    input.extend_from_slice(accepting_peer.as_slice());
    let mut expected = 5_u128.to_le_bytes().to_vec();
    expected.extend_from_slice(&12_u128.to_le_bytes());
    expected.extend_from_slice(&29_u128.to_le_bytes());
    assert_eq!(
        pic.update_call(canister, caller, "partial_refund_probe", input)
            .await
            .unwrap(),
        expected
    );
    println!(
        "partial refund passed: accepted 5 of 17, refunded 12; trapped acceptance refunded all 29"
    );
    Ok(remote)
}
