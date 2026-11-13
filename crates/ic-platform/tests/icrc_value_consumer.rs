//! Non-cryptographic public ICRC value observations from the reviewed contract.

use ic_platform::Principal;
use ic_platform::candid_codec::decoder::{Decode, Limits, decode_one};
use ic_platform::candid_codec::{Encode, Int, Nat, encode_one};
use ic_platform::icrc_ledger_types::{Account, ICRC3Value, Value};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use std::collections::BTreeMap;
use std::fmt::{self, Debug, Display, Write};

fn huge() -> Nat {
    "340282366920938463463374607431768211456".parse().unwrap()
}
fn limits(length: usize) -> Limits {
    Limits {
        bytes: length,
        header: 16_384,
        types: 1024,
        work: 500_000,
        allocation: 500_000,
    }
}

fn variants() -> Vec<Value> {
    vec![
        Value::blob(vec![0, 255, 16]),
        Value::text("literal\0λ"),
        Value::Nat(huge()),
        Value::Nat64(u64::MAX),
        Value::Int(Int::from(-7i64)),
        Value::Array(vec![]),
        Value::Map(BTreeMap::new()),
    ]
}

fn nested() -> Value {
    Value::map([
        (
            "z",
            Value::Array(vec![
                Value::Nat64(u64::MAX),
                Value::Nat(huge()),
                Value::Int(Int::from(i128::MIN)),
            ]),
        ),
        (
            "a:λ",
            Value::map([
                ("unescaped, key", Value::text("line\n\0text")),
                ("bytes", Value::blob(vec![0, 255])),
            ]),
        ),
    ])
}

fn negative_nat_error() -> String {
    let error = Int::from(-7i64).0.try_into().map(Nat).unwrap_err();
    format!("Failed to convert Int to Nat: {error:?}")
}

#[test]
fn constructors_duplicate_keys_and_declared_order_remain_exact() {
    fn traits<T: Serialize + for<'a> Deserialize<'a> + Clone + Debug + Eq + Ord>() {}
    traits::<Value>();
    traits::<ICRC3Value>();
    assert_eq!(Value::text(42), Value::Text("42".to_owned()));
    assert_eq!(
        Value::blob(&[0u8, 255][..]),
        Value::Blob(ByteBuf::from(vec![0, 255]))
    );
    let map = Value::map([
        (2, Value::text("two")),
        (10, Value::text("ten")),
        (1, Value::text("first")),
        (1, Value::text("last")),
    ])
    .as_map()
    .unwrap();
    assert_eq!(
        map.keys().map(String::as_str).collect::<Vec<_>>(),
        ["1", "10", "2"]
    );
    assert_eq!(map["1"], Value::text("last"));
    let values = variants();
    assert_eq!(
        values.iter().map(Value::variant_name).collect::<Vec<_>>(),
        ["Blob", "Text", "Nat", "Nat64", "Int", "Array", "Map"]
    );
    assert!(values.windows(2).all(|pair| pair[0] < pair[1]));
    let standard: Vec<ICRC3Value> = values
        .into_iter()
        .filter(|value| !matches!(value, Value::Nat64(_)))
        .map(Into::into)
        .collect();
    assert!(standard.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn every_accessor_and_try_from_mismatch_retains_its_error_contract() {
    for value in variants() {
        let name = value.variant_name();
        if name != "Blob" {
            assert_eq!(value.clone().as_blob().unwrap_err(), name);
            let expected = format!("Expecting variant Blob but found {name}");
            assert_eq!(ByteBuf::try_from(value.clone()).unwrap_err(), expected);
            assert_eq!(Vec::<u8>::try_from(value.clone()).unwrap_err(), expected);
        }
        if name != "Text" {
            assert_eq!(value.clone().as_text().unwrap_err(), name);
            assert_eq!(
                String::try_from(value.clone()).unwrap_err(),
                format!("Expecting variant Text but found {name}")
            );
        }
        if name != "Int" {
            assert_eq!(value.clone().as_int().unwrap_err(), name);
            assert_eq!(
                Int::try_from(value.clone()).unwrap_err(),
                format!("Expecting variant Int but found {name}")
            );
        }
        if name != "Array" {
            assert_eq!(value.clone().as_array().unwrap_err(), name);
            assert_eq!(
                Vec::<Value>::try_from(value.clone()).unwrap_err(),
                format!("Expecting variant Array but found {name}")
            );
        }
        if name != "Map" {
            assert_eq!(value.clone().as_map().unwrap_err(), name);
            assert_eq!(
                BTreeMap::<String, Value>::try_from(value.clone()).unwrap_err(),
                format!("Expecting variant Map but found {name}")
            );
        }
        if name != "Nat" && name != "Nat64" {
            let detail = if name == "Int" {
                negative_nat_error()
            } else {
                name
            };
            assert_eq!(value.clone().as_nat().unwrap_err(), detail);
            assert_eq!(
                Nat::try_from(value.clone()).unwrap_err(),
                format!("Expecting variant Nat but found {detail}")
            );
            assert_eq!(
                u64::try_from(value).unwrap_err(),
                format!("Expecting variant Nat but found {detail}")
            );
        }
    }
    assert_eq!(
        u64::try_from(Value::Nat(huge())).unwrap_err(),
        "Unable to convert nat {nat} to u64"
    );
}

#[test]
fn successful_consuming_conversions_preserve_values_without_new_limits() {
    assert_eq!(
        ByteBuf::try_from(Value::blob(vec![0, 255]))
            .unwrap()
            .as_slice(),
        [0, 255]
    );
    assert_eq!(
        Vec::<u8>::try_from(Value::blob(vec![0, 255])).unwrap(),
        [0, 255]
    );
    assert_eq!(String::try_from(Value::text("\0λ")).unwrap(), "\0λ");
    assert_eq!(
        Int::try_from(Value::Int(Int::from(i128::MIN))).unwrap(),
        Int::from(i128::MIN)
    );
    assert_eq!(
        Vec::<Value>::try_from(Value::Array(vec![Value::Nat64(0)])).unwrap(),
        [Value::Nat64(0)]
    );
    assert_eq!(
        BTreeMap::<String, Value>::try_from(Value::map([("k", Value::Nat64(1))])).unwrap()["k"],
        Value::Nat64(1)
    );
    for value in [
        Value::Nat(Nat::from(0u64)),
        Value::Nat64(0),
        Value::Int(Int::from(0i64)),
    ] {
        assert_eq!(value.clone().as_nat().unwrap(), Nat::from(0u64));
        assert_eq!(u64::try_from(value).unwrap(), 0);
    }
    for value in [
        Value::Nat(Nat::from(u64::MAX)),
        Value::Nat64(u64::MAX),
        Value::Int(Int::from(u64::MAX)),
    ] {
        assert_eq!(u64::try_from(value).unwrap(), u64::MAX);
    }
    assert_eq!(Value::Nat(huge()).as_nat().unwrap(), huge());
    let positive: Int = huge().0.to_string().parse().unwrap();
    assert_eq!(Value::Int(positive).as_nat().unwrap(), huge());
    let converted: ICRC3Value = nested().into();
    let restored: Value = converted.clone().into();
    let Value::Map(fields) = &restored else {
        panic!("map")
    };
    let Value::Array(values) = &fields["z"] else {
        panic!("array")
    };
    assert_eq!(values[0], Value::Nat(Nat::from(u64::MAX)));
    assert_eq!(values[1], Value::Nat(huge()));
    assert_eq!(ICRC3Value::from(restored), converted);
}

#[test]
fn principal_and_account_conversions_keep_bytes_and_refusal_precedence() {
    for principal in [
        Principal::management_canister(),
        Principal::anonymous(),
        Principal::from_slice(&[0, 255, 1]),
    ] {
        let value = Value::from(principal);
        assert_eq!(
            value.clone().as_blob().unwrap().as_slice(),
            principal.as_slice()
        );
        assert_eq!(Principal::try_from(value).unwrap(), principal);
        for subaccount in [None, Some([0; 32]), Some([255; 32])] {
            let account = Account {
                owner: principal,
                subaccount,
            };
            let value = Value::from(account);
            assert_eq!(
                value.clone().as_array().unwrap().len(),
                if subaccount.is_some() { 2 } else { 1 }
            );
            let restored = Account::try_from(value).unwrap();
            assert_eq!(restored.owner, principal);
            assert_eq!(restored.subaccount, subaccount);
        }
    }
    assert_eq!(Principal::try_from(Value::Nat64(0)).unwrap_err(), "Nat64");
    assert_eq!(Account::try_from(Value::Nat64(0)).unwrap_err(), "Nat64");
    assert_eq!(
        Account::try_from(Value::Array(vec![Value::text("bad")])).unwrap_err(),
        "Text"
    );
    assert_eq!(
        Account::try_from(Value::Array(vec![
            Value::from(Principal::anonymous()),
            Value::text("bad")
        ]))
        .unwrap_err(),
        "Text"
    );
    assert_eq!(
        Account::try_from(Value::Array(vec![
            Value::text("bad"),
            Value::Nat64(0),
            Value::Nat64(0)
        ]))
        .unwrap_err(),
        "Account should be an array of either one or two elements but found an array of 3 elements"
    );
    let invalid = vec![0; 30];
    let error = Principal::try_from_slice(&invalid).unwrap_err();
    assert_eq!(
        Principal::try_from(Value::blob(invalid.clone())).unwrap_err(),
        format!("Unable to decode the principal, error {error}")
    );
    assert_eq!(
        Account::try_from(Value::Array(vec![Value::blob(invalid)])).unwrap_err(),
        format!("Unable to decode the owner of the account, error {error}")
    );
    for length in [0, 31, 33] {
        let bytes = vec![0; length];
        let error = <&[u8; 32]>::try_from(bytes.as_slice()).unwrap_err();
        assert_eq!(
            Account::try_from(Value::Array(vec![
                Value::from(Principal::anonymous()),
                Value::blob(bytes)
            ]))
            .unwrap_err(),
            format!("Unable to decode the subaccount of the account, error {error}")
        );
    }
    assert_eq!(
        Account::try_from(Value::Array(vec![])).unwrap_err(),
        "Account should be an array of either one or two elements but found an array of 0 elements"
    );
}

struct RejectFormatting;
impl Write for RejectFormatting {
    fn write_str(&mut self, _text: &str) -> fmt::Result {
        Err(fmt::Error)
    }
}

#[test]
fn display_preserves_literal_shapes_and_formatter_failure() {
    let cases = [
        (Value::blob(vec![]), "".to_owned()),
        (Value::blob(vec![0, 15, 255]), "000fff".to_owned()),
        (Value::text("a\n\0λ"), "a\n\0λ".to_owned()),
        (Value::Nat64(u64::MAX), u64::MAX.to_string()),
        (Value::Nat(huge()), huge().to_string()),
        (
            Value::Int(Int::from(i128::MIN)),
            Int::from(i128::MIN).to_string(),
        ),
        (Value::Array(vec![]), "Array()".to_owned()),
        (
            Value::Array(vec![Value::text("a"), Value::Nat64(2)]),
            "Array(a, 2)".to_owned(),
        ),
        (
            Value::map(Vec::<(String, Value)>::new()),
            "Map()".to_owned(),
        ),
        (
            Value::map([
                ("z", Value::text("last")),
                ("a: key", Value::blob(vec![0, 255])),
            ]),
            "Map(a: key: 00ff, z: last)".to_owned(),
        ),
    ];
    for (value, expected) in cases {
        assert_eq!(value.to_string(), expected);
        assert_eq!(format!("{value:>40.2}"), expected);
        assert_eq!(format!("{value:#}"), expected);
        let standard_expected = match &value {
            Value::Nat64(value) => Nat::from(*value).to_string(),
            _ => expected,
        };
        let standard = ICRC3Value::from(value);
        assert_eq!(standard.to_string(), standard_expected);
        assert_eq!(format!("{standard:>40.2}"), standard_expected);
    }
    assert!(write!(&mut RejectFormatting, "{}", nested()).is_err());
    assert!(write!(&mut RejectFormatting, "{}", ICRC3Value::from(nested())).is_err());
}

fn observation<T>(label: &str, value: &T) -> serde_json::Value
where
    T: Serialize + for<'a> Deserialize<'a> + Encode + for<'a> Decode<'a> + Debug + Display + Eq,
{
    let json = serde_json::to_string(value).unwrap();
    let json_decoding = serde_json::from_str::<T>(&json)
        .map(|decoded| decoded == *value)
        .map_err(|error| error.to_string());
    assert_eq!(json_decoding, Ok(true));
    let cbor = serde_cbor::to_vec(value).unwrap();
    let cbor_decoding = serde_cbor::from_slice::<T>(&cbor)
        .map(|decoded| decoded == *value)
        .map_err(|error| error.to_string());
    assert_eq!(cbor_decoding, Ok(true));
    let candid = encode_one(value, 65_536).unwrap();
    assert_eq!(
        decode_one::<T>(&candid, limits(candid.len())).unwrap(),
        *value
    );
    serde_json::json!({
        "label": label,
        "json": json,
        "json_decode": json_decoding,
        "cbor": cbor,
        "cbor_decode": cbor_decoding,
        "candid": candid,
        "display": value.to_string(),
        "debug": format!("{value:?}")
    })
}

#[test]
fn deterministic_noncrypto_value_observations() {
    let int_json = serde_json::to_value(Int::from(-7i64)).unwrap();
    assert!(int_json.is_array());
    assert_eq!(
        serde_json::from_value::<Int>(int_json).unwrap(),
        Int::from(-7i64)
    );
    let mut observations = Vec::new();
    for value in variants() {
        observations.push(observation(
            &format!("Value::{}", value.variant_name()),
            &value,
        ));
        if !matches!(value, Value::Nat64(_)) {
            observations.push(observation(
                &format!("ICRC3Value::{}", value.variant_name()),
                &ICRC3Value::from(value),
            ));
        }
    }
    observations.push(observation("Value::nested", &nested()));
    observations.push(observation(
        "ICRC3Value::nested",
        &ICRC3Value::from(nested()),
    ));
    let errors = vec![
        (
            "negative-as-nat",
            Value::Int(Int::from(-7i64)).as_nat().unwrap_err(),
        ),
        (
            "negative-try-nat",
            Nat::try_from(Value::Int(Int::from(-7i64))).unwrap_err(),
        ),
        (
            "negative-try-u64",
            u64::try_from(Value::Int(Int::from(-7i64))).unwrap_err(),
        ),
        (
            "oversized-u64",
            u64::try_from(Value::Nat(huge())).unwrap_err(),
        ),
        (
            "principal-too-long",
            Principal::try_from(Value::blob(vec![0; 30])).unwrap_err(),
        ),
        (
            "account-too-long-owner",
            Account::try_from(Value::Array(vec![Value::blob(vec![0; 30])])).unwrap_err(),
        ),
        (
            "account-short-subaccount",
            Account::try_from(Value::Array(vec![
                Value::from(Principal::anonymous()),
                Value::blob(vec![0; 31]),
            ]))
            .unwrap_err(),
        ),
        (
            "account-too-many",
            Account::try_from(Value::Array(vec![Value::Nat64(0); 3])).unwrap_err(),
        ),
    ];
    let transcript =
        serde_json::to_string(&serde_json::json!({"values":observations,"errors":errors})).unwrap();
    assert!(
        transcript.len() < 30_000,
        "finite value fixture: {} bytes",
        transcript.len()
    );
    // Preserve every original serialized byte, display and conversion error.
    // The authorized bug fix changes only the previously failing Serde decodes.
    let mut expected: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/icrc-value-observations.json")).unwrap();
    for value in expected["values"].as_array_mut().unwrap() {
        value["json_decode"] = serde_json::json!({"Ok": true});
        value["cbor_decode"] = serde_json::json!({"Ok": true});
    }
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&transcript).unwrap(),
        expected
    );
    println!("ICRC_VALUE_OBSERVATIONS={transcript}");
}

#[test]
fn owned_blob_allocation_observation() {
    let mut bytes = Vec::with_capacity(256);
    bytes.extend_from_slice(&[0, 1, 127, 255]);
    let original_pointer = bytes.as_ptr();
    let original_capacity = bytes.capacity();
    let recovered = Vec::<u8>::try_from(Value::blob(bytes)).unwrap();
    assert_eq!(recovered, [0, 1, 127, 255]);
    assert_eq!(original_pointer, recovered.as_ptr());
    assert_eq!(original_capacity, recovered.capacity());
    println!(
        "ICRC_VALUE_ALLOCATION={{\"pointer_preserved\":{},\"input_capacity\":{},\"output_capacity\":{}}}",
        original_pointer == recovered.as_ptr(),
        original_capacity,
        recovered.capacity()
    );
}
