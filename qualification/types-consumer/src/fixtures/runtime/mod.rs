//! Public local Types/Runtime crossing without an executor or IC imports.

use candidate_runtime::call::{Call, Completion, Outcome, RejectCode, Wait};
use candidate_types::{Account, Principal};
use std::num::NonZeroU32;

#[cfg(all(feature = "runtime-ic0", target_arch = "wasm32"))]
mod ic0;

pub fn check() {
    #[cfg(all(feature = "runtime-ic0", target_arch = "wasm32"))]
    ic0::check();
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
