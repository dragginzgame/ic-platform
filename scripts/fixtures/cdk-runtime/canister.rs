//! A standalone Wire canister consuming Platform's owned runtime and generator.
#[cfg(feature = "external-candid")]
use ic_platform::candid::contract;
use ic_platform_runtime::{
    Principal,
    cdk::{api, futures},
};
use std::cell::Cell;
#[cfg(not(feature = "prepared-call"))]
use transport::Execute;
#[cfg(feature = "external-candid")]
// Copied from the workspace-formatted management_consumer.rs by the runtime script.
#[rustfmt::skip]
mod management_proof;
mod cycles;
mod execution;
mod stable_checks;
mod system_checks;
#[cfg(feature = "timers-composition")]
mod timer_checks;
mod transport;

#[cfg(feature = "external-candid")]
#[ic_platform_runtime::cdk::query(
    decode_with = "codec::decode::<_, { protocol::Method::ManagementProof as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::ManagementProof as u64 }>"
)]
fn management_proof() -> u8 {
    management_proof::roundtrip();
    management_proof::history_roundtrip();
    management_proof::canister_snapshot_status_roundtrip();
    management_proof::lifecycle_settings_and_ecdsa_contracts();
    4
}

thread_local! {
    static VALUE: Cell<u64> = const { Cell::new(0) };
    static CLEANED: Cell<u64> = const { Cell::new(0) };
}

// Compare just the message-buffer read, excluding Wire decoding/encoding.
// Both variants remain in this qualification fixture, not the production API.
#[ic_platform_runtime::cdk::query(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::BufferCopy as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::BufferCopy as u64 }>"
)]
fn buffer_copy(baseline: bool, _payload: Vec<u8>) -> (u64, Vec<u8>) {
    let start = api::instruction_counter();
    let bytes = if baseline {
        let len = ic_platform_runtime::ic0::msg_arg_data_size();
        let mut bytes = vec![0; len];
        ic_platform_runtime::ic0::msg_arg_data_copy(&mut bytes, 0);
        bytes
    } else {
        api::msg_arg_data()
    };
    let instructions = api::instruction_counter() - start;
    (instructions, bytes)
}

#[ic_platform_runtime::cdk::init(
    decode_with = "codec::decode::<_, { protocol::Method::Initialize as u64 }>"
)]
fn initialize(value: u64) {
    VALUE.set(value);
    #[cfg(feature = "timers-composition")]
    ic_timers::initialize_runtime().unwrap();
}

#[ic_platform_runtime::cdk::post_upgrade(
    decode_with = "codec::decode::<_, { protocol::Method::AfterUpgrade as u64 }>"
)]
fn after_upgrade(value: u64) {
    #[cfg(feature = "external-candid")]
    {
        // Fixture-owned legacy bytes: deliberately retain the old no-finalization
        // decoding policy. Platform no longer owns a checkpoint API.
        let mut bytes = vec![0; (api::stable_size() * 65_536) as usize];
        ic_platform_runtime::cdk::stable::stable_read(0, &mut bytes);
        let mut decoder = ic_platform::candid::decoder::IDLDeserialize::new(&bytes).unwrap();
        CHECKPOINT.set(contract::ArgumentDecoder::decode(&mut decoder).unwrap());
    }
    VALUE.set(value);
    #[cfg(feature = "timers-composition")]
    ic_timers::initialize_runtime().unwrap();
}

struct Cleanup;

impl Drop for Cleanup {
    fn drop(&mut self) {
        if futures::is_recovering_from_trap() {
            CLEANED.set(CLEANED.get() + 1);
        }
    }
}

thread_local! {
    static RECEIVED: Cell<u128> = const { Cell::new(0) };
}

#[cfg(test)]
mod tests;

thread_local! {
    static CHECKPOINT: Cell<(bool, bool, u64, u64)> = const { Cell::new((false, false, 0, 0)) };
}

#[ic_platform_runtime::cdk::pre_upgrade]
fn save_checkpoint() {
    #[cfg(feature = "external-candid")]
    {
        // This fixed fixture record fits one page. Preserve its original Candid
        // bytes without introducing a production checkpoint replacement.
        let bytes = contract::encode_args((true, false, VALUE.get(), CLEANED.get())).unwrap();
        assert!(bytes.len() <= 65_536);
        if api::stable_size() == 0 {
            ic_platform_runtime::cdk::stable::stable_grow(1).unwrap();
        }
        ic_platform_runtime::cdk::stable::stable_write(0, &bytes);
    }
}

#[ic_platform_runtime::cdk::query(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::Checkpoint as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::Checkpoint as u64 }>"
)]
fn checkpoint() -> (bool, bool, u64, u64) {
    CHECKPOINT.get()
}

#[ic_platform_runtime::cdk::query(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::CheckpointBytes as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::CheckpointBytes as u64 }>"
)]
fn checkpoint_bytes() -> Vec<u8> {
    #[cfg(feature = "external-candid")]
    {
        let expected = contract::encode_args(CHECKPOINT.get()).unwrap();
        let mut actual = vec![0; expected.len()];
        ic_platform_runtime::cdk::stable::stable_read(0, &mut actual);
        assert_eq!(actual, expected);
        actual
    }
    #[cfg(not(feature = "external-candid"))]
    Vec::new()
}

#[ic_platform_runtime::cdk::inspect_message]
fn inspect() {
    assert!(!api::msg_method_name().is_empty());
    api::accept_message();
}

#[cfg(feature = "external-candid")]
#[ic_platform_runtime::cdk::update(
    guard = "codec::admit",
    decode_with = "codec::decode::<_, { protocol::Method::ExternalManagement as u64 }>",
    encode_with = "codec::encode::<_, { protocol::Method::ExternalManagement as u64 }>"
)]
async fn external_management() -> bool {
    let response = transport::unbounded_wait(
        Principal::management_canister(),
        ic_platform::raw_rand::METHOD,
    )
    .with_raw_args(ic_platform::raw_rand::encode().as_slice())
    .execute()
    .await
    .unwrap();
    ic_platform::raw_rand::decode(response.as_ref()).is_ok()
}

mod codec;
#[cfg(feature = "external-adapters")]
mod external_canister;
mod protocol;
mod wire_canister;
mod wire_protocol;
