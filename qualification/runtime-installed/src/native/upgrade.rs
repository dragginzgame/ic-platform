use candid::Principal;
use ic_management_canister_types::{CanisterIdRecord, CanisterStatusType};
use ic_testkit::pocket_ic::{
    ErrorCode, common::rest::RawEffectivePrincipal, nonblocking::PocketIc,
};

pub(super) async fn verify(
    pic: &PocketIc,
    canister: Principal,
    peer: Principal,
    remote: Principal,
    wasm: Vec<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
    let caller = canister;
    let mut input = vec![u8::try_from(remote.as_slice().len()).unwrap()];
    input.extend_from_slice(remote.as_slice());
    input.extend_from_slice(canister.as_slice());
    let in_flight = pic
        .submit_call(canister, caller, "upgrade_probe", input)
        .await
        .unwrap();
    let mut started = false;
    for _ in 0..20 {
        if pic
            .update_call(remote, caller, "upgrade_status", vec![])
            .await
            .unwrap()
            == [1, 0]
        {
            started = true;
            break;
        }
    }
    assert!(started, "peer must be suspended before stop");
    assert!(pic.ingress_status(in_flight.clone()).await.is_none());
    println!("upgrade: requesting stop; waiting for outstanding callbacks");
    let stop_request = pic
        .submit_call_with_effective_principal(
            Principal::management_canister(),
            RawEffectivePrincipal::CanisterId(canister.as_slice().to_vec()),
            Principal::anonymous(),
            "stop_canister",
            candid::encode_one(CanisterIdRecord {
                canister_id: canister,
            })?,
        )
        .await
        .unwrap();
    let mut stopping = false;
    for _ in 0..20 {
        if pic.canister_status(canister, None).await.unwrap().status == CanisterStatusType::Stopping
        {
            stopping = true;
            break;
        }
    }
    assert!(stopping, "stop must wait for the held call");
    assert!(pic.ingress_status(in_flight.clone()).await.is_none());
    assert!(pic.ingress_status(stop_request.clone()).await.is_none());
    println!("upgrade: Stopping observed; allowing the peer to finish");
    assert_eq!(
        pic.update_call(remote, caller, "release_upgrade", vec![])
            .await
            .unwrap(),
        b"released"
    );
    pic.await_call(stop_request).await.unwrap();
    assert_eq!(
        pic.canister_status(canister, None).await.unwrap().status,
        CanisterStatusType::Stopped
    );
    assert_eq!(
        pic.ingress_status(in_flight).await.unwrap().unwrap(),
        b"released",
        "old handler must complete before upgrade"
    );
    println!("upgrade: Stopped; old response drained; installing");
    pic.upgrade_canister(canister, wasm, vec![], None)
        .await
        .unwrap();
    assert_eq!(
        pic.canister_status(canister, None).await.unwrap().status,
        CanisterStatusType::Stopped
    );
    pic.start_canister(canister, None).await.unwrap();
    assert_eq!(
        pic.canister_status(canister, None).await.unwrap().status,
        CanisterStatusType::Running
    );
    assert_eq!(
        pic.update_call(canister, caller, "observe", vec![])
            .await
            .unwrap(),
        vec![0, 0, 0]
    );
    let mut input = vec![0];
    input.extend_from_slice(peer.as_slice());
    assert_eq!(
        pic.update_call(canister, caller, "await_probe", input)
            .await
            .unwrap(),
        vec![16]
    );
    assert_eq!(
        pic.update_call(canister, caller, "observe", vec![])
            .await
            .unwrap(),
        vec![1, 1, 1, 1]
    );
    println!("upgrade: Running; fresh callback completed after drained upgrade");
    let mut input = vec![6];
    input.extend_from_slice(peer.as_slice());
    let missing = pic
        .update_call(canister, caller, "await_probe", input)
        .await
        .expect_err("manual handler must not receive an automatic reply");
    assert_eq!(missing.error_code, ErrorCode::CanisterDidNotReply);
    assert_eq!(
        pic.update_call(canister, caller, "observe", vec![])
            .await
            .unwrap(),
        vec![2, 2, 2, 1, 1],
        "missing reply still completes and destroys the task"
    );
    assert_eq!(
        pic.update_call(canister, caller, "observe_custody", vec![])
            .await
            .unwrap(),
        vec![0, 0]
    );
    assert_eq!(
        pic.update_call(peer, caller, "probe", vec![0])
            .await
            .unwrap(),
        vec![17],
        "missing reply must not roll back the remote effect"
    );
    assert_eq!(
        pic.update_call(canister, caller, "observe_dispatch", vec![])
            .await
            .unwrap(),
        vec![1, 1],
        "observing the peer reply is distinct from replying to the original caller"
    );
    println!("manual reply omission passed: DidNotReply, Completed, released handler ownership");
    Ok(())
}
