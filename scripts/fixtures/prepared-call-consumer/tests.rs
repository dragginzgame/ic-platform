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
