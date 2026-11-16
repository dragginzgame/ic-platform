//! Independent schema decoders for the bounded management request encoder.
#![forbid(unsafe_code)]
#![cfg(all(feature = "http-request", not(target_family = "wasm")))]

use ic_platform_protocol::http_request::{EncodeError, encode_http_get_args};

#[test]
fn entropy_pricing_is_explicit_and_bounded() {
    use ic_platform_protocol::http_request::encode_http_get_args_v1;
    // The pinned upstream DTO predates this optional nat32 field.
    #[derive(candid::CandidType, serde::Deserialize)]
    struct Pricing {
        pricing_version: Option<u32>,
    }
    let url = "https://entropy-probe.invalid/hold";
    let bytes = encode_http_get_args_v1(url, None, 256, true, 4096).unwrap();
    let request: ic_management_canister_types::HttpRequestArgs =
        candid::decode_one(&bytes).unwrap();
    assert_eq!(
        candid::decode_one::<Pricing>(&bytes)
            .unwrap()
            .pricing_version,
        Some(1)
    );
    assert_eq!(request.url, url);
    assert_eq!(request.max_response_bytes, Some(256));
    assert_eq!(request.body, None);
    assert!(request.transform.is_none());
    assert!(request.headers.is_empty());
    assert_eq!(
        encode_http_get_args_v1(url, None, 256, true, bytes.len()).unwrap(),
        bytes
    );
    assert!(encode_http_get_args_v1(url, None, 256, true, bytes.len() - 1).is_err());
    let default = encode_http_get_args(url, None, 256, true, 4096).unwrap();
    let request: Pricing = candid::decode_one(&default).unwrap();
    assert_eq!(request.pricing_version, None);
}

#[test]
fn management_schema_preserve_every_selected_field() {
    let urls = [
        String::new(),
        " https://ic-testkit.invalid/held/a/0\0é".into(),
        "u".repeat(127),
        "u".repeat(128),
        "u".repeat(16383),
        "u".repeat(16384),
    ];
    for url in urls {
        for length in [0, 8, 127, 128, 16383, 16384] {
            let body: Vec<u8> = (0..length).map(|n| n as u8).collect();
            for selected in [None, Some(body.as_slice())] {
                for replicated in [false, true] {
                    let bytes =
                        encode_http_get_args(&url, selected, u64::MAX, replicated, 65536).unwrap();
                    let decoded: ic_management_canister_types::HttpRequestArgs =
                        candid::decode_one(&bytes).unwrap();
                    assert_eq!(decoded.url, url);
                    assert_eq!(
                        decoded.method,
                        ic_management_canister_types::HttpMethod::GET
                    );
                    assert_eq!(decoded.max_response_bytes, Some(u64::MAX));
                    assert_eq!(decoded.body.as_deref(), selected);
                    assert_eq!(decoded.is_replicated, Some(replicated));
                    assert!(decoded.headers.is_empty());
                    assert!(decoded.transform.is_none());
                    assert_eq!(
                        encode_http_get_args(&url, selected, u64::MAX, replicated, bytes.len())
                            .unwrap(),
                        bytes
                    );
                    assert_eq!(
                        encode_http_get_args(&url, selected, u64::MAX, replicated, bytes.len() - 1),
                        Err(EncodeError::OutputLimit {
                            max_bytes: bytes.len() - 1
                        })
                    );
                }
            }
        }
    }
}

#[test]
fn held_reply_slot_is_one_complete_argument() {
    let url = "https://ic-testkit.invalid/held/aaaaa-aa/fixture/42";
    let slot = 42u64.to_le_bytes();
    let bytes = encode_http_get_args(url, Some(&slot), 256, true, 4096).unwrap();
    let (request,): (ic_management_canister_types::HttpRequestArgs,) =
        candid::decode_args(&bytes).unwrap();
    assert_eq!(request.body, Some(slot.to_vec()));
    assert_eq!(request.max_response_bytes, Some(256));
    assert_eq!(request.url, url);
    let mut extra = bytes;
    extra.push(0);
    assert!(candid::decode_one::<ic_management_canister_types::HttpRequestArgs>(&extra).is_err());
    assert_eq!(
        encode_http_get_args(url, Some(&slot), 256, true, 0),
        Err(EncodeError::OutputLimit { max_bytes: 0 })
    );
}
