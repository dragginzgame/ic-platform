use super::{raw, system};
use crate::call::{Call, SubmitError, Wait};
use ic_platform_types::Principal;
use std::{
    future::Future,
    panic::catch_unwind,
    pin::pin,
    task::{Context, Poll, Waker},
};

#[test]
fn native_observations_refuse_and_no_request_can_be_submitted() {
    assert!(catch_unwind(system::caller).is_err());
    assert!(catch_unwind(system::canister_self).is_err());
    assert!(catch_unwind(system::time).is_err());
    assert!(catch_unwind(system::now).is_err());
    assert!(catch_unwind(system::now_millis).is_err());
    assert!(catch_unwind(system::now_secs).is_err());
    assert!(catch_unwind(|| system::is_controller(&Principal::anonymous())).is_err());
    assert!(catch_unwind(raw::msg_arg_data_size).is_err());
    let request = Call::new(
        Principal::management_canister(),
        "example",
        Wait::Bounded { seconds: 1 },
    );
    let mut response = pin!(request.execute());
    assert!(matches!(
        response
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop())),
        Poll::Ready(Err(SubmitError::OutsideCanister))
    ));
}

#[test]
fn native_lifecycle_fails_before_application_callbacks() {
    let guarded = std::cell::Cell::new(false);
    let result = catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = crate::entrypoints::lifecycle(
            0,
            false,
            || {
                guarded.set(true);
                Ok::<_, ()>(())
            },
            |_| Ok::<_, ()>(()),
            |()| {},
        );
    }));
    assert!(result.is_err());
    assert!(!guarded.get());
}
