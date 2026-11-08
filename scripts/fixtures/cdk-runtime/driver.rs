//! Installed Wire runtime checks. Testkit owns server lifecycle and transport.
mod benchmarks;
mod external_driver;
mod pocketic_support;
mod protocol;
mod wire_driver;
mod wire_protocol;

use ic_platform::principal::ExternalPrincipal as Principal;
use ic_testkit::pocket_ic::{PocketIc, RejectCode, RejectResponse};
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
async fn query<R: DeserializeOwned>(
    pic: &PocketIc,
    id: Principal,
    method: &str,
    args: impl Serialize,
) -> R {
    response(
        method,
        &pic.query_call(id, Principal::anonymous(), method, request(method, args))
            .await
            .unwrap(),
    )
}
async fn update<R: DeserializeOwned>(
    pic: &PocketIc,
    id: Principal,
    method: &str,
    args: impl Serialize,
) -> R {
    response(
        method,
        &pic.update_call(id, Principal::anonymous(), method, request(method, args))
            .await
            .unwrap(),
    )
}
#[cfg(feature = "timers-composition")]
async fn advance(pic: &PocketIc) {
    pic.advance_time(Duration::from_secs(2)).await.unwrap();
    for _ in 0..20 {
        pic.tick().await.unwrap();
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
    pocketic_support::runtime().block_on(run_fixture());
}

async fn run_fixture() {
    let wasm = std::fs::read(std::env::args().nth(1).expect("canister Wasm path")).unwrap();
    let legacy = std::env::var_os("PLATFORM_EXTERNAL_CANDID").is_some();
    // Testkit owns server startup/teardown; cross the client-owner boundary by URL.
    let (server, builder) = pocketic_support::builder(Duration::from_secs(60));
    let builder = builder.with_application_subnet();
    // The optional root-key measurement compares against Testkit's NNS key.
    // Keep the ordinary owner/Timers qualification topology unchanged.
    let builder = if std::env::var("PLATFORM_CODEC_BENCHMARK").as_deref() == Ok("system") {
        builder.with_nns_subnet()
    } else {
        builder
    };
    let instance = builder.build().await.unwrap();
    let pic = &instance;
    let id = pic.create_canister().await.unwrap();
    pic.add_cycles(id, 10_000_000_000_000).await.unwrap();
    if let Ok(mode) = std::env::var("PLATFORM_CODEC_BENCHMARK") {
        benchmarks::run(pic, id, wasm, &mode).await;
        instance.drop().await.unwrap();
        drop(server);
        return;
    }
    pic.install_canister(id, wasm.clone(), request("initialize", 10u64), None)
        .await
        .unwrap();
    if std::env::var_os("PLATFORM_EXECUTOR_BENCHMARK").is_some() {
        wire_driver::benchmark_fanout(pic, id).await;
        instance.drop().await.unwrap();
        drop(server);
        return;
    }
    let measure_copies = std::env::var_os("PLATFORM_BUFFER_COPY_BENCHMARK").is_some();
    benchmarks::buffer_copy(pic, id, measure_copies).await;
    if measure_copies {
        instance.drop().await.unwrap();
        drop(server);
        return;
    }
    assert_eq!(query::<u64>(pic, id, "value", ()).await, 10);
    assert_eq!(query::<u64>(pic, id, "scalar", 5u64).await, 6);
    assert_eq!(query::<(u64, bool)>(pic, id, "tuple", ()).await, (7, true));
    query::<()>(pic, id, "unit", ()).await;
    assert_eq!(update::<u64>(pic, id, "awaited", 8u64).await, 9);
    assert_eq!(query::<u64>(pic, id, "composite", 9u64).await, 10);
    assert_eq!(query::<u64>(pic, id, "self_wake_query", ()).await, 2);
    if legacy {
        assert_eq!(query::<u8>(pic, id, "management_proof", ()).await, 4);
        println!("Owned management HTTP/history/status/snapshot/lifecycle wire proofs passed");
    }
    assert_guard_rejection(
        &pic.query_call(
            id,
            Principal::anonymous(),
            "guarded",
            request("guarded", ()),
        )
        .await
        .map_err(rejection)
        .unwrap_err(),
    );
    for bytes in [vec![], vec![0, 1]] {
        let error = pic
            .query_call(id, Principal::anonymous(), "scalar", bytes)
            .await
            .map_err(rejection)
            .unwrap_err();
        // Empty input must reach the decoder, not fail in buffer initialization.
        assert!(error.reject_message.contains("fixture request:"));
    }
    assert_eq!(query::<Vec<u8>>(pic, id, "manual", ()).await, vec![0, 255]);
    assert!(update::<bool>(pic, id, "missing", ()).await);
    assert!(update::<bool>(pic, id, "insufficient_cycles", ()).await);
    let (replies, refused): (u64, u64) = update(pic, id, "submission_pressure", ()).await;
    assert!(replies > 0 && refused > 0);
    assert_eq!(replies + refused, 600);
    assert!(
        pic.update_call(
            id,
            Principal::anonymous(),
            "callback_trap",
            request("callback_trap", ())
        )
        .await
        .is_err()
    );
    assert_eq!(query::<u64>(pic, id, "cleaned", ()).await, 1);
    assert_eq!(update::<u64>(pic, id, "awaited", 3u64).await, 4);
    update::<()>(pic, id, "migratory", ()).await;
    assert_eq!(query::<u64>(pic, id, "value", ()).await, 11);
    #[cfg(feature = "timers-composition")]
    for (trap, cancel, value, cleaned) in [
        (false, false, 12, 1),
        (true, false, 12, 2),
        (false, true, 12, 2),
        (false, false, 13, 2),
    ] {
        update::<()>(pic, id, "timer_start", (trap, cancel)).await;
        advance(pic).await;
        assert_eq!(query::<u64>(pic, id, "value", ()).await, value);
        assert_eq!(query::<u64>(pic, id, "cleaned", ()).await, cleaned);
    }
    update::<()>(pic, id, "weak_pending", ()).await;
    // Protected unfinished tasks must still panic, rather than quietly outlive the method.
    assert!(
        pic.update_call(
            id,
            Principal::anonymous(),
            "protected_pending",
            request("protected_pending", ())
        )
        .await
        .map_err(rejection)
        .unwrap_err()
        .reject_message
        .contains("protected task outlived its canister method")
    );
    let checkpoint_value = query::<u64>(pic, id, "value", ()).await;
    let checkpoint_cleaned = query::<u64>(pic, id, "cleaned", ()).await;
    assert_eq!(
        (checkpoint_value, checkpoint_cleaned),
        if cfg!(feature = "timers-composition") {
            (13, 2)
        } else {
            (11, 1)
        }
    );
    pic.upgrade_canister(id, wasm.clone(), request("after_upgrade", 30u64), None)
        .await
        .unwrap();
    assert_eq!(query::<u64>(pic, id, "value", ()).await, 30);
    if legacy {
        assert_eq!(
            query::<(bool, bool, u64, u64)>(pic, id, "checkpoint", ()).await,
            (true, false, checkpoint_value, checkpoint_cleaned)
        );
        let bytes: Vec<u8> = query(pic, id, "checkpoint_bytes", ()).await;
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
        assert!(update::<bool>(pic, id, "external_management", ()).await);
    }
    #[cfg(feature = "timers-composition")]
    {
        update::<()>(pic, id, "timer_start", (false, false)).await;
        advance(pic).await;
        assert_eq!(query::<u64>(pic, id, "value", ()).await, 31);
    }
    let receiver = pic.create_canister().await.unwrap();
    pic.add_cycles(receiver, 10_000_000_000_000).await.unwrap();
    pic.install_canister(receiver, wasm, request("initialize", 0u64), None)
        .await
        .unwrap();
    assert_eq!(update::<u64>(pic, id, "cancel_task", ()).await, 2);
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
        )
        .await;
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
        .await
        .map_err(rejection)
        .unwrap_err();
    assert_eq!(error.reject_code, RejectCode::CanisterError);
    assert!(
        error
            .reject_message
            .contains("fixture cycle callback write trap")
    );
    assert_eq!(
        query::<u64>(pic, id, "value", ()).await,
        if cfg!(feature = "timers-composition") {
            31
        } else {
            30
        }
    );
    assert_eq!(
        query::<u128>(pic, receiver, "cycle_total", ()).await,
        1_900_000_000
    );
    assert_eq!(query::<u64>(pic, id, "cleaned", ()).await, 1);
    assert_eq!(
        pic.query_call(
            receiver,
            Principal::anonymous(),
            "stable_value",
            request("stable_value", ())
        )
        .await
        .map_err(rejection)
        .unwrap_err()
        .reject_code,
        RejectCode::CanisterError
    );
    assert_eq!(
        update::<(u64, u64)>(pic, receiver, "stable_edit", (123u64, false, false)).await,
        (0, 1)
    );
    assert!(
        pic.update_call(
            receiver,
            Principal::anonymous(),
            "stable_edit",
            request("stable_edit", (999u64, true, true))
        )
        .await
        .map_err(rejection)
        .unwrap_err()
        .reject_message
        .contains("fixture stable rollback")
    );
    assert_eq!(
        query::<(u64, u64)>(pic, receiver, "stable_value", ()).await,
        (1, 123)
    );
    assert_eq!(
        update::<(u64, u64)>(pic, receiver, "stable_edit", (456u64, true, false)).await,
        (1, 2)
    );
    assert_eq!(
        update::<(u64, u64)>(pic, receiver, "stable_refused_growth", ()).await,
        (u64::MAX, 2)
    );
    assert_eq!(
        query::<(u64, u64)>(pic, receiver, "stable_value", ()).await,
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
            .await
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
            query::<(u64, u64)>(pic, receiver, "stable_value", ()).await,
            (2, 456)
        );
    }
    let root = vec![42u8; 32];
    assert!(update::<bool>(pic, receiver, "publish_root", root.clone()).await);
    let certificate: Option<Vec<u8>> = query(pic, receiver, "certificate", ()).await;
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
    let (version, burned, absent): (u64, u128, bool) =
        update(pic, receiver, "system_helpers", ()).await;
    assert!(version > 0);
    assert_eq!(burned, 123);
    assert!(absent);
    wire_driver::run(pic, receiver).await;
    if legacy && std::env::var_os("PLATFORM_EXTERNAL_ADAPTERS").is_some() {
        external_driver::qualify(pic, id).await;
    }
    println!(
        "Runtime: Wire RPCs, raw awaits, trap cleanup, upgrade, cycles, stable rollback and certificates passed (external Candid: {legacy}, Timers composition: {})",
        cfg!(feature = "timers-composition")
    );
    instance.drop().await.unwrap();
    drop(server);
}
