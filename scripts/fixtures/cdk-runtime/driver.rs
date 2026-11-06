//! Installed Wire runtime checks. Testkit owns server lifecycle and transport.
mod external_driver;
mod protocol;
mod wire_driver;
mod wire_protocol;

use ic_platform::principal::ExternalPrincipal as Principal;
use ic_testkit::pic::PocketIcStartupConfig;
use ic_testkit::pocket_ic::{PocketIc, PocketIcBuilder, RejectCode, RejectResponse};
use protocol::{LIMITS, method_id};
use serde::{Serialize, de::DeserializeOwned};
use std::time::Duration;

// Keep the existing copy corpus and its explicit fixture-owned frame allowance.
const BUFFER_COPY_SIZES: [usize; 5] = [0, 1, 63, 1024, 2048];

fn request(method: &str, args: impl Serialize) -> Vec<u8> {
    ic_wire::native::encode_request(method_id(method), &args, LIMITS).unwrap()
}
fn response<R: DeserializeOwned>(method: &str, bytes: &[u8]) -> R {
    ic_wire::native::decode_response(method_id(method), bytes, LIMITS).unwrap()
}
fn query<R: DeserializeOwned>(
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
fn update<R: DeserializeOwned>(
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
#[cfg(feature = "timers-composition")]
fn advance(pic: &PocketIc) {
    pic.advance_time(Duration::from_secs(2)).unwrap();
    for _ in 0..20 {
        pic.tick().unwrap();
    }
}
fn rejection(error: ic_testkit::pocket_ic::ClientError) -> RejectResponse {
    match error {
        ic_testkit::pocket_ic::ClientError::Canister(rejection) => rejection,
        other => panic!("expected canister rejection, got {other}"),
    }
}

fn assert_guard_rejection(error: &RejectResponse) {
    assert_eq!(error.reject_code, RejectCode::CanisterReject);
    assert_eq!(error.reject_message, "fixture guard refused");
}

#[cfg(test)]
mod tests;

fn main() {
    let wasm = std::fs::read(std::env::args().nth(1).expect("canister Wasm path")).unwrap();
    let legacy = std::env::var_os("PLATFORM_EXTERNAL_CANDID").is_some();
    // Testkit owns server startup/teardown; cross the client-owner boundary by URL.
    let config = PocketIcStartupConfig::from_env(Duration::from_secs(60)).unwrap();
    let (server, url) = match config {
        PocketIcStartupConfig::Connect { server_url, .. } => (None, server_url),
        spawn => {
            let server = spawn.start_managed_server().unwrap();
            let url = server.url().to_owned();
            (Some(server), url)
        }
    };
    let instance = PocketIcBuilder::new()
        .with_server_url(url.parse().unwrap())
        .with_max_request_time_ms(Some(60_000))
        .with_application_subnet()
        .build()
        .unwrap();
    let pic = &instance;
    let id = pic.create_canister().unwrap();
    pic.add_cycles(id, 10_000_000_000_000).unwrap();
    pic.install_canister(id, wasm.clone(), request("initialize", 10u64), None)
        .unwrap();
    for size in BUFFER_COPY_SIZES {
        let payload: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        let mut baseline = Vec::new();
        let mut candidate = Vec::new();
        for round in 0..6 {
            for old in if round % 2 == 0 {
                [true, false]
            } else {
                [false, true]
            } {
                let args = (old, payload.clone());
                let expected = request("buffer_copy", &args);
                let (instructions, bytes): (u64, Vec<u8>) = query(pic, id, "buffer_copy", args);
                assert_eq!(bytes, expected);
                if round != 0 {
                    if old {
                        baseline.push(instructions);
                    } else {
                        candidate.push(instructions);
                    }
                }
            }
        }
        baseline.sort_unstable();
        candidate.sort_unstable();
        println!(
            "buffer-copy payload={size} baseline={} candidate={} instructions",
            baseline[2], candidate[2]
        );
    }
    assert_eq!(query::<u64>(pic, id, "value", ()), 10);
    assert_eq!(query::<u64>(pic, id, "scalar", 5u64), 6);
    assert_eq!(query::<(u64, bool)>(pic, id, "tuple", ()), (7, true));
    query::<()>(pic, id, "unit", ());
    assert_eq!(update::<u64>(pic, id, "awaited", 8u64), 9);
    assert_eq!(query::<u64>(pic, id, "composite", 9u64), 10);
    assert_eq!(query::<u64>(pic, id, "self_wake_query", ()), 2);
    if legacy {
        assert_eq!(query::<u8>(pic, id, "management_proof", ()), 4);
        println!("Owned management HTTP/history/status/snapshot/lifecycle wire proofs passed");
    }
    assert_guard_rejection(
        &pic.query_call(
            id,
            Principal::anonymous(),
            "guarded",
            request("guarded", ()),
        )
        .map_err(rejection)
        .unwrap_err(),
    );
    for bytes in [vec![], vec![0, 1]] {
        let error = pic
            .query_call(id, Principal::anonymous(), "scalar", bytes)
            .map_err(rejection)
            .unwrap_err();
        // Empty input must reach the decoder, not fail in buffer initialization.
        assert!(error.reject_message.contains("fixture request:"));
    }
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
    #[cfg(feature = "timers-composition")]
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
        .map_err(rejection)
        .unwrap_err()
        .reject_message
        .contains("protected task outlived its canister method")
    );
    let checkpoint_value = query::<u64>(pic, id, "value", ());
    let checkpoint_cleaned = query::<u64>(pic, id, "cleaned", ());
    assert_eq!(
        (checkpoint_value, checkpoint_cleaned),
        if cfg!(feature = "timers-composition") {
            (13, 2)
        } else {
            (11, 1)
        }
    );
    pic.upgrade_canister(id, wasm.clone(), request("after_upgrade", 30u64), None)
        .unwrap();
    assert_eq!(query::<u64>(pic, id, "value", ()), 30);
    if legacy {
        assert_eq!(
            query::<(bool, bool, u64, u64)>(pic, id, "checkpoint", ()),
            (true, false, checkpoint_value, checkpoint_cleaned)
        );
        let bytes: Vec<u8> = query(pic, id, "checkpoint_bytes", ());
        // Candid is retained only for this pre-existing persisted checkpoint format.
        assert_eq!(
            bytes,
            ic_platform::candid::contract::encode_args((
                true,
                false,
                checkpoint_value,
                checkpoint_cleaned
            ))
            .unwrap()
        );
        assert!(update::<bool>(pic, id, "external_management", ()));
    }
    #[cfg(feature = "timers-composition")]
    {
        update::<()>(pic, id, "timer_start", (false, false));
        advance(pic);
        assert_eq!(query::<u64>(pic, id, "value", ()), 31);
    }
    let receiver = pic.create_canister().unwrap();
    pic.add_cycles(receiver, 10_000_000_000_000).unwrap();
    pic.install_canister(receiver, wasm, request("initialize", 0u64), None)
        .unwrap();
    assert_eq!(update::<u64>(pic, id, "cancel_task", ()), 2);
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
        .map_err(rejection)
        .unwrap_err();
    assert_eq!(error.reject_code, RejectCode::CanisterError);
    assert!(
        error
            .reject_message
            .contains("fixture cycle callback write trap")
    );
    assert_eq!(
        query::<u64>(pic, id, "value", ()),
        if cfg!(feature = "timers-composition") {
            31
        } else {
            30
        }
    );
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
        .map_err(rejection)
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
        .map_err(rejection)
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
    assert_eq!(
        update::<(u64, u64)>(pic, receiver, "stable_refused_growth", ()),
        (u64::MAX, 2)
    );
    assert_eq!(
        query::<(u64, u64)>(pic, receiver, "stable_value", ()),
        (2, 456)
    );
    for write in [false, true] {
        let rejection = pic
            .update_call(
                receiver,
                Principal::anonymous(),
                "stable_bounds_trap",
                request("stable_bounds_trap", write),
            )
            .map_err(rejection)
            .unwrap_err();
        assert_eq!(rejection.reject_code, RejectCode::CanisterError);
        assert!(
            rejection
                .reject_message
                .to_ascii_lowercase()
                .contains("stable memory"),
            "expected a stable-memory bounds trap: {rejection:?}"
        );
        assert_eq!(
            query::<(u64, u64)>(pic, receiver, "stable_value", ()),
            (2, 456)
        );
    }
    let root = vec![42u8; 32];
    assert!(update::<bool>(pic, receiver, "publish_root", root.clone()));
    let certificate: Option<Vec<u8>> = query(pic, receiver, "certificate", ());
    let certificate = certificate.unwrap();
    let certificate =
        ic_platform::certification::cbor::decode_certificate(&certificate, certificate.len())
            .unwrap();
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
    if legacy && std::env::var_os("PLATFORM_EXTERNAL_ADAPTERS").is_some() {
        external_driver::qualify(pic, id);
    }
    println!(
        "Runtime: Wire RPCs, raw awaits, trap cleanup, upgrade, cycles, stable rollback and certificates passed (external Candid: {legacy}, Timers composition: {})",
        cfg!(feature = "timers-composition")
    );
    drop(instance);
    drop(server);
}
