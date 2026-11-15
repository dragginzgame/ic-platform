use super::{Call, Principal, Wait};

mod custody;

#[test]
fn borrowed_inputs_and_request_facts_remain_exact() {
    let method = String::from("  方法\0 ");
    let args = [0, 255, 128];
    let target = Principal::anonymous();
    let call = Call::new(target, method.as_str(), Wait::Bounded { seconds: 0 })
        .args(&args)
        .cycles(u128::MAX);
    assert_eq!(call.target(), target);
    assert_eq!(call.method().as_ptr(), method.as_ptr());
    assert_eq!(call.method(), method);
    assert_eq!(call.raw_args().as_ptr(), args.as_ptr());
    assert_eq!(call.raw_args(), args);
    assert_eq!(call.wait(), Wait::Bounded { seconds: 0 });
    assert_eq!(call.attached_cycles(), u128::MAX);
}

#[test]
fn ownership_outlives_sources_and_releases_old_borrows() {
    let call = {
        let old = vec![1, 2];
        Call::new(
            Principal::management_canister(),
            String::from("call"),
            Wait::Unbounded,
        )
        .args(&old)
        .owned_args(vec![3, 4])
    };
    assert_eq!(call.raw_args(), &[3, 4]);
    assert_eq!(call.method(), "call");
    assert_eq!(call.wait(), Wait::Unbounded);
    assert_eq!(call.attached_cycles(), 0);
    let replacement = [5];
    let call = call.args(&replacement);
    assert_eq!(call.raw_args(), &[5]);
}

#[test]
fn default_arguments_are_empty_raw_bytes() {
    let call = Call::new(
        Principal::anonymous(),
        "",
        Wait::Bounded { seconds: u32::MAX },
    );
    assert!(call.raw_args().is_empty());
    assert_eq!(call.method(), "");
    assert_eq!(call.wait(), Wait::Bounded { seconds: u32::MAX });
}
