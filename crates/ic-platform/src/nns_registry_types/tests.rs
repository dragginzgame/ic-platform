use super::*;

#[expect(
    clippy::needless_pass_by_value,
    reason = "Each differential case owns its selected DTO through the oracle roundtrip"
)]
fn differential<
    T: crate::candid_codec::Encode
        + for<'a> crate::candid_codec::decoder::Decode<'a>
        + std::fmt::Debug
        + PartialEq,
>(
    value: T,
) {
    crate::tests::reference::roundtrip(&value);
}

#[test]
fn registry_requests_and_all_response_states_preserve_bytes() {
    for principal in [
        None,
        Some(Principal::management_canister()),
        Some(Principal::anonymous()),
        Some(Principal::from_slice(&[42; 29])),
    ] {
        differential(GetSubnetForCanisterRequest { principal });
        differential(GetSubnetForCanisterResponse::Ok(
            GetSubnetForCanisterResponseOk {
                subnet_id: principal,
            },
        ));
    }
    differential(GetSubnetForCanisterResponse::Err("not found 🌍".to_owned()));
}

#[test]
fn owned_principal_fields_match_independent_registry_values_at_every_length() {
    let env = crate::tests::reference::frozen_schema(include_str!(
        "../../tests/fixtures/registry-schema.json"
    ));
    let ty = env.find_type("Request").unwrap().clone();
    for length in 0..=Principal::MAX_LENGTH_IN_BYTES {
        for fill in [0, 4, 255] {
            let principal = Principal::from_slice(&vec![fill; length]);
            let upstream = candid::Principal::from_slice(principal.as_slice());
            let expected = candid::encode_one(upstream).unwrap();
            assert_eq!(
                crate::candid_codec::encode_one(&principal, 128).unwrap(),
                expected
            );
            assert_eq!(
                crate::candid_codec::decoder::decode_one::<Principal>(
                    &expected,
                    crate::tests::reference::limits()
                )
                .unwrap(),
                principal
            );
            let values = candid::IDLArgs::new(&[candid::IDLValue::Record(vec![
                candid::types::value::IDLField {
                    id: candid::types::Label::Named("principal".into()),
                    val: candid::IDLValue::Opt(Box::new(candid::IDLValue::Principal(upstream))),
                },
            ])]);
            let expected = values
                .to_bytes_with_types(&env, std::slice::from_ref(&ty))
                .unwrap();
            let request = GetSubnetForCanisterRequest {
                principal: Some(principal),
            };
            assert_eq!(
                crate::candid_codec::encode_one(&request, 256).unwrap(),
                expected
            );
            assert_eq!(
                crate::candid_codec::decoder::decode_one::<GetSubnetForCanisterRequest>(
                    &expected,
                    crate::tests::reference::limits()
                )
                .unwrap(),
                request
            );
        }
    }
}
