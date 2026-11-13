//! Public call contracts that do not invoke the IC System API.

use ic_platform_runtime::Principal;
use ic_platform_runtime::cdk::call::{
    Call, CallFailed, CallPerformFailed, CallRejected, InsufficientLiquidCycleBalance, OnewayError,
    RejectCode, Response, UnrecognizedRejectCode, WaitPolicy,
};
use std::borrow::Borrow;
use std::error::Error;
use std::fmt::Debug;
use std::future::IntoFuture;
use std::ops::Deref;

#[test]
fn public_transport_types_keep_their_traits() {
    fn transport<T: Send + Sync + Unpin + Debug>() {}
    fn error<T: Error + Clone + Debug>() {}
    fn call<T: Clone + Debug + IntoFuture<Output = Result<Response, CallFailed>>>() {}
    fn response<T>()
    where
        T: Debug
            + Deref<Target = [u8]>
            + AsRef<[u8]>
            + Borrow<[u8]>
            + PartialEq
            + PartialEq<Vec<u8>>,
        for<'a> T: PartialEq<&'a [u8]>,
    {
    }

    transport::<Call<'static, 'static>>();
    transport::<<Call<'static, 'static> as IntoFuture>::IntoFuture>();
    call::<Call<'static, 'static>>();
    response::<Response>();
    error::<CallFailed>();
    error::<OnewayError>();
    error::<InsufficientLiquidCycleBalance>();
    error::<CallPerformFailed>();
    error::<CallRejected>();
    error::<UnrecognizedRejectCode>();
    let _: fn(Response) -> Vec<u8> = Response::into_bytes;
}

#[test]
fn construction_cloning_and_unpolled_drop_need_no_ic_context() {
    let method = String::from("borrowed_method_λ");
    let mut bytes = vec![0, 255, 1, 128];
    let borrowed = Call::bounded_wait(Principal::anonymous(), &method)
        .with_raw_args(&bytes)
        .with_cycles(u128::MAX);
    let borrowed_clone = borrowed.clone();
    drop(borrowed.into_future());
    drop(borrowed_clone.into_future());
    assert_eq!(bytes, [0, 255, 1, 128]);
    bytes.push(7);

    let owned = Call::unbounded_wait(Principal::management_canister(), &method)
        .take_raw_args(bytes)
        .with_cycles(1);
    let owned_clone = owned.clone();
    drop(owned.into_future());
    drop(owned_clone.into_future());

    drop(Call::bounded_wait(Principal::anonymous(), &method).into_future());
    drop(Call::unbounded_wait(Principal::anonymous(), &method).into_future());
    assert_eq!(method, "borrowed_method_λ");
}

#[test]
fn wait_policy_extrema_and_repeated_builder_selection_are_admitted_lazily() {
    fn copy_eq_debug<T: Copy + Eq + Debug>() {}
    copy_eq_debug::<WaitPolicy>();

    let zero = WaitPolicy::Bounded { timeout_seconds: 0 };
    let maximum = WaitPolicy::Bounded {
        timeout_seconds: u32::MAX,
    };
    assert_ne!(zero, maximum);
    assert_ne!(zero, WaitPolicy::Unbounded);
    assert_ne!(maximum, WaitPolicy::Unbounded);

    for policy in [zero, maximum, WaitPolicy::Unbounded] {
        let copied = policy;
        assert_eq!(copied, policy);
        match copied {
            WaitPolicy::Bounded { timeout_seconds } => {
                assert!(timeout_seconds == 0 || timeout_seconds == u32::MAX);
            }
            WaitPolicy::Unbounded => {}
        }
        let bytes = [1, 0, 255];
        let call = Call::bounded_wait(Principal::anonymous(), "selection")
            .with_wait_policy(WaitPolicy::Unbounded)
            .with_wait_policy(policy)
            .with_cycles(u128::MAX)
            .with_cycles(0)
            .take_raw_args(vec![9])
            .with_raw_args(&bytes);
        drop(call.clone().into_future());
        drop(call.into_future());
        let call = Call::unbounded_wait(Principal::anonymous(), "selection")
            .with_raw_args(&bytes)
            .take_raw_args(Vec::new())
            .with_wait_policy(policy);
        drop(call.into_future());
    }
}

#[test]
#[expect(
    clippy::redundant_clone,
    reason = "This test observes Clone preserving the public error payload and variant"
)]
fn balance_errors_preserve_amounts_display_and_transparent_conversions() {
    for (available, required) in [(0, 1), (u128::MAX - 1, u128::MAX)] {
        let leaf = InsufficientLiquidCycleBalance {
            available,
            required,
        };
        let expected = format!(
            "insufficient liquid cycles balance, available: {available}, required: {required}"
        );
        assert_eq!(leaf.to_string(), expected);
        assert!(leaf.source().is_none());

        let awaited = CallFailed::from(leaf.clone());
        assert_eq!(awaited.to_string(), expected);
        assert!(awaited.source().is_none());
        match awaited.clone() {
            CallFailed::InsufficientLiquidCycleBalance(inner) => {
                assert_eq!(inner.available, available);
                assert_eq!(inner.required, required);
            }
            other => panic!("unexpected awaited error: {other:?}"),
        }

        let oneway = OnewayError::from(leaf);
        assert_eq!(oneway.to_string(), expected);
        assert!(oneway.source().is_none());
        match oneway.clone() {
            OnewayError::InsufficientLiquidCycleBalance(inner) => {
                assert_eq!(inner.available, available);
                assert_eq!(inner.required, required);
            }
            other => panic!("unexpected oneway error: {other:?}"),
        }
    }
}

#[test]
#[expect(
    clippy::redundant_clone,
    reason = "This test observes Clone preserving the public error variant"
)]
fn perform_error_preserves_display_and_transparent_conversions() {
    let leaf = CallPerformFailed;
    assert_eq!(leaf.to_string(), "call perform failed");
    assert!(leaf.source().is_none());

    let awaited = CallFailed::from(leaf.clone());
    assert_eq!(awaited.to_string(), "call perform failed");
    assert!(awaited.source().is_none());
    assert!(matches!(awaited.clone(), CallFailed::CallPerformFailed(_)));

    let oneway = OnewayError::from(leaf);
    assert_eq!(oneway.to_string(), "call perform failed");
    assert!(oneway.source().is_none());
    assert!(matches!(oneway.clone(), OnewayError::CallPerformFailed(_)));
}

#[expect(
    clippy::redundant_clone,
    reason = "The cloned rejection wrapper must retain the public message and code"
)]
fn assert_rejection_text_and_conversion(raw: u32, message: &str) -> CallRejected {
    let rejection = CallRejected::with_rejection(raw, message.to_owned());
    assert_eq!(rejection.raw_reject_code(), raw);
    assert_eq!(rejection.reject_message(), message);
    assert_eq!(
        rejection.to_string(),
        format!("call rejected: {raw} - {message}")
    );
    assert!(rejection.source().is_none());

    let wrapped = CallFailed::from(rejection.clone());
    assert_eq!(wrapped.to_string(), rejection.to_string());
    assert!(wrapped.source().is_none());
    match wrapped.clone() {
        CallFailed::CallRejected(inner) => {
            assert_eq!(inner.raw_reject_code(), raw);
            assert_eq!(inner.reject_message(), message);
        }
        other => panic!("unexpected rejection wrapper: {other:?}"),
    }
    rejection
}

#[test]
fn every_known_rejection_code_keeps_its_catalogue_identity() {
    for (raw, expected) in [
        (1, RejectCode::SysFatal),
        (2, RejectCode::SysTransient),
        (3, RejectCode::DestinationInvalid),
        (4, RejectCode::CanisterReject),
        (5, RejectCode::CanisterError),
        (6, RejectCode::SysUnknown),
    ] {
        let rejection = assert_rejection_text_and_conversion(raw, "line one\nline two\0 café 🦀");
        assert_eq!(rejection.reject_code(), Ok(expected));
        assert_eq!(RejectCode::try_from(u64::from(raw)).ok(), Some(expected));
    }
}

#[test]
fn unknown_rejection_codes_keep_raw_u32_values_and_exact_text() {
    for raw in [0, 7, 255, 256, u32::from(u16::MAX), u32::MAX - 1, u32::MAX] {
        for message in ["", "unrecognized\0code\nλ"] {
            let rejection = assert_rejection_text_and_conversion(raw, message);
            let unknown = rejection.reject_code().expect_err("unknown catalogue code");
            assert_eq!(
                unknown.to_string(),
                format!("unrecognized reject code: {raw}")
            );
            assert_eq!(unknown, unknown.clone());
            assert!(unknown.source().is_none());
            assert!(RejectCode::try_from(u64::from(raw)).is_err());
        }
    }
    let zero = CallRejected::with_rejection(0, String::new())
        .reject_code()
        .unwrap_err();
    let maximum = CallRejected::with_rejection(u32::MAX, String::new())
        .reject_code()
        .unwrap_err();
    assert_ne!(zero, maximum);
}
