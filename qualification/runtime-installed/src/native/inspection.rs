//! Installed ingress decisions and the bypass for actual inter-canister calls.

use candid::Principal;
use ic_testkit::pocket_ic::{ErrorCode, nonblocking::PocketIc};

pub(super) async fn verify(pic: &PocketIc, caller_wasm: &[u8]) -> std::io::Result<()> {
    let inspected_wasm = std::fs::read(
        "target/runtime-exports/wasm32-unknown-unknown/release/runtime_exports.wasm",
    )?;
    let inspected = pic.create_canister().await;
    pic.add_cycles(inspected, 10_000_000_000_000).await;
    pic.install_canister(inspected, inspected_wasm, vec![], None)
        .await;

    let admitted = Principal::from_slice(&[42, 2]);
    pic.set_controllers(inspected, None, vec![admitted])
        .await
        .expect("fixture must assign a controller for the ingress policy");
    let payload = vec![0, 128, 254, 7];
    assert_eq!(
        pic.update_call(inspected, admitted, "echo", payload.clone())
            .await
            .expect("explicitly accepted ingress must run the update"),
        payload
    );
    for (caller, method, payload) in [
        (Principal::from_slice(&[43, 2]), "echo", vec![1]),
        (admitted, "echo", vec![1; 5]),
        (admitted, "probe", vec![1]),
    ] {
        let rejection = pic
            .submit_call(inspected, caller, method, payload)
            .await
            .expect_err("inspection must refuse at submission, before update execution");
        assert_eq!(rejection.error_code, ErrorCode::CanisterRejectedMessage);
    }

    let subnet = pic.get_subnet(inspected).await.unwrap();
    let sender = pic.create_canister_on_subnet(None, None, subnet).await;
    pic.add_cycles(sender, 10_000_000_000_000).await;
    pic.install_canister(sender, caller_wasm.to_vec(), vec![], None)
        .await;
    let mut input = vec![0];
    input.extend_from_slice(inspected.as_slice());
    assert_eq!(
        pic.update_call(sender, admitted, "await_probe", input)
            .await
            .expect("the inter-canister call must reach the update rejected for ingress"),
        [1]
    );
    println!(
        "installed ingress inspection passed: one admitted call, three induction refusals and one inter-canister reply"
    );

    Ok(())
}
