//! Installed runtime regression checks; Testkit owns bounded server startup.
use ic_platform::{
    Principal,
    candid::contract::{self, ArgumentEncoder},
};
use ic_testkit::pic::{PocketIc, PocketIcBuilder, PocketIcStartupConfig};
use ic_testkit::pocket_ic::{RejectCode, RejectResponse};
use std::time::Duration;

fn query(pic: &PocketIc, id: Principal, method: &str, args: impl ArgumentEncoder) -> Vec<u8> {
    pic.query_call(
        id,
        Principal::anonymous(),
        method,
        contract::encode_args(args).unwrap(),
    )
    .unwrap()
}
fn update(pic: &PocketIc, id: Principal, method: &str, args: impl ArgumentEncoder) -> Vec<u8> {
    pic.update_call(
        id,
        Principal::anonymous(),
        method,
        contract::encode_args(args).unwrap(),
    )
    .unwrap()
}
fn number(pic: &PocketIc, id: Principal, method: &str) -> u64 {
    ic_platform::candid::decoder::decode_one(&query(pic, id, method, ())).unwrap()
}
fn advance(pic: &PocketIc) {
    pic.advance_time(Duration::from_secs(2));
    for _ in 0..20 {
        pic.tick();
    }
}
fn assert_guard_rejection(error: &RejectResponse) {
    assert_eq!(error.reject_code, RejectCode::CanisterReject);
    assert_eq!(error.reject_message, "fixture guard refused");
}

#[test]
#[should_panic]
fn missing_entrypoint_is_not_guard_rejection() {
    assert_guard_rejection(&RejectResponse {
        reject_code: RejectCode::CanisterError,
        reject_message: "fixture guard refused".into(),
        error_code: ic_testkit::pocket_ic::ErrorCode::CanisterMethodNotFound,
        certified: false,
    });
}

#[test]
#[should_panic]
fn unrelated_rejection_is_not_guard_rejection() {
    assert_guard_rejection(&RejectResponse {
        reject_code: RejectCode::CanisterReject,
        reject_message: "unrelated rejection".into(),
        error_code: ic_testkit::pocket_ic::ErrorCode::CanisterRejectedMessage,
        certified: false,
    });
}

fn main() {
    let wasm = std::fs::read(std::env::args().nth(1).expect("canister Wasm path")).unwrap();
    let owned = PocketIcStartupConfig::from_env(Duration::from_secs(60))
        .unwrap()
        .build_instance(PocketIcBuilder::new().with_application_subnet())
        .unwrap();
    let pic = &owned.instance;
    let id = pic.create_canister();
    pic.add_cycles(id, 10_000_000_000_000);
    pic.install_canister(
        id,
        wasm.clone(),
        contract::encode_args((10u64,)).unwrap(),
        None,
    );
    assert_eq!(number(pic, id, "value"), 10);
    assert_eq!(
        query(pic, id, "scalar", (5u64,)),
        contract::encode_args((6u64,)).unwrap()
    );
    assert_eq!(
        query(pic, id, "tuple", ()),
        contract::encode_args((7u64, true)).unwrap()
    );
    assert_eq!(
        query(pic, id, "unit", ()),
        contract::encode_one(()).unwrap()
    );
    assert_eq!(
        update(pic, id, "awaited", (8u64,)),
        contract::encode_args((9u64,)).unwrap()
    );
    assert_eq!(
        query(pic, id, "composite", (9u64,)),
        contract::encode_args((10u64,)).unwrap()
    );
    assert_guard_rejection(
        &pic.query_call(
            id,
            Principal::anonymous(),
            "guarded",
            contract::encode_args(()).unwrap(),
        )
        .unwrap_err(),
    );
    assert!(
        pic.query_call(id, Principal::anonymous(), "scalar", vec![0, 1])
            .is_err()
    );
    assert_eq!(
        pic.query_call(id, Principal::anonymous(), "custom", vec![0, 255])
            .unwrap(),
        vec![0, 255]
    );
    assert_eq!(query(pic, id, "manual", ()), b"\0manual\xff");
    assert_eq!(
        update(pic, id, "missing", ()),
        contract::encode_args((true,)).unwrap()
    );
    assert!(
        pic.update_call(
            id,
            Principal::anonymous(),
            "callback_trap",
            contract::encode_args(()).unwrap()
        )
        .is_err()
    );
    assert_eq!(number(pic, id, "cleaned"), 1);
    assert_eq!(
        update(pic, id, "awaited", (3u64,)),
        contract::encode_args((4u64,)).unwrap()
    );
    update(pic, id, "migratory", ());
    assert_eq!(number(pic, id, "value"), 11);
    update(pic, id, "timer_start", (false, false));
    advance(pic);
    assert_eq!(number(pic, id, "value"), 12);
    update(pic, id, "timer_start", (true, false));
    advance(pic);
    assert_eq!(number(pic, id, "cleaned"), 2);
    update(pic, id, "timer_start", (false, true));
    advance(pic);
    assert_eq!(number(pic, id, "value"), 12);
    update(pic, id, "timer_start", (false, false));
    advance(pic);
    assert_eq!(number(pic, id, "value"), 13);
    pic.upgrade_canister(id, wasm, contract::encode_args((30u64,)).unwrap(), None)
        .unwrap();
    assert_eq!(number(pic, id, "value"), 30);
    assert_eq!(
        query(pic, id, "checkpoint", ()),
        contract::encode_args((true, false, 13u64, 2u64)).unwrap()
    );
    let checkpoint_bytes: Vec<u8> =
        ic_platform::candid::decoder::decode_one(&query(pic, id, "checkpoint_bytes", ())).unwrap();
    assert_eq!(
        checkpoint_bytes,
        contract::encode_args((true, false, 13u64, 2u64)).unwrap()
    );
    update(pic, id, "timer_start", (false, false));
    advance(pic);
    assert_eq!(number(pic, id, "value"), 31);
    let receiver = pic.create_canister();
    pic.add_cycles(receiver, 10_000_000_000_000);
    // The original Wasm was moved into upgrade_canister; reload the same artifact.
    let wasm = std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    pic.install_canister(
        receiver,
        wasm,
        contract::encode_args((0u64,)).unwrap(),
        None,
    );
    let offered = 1_000_000_000u128;
    for (limit, mode, expected_refund) in [
        (300_000_000u128, 0u8, 700_000_000u128),
        (2 * offered, 0, 0),
        (300_000_000, 1, 700_000_000),
        (300_000_000, 2, offered),
    ] {
        let reply = update(
            pic,
            id,
            "cycle_roundtrip",
            (receiver, offered, limit, mode, false),
        );
        let (ok, refunded, receipt, counters): (bool, u128, Vec<u128>, Vec<u64>) =
            ic_platform::candid::decoder::decode_args(&reply).unwrap();
        assert_eq!(ok, mode == 0);
        assert_eq!(refunded, expected_refund);
        if ok {
            assert_eq!(
                receipt,
                vec![offered, offered.min(limit), offered.saturating_sub(limit)]
            );
        }
        assert_eq!(counters.len(), 4);
        assert!(counters[0] > 0 && counters[2] > 0);
        assert!(counters[1] >= counters[0]);
        assert!(counters[3] > counters[1] && counters[3] >= counters[2]);
    }
    let rejected = pic
        .update_call(
            id,
            Principal::anonymous(),
            "cycle_roundtrip",
            contract::encode_args((receiver, offered, 300_000_000u128, 0u8, true)).unwrap(),
        )
        .unwrap_err();
    assert_eq!(rejected.reject_code, RejectCode::CanisterError);
    assert!(
        rejected
            .reject_message
            .contains("fixture cycle callback write trap")
    );
    assert_eq!(number(pic, id, "value"), 31);
    let total: u128 =
        ic_platform::candid::decoder::decode_one(&query(pic, receiver, "cycle_total", ())).unwrap();
    assert_eq!(total, 1_900_000_000);
    assert_eq!(number(pic, id, "cleaned"), 1);
    // A fresh receiver has zero stable pages; a failed read must remain a trap.
    assert_eq!(
        pic.query_call(
            receiver,
            Principal::anonymous(),
            "stable_value",
            contract::encode_args(()).unwrap()
        )
        .unwrap_err()
        .reject_code,
        RejectCode::CanisterError
    );
    assert_eq!(
        update(pic, receiver, "stable_edit", (123u64, false, false)),
        contract::encode_args((0u64, 1u64)).unwrap()
    );
    let failure = pic
        .update_call(
            receiver,
            Principal::anonymous(),
            "stable_edit",
            contract::encode_args((999u64, true, true)).unwrap(),
        )
        .unwrap_err();
    assert!(failure.reject_message.contains("fixture stable rollback"));
    assert_eq!(
        query(pic, receiver, "stable_value", ()),
        contract::encode_args((1u64, 123u64)).unwrap()
    );
    assert_eq!(
        update(pic, receiver, "stable_edit", (456u64, true, false)),
        contract::encode_args((1u64, 2u64)).unwrap()
    );
    let root = vec![42u8; 32];
    assert_eq!(
        update(pic, receiver, "publish_root", (root.clone(),)),
        contract::encode_args((true,)).unwrap()
    );
    let certificate: Option<Vec<u8>> =
        ic_platform::candid::decoder::decode_one(&query(pic, receiver, "certificate", ())).unwrap();
    let certificate: ic_platform::certification::Certificate =
        serde_cbor::from_slice(&certificate.unwrap()).unwrap();
    assert_eq!(
        certificate.tree.lookup_path([
            b"canister".as_slice(),
            receiver.as_slice(),
            b"certified_data".as_slice()
        ]),
        ic_platform::certification::LookupResult::Found(root.as_slice())
    );
    let (version, burned, absent): (u64, u128, bool) =
        ic_platform::candid::decoder::decode_args(&update(pic, receiver, "system_helpers", ()))
            .unwrap();
    assert!(version > 0);
    assert_eq!(burned, 123);
    assert!(absent);
    println!(
        "CDK runtime, checkpoint bytes, stable rollback, certificates and system helpers passed"
    );
}
