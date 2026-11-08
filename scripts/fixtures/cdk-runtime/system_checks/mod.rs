//! Installed system checks qualification scenarios.
use super::*;

#[ic_platform_runtime::cdk::update(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::PublishRoot as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::PublishRoot as u64 }>"
)]
fn publish_root(root: Vec<u8>) -> bool {
    api::certified_data_set(root);
    api::data_certificate().is_none()
}

#[ic_platform_runtime::cdk::query(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::Certificate as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::Certificate as u64 }>"
)]
fn certificate() -> Option<Vec<u8>> {
    api::data_certificate()
}

#[ic_platform_runtime::cdk::update(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::SystemHelpers as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::SystemHelpers as u64 }>"
)]
fn system_helpers() -> (u64, u128, bool) {
    assert!(!api::root_key().is_empty());
    assert_ne!(api::subnet_self(), Principal::anonymous());
    assert!(api::canister_cycle_balance() >= api::canister_liquid_cycle_balance());
    assert!(api::cost_create_canister() > 0);
    assert!(api::cost_http_request(100, 100) > 0);
    assert!(api::cost_sign_with_ecdsa("missing-fixture-key", 0).is_err());
    assert!(api::performance_counter(api::PerformanceCounterType::InstructionCounter) > 0);
    let previous = api::global_timer_set(0);
    api::global_timer_set(previous);
    api::debug_print("platform system fixture");
    ic_platform_runtime::cdk::println!("platform {}", "macro");
    (
        api::canister_version(),
        api::cycles_burn(123),
        api::data_certificate().is_none(),
    )
}
