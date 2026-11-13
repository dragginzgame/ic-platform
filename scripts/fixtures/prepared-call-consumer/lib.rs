//! This package selects only Platform runtime for raw calls; codecs are separate selections.
use ic_platform::call::{Call, Principal, WaitPolicy};

/// Prepare a bounded raw call whose method and arguments outlive this function.
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

/// Execute the prepared raw call and return its response bytes.
pub async fn roundtrip_raw(target: Principal) -> Result<Vec<u8>, ic_platform::call::CallFailed> {
    prepare(target)
        .execute()
        .await
        .map(ic_platform::call::Response::into_bytes)
}

#[cfg(feature = "wire")]
/// Prepare a caller-encoded Wire request independently of the raw transport.
///
/// # Panics
/// Panics if the fixture's scalar request exceeds its fixed encoding limits.
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
/// Prepare an empty Candid argument list for an external interface.
///
/// # Panics
/// Panics if the empty argument list exceeds the fixture's encoding allowance.
pub fn external_request(target: Principal) -> Call<'static, 'static> {
    let bytes = ic_platform::candid_codec::arguments::encode_args(&(), 64).unwrap();
    Call::new(
        target,
        String::from("external_method"),
        WaitPolicy::Unbounded,
    )
    .take_raw_args(bytes)
}

#[cfg(test)]
mod tests;
