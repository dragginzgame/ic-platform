//! Installed effect/refusal checks; Testkit owns provider admission/lifecycle.
use ic_platform::{
    candid::{self, contract},
    principal::ExternalPrincipal as Principal,
};
use ic_testkit::pic::PocketIc;
const LIMITS: candid::DecodeLimits = candid::DecodeLimits {
    reply_bytes: 65536,
    decoding_quota: 100000,
    skipping_quota: 1000,
    type_entries: 128,
    header_bytes: 8192,
};
fn observed(pic: &PocketIc, id: Principal) -> u64 {
    let bytes = pic
        .query_call(
            id,
            Principal::anonymous(),
            "external_observed",
            b"DIDL\0\0".to_vec(),
        )
        .unwrap();
    candid::decode_exactly_one(&bytes, LIMITS).unwrap()
}
pub fn qualify(pic: &PocketIc, id: Principal) {
    let args = contract::encode_one(7u64).unwrap();
    let reply = pic
        .update_call(id, Principal::anonymous(), "external_effect", args)
        .unwrap();
    assert_eq!(
        candid::decode_exactly_one::<u64>(&reply, LIMITS).unwrap(),
        7
    );
    for bytes in [
        vec![],
        vec![0],
        vec![0; 1025],
        contract::encode_one("wrong type").unwrap(),
    ] {
        assert!(
            pic.update_call(id, Principal::anonymous(), "external_effect", bytes)
                .is_err()
        );
        assert_eq!(observed(pic, id), 7);
    }
    for (method, bytes) in [
        ("external_no_work", contract::encode_one(8u64).unwrap()),
        ("external_no_header", contract::encode_one(8u64).unwrap()),
        (
            "external_no_types",
            contract::encode_one(Some(8u64)).unwrap(),
        ),
        (
            "external_no_skip",
            contract::encode_args((8u64, 9u64)).unwrap(),
        ),
    ] {
        assert!(
            pic.update_call(id, Principal::anonymous(), method, bytes)
                .is_err()
        );
        assert_eq!(observed(pic, id), 7);
    }
    let denied = pic
        .update_call(id, Principal::anonymous(), "external_denied", vec![0; 1025])
        .unwrap_err();
    assert!(denied.reject_message.contains("fixture access refused"));
    assert_eq!(observed(pic, id), 7);
    assert!(
        pic.update_call(
            id,
            Principal::anonymous(),
            "external_encoding_trap",
            b"DIDL\0\0".to_vec()
        )
        .is_err()
    );
    assert_eq!(observed(pic, id), 7);
    let tuple = pic
        .update_call(
            id,
            Principal::anonymous(),
            "external_tuple",
            contract::encode_args((8u64, true)).unwrap(),
        )
        .unwrap();
    assert_eq!(
        candid::decode_args::<(u64, bool)>(&tuple, LIMITS).unwrap(),
        (8, true)
    );
    let unit = pic
        .update_call(
            id,
            Principal::anonymous(),
            "external_zero",
            b"DIDL\0\0".to_vec(),
        )
        .unwrap();
    assert_eq!(unit, b"DIDL\0\0");
    let null = pic
        .query_call(
            id,
            Principal::anonymous(),
            "external_one_null",
            b"DIDL\0\0".to_vec(),
        )
        .unwrap();
    assert_eq!(null, b"DIDL\0\x01\x7f");
    assert_eq!(
        pic.update_call(id, Principal::anonymous(), "external_legacy_empty", vec![])
            .unwrap(),
        b"DIDL\0\0"
    );
    let composite = pic
        .query_call(
            id,
            Principal::anonymous(),
            "external_composite",
            contract::encode_one(8u64).unwrap(),
        )
        .unwrap();
    assert_eq!(
        candid::decode_exactly_one::<u64>(&composite, LIMITS).unwrap(),
        9
    );
    let random = pic
        .update_call(
            id,
            Principal::anonymous(),
            "external_random",
            b"DIDL\0\0".to_vec(),
        )
        .unwrap();
    assert_eq!(
        candid::decode_exactly_one::<Vec<u8>>(&random, LIMITS)
            .unwrap()
            .len(),
        32
    );
    println!(
        "External endpoint admission, auth-before-decode, scalar/tuple/zero/null replies, mutation rollback, composite query and management await passed"
    );
}
