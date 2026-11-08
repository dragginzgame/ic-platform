//! Ordered installed assertions; lifecycle remains in the driver.
use super::*;

pub(super) async fn execution(pic: &PocketIc, id: Principal, legacy: bool) {
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
}

pub(super) async fn upgrade(pic: &PocketIc, id: Principal, wasm: Vec<u8>, legacy: bool) {
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
}

pub(super) async fn cycles(pic: &PocketIc, id: Principal, receiver: Principal) {
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
}

pub(super) async fn stable_memory(pic: &PocketIc, receiver: Principal) {
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
}

pub(super) async fn system(pic: &PocketIc, receiver: Principal) {
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
}
