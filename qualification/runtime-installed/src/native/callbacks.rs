use candid::Principal;
use ic_testkit::pocket_ic::{ErrorCode, nonblocking::PocketIc};

pub(super) async fn verify(pic: &PocketIc, canister: Principal, peer: Principal) {
    let caller = canister;
    for (index, (mode, completed, dropped)) in [
        (0, 1, 1),
        (1, 1, 2),
        (2, 1, 3),
        (0, 2, 4),
        (3, 2, 4),
        (0, 3, 5),
        (4, 4, 6),
        (0, 5, 7),
    ]
    .into_iter()
    .enumerate()
    {
        let mut input = vec![mode];
        input.extend_from_slice(peer.as_slice());
        let result = pic
            .update_call(canister, caller, "await_probe", input)
            .await;
        let peer_count = u8::try_from(index + 1 + usize::from(index >= 6)).unwrap();
        if mode == 0 || mode == 4 {
            assert_eq!(
                result.expect("resumed handler must reply"),
                vec![peer_count]
            );
        } else {
            let rejection = result.expect_err("callback trap must roll back its reply");
            assert_eq!(rejection.error_code, ErrorCode::CanisterTrapped);
        }
        assert_eq!(
            pic.update_call(canister, caller, "observe_custody", vec![])
                .await
                .unwrap(),
            [0, 0, 0, 0, 1, 0, 0, 0][..=index],
            "only the trapping cleanup retains its handler-owned allocation"
        );
        assert_eq!(
            pic.update_call(canister, caller, "observe_dispatch", vec![])
                .await
                .unwrap(),
            [1, 0, 0, 1, 0, 1, 1, 1][..=index],
            "consumer reply records commit only when the callback returns successfully"
        );
        let started = u8::try_from(index + 1).unwrap();
        let mut expected = vec![started, completed, dropped];
        // Every prior handle is re-observed: later success must not turn
        // failed cleanup into a committed termination claim.
        expected.extend_from_slice(&[1, 2, 2, 1, 0, 1, 1, 1][..=index]);
        assert_eq!(
            pic.update_call(canister, caller, "observe", vec![])
                .await
                .unwrap(),
            expected,
            "committed counters and task lifecycle must match"
        );
        assert_eq!(
            pic.update_call(peer, caller, "probe", vec![0])
                .await
                .unwrap(),
            vec![peer_count],
            "caller rollback must not undo committed peer effects"
        );
    }
    println!(
        "cleanup custody passed: successful paths release; trapping cleanup retains one handler allocation"
    );
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
        assert_eq!(
            pic.update_call(canister, caller, "refund_probe", input)
                .await
                .unwrap(),
            refunds
        );
    }
    assert_eq!(
        pic.update_call(peer, caller, "probe", vec![0])
            .await
            .unwrap(),
        vec![9],
        "rejected peer update must roll back its increment"
    );
    assert_eq!(
        pic.update_call(canister, caller, "observe", vec![])
            .await
            .unwrap(),
        vec![8, 5, 7, 1, 2, 2, 1, 0, 1, 1, 1],
        "refund callbacks must not alter retained task states"
    );
    let mut input = vec![0];
    input.extend_from_slice(peer.as_slice());
    assert_eq!(
        pic.update_call(canister, caller, "resource_probe", input)
            .await
            .unwrap(),
        vec![9]
    );
    assert_eq!(
        pic.update_call(peer, caller, "probe", vec![0])
            .await
            .unwrap(),
        vec![9],
        "resource refusals must not increment the peer"
    );
    for (mode, expected) in [(0, 10), (1, 10), (2, 10), (0, 11)] {
        let mut input = vec![mode];
        input.extend_from_slice(peer.as_slice());
        let result = pic
            .update_call(canister, caller, "notify_probe", input)
            .await;
        if mode == 1 {
            assert_eq!(
                result
                    .expect_err("initial trap must roll back notification")
                    .error_code,
                ErrorCode::CanisterTrapped
            );
        } else {
            assert_eq!(
                result.expect("notification receipt does not require remote success"),
                vec![expected]
            );
        }
        assert_eq!(
            pic.update_call(peer, caller, "probe", vec![0])
                .await
                .unwrap(),
            vec![expected]
        );
    }
    assert_eq!(
        pic.update_call(canister, caller, "observe", vec![])
            .await
            .unwrap(),
        vec![8, 5, 7, 1, 2, 2, 1, 0, 1, 1, 1],
        "notifications must not change prior task states"
    );
    let query_error = pic
        .query_call(canister, caller, "refuse_update", vec![])
        .await
        .expect_err("query intentionally returns without a reply after asserting refusal");
    assert_eq!(query_error.error_code, ErrorCode::CanisterDidNotReply);
    assert_eq!(
        pic.update_call(canister, caller, "nested_probe", vec![0])
            .await
            .unwrap(),
        b"nested-refused"
    );
    assert_eq!(
        pic.update_call(canister, caller, "probe", vec![1])
            .await
            .unwrap(),
        vec![3],
        "ordinary update must still work after context refusals"
    );
    for capture in [1, 2] {
        for mode in [0, 1] {
            let mut input = vec![mode];
            input.extend_from_slice(peer.as_slice());
            let expected = if mode == 0 {
                vec![capture]
            } else {
                vec![capture, 2, 11]
            };
            assert_eq!(
                pic.update_call(canister, caller, "stale_probe", input)
                    .await
                    .unwrap(),
                expected
            );
        }
    }
}
