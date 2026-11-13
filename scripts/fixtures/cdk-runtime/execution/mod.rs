//! Installed execution qualification scenarios.
#[cfg(not(feature = "prepared-call"))]
use super::Execute;
use super::{CLEANED, Cell, Cleanup, VALUE, api, codec, futures, protocol, transport};

thread_local! {
    static MULTIPLEXED_CLEANED: Cell<u64> = const { Cell::new(0) };
}

#[ic_platform_runtime::cdk::query(
    decode_with = "codec::decode::<_, { protocol::Method::MultiplexedCleaned as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::MultiplexedCleaned as u64 }>"
)]
fn multiplexed_cleaned() -> u64 {
    MULTIPLEXED_CLEANED.get()
}

// Both requests are enqueued before either can reply. Trapping while processing
// the first response rolls back that callback; cleanup must destroy both pending
// futures, and the later callback must not revive the cancelled method.
#[ic_platform_runtime::cdk::update(
    decode_with = "codec::decode::<_, { protocol::Method::MultiplexedTrap as u64 }>",
    encode_with = "codec::encode_unit::<{ protocol::Method::MultiplexedTrap as u64 }>"
)]
async fn multiplexed_trap() {
    struct RecoveryDrop;
    impl Drop for RecoveryDrop {
        fn drop(&mut self) {
            if futures::is_recovering_from_trap() {
                MULTIPLEXED_CLEANED.set(MULTIPLEXED_CLEANED.get() + 1);
            }
        }
    }
    let _drop = RecoveryDrop;
    let mut calls: Vec<_> = (0..2)
        .map(|_| {
            Box::pin(
                transport::unbounded_wait(api::canister_self(), "scalar")
                    .take_raw_args(codec::request("scalar", &1u64))
                    .execute(),
            )
        })
        .collect();
    std::future::poll_fn(|cx| {
        let mut replied = false;
        for call in &mut calls {
            if let std::task::Poll::Ready(result) = call.as_mut().poll(cx) {
                result.unwrap();
                replied = true;
            }
        }
        if replied {
            std::task::Poll::Ready(())
        } else {
            std::task::Poll::Pending
        }
    })
    .await;
    api::trap("fixture multiplexed callback trap");
}

// Owner-local cancellation: dropping a queued future exactly once, and refusing
// to cancel a new task through a stale handle. No scheduler is involved.
#[ic_platform_runtime::cdk::update(
    decode_with = "codec::decode::<_, { protocol::Method::CancelTask as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::CancelTask as u64 }>"
)]
fn cancel_task() -> u64 {
    struct Dropped(std::rc::Rc<Cell<u64>>);
    impl Drop for Dropped {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let count = std::rc::Rc::new(Cell::new(0));
    let spawn = || {
        let dropped = Dropped(count.clone());
        ic_platform_runtime::executor::spawn_migratory(async move {
            std::future::pending::<()>().await;
            drop(dropped);
        })
    };
    let first = spawn();
    ic_platform_runtime::executor::cancel_task(&first);
    assert_eq!(count.get(), 1);
    let second = spawn();
    ic_platform_runtime::executor::cancel_task(&first);
    assert_eq!(count.get(), 1);
    ic_platform_runtime::executor::cancel_task(&second);
    count.get()
}

#[ic_platform_runtime::cdk::query(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::Value as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::Value as u64 }>"
)]
fn value() -> u64 {
    VALUE.get()
}

// A query must drain protected wakes produced while its own future is polling.
#[ic_platform_runtime::cdk::query(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::SelfWakeQuery as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::SelfWakeQuery as u64 }>"
)]
async fn self_wake_query() -> u64 {
    let mut polls = 0;
    let mut saved: Option<std::task::Waker> = None;
    std::future::poll_fn(|cx| {
        polls += 1;
        if let Some(first) = &saved {
            assert!(first.will_wake(cx.waker()));
            std::task::Poll::Ready(polls)
        } else {
            saved = Some(cx.waker().clone());
            for _ in 0..32 {
                cx.waker().wake_by_ref();
            }
            std::task::Poll::Pending
        }
    })
    .await
}

#[ic_platform_runtime::cdk::query(
    name = "scalar",
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::Scalar as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::Scalar as u64 }>"
)]
fn r#match(input: u64) -> u64 {
    input + 1
}

#[ic_platform_runtime::cdk::query(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::Tuple as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::Tuple as u64 }>"
)]
fn tuple() -> (u64, bool) {
    (7, true)
}

#[ic_platform_runtime::cdk::query(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::Unit as u64 }>",
    encode_with = "codec::encode_unit::<{ protocol::Method::Unit as u64 }>"
)]
fn unit() {}

#[ic_platform_runtime::cdk::update(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::Awaited as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::Awaited as u64 }>"
)]
async fn awaited(input: u64) -> u64 {
    let bytes = transport::unbounded_wait(api::canister_self(), "scalar")
        .with_wait_policy(ic_platform_runtime::cdk::call::WaitPolicy::Bounded {
            timeout_seconds: 300,
        })
        .take_raw_args(codec::request("scalar", &input))
        .execute()
        .await
        .unwrap()
        .into_bytes();
    codec::response("scalar", &bytes)
}

#[ic_platform_runtime::cdk::query(
    guard = "codec::admit",
    composite = true,
    decode_with = "codec::decode::<_, { protocol::Method::Composite as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::Composite as u64 }>"
)]
async fn composite(input: u64) -> u64 {
    let bytes = transport::bounded_wait(api::canister_self(), "scalar")
        .with_wait_policy(ic_platform_runtime::cdk::call::WaitPolicy::Unbounded)
        .take_raw_args(codec::request("scalar", &input))
        .execute()
        .await
        .unwrap()
        .into_bytes();
    codec::response("scalar", &bytes)
}

fn refuse() -> Result<(), String> {
    Err("fixture guard refused".into())
}

#[ic_platform_runtime::cdk::query(
    guard = "codec::admit",
    guard = "refuse",
    decode_with = "codec::decode::<_, { protocol::Method::Guarded as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::Guarded as u64 }>"
)]
fn guarded() -> u64 {
    999
}

#[ic_platform_runtime::cdk::query(
    guard = "codec::admit",
    manual_reply = true,
    decode_with = "codec::decode::<_, { protocol::Method::Manual as u64 }>"
)]
fn manual() {
    api::msg_reply(codec::encode::<_, { protocol::Method::Manual as u64 }>(
        vec![0u8, 255],
    ));
}

#[ic_platform_runtime::cdk::update(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::CallbackTrap as u64 }>",
    encode_with = "codec::encode_unit::<{ protocol::Method::CallbackTrap as u64 }>"
)]
async fn callback_trap() {
    let _cleanup = Cleanup;
    transport::unbounded_wait(api::canister_self(), "scalar")
        .take_raw_args(codec::request("scalar", &1u64))
        .execute()
        .await
        .unwrap();
    api::trap("fixture callback trap");
}

#[ic_platform_runtime::cdk::query(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::Cleaned as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::Cleaned as u64 }>"
)]
fn cleaned() -> u64 {
    CLEANED.get()
}

#[ic_platform_runtime::cdk::update(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::ProtectedPending as u64 }>",
    encode_with = "codec::encode_unit::<{ protocol::Method::ProtectedPending as u64 }>"
)]
fn protected_pending() {
    futures::spawn(std::future::pending());
}

#[ic_platform_runtime::cdk::update(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::Migratory as u64 }>",
    encode_with = "codec::encode_unit::<{ protocol::Method::Migratory as u64 }>"
)]
fn migratory() {
    futures::spawn_migratory(async { VALUE.set(VALUE.get() + 1) });
}

#[ic_platform_runtime::cdk::query(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::Controller as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::Controller as u64 }>"
)]
fn controller() -> bool {
    api::is_controller(&api::msg_caller())
}

#[ic_platform_runtime::cdk::update(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::Missing as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::Missing as u64 }>"
)]
async fn missing() -> bool {
    // A successful transport can return bytes invalid for the chosen codec.
    // Do not reclassify that reply as a submission failure or retry permission.
    let reply = transport::bounded_wait(api::canister_self(), "manual")
        .take_raw_args(codec::request("manual", &()))
        .execute()
        .await
        .unwrap();
    assert!(
        ic_wire::native::decode_response::<()>(
            protocol::Method::Manual as u64,
            reply.as_ref(),
            protocol::LIMITS,
        )
        .is_err()
    );
    matches!(
        transport::bounded_wait(api::canister_self(), "no_such_method")
            .take_raw_args(codec::request("no_such_method", &()))
            .execute()
            .await,
        Err(ic_platform_runtime::cdk::call::CallFailed::CallRejected(_))
    )
}

#[ic_platform_runtime::cdk::update(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::SubmissionPressure as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::SubmissionPressure as u64 }>"
)]
async fn submission_pressure() -> (u64, u64) {
    let mut calls: Vec<_> = (0..600)
        .map(|_| {
            Some(Box::pin(
                transport::unbounded_wait(api::canister_self(), "scalar")
                    .take_raw_args(codec::request("scalar", &1u64))
                    .execute(),
            ))
        })
        .collect();
    let (mut replies, mut refused) = (0u64, 0u64);
    std::future::poll_fn(move |cx| {
        for slot in &mut calls {
            let Some(call) = slot else { continue };
            match call.as_mut().poll(cx) {
                std::task::Poll::Ready(result) => {
                    match result {
                        Ok(_) => replies += 1,
                        Err(ic_platform_runtime::cdk::call::CallFailed::CallPerformFailed(_)) => {
                            refused += 1
                        }
                        Err(error) => panic!("unexpected submission error: {error}"),
                    }
                    *slot = None;
                }
                std::task::Poll::Pending => {}
            }
        }
        if calls.iter().all(Option::is_none) {
            std::task::Poll::Ready((replies, refused))
        } else {
            std::task::Poll::Pending
        }
    })
    .await
}

#[ic_platform_runtime::cdk::update(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::InsufficientCycles as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::InsufficientCycles as u64 }>"
)]
async fn insufficient_cycles() -> bool {
    matches!(transport::bounded_wait(api::canister_self(), "scalar")
        .take_raw_args(codec::request("scalar", &1u64))
        .with_cycles(api::canister_liquid_cycle_balance()).execute().await,
        Err(ic_platform_runtime::cdk::call::CallFailed::InsufficientLiquidCycleBalance(error))
            if error.available < error.required)
}

#[ic_platform_runtime::cdk::update(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::WeakPending as u64 }>",
    encode_with = "codec::encode_unit::<{ protocol::Method::WeakPending as u64 }>"
)]
fn weak_pending() {
    futures::spawn_weak(std::future::pending());
}
