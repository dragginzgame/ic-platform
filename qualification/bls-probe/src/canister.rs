//! Measurement-only cache and fixed framing, shared by both implementation arms.
use ic_platform_runtime::ic0;
use std::cell::RefCell;

#[cfg(feature = "candidate")]
use ic_platform::bls as backend;
#[cfg(feature = "baseline")]
use platform_baseline::bls as backend;

type CachedKey = backend::PreparedPublicKey;

thread_local! {
    static KEY: RefCell<Option<([u8; 96], CachedKey)>> = const { RefCell::new(None) };
}

#[expect(
    unsafe_code,
    reason = "The benchmark explicitly exports its IC update ABI"
)]
#[unsafe(export_name = "canister_update verify_once")]
pub extern "C" fn verify_once() {
    run(false);
}

#[expect(
    unsafe_code,
    reason = "The benchmark explicitly exports its IC update ABI"
)]
#[unsafe(export_name = "canister_update verify_cached")]
pub extern "C" fn verify_cached() {
    run(true);
}

fn run(cached: bool) {
    let mut input = vec![0; ic0::msg_arg_data_size()];
    ic0::msg_arg_data_copy(&mut input, 0);
    let start = ic0::performance_counter(0);
    let valid = input.len() >= 144 && {
        let (signature, rest) = input.split_at(48);
        let (key, message) = rest.split_at(96);
        if cached {
            cached_verify(signature, message, key.try_into().unwrap())
        } else {
            backend::verify_bls_signature(signature, message, key).is_ok()
        }
    };
    let instructions = ic0::performance_counter(0) - start;
    let mut response = [0; 9];
    response[0] = u8::from(valid);
    response[1..].copy_from_slice(&instructions.to_le_bytes());
    ic0::msg_reply_data_append(&response);
    ic0::msg_reply();
}

fn cached_verify(signature: &[u8], message: &[u8], key: &[u8; 96]) -> bool {
    let Ok(signature) = backend::Signature::deserialize(signature) else {
        return false;
    };
    KEY.with(|cache| {
        let mut cached = cache.borrow_mut();
        if cached.as_ref().is_none_or(|(encoded, _)| encoded != key) {
            let Ok(public) = backend::PublicKey::deserialize(key) else {
                return false;
            };
            let public = public.prepare();
            *cached = Some((*key, public));
        }
        cached
            .as_ref()
            .unwrap()
            .1
            .verify(message, &signature)
            .is_ok()
    })
}
