//! Installed timer checks qualification scenarios.
use super::*;
#[cfg(feature = "timers-composition")]
use ic_timers::{
    OnceDecision, OnceRunResult, TimerCompletion, TimerIdentity, TimerSchedule, TimerStart,
};
#[cfg(feature = "timers-composition")]
use std::time::Duration;

#[cfg(feature = "timers-composition")]
#[ic_platform_runtime::cdk::update(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::TimerStart as u64 }>",
    encode_with = "codec::encode_unit::<{ protocol::Method::TimerStart as u64 }>"
)]
fn timer_start(trap: bool, cancel: bool) {
    let registration = ic_timers::register_once(
        TimerIdentity::try_new("platform", "qualification", "once").unwrap(),
        TimerStart::RemoveWhenStopped {
            initial: TimerSchedule::After(Duration::from_secs(1)),
        },
        move |_| async move {
            let _cleanup = Cleanup;
            transport::unbounded_wait(api::canister_self(), "scalar")
                .take_raw_args(codec::request("scalar", &1u64))
                .execute()
                .await
                .unwrap();
            if trap {
                api::trap("fixture timer callback trap");
            }
            VALUE.set(VALUE.get() + 1);
            OnceRunResult::new(TimerCompletion::no_work(), OnceDecision::Stop)
        },
    )
    .unwrap();
    if cancel {
        registration.cancel().unwrap();
    }
}
