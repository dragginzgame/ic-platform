use super::*;

fn prepared() -> Call<'static, 'static> {
    let method = String::from("runtime-selected-method");
    let source = vec![1, 2, 3];
    Call::new(Principal::anonymous(), method, WaitPolicy::Unbounded).take_raw_args(source)
}

#[test]
fn owned_request_outlives_its_sources_and_future_creation_is_inert() {
    let call = prepared();
    assert_eq!(call.method(), "runtime-selected-method");
    assert_eq!(call.raw_args(), [1, 2, 3]);
    assert_eq!(call.target(), Principal::anonymous());
    // Polling would require IC system calls; creation/drop must not make any.
    drop(call.execute());
}

#[test]
fn borrowed_inputs_are_not_copied_and_can_be_replaced_by_owned_bytes() {
    let method = String::from("borrowed");
    let args = vec![5, 6];
    let call = Call::new(
        Principal::anonymous(),
        method.as_str(),
        WaitPolicy::Unbounded,
    )
    .with_raw_args(&args);
    assert_eq!(call.method().as_ptr(), method.as_ptr());
    assert_eq!(call.raw_args().as_ptr(), args.as_ptr());
    let call = call.take_raw_args(vec![9]);
    drop(args);
    assert_eq!(call.raw_args(), [9]);
}

#[test]
fn explicit_configuration_preserves_zero_and_last_attachment() {
    let call = prepared()
        .with_cycles(4)
        .with_cycles(u128::MAX)
        .with_wait_policy(WaitPolicy::Bounded { timeout_seconds: 0 });
    assert_eq!(call.attached_cycles(), u128::MAX);
    assert_eq!(
        call.wait_policy(),
        WaitPolicy::Bounded { timeout_seconds: 0 }
    );
    assert_eq!(
        call.with_wait_policy(WaitPolicy::Unbounded).wait_policy(),
        WaitPolicy::Unbounded
    );
    let empty = Call::new(Principal::anonymous(), "empty", WaitPolicy::Unbounded);
    assert!(empty.raw_args().is_empty());
    assert_eq!(empty.attached_cycles(), 0);
}

#[test]
fn rejection_types_retain_platform_identity_and_unknown_codes() {
    let rejected: ic_platform::cdk::call::CallRejected =
        CallRejected::with_rejection(12345, "opaque".into());
    assert_eq!(rejected.raw_reject_code(), 12345);
    assert!(rejected.reject_code().is_err());
    let failure: CallFailed = ic_platform::cdk::call::CallFailed::CallRejected(rejected);
    assert!(matches!(failure, CallFailed::CallRejected(error) if error.raw_reject_code() == 12345));
}

#[test]
fn uncertain_system_rejection_is_not_reclassified_as_non_submission() {
    let rejected =
        CallRejected::with_rejection(RejectCode::SysUnknown as u32, "unknown outcome".into());
    assert_eq!(rejected.reject_code().unwrap(), RejectCode::SysUnknown);
    assert!(matches!(
        CallFailed::CallRejected(rejected),
        CallFailed::CallRejected(_)
    ));
}
