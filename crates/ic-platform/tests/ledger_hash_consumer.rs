//! Independent public-consumer checks of the ledger hash composition contracts.

use std::{
    collections::BTreeMap,
    path::{Component, PathBuf},
};

use ic_platform::{
    Principal,
    candid_codec::{Int, Nat},
    icp_ledger_types::account_identifier,
    icrc_ledger_types::{ICRC3Value, Value},
};
use num_bigint::{BigInt, BigUint};
use sha2_11::{Digest, Sha224, Sha256};

fn digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn unsigned_leb(value: &BigUint) -> Vec<u8> {
    let bytes = value.to_bytes_le();
    let groups = value.bits().max(1).div_ceil(7);
    (0..groups)
        .map(|group| {
            let bit = group * 7;
            let index = usize::try_from(bit / 8).unwrap();
            let shift = bit % 8;
            let low = u16::from(bytes.get(index).copied().unwrap_or(0));
            let high = u16::from(bytes.get(index + 1).copied().unwrap_or(0));
            let payload = (((low | (high << 8)) >> shift) & 0x7f) as u8;
            payload | if group + 1 < groups { 0x80 } else { 0 }
        })
        .collect()
}

fn negative_leb(mut value: i128) -> Vec<u8> {
    let mut result = Vec::new();
    loop {
        let payload = (value & 0x7f) as u8;
        value >>= 7;
        let done = value == -1 && payload & 0x40 != 0;
        result.push(payload | if done { 0 } else { 0x80 });
        if done {
            return result;
        }
    }
}

fn reference_value_hash(value: &Value) -> [u8; 32] {
    match value {
        Value::Blob(bytes) => digest(bytes),
        Value::Text(text) => digest(text.as_bytes()),
        Value::Nat(value) => digest(&unsigned_leb(&value.0)),
        Value::Nat64(value) => digest(&unsigned_leb(&BigUint::from(*value))),
        Value::Int(value) => {
            let number = value.0.to_string().parse::<i128>().unwrap();
            if number < 0 {
                digest(&negative_leb(number))
            } else {
                digest(&unsigned_leb(&BigUint::from(number as u128)))
            }
        }
        Value::Array(values) => {
            let bytes: Vec<_> = values.iter().flat_map(reference_value_hash).collect();
            digest(&bytes)
        }
        Value::Map(values) => {
            let mut pairs: Vec<_> = values
                .iter()
                .map(|(key, value)| (digest(key.as_bytes()), reference_value_hash(value)))
                .collect();
            pairs.sort_unstable();
            let bytes: Vec<_> = pairs
                .into_iter()
                .flat_map(|(key, value)| key.into_iter().chain(value))
                .collect();
            digest(&bytes)
        }
    }
}

fn integer(number: i128) -> Value {
    Value::Int(Int::from(BigInt::from(number)))
}

fn value_cases() -> Vec<(String, Value)> {
    let mut cases = vec![
        ("empty-blob".into(), Value::Blob(Vec::new().into())),
        (
            "binary-blob".into(),
            Value::Blob(vec![0, 1, 127, 128, 254, 255].into()),
        ),
        ("empty-text".into(), Value::Text(String::new())),
        ("unicode-nul-text".into(), Value::Text("a\0é水🦀".into())),
        ("empty-array".into(), Value::Array(Vec::new())),
        ("empty-map".into(), Value::Map(BTreeMap::new())),
        ("int-min".into(), integer(i128::MIN)),
        ("int-max".into(), integer(i128::MAX)),
    ];
    // Exercise both unsigned group boundaries and signed terminal-sign boundaries.
    for bit in (0..127).filter(|bit| bit % 7 == 0 || bit % 7 == 6) {
        let boundary = 1_i128 << bit;
        for offset in [-1, 0, 1] {
            let positive = boundary + offset;
            for number in [positive, -positive] {
                cases.push((format!("int-b{bit}-o{offset}-v{number}"), integer(number)));
            }
        }
    }
    for bit in (0..64).step_by(7) {
        let boundary = 1_u64 << bit;
        for number in [boundary - 1, boundary, boundary + 1] {
            cases.push((format!("nat64-b{bit}-v{number}"), Value::Nat64(number)));
        }
    }
    cases.push(("nat64-max".into(), Value::Nat64(u64::MAX)));
    for bit in [
        1_usize, 7, 8, 63, 64, 127, 128, 255, 256, 511, 1024, 2048, 4096, 8191,
    ] {
        let boundary = BigUint::from(1_u8) << bit;
        for (label, number) in [
            ("below", &boundary - BigUint::from(1_u8)),
            ("at", boundary.clone()),
            ("above", &boundary + BigUint::from(1_u8)),
        ] {
            cases.push((format!("nat-b{bit}-{label}"), Value::Nat(Nat::from(number))));
        }
    }
    cases.push(("nat-zero".into(), Value::Nat(Nat::from(0_u8))));
    for length in [1_usize, 2, 16, 33, 128, 512, 1024] {
        let mut bytes: Vec<_> = (0..length)
            .map(|index| ((index * 73 + length * 19) % 256) as u8)
            .collect();
        *bytes.last_mut().unwrap() |= 0x80;
        cases.push((
            format!("nat-pattern-{}bits", length * 8),
            Value::Nat(Nat::from(BigUint::from_bytes_le(&bytes))),
        ));
    }
    let map = BTreeMap::from([
        (
            String::new(),
            Value::Array(vec![integer(-65), Value::Text("\0".into())]),
        ),
        ("a".into(), Value::Nat64(128)),
        ("b".into(), Value::Blob(vec![9, 0, 255].into())),
        ("é水".into(), Value::Map(BTreeMap::new())),
    ]);
    cases.push(("mixed-map".into(), Value::Map(map.clone())));
    cases.push((
        "mixed-array".into(),
        Value::Array(vec![
            Value::Map(map),
            Value::Array(vec![Value::Nat(Nat::from(129_u16)), integer(-129)]),
            Value::Text("tail".into()),
        ]),
    ));
    cases.push((
        "ordered-array-ab".into(),
        Value::Array(vec![integer(1), integer(2)]),
    ));
    cases.push((
        "ordered-array-ba".into(),
        Value::Array(vec![integer(2), integer(1)]),
    ));
    cases
}

fn account_cases() -> Vec<(String, Principal, [u8; 32])> {
    let subaccounts = [
        ("zero", [0; 32]),
        ("ones", [255; 32]),
        ("ascending", std::array::from_fn(|index| index as u8)),
        ("first", std::array::from_fn(|index| u8::from(index == 0))),
        (
            "last",
            std::array::from_fn(|index| if index == 31 { 128 } else { 0 }),
        ),
    ];
    let mut cases = Vec::new();
    for length in 0..=29 {
        let bytes: Vec<_> = (0..length)
            .map(|index| (index * 17 + length) as u8)
            .collect();
        let principal = Principal::from_slice(&bytes);
        for (label, subaccount) in subaccounts {
            cases.push((format!("owner-{length}-{label}"), principal, subaccount));
        }
    }
    cases
}

fn reference_account(owner: Principal, subaccount: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha224::new();
    hasher.update([10]);
    hasher.update(b"account-id");
    hasher.update(owner.as_slice());
    hasher.update(subaccount);
    let hash = hasher.finalize();
    let checksum = crc32fast::hash(&hash).to_be_bytes();
    let mut result = [0; 32];
    result[..4].copy_from_slice(&checksum);
    result[4..].copy_from_slice(&hash);
    result
}

#[test]
fn public_value_hashes_follow_the_protocol_composition() {
    for (name, value) in value_cases() {
        let expected = reference_value_hash(&value);
        assert_eq!(value.hash(), expected, "Value case {name}");
        assert_eq!(
            ICRC3Value::from(value).hash(),
            expected,
            "ICRC3Value case {name}"
        );
    }
}

#[test]
fn map_composition_orders_digest_pairs_and_ignores_insertion_order() {
    let entries = [("a".to_owned(), integer(1)), ("b".to_owned(), integer(2))];
    let forward = Value::Map(entries.clone().into_iter().collect());
    let reverse = Value::Map(entries.into_iter().rev().collect());
    assert_eq!(forward.hash(), reverse.hash());
    let Value::Map(values) = &forward else {
        unreachable!()
    };
    let lexical_key_order: Vec<_> = values
        .iter()
        .flat_map(|(key, value)| digest(key.as_bytes()).into_iter().chain(value.hash()))
        .collect();
    assert_ne!(digest(b"a").cmp(&digest(b"b")), "a".cmp("b"));
    assert_ne!(forward.hash(), digest(&lexical_key_order));
    assert_eq!(forward.hash(), reference_value_hash(&forward));
}

#[test]
fn arrays_preserve_order_and_nonnegative_integer_forms_agree() {
    assert_ne!(
        Value::Array(vec![integer(1), integer(2)]).hash(),
        Value::Array(vec![integer(2), integer(1)]).hash()
    );
    for number in [0_u64, 63, 64, 127, 128, 8191, 8192, u64::MAX] {
        let expected = Value::Nat64(number).hash();
        assert_eq!(Value::Nat(Nat::from(number)).hash(), expected);
        assert_eq!(integer(i128::from(number)).hash(), expected);
    }
}

fn outside_integer_cases() -> [(String, Value); 2] {
    [
        (
            "above-i128".into(),
            Value::Int(Int::from(BigInt::from(i128::MAX) + 1)),
        ),
        (
            "below-i128".into(),
            Value::Int(Int::from(BigInt::from(i128::MIN) - 1)),
        ),
    ]
}

fn panic_message(result: std::thread::Result<[u8; 32]>) -> String {
    let payload = result.expect_err("outside-i128 Int unexpectedly hashed");
    if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else {
        panic!("outside-i128 panic payload is not text")
    }
}

#[test]
fn integers_outside_i128_preserve_the_public_panic() {
    for (name, value) in outside_integer_cases() {
        let expected =
            "BUG: blocks cannot contain integers that do not fit into the 128-bit representation";
        assert_eq!(
            panic_message(std::panic::catch_unwind(|| value.hash())),
            expected,
            "{name}"
        );
        assert_eq!(
            panic_message(std::panic::catch_unwind(|| ICRC3Value::from(value).hash())),
            expected,
            "ICRC3 {name}"
        );
    }
}

#[test]
fn account_identifiers_follow_the_public_envelope() {
    for (name, owner, subaccount) in account_cases() {
        assert_eq!(
            account_identifier(owner, &subaccount),
            reference_account(owner, &subaccount),
            "{name}"
        );
    }
}

fn hexadecimal(bytes: [u8; 32]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    bytes
        .into_iter()
        .flat_map(|byte| {
            [
                char::from(DIGITS[usize::from(byte >> 4)]),
                char::from(DIGITS[usize::from(byte & 15)]),
            ]
        })
        .collect()
}

fn observations() -> serde_json::Value {
    let mut values = BTreeMap::new();
    let mut icrc3 = BTreeMap::new();
    for (name, value) in value_cases() {
        values.insert(name.clone(), hexadecimal(value.hash()));
        icrc3.insert(name, hexadecimal(ICRC3Value::from(value).hash()));
    }
    let accounts: BTreeMap<_, _> = account_cases()
        .into_iter()
        .map(|(name, owner, subaccount)| {
            (name, hexadecimal(account_identifier(owner, &subaccount)))
        })
        .collect();
    let mut panics = BTreeMap::new();
    for (name, value) in outside_integer_cases() {
        panics.insert(
            format!("value-{name}"),
            panic_message(std::panic::catch_unwind(|| value.hash())),
        );
        panics.insert(
            format!("icrc3-{name}"),
            panic_message(std::panic::catch_unwind(|| ICRC3Value::from(value).hash())),
        );
    }
    serde_json::json!({"schema": 1, "value": values, "icrc3": icrc3, "account": accounts, "panic": panics})
}

#[test]
fn public_hashes_match_frozen_observations() {
    let frozen: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/ledger-hash-observations.json")).unwrap();
    assert_eq!(observations(), frozen);
}

#[test]
#[ignore = "writes black-box observations only when IC_PLATFORM_LEDGER_OBSERVATIONS selects a target/ path"]
fn write_ledger_hash_observations() {
    let path = PathBuf::from(
        std::env::var("IC_PLATFORM_LEDGER_OBSERVATIONS")
            .expect("set IC_PLATFORM_LEDGER_OBSERVATIONS to a relative file below target/"),
    );
    assert!(
        path.is_relative()
            && path.starts_with("target")
            && path
                .components()
                .all(|part| matches!(part, Component::Normal(_) | Component::CurDir)),
        "output must stay below target/"
    );
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let json = serde_json::to_string_pretty(&observations()).unwrap();
    std::fs::write(path, format!("{json}\n")).unwrap();
}
