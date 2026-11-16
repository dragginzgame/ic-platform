//! Public local Types/Runtime crossing without an executor or IC imports.

use candidate_runtime::call::{Call, Completion, Outcome, RejectCode, Wait};
use candidate_types::{Account, Principal};
use std::num::NonZeroU32;

pub fn check() {
    #[cfg(all(feature = "runtime-ic0", target_arch = "wasm32"))]
    ic0_cost();
    for len in 0..=29 {
        let owner = Principal::from_slice(&vec![0xa5; len]);
        let account = Account::from(owner);
        let method = String::from(" méthode\0 ");
        let args = [0, 128, 255];
        let call = Call::new(account.owner, method.as_str(), Wait::Bounded { seconds: 0 })
            .args(&args)
            .cycles(u128::MAX);
        let recovered: Principal = call.target();
        assert_eq!(recovered, owner);
        assert_eq!(call.raw_args().as_ptr(), args.as_ptr());
        assert_eq!(call.method(), method);
        assert_eq!(call.wait(), Wait::Bounded { seconds: 0 });
        assert_eq!(call.attached_cycles(), u128::MAX);
    }
    let owned = {
        let source = vec![1, 2];
        Call::new(
            Principal::anonymous(),
            String::from("owned"),
            Wait::Unbounded,
        )
        .args(&source)
        .owned_args(vec![3, 4])
    };
    assert_eq!(owned.raw_args(), &[3, 4]);
    for refund in [None, Some(0), Some(u128::MAX)] {
        let completion = Completion::from_reply(vec![0, 255], refund);
        assert_eq!(
            completion.into_parts(),
            (Outcome::Reply(vec![0, 255]), refund)
        );
        for code in [1, 2, 3, 4, 5, 6, 7, u32::MAX] {
            let completion =
                Completion::from_rejection(NonZeroU32::new(code).unwrap(), "拒否\0".into(), refund);
            assert_eq!(completion.refunded_cycles(), refund);
            assert_eq!(
                matches!(completion.outcome(), Outcome::OutcomeUnknown(_)),
                code == 6
            );
            let (Outcome::Rejected(rejection) | Outcome::OutcomeUnknown(rejection)) =
                completion.outcome()
            else {
                panic!("expected rejection")
            };
            assert_eq!(rejection.code(), code);
            assert_eq!(rejection.message(), "拒否\0");
            if code == 6 {
                assert_eq!(rejection.kind(), Some(RejectCode::SysUnknown));
            }
            if code > 6 {
                assert_eq!(rejection.kind(), None);
            }
        }
    }
}

#[cfg(all(feature = "runtime-ic0", target_arch = "wasm32"))]
fn ic0_cost() {
    use candidate_runtime::call::CostOverflow;
    // The external Wasm host supplies independent u128 values in this order.
    for (attachment, expected) in [
        (0, Ok(0)),
        (9, Ok((1_u128 << 64) + 16)),
        (0, Ok(u128::MAX)),
        (1, Err(CostOverflow)),
        (u128::MAX, Err(CostOverflow)),
    ] {
        let call = Call::new(Principal::anonymous(), "mé\0", Wait::Unbounded)
            .args(&[0, 128, 255])
            .cycles(attachment);
        assert_eq!(call.required_cycles(), expected);
    }
    ic0_notify();
}

#[cfg(all(feature = "runtime-ic0", target_arch = "wasm32"))]
fn ic0_notify() {
    use candidate_runtime::call::SubmitError;
    let args = [0, 128, 255];
    // Exact-balance success, including a nonzero high cycle word. A bounded
    // request must still install no wait or callback for notify.
    let result = Call::new(Principal::anonymous(), "mé\0", Wait::Bounded { seconds: 0 })
        .args(&args)
        .cycles((1_u128 << 100) + 7)
        .notify();
    assert!(result.is_ok());
    // Empty method/argument bytes and a zero-cost/zero-balance observation.
    assert!(
        Call::new(Principal::management_canister(), "", Wait::Unbounded)
            .notify()
            .is_ok()
    );
    assert_eq!(
        Call::new(Principal::anonymous(), "mé\0", Wait::Unbounded)
            .args(&args)
            .cycles(5)
            .notify(),
        Err(SubmitError::InsufficientBalance {
            available: 17,
            required: 18
        }),
    );
    assert_eq!(
        Call::new(Principal::anonymous(), "mé\0", Wait::Unbounded)
            .args(&args)
            .cycles(1)
            .notify(),
        Err(SubmitError::CostOverflow),
    );
    for code in [2, u32::MAX] {
        assert_eq!(
            Call::new(
                Principal::anonymous(),
                "mé\0",
                Wait::Bounded { seconds: u32::MAX }
            )
            .args(&args)
            .notify(),
            Err(SubmitError::Refused(NonZeroU32::new(code).unwrap())),
        );
    }
}
