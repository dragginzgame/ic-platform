//! This package declares only ic-call for raw calls; codecs are separate selections.
use ic_call::{Call, Principal, WaitPolicy};

pub fn prepare(target: Principal) -> Call<'static, 'static> {
    let method = String::from("runtime-selected-method");
    let args = vec![1, 2, 3];
    Call::new(
        target,
        method,
        WaitPolicy::Bounded {
            timeout_seconds: 300,
        },
    )
    .take_raw_args(args)
}

pub async fn roundtrip_raw(target: Principal) -> Result<Vec<u8>, ic_call::CallFailed> {
    prepare(target)
        .execute()
        .await
        .map(ic_call::Response::into_bytes)
}

#[cfg(feature = "wire")]
pub fn wire_request(target: Principal) -> Call<'static, 'static> {
    let bytes = ic_wire::native::encode_request(17, &42u64, limits()).unwrap();
    Call::new(target, String::from("wire"), WaitPolicy::Unbounded).take_raw_args(bytes)
}

#[cfg(feature = "wire")]
fn limits() -> ic_wire::Limits {
    ic_wire::Limits {
        bytes: 1024,
        allocation: 4096,
        nodes: 64,
        depth: 8,
    }
}

#[cfg(feature = "external-candid")]
pub fn external_request(target: Principal) -> Call<'static, 'static> {
    let bytes = ic_platform::candid::contract::encode_args(()).unwrap();
    Call::new(target, String::from("raw_rand"), WaitPolicy::Unbounded).take_raw_args(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ownership_and_lazy_execution() {
        let call = prepare(Principal::anonymous());
        assert_eq!(call.method(), "runtime-selected-method");
        assert_eq!(call.raw_args(), [1, 2, 3]);
        drop(call.execute());
    }
    #[cfg(feature = "wire")]
    #[test]
    fn wire_bytes_are_unchanged() {
        let call = wire_request(Principal::anonymous());
        let decoded: u64 = ic_wire::native::decode_request(17, call.raw_args(), limits()).unwrap();
        assert_eq!(decoded, 42);
    }
    #[cfg(feature = "external-candid")]
    #[test]
    fn external_candid_is_explicit_and_unchanged() {
        let call = external_request(Principal::management_canister());
        assert_eq!(call.raw_args(), b"DIDL\0\0");
        ic_platform::candid::decode_args::<()>(
            call.raw_args(),
            ic_platform::candid::DecodeLimits {
                reply_bytes: 64,
                decoding_quota: 1000,
                skipping_quota: 1000,
                type_entries: 8,
                header_bytes: 64,
            },
        )
        .unwrap();
    }
}
