//! Cost and submission checks against the independent Wasm IC0 host.

use candidate_runtime::call::{Call, CostOverflow, SubmitError, Wait};
use candidate_types::Principal;
use std::num::NonZeroU32;

pub(super) fn check() {
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

fn ic0_notify() {
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
