//! Raw callers use the same adapters without selecting entrypoint generation.
use ic_platform::{Principal, cdk::call};

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
    use ic_platform::cdk::{api, stable};

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
/// Consumer names must not capture standard-library paths inside exported macros.
pub fn print_with_shadowed_names() {
    mod std {}
    macro_rules! format {
        () => {
            0_u8
        };
        ($($tokens:tt)+) => {
            compile_error!("consumer format macro captured Platform print")
        };
    }
    let value = format!();
    ic_platform::cdk::println!("platform print hygiene");
    ic_platform::cdk::println!("platform print value {}", value);
}

#[test]
fn native_print_ignores_consumer_names() {
    print_with_shadowed_names();
}
