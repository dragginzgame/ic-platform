use super::*;
use std::{fmt::Debug, future::IntoFuture};
use system::{Event, with_mock};

fn isolated(test: impl FnOnce() + Send + 'static) {
    std::thread::spawn(test)
        .join()
        .expect("isolated call test failed");
}

#[test]
fn builders_preserve_borrowing_ownership_defaults_and_last_selection() {
    let owner = Principal::from_slice(&[1, 2]);
    let method = String::from("méthod");
    let borrowed = [0_u8, 0xff, 7];
    let call = Call::bounded_wait(owner, &method);
    assert_eq!(call.callee, owner);
    assert_eq!(call.method.as_ptr(), method.as_ptr());
    assert!(matches!(call.args, Cow::Borrowed([])));
    assert_eq!(call.cycles, 0);
    assert_eq!(
        call.wait,
        WaitPolicy::Bounded {
            timeout_seconds: 300
        }
    );

    let call = call.with_raw_args(&borrowed);
    assert!(matches!(call.args, Cow::Borrowed(_)));
    assert_eq!(call.args.as_ptr(), borrowed.as_ptr());
    let owned = vec![8, 9, 10];
    let allocation = owned.as_ptr();
    let call = call.take_raw_args(owned);
    assert!(matches!(call.args, Cow::Owned(_)));
    assert_eq!(call.args.as_ptr(), allocation);
    let cloned = call.clone();
    assert_eq!(cloned.args.as_ref(), [8, 9, 10]);
    assert_eq!(cloned.method.as_ptr(), method.as_ptr());

    let call = call
        .with_raw_args(&borrowed)
        .with_cycles(u128::MAX)
        .with_cycles(7)
        .with_wait_policy(WaitPolicy::Bounded { timeout_seconds: 0 })
        .with_wait_policy(WaitPolicy::Unbounded);
    assert!(matches!(call.args, Cow::Borrowed(_)));
    assert_eq!(call.args.as_ptr(), borrowed.as_ptr());
    assert_eq!(call.cycles, 7);
    assert_eq!(call.wait, WaitPolicy::Unbounded);
    assert_eq!(Call::unbounded_wait(owner, "x").wait, WaitPolicy::Unbounded);
}

#[test]
fn cost_uses_utf8_byte_lengths_and_saturating_u128_addition() {
    isolated(|| {
        with_mock(|mock| mock.cost = u128::MAX - 3);
        let call = Call::bounded_wait(Principal::anonymous(), "méthod")
            .with_raw_args(&[0, 1, 2])
            .with_cycles(7);
        assert_eq!(call.get_cost(), u128::MAX);
        with_mock(|mock| assert_eq!(mock.events, [Event::Cost(7, 3)]));
        with_mock(|mock| mock.cost = 1_u128 << 80);
        let call = call.with_cycles(1_u128 << 90);
        assert_eq!(call.get_cost(), (1_u128 << 80) + (1_u128 << 90));
    });
}

#[test]
fn oneway_submits_in_order_without_context_or_callback_retention() {
    isolated(|| {
        with_mock(|mock| {
            mock.cost = 5;
            mock.balance = 5 + (1_u128 << 100);
        });
        let call = Call::bounded_wait(Principal::from_slice(&[1, 2]), "raw")
            .take_raw_args(vec![0, 0xff, 0x44])
            .with_cycles(1_u128 << 100)
            .with_wait_policy(WaitPolicy::Bounded { timeout_seconds: 0 });
        call.oneway().unwrap();
        let expected = vec![
            Event::Cost(3, 3),
            Event::Balance,
            Event::Oneway {
                callee: vec![1, 2],
                method: "raw".to_owned(),
            },
            Event::Append(vec![0, 0xff, 0x44]),
            Event::Cycles(1_u128 << 100),
            Event::Timeout(0),
            Event::Perform,
        ];
        with_mock(|mock| {
            assert_eq!(mock.events, expected);
            assert!(mock.environments.is_empty());
            mock.events.clear();
        });
        // Submission borrows the builder, so an explicit second oneway is valid.
        call.oneway().unwrap();
        with_mock(|mock| assert_eq!(mock.events, expected));
    });
}

#[test]
fn zero_payload_cycles_and_unbounded_wait_skip_only_their_operations() {
    isolated(|| {
        let owner = Principal::management_canister();
        Call::unbounded_wait(owner, "")
            .take_raw_args(Vec::new())
            .oneway()
            .unwrap();
        with_mock(|mock| {
            assert_eq!(
                mock.events,
                [
                    Event::Cost(0, 0),
                    Event::Balance,
                    Event::Oneway {
                        callee: Vec::new(),
                        method: String::new()
                    },
                    Event::Perform,
                ]
            );
            mock.events.clear();
        });
        for timeout in [0, 300, u32::MAX] {
            Call::bounded_wait(owner, "x")
                .with_wait_policy(WaitPolicy::Bounded {
                    timeout_seconds: timeout,
                })
                .oneway()
                .unwrap();
            with_mock(|mock| {
                assert_eq!(
                    mock.events,
                    [
                        Event::Cost(1, 0),
                        Event::Balance,
                        Event::Oneway {
                            callee: Vec::new(),
                            method: "x".to_owned()
                        },
                        Event::Timeout(timeout),
                        Event::Perform,
                    ]
                );
                mock.events.clear();
            });
        }
    });
}

#[test]
fn oneway_preflight_refuses_before_construction_and_all_nonzero_statuses_fail() {
    isolated(|| {
        let call = Call::bounded_wait(Principal::anonymous(), "x").with_cycles(u128::MAX);
        with_mock(|mock| {
            mock.cost = 1;
            mock.balance = u128::MAX - 1;
        });
        match call.oneway() {
            Err(OnewayError::InsufficientLiquidCycleBalance(error)) => {
                assert_eq!(error.available, u128::MAX - 1);
                assert_eq!(error.required, u128::MAX);
            }
            other => panic!("unexpected preflight result: {other:?}"),
        }
        with_mock(|mock| {
            assert_eq!(mock.events, [Event::Cost(1, 0), Event::Balance]);
            mock.balance = u128::MAX;
            mock.events.clear();
        });
        // Equality with the saturated required value succeeds.
        call.oneway().unwrap();
        with_mock(|mock| assert!(mock.events.contains(&Event::Cycles(u128::MAX))));
        for status in [1, 2, u32::MAX] {
            with_mock(|mock| {
                mock.status = status;
                mock.events.clear();
            });
            assert!(matches!(
                call.oneway(),
                Err(OnewayError::CallPerformFailed(_))
            ));
            with_mock(|mock| {
                assert_eq!(mock.events.last(), Some(&Event::Perform));
                assert!(mock.environments.is_empty());
            });
        }
    });
}

#[test]
fn unpolled_futures_are_lazy_and_preserve_required_auto_traits() {
    fn required<T: Send + Sync + Unpin + Debug>() {}
    required::<Call<'static, 'static>>();
    required::<future::CallFuture<'static, 'static>>();
    isolated(|| {
        let method = String::from("temporary");
        let args = [1, 2, 3];
        drop(
            Call::bounded_wait(Principal::anonymous(), &method)
                .with_raw_args(&args)
                .into_future(),
        );
        with_mock(|mock| {
            assert!(mock.events.is_empty());
            assert!(mock.environments.is_empty());
        });
    });
}

#[test]
fn response_preserves_owned_transport_bytes_and_comparison_traits() {
    let bytes = vec![0, 0xff, 0x80, 7];
    let allocation = bytes.as_ptr();
    let response = Response(bytes);
    assert_eq!(response, vec![0, 0xff, 0x80, 7]);
    assert_eq!(response, &[0, 0xff, 0x80, 7][..]);
    assert_eq!(response, Response(vec![0, 0xff, 0x80, 7]));
    assert_eq!(response.as_ref(), [0, 0xff, 0x80, 7]);
    assert_eq!(&*response, [0, 0xff, 0x80, 7]);
    let borrowed: &[u8] = Borrow::borrow(&response);
    assert_eq!(borrowed, [0, 0xff, 0x80, 7]);
    let recovered = response.into_bytes();
    assert_eq!(recovered.as_ptr(), allocation);
    assert_eq!(recovered, [0, 0xff, 0x80, 7]);
}

#[test]
fn raw_rejections_and_transparent_errors_preserve_public_data() {
    for (code, expected) in [
        (1, RejectCode::SysFatal),
        (2, RejectCode::SysTransient),
        (3, RejectCode::DestinationInvalid),
        (4, RejectCode::CanisterReject),
        (5, RejectCode::CanisterError),
        (6, RejectCode::SysUnknown),
    ] {
        let error = CallRejected::with_rejection(code, "é\0detail".to_owned());
        assert_eq!(error.raw_reject_code(), code);
        assert_eq!(error.reject_message(), "é\0detail");
        assert_eq!(error.reject_code(), Ok(expected));
        assert_eq!(
            error.to_string(),
            format!("call rejected: {code} - é\0detail")
        );
    }
    for code in [0, 7, u32::MAX] {
        let rejection = CallRejected::with_rejection(code, "raw".to_owned());
        let unknown = rejection.reject_code().unwrap_err();
        assert_eq!(
            unknown.to_string(),
            format!("unrecognized reject code: {code}")
        );
        assert_eq!(unknown.clone(), unknown);
        let wrapped = CallFailed::from(rejection);
        assert_eq!(wrapped.to_string(), format!("call rejected: {code} - raw"));
    }
    let insufficient = InsufficientLiquidCycleBalance {
        available: u128::MAX - 1,
        required: u128::MAX,
    };
    let message = format!(
        "insufficient liquid cycles balance, available: {}, required: {}",
        u128::MAX - 1,
        u128::MAX
    );
    assert_eq!(insufficient.to_string(), message);
    assert_eq!(CallFailed::from(insufficient.clone()).to_string(), message);
    assert_eq!(OnewayError::from(insufficient).to_string(), message);
    assert_eq!(CallPerformFailed.to_string(), "call perform failed");
    assert_eq!(
        CallFailed::from(CallPerformFailed).to_string(),
        "call perform failed"
    );
    assert_eq!(
        OnewayError::from(CallPerformFailed).to_string(),
        "call perform failed"
    );
}
