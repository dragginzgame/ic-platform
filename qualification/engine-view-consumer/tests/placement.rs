//! Independent upstream schema qualification for Registry placement.
#![forbid(unsafe_code)]
#![cfg(all(feature = "registry", not(target_family = "wasm")))]
use candid::{CandidType, Deserialize, Principal};
use ic_platform_protocol::{candid::Limits, placement::*};
#[derive(CandidType, Deserialize)]
struct Request {
    principal: Option<Principal>,
}
#[derive(CandidType, Deserialize)]
struct Response {
    subnet_id: Option<Principal>,
}

fn limits() -> Limits {
    Limits {
        bytes: 4096,
        header: 1024,
        types: 32,
        work: 8192,
        allocation: 8192,
        vector: 128,
        skipping: 4096,
    }
}

#[test]
fn upstream_schema_preserves_exact_principals_and_result_variants() {
    for bytes in [&[][..], &[4], &[7; 29]] {
        let source = Principal::from_slice(bytes);
        let local = candidate_types::Principal::from_slice(bytes);
        let encoded = encode_get_subnet_for_canister_args(local, 256).unwrap();
        let decoded: Request = candid::decode_one(&encoded).unwrap();
        assert_eq!(decoded.principal, Some(source));
        assert_eq!(
            encode_get_subnet_for_canister_args(local, encoded.len()).unwrap(),
            encoded
        );
        assert!(encode_get_subnet_for_canister_args(local, encoded.len() - 1).is_err());
        for subnet in [None, Some(source)] {
            let response: Result<Response, String> = Ok(Response { subnet_id: subnet });
            let wire = candid::encode_one(response).unwrap();
            let PlacementResult::Ok(value) =
                decode_get_subnet_for_canister_response(&wire, limits()).unwrap()
            else {
                panic!("expected success")
            };
            assert_eq!(value.subnet_id, subnet.map(|_| local));
            for end in 0..wire.len() {
                assert!(decode_get_subnet_for_canister_response(&wire[..end], limits()).is_err());
            }
            let mut trailing = wire;
            trailing.push(0);
            assert!(decode_get_subnet_for_canister_response(&trailing, limits()).is_err());
        }
    }
    let response: Result<Response, String> = Err("source\0é".into());
    let wire = candid::encode_one(response).unwrap();
    assert_eq!(
        decode_get_subnet_for_canister_response(&wire, limits()).unwrap(),
        PlacementResult::Err("source\0é".into())
    );
    assert_eq!(
        ic_platform_protocol::cloud_engine::encode_empty_args(6).unwrap(),
        b"DIDL\0\0"
    );
    assert!(ic_platform_protocol::cloud_engine::encode_empty_args(5).is_err());
}
