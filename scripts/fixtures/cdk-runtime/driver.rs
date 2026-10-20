//! Installed Wire runtime checks. Testkit owns server lifecycle and transport.
mod protocol;
mod wire_driver;
mod wire_protocol;

use ic_platform::principal::ExternalPrincipal as Principal;
use ic_testkit::pic::{PocketIc, PocketIcBuilder, PocketIcStartupConfig};
use ic_testkit::pocket_ic::{RejectCode, RejectResponse};
use protocol::{LIMITS, method_id};
use serde::{Serialize, de::DeserializeOwned};
use std::time::Duration;

fn request(method: &str, args: impl Serialize) -> Vec<u8> {
    ic_wire::native::encode_request(method_id(method), &args, LIMITS).unwrap()
}
fn response<R: Serialize + DeserializeOwned>(method: &str, bytes: &[u8]) -> R {
    ic_wire::native::decode_response(method_id(method), bytes, LIMITS).unwrap()
}
fn query<R: Serialize + DeserializeOwned>(
    pic: &PocketIc,
    id: Principal,
    method: &str,
    args: impl Serialize,
) -> R {
    response(
        method,
        &pic.query_call(id, Principal::anonymous(), method, request(method, args))
            .unwrap(),
    )
}
fn update<R: Serialize + DeserializeOwned>(
    pic: &PocketIc,
    id: Principal,
    method: &str,
    args: impl Serialize,
) -> R {
    response(
        method,
        &pic.update_call(id, Principal::anonymous(), method, request(method, args))
            .unwrap(),
    )
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
    let legacy = std::env::var_os("PLATFORM_EXTERNAL_CANDID").is_some();
    let owned = PocketIcStartupConfig::from_env(Duration::from_secs(60))
        .unwrap()
        .build_instance(PocketIcBuilder::new().with_application_subnet())
        .unwrap();
    let pic = &owned.instance;
    let id = pic.create_canister();
    pic.add_cycles(id, 10_000_000_000_000);
    pic.install_canister(id, wasm.clone(), request("initialize", 10u64), None);
    assert_eq!(query::<u64>(pic, id, "value", ()), 10);
    assert_eq!(query::<u64>(pic, id, "scalar", 5u64), 6);
    assert_eq!(query::<(u64, bool)>(pic, id, "tuple", ()), (7, true));
    query::<()>(pic, id, "unit", ());
    assert_eq!(update::<u64>(pic, id, "awaited", 8u64), 9);
    assert_eq!(query::<u64>(pic, id, "composite", 9u64), 10);
    assert_guard_rejection(
        &pic.query_call(
            id,
            Principal::anonymous(),
            "guarded",
            request("guarded", ()),
        )
        .unwrap_err(),
    );
    assert!(
        pic.query_call(id, Principal::anonymous(), "scalar", vec![0, 1])
            .is_err()
    );
    assert_eq!(query::<Vec<u8>>(pic, id, "manual", ()), vec![0, 255]);
    assert!(update::<bool>(pic, id, "missing", ()));
    assert!(update::<bool>(pic, id, "insufficient_cycles", ()));
    let (replies, refused): (u64, u64) = update(pic, id, "submission_pressure", ());
    assert!(replies > 0 && refused > 0);
    assert_eq!(replies + refused, 600);
    assert!(
        pic.update_call(
            id,
            Principal::anonymous(),
            "callback_trap",
            request("callback_trap", ())
        )
        .is_err()
    );
    assert_eq!(query::<u64>(pic, id, "cleaned", ()), 1);
    assert_eq!(update::<u64>(pic, id, "awaited", 3u64), 4);
    update::<()>(pic, id, "migratory", ());
    assert_eq!(query::<u64>(pic, id, "value", ()), 11);
    for (trap, cancel, value, cleaned) in [
        (false, false, 12, 1),
        (true, false, 12, 2),
        (false, true, 12, 2),
        (false, false, 13, 2),
    ] {
        update::<()>(pic, id, "timer_start", (trap, cancel));
        advance(pic);
        assert_eq!(query::<u64>(pic, id, "value", ()), value);
        assert_eq!(query::<u64>(pic, id, "cleaned", ()), cleaned);
    }
    update::<()>(pic, id, "weak_pending", ());
    // Protected unfinished tasks must still panic, rather than quietly outlive the method.
    assert!(
        pic.update_call(
            id,
            Principal::anonymous(),
            "protected_pending",
            request("protected_pending", ())
        )
        .unwrap_err()
        .reject_message
        .contains("protected task outlived its canister method")
    );
    pic.upgrade_canister(id, wasm.clone(), request("after_upgrade", 30u64), None)
        .unwrap();
    assert_eq!(query::<u64>(pic, id, "value", ()), 30);
    if legacy {
        assert_eq!(
            query::<(bool, bool, u64, u64)>(pic, id, "checkpoint", ()),
            (true, false, 13, 2)
        );
        let bytes: Vec<u8> = query(pic, id, "checkpoint_bytes", ());
        // Candid is retained only for this pre-existing persisted checkpoint format.
        assert_eq!(
            bytes,
            ic_platform::candid::contract::encode_args((true, false, 13u64, 2u64)).unwrap()
        );
        assert!(update::<bool>(pic, id, "external_management", ()));
    }
    update::<()>(pic, id, "timer_start", (false, false));
    advance(pic);
    assert_eq!(query::<u64>(pic, id, "value", ()), 31);
    let receiver = pic.create_canister();
    pic.add_cycles(receiver, 10_000_000_000_000);
    pic.install_canister(receiver, wasm, request("initialize", 0u64), None);
    let offered = 1_000_000_000u128;
    for (limit, mode, refund) in [
        (300_000_000u128, 0u8, 700_000_000u128),
        (2 * offered, 0, 0),
        (300_000_000, 1, 700_000_000),
        (300_000_000, 2, offered),
    ] {
        let (ok, refunded, receipt, counters): (bool, u128, Vec<u128>, Vec<u64>) = update(
            pic,
            id,
            "cycle_roundtrip",
            (receiver, offered, limit, mode, false),
        );
        assert_eq!(ok, mode == 0);
        assert_eq!(refunded, refund);
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
    let error = pic
        .update_call(
            id,
            Principal::anonymous(),
            "cycle_roundtrip",
            request(
                "cycle_roundtrip",
                (receiver, offered, 300_000_000u128, 0u8, true),
            ),
        )
        .unwrap_err();
    assert_eq!(error.reject_code, RejectCode::CanisterError);
    assert!(
        error
            .reject_message
            .contains("fixture cycle callback write trap")
    );
    assert_eq!(query::<u64>(pic, id, "value", ()), 31);
    assert_eq!(
        query::<u128>(pic, receiver, "cycle_total", ()),
        1_900_000_000
    );
    assert_eq!(query::<u64>(pic, id, "cleaned", ()), 1);
    assert_eq!(
        pic.query_call(
            receiver,
            Principal::anonymous(),
            "stable_value",
            request("stable_value", ())
        )
        .unwrap_err()
        .reject_code,
        RejectCode::CanisterError
    );
    assert_eq!(
        update::<(u64, u64)>(pic, receiver, "stable_edit", (123u64, false, false)),
        (0, 1)
    );
    assert!(
        pic.update_call(
            receiver,
            Principal::anonymous(),
            "stable_edit",
            request("stable_edit", (999u64, true, true))
        )
        .unwrap_err()
        .reject_message
        .contains("fixture stable rollback")
    );
    assert_eq!(
        query::<(u64, u64)>(pic, receiver, "stable_value", ()),
        (1, 123)
    );
    assert_eq!(
        update::<(u64, u64)>(pic, receiver, "stable_edit", (456u64, true, false)),
        (1, 2)
    );
    let root = vec![42u8; 32];
    assert!(update::<bool>(pic, receiver, "publish_root", root.clone()));
    let certificate: Option<Vec<u8>> = query(pic, receiver, "certificate", ());
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
    let (version, burned, absent): (u64, u128, bool) = update(pic, receiver, "system_helpers", ());
    assert!(version > 0);
    assert_eq!(burned, 123);
    assert!(absent);
    wire_driver::run(pic, receiver);
    println!(
        "Owned runtime: Wire RPCs, raw awaits, trap cleanup, timer cancellation, upgrade, cycles, stable rollback and certificates passed (external Candid: {legacy})"
    );
}
