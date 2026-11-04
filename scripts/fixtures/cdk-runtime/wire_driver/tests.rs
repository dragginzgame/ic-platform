use super::*;

#[test]
fn binary_roundtrip_frames_fit_fixture_budget() {
    for size in ROUNDTRIP_SIZES {
        let input = vec![255u8; size];
        for id in [AWAIT, ECHO] {
            let request = ic_wire::native::encode_request(id, &input, REQUEST).unwrap();
            let decoded: (Vec<u8>,) =
                ic_wire::native::decode_request(id, &request, REQUEST).unwrap();
            assert_eq!(decoded.0, input);
            let reply = ic_wire::native::encode_response(id, &input, RESPONSE).unwrap();
            let decoded: Vec<u8> = ic_wire::native::decode_response(id, &reply, RESPONSE).unwrap();
            assert_eq!(decoded, input);
        }
    }
}

#[test]
fn responses_decode_without_a_serializer() {
    #[derive(Debug, PartialEq, serde::Deserialize)]
    struct ReadOnly {
        value: u64,
    }
    let bytes = crate::request("scalar", 128_u64);
    let decoded: ReadOnly = ic_wire::native::decode_request(
        crate::protocol::method_id("scalar"),
        &bytes,
        crate::protocol::LIMITS,
    )
    .unwrap();
    assert_eq!(decoded, ReadOnly { value: 128 });
    let bytes = ic_wire::native::encode_response(
        crate::protocol::method_id("scalar"),
        &128_u64,
        crate::protocol::LIMITS,
    )
    .unwrap();
    assert_eq!(crate::response::<ReadOnly>("scalar", &bytes), decoded);
}

#[test]
fn request_resource_refusals_keep_fixture_policy() {
    for id in [TIGHT, ALLOCATION, DEPTH] {
        let bytes = ic_wire::native::encode_request(id, &vec![0_u8, 255, 3], REQUEST).unwrap();
        assert_eq!(
            ic_wire::native::decode_request::<(Vec<u8>,)>(id, &bytes, request_limits(id)),
            Err(ic_wire::Error::Limit)
        );
    }
}
