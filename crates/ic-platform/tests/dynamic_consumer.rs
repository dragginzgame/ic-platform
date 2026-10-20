//! Native refusal-fixture construction with only Platform at the Candid boundary.
#![cfg(not(target_family = "wasm"))]

#[cfg(test)]
use ic_platform::{
    candid::{
        contract::{
            self, CandidType,
            types::{Field, Label, Type, TypeInner},
        },
        decoder::{DecoderConfig, decode_one_with_config},
    },
    candid_parser::{IDLArgs, IDLField, IDLValue, TypeSerialize, ValueSerializer},
    principal::ExternalPrincipal as Principal,
};

#[cfg(test)]
#[derive(Debug, PartialEq, contract::CandidType, contract::serde::Deserialize)]
#[candid_path("ic_platform::candid::contract")]
#[serde(crate = "ic_platform::candid::contract::serde")]
struct PreparationInput {
    value: u64,
}

#[cfg(test)]
fn fixture(hex: &str) -> Vec<u8> {
    let (pairs, remainder) = hex.trim().as_bytes().as_chunks::<2>();
    assert!(remainder.is_empty());
    pairs
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn old_field_fixture_retains_original_owner_bytes() {
    let input = PreparationInput { value: 42 };
    let bytes = contract::encode_one(&input).unwrap();
    let mut args = IDLArgs::from_bytes(&bytes).unwrap();
    let IDLValue::Record(fields) = &mut args.args[0] else {
        panic!("record")
    };
    fields.push(IDLField {
        id: Label::Named("trusted_uploader".into()),
        val: IDLValue::Principal(Principal::anonymous()),
    });
    fields.sort_by_key(|field| field.id.get_id());
    let bytes = args.to_bytes().unwrap();
    assert_eq!(
        bytes,
        fixture(include_str!("fixtures/dynamic-old-field.hex"))
    );
    assert_eq!(
        IDLArgs::from_bytes(&bytes).unwrap().to_bytes().unwrap(),
        bytes
    );
    // Typed record projection continues skipping the historical field.
    assert_eq!(
        decode_one_with_config::<PreparationInput>(&bytes, &DecoderConfig::new()).unwrap(),
        input
    );
}

#[test]
fn oversized_type_fixtures_preserve_bytes_and_caller_selected_refusals() {
    // The same independent serializer construction used by Blob's admission corpus.
    let input = PreparationInput { value: 42 };
    let mut nested: Type = TypeInner::Null.into();
    for _ in 0..129 {
        nested = TypeInner::Opt(nested).into();
    }
    let wide = TypeInner::Record(
        (0..2048)
            .map(|id| Field {
                id: Label::Id(id).into(),
                ty: TypeInner::Null.into(),
            })
            .collect(),
    )
    .into();
    for (extra, value, expected) in [
        (nested, vec![0], include_str!("fixtures/dynamic-nested.hex")),
        (wide, vec![], include_str!("fixtures/dynamic-wide.hex")),
    ] {
        let mut types = TypeSerialize::new();
        types.push_type(&PreparationInput::ty()).unwrap();
        types.push_type(&extra).unwrap();
        types.serialize().unwrap();
        let mut values = ValueSerializer::new();
        input.idl_serialize(&mut values).unwrap();
        values.write(&value).unwrap();
        let mut bytes = b"DIDL".to_vec();
        bytes.extend(types.get_result());
        bytes.extend(values.get_result());
        assert_eq!(bytes, fixture(expected));
        assert_eq!(
            decode_one_with_config::<PreparationInput>(&bytes, &DecoderConfig::new()).unwrap(),
            input
        );
        let mut config = DecoderConfig::new();
        config.set_max_header_len(64);
        let _: contract::Error =
            decode_one_with_config::<PreparationInput>(&bytes, &config).unwrap_err();
    }
}
