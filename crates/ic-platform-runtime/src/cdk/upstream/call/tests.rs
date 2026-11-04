use super::*;

#[test]
fn wait_policy_replaces_mode_without_changing_payload_or_payment() {
    for initial in [
        Call::bounded_wait(Principal::anonymous(), "probe"),
        Call::unbounded_wait(Principal::anonymous(), "probe"),
    ] {
        let initial = initial.take_raw_args(vec![1, 2]).with_cycles(123);
        for timeout_seconds in [0, 1, 300, u32::MAX] {
            let call = initial
                .clone()
                .with_wait_policy(WaitPolicy::Bounded { timeout_seconds });
            assert_eq!(call.timeout_seconds, Some(timeout_seconds));
            assert_eq!(call.encoded_args.as_ref(), [1, 2]);
            assert_eq!(call.cycles, 123);
            assert_eq!(call.method, "probe");
            assert_eq!(call.canister_id, Principal::anonymous());
            let call = call.with_wait_policy(WaitPolicy::Unbounded);
            assert_eq!(call.timeout_seconds, None);
        }
    }
    assert_eq!(
        Call::bounded_wait(Principal::anonymous(), "probe").timeout_seconds,
        Some(300)
    );
    assert_eq!(
        Call::unbounded_wait(Principal::anonymous(), "probe").timeout_seconds,
        None
    );
}

#[test]
fn raw_constructors_do_not_choose_a_codec() {
    for call in [
        Call::bounded_wait(Principal::anonymous(), "probe"),
        Call::unbounded_wait(Principal::anonymous(), "probe"),
    ] {
        assert!(call.encoded_args.is_empty());
        assert_eq!(call.with_raw_args(b"raw").encoded_args.as_ref(), b"raw");
    }
}

#[test]
fn external_unit_arguments_are_explicit() {
    let args = b"DIDL\0\0".to_vec();
    let call =
        Call::unbounded_wait(Principal::management_canister(), "raw_rand").with_raw_args(&args);
    assert_eq!(call.encoded_args.as_ref(), b"DIDL\0\0");
}
