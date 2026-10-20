//! Raw callers use the same adapters without selecting entrypoint generation.
use ic_platform::{
    Principal,
    cdk::{api, call, stable},
};

/// Compile an actual awaited raw call without a generated entrypoint or codec.
pub async fn forward(destination: Principal, bytes: Vec<u8>) -> Result<Vec<u8>, call::CallFailed> {
    let response = call::Call::bounded_wait(destination, "forward")
        .with_wait_policy(call::WaitPolicy::Bounded {
            timeout_seconds: 30,
        })
        .take_raw_args(bytes)
        .await?;
    Ok(response.into_bytes())
}

#[test]
fn transport_and_system_types_are_available_without_entrypoints() {
    let rejected = call::CallRejected::with_rejection(3, "missing".into());
    assert_eq!(
        rejected.reject_code(),
        Ok(call::RejectCode::DestinationInvalid)
    );
    let _: fn() -> Principal = api::canister_self;
    let _: fn(u64, &[u8]) = stable::stable_write;
    let _: fn(u64, &mut [u8]) = stable::stable_read;
    let _: fn() -> bool = ic_platform::cdk::futures::is_recovering_from_trap;
}
