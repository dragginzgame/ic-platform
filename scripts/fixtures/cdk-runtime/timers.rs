use ic_timers::{
    OnceDecision, OnceRunResult, TimerCompletion, TimerIdentity, TimerSchedule, TimerStart,
};
use std::time::Duration;

#[ic_platform::cdk::update]
fn timer_start(trap: bool, cancel: bool) {
    let registration = ic_timers::register_once(
        TimerIdentity::try_new("platform", "qualification", "once").unwrap(),
        TimerStart::RemoveWhenStopped {
            initial: TimerSchedule::After(Duration::from_secs(1)),
        },
        move |_| async move {
            let _cleanup = Cleanup;
            Call::unbounded_wait(api::canister_self(), "scalar")
                .with_arg(1u64)
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
