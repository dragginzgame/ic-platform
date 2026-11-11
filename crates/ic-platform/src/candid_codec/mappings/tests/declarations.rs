use super::limits;
use crate::candid_codec::{
    Nat,
    decoder::{Error, decode_one, decode_one_with_skip_limit},
    encode_one,
};
use candid as reference;

crate::candid_record! {
    #[derive(Debug, PartialEq, reference::CandidType, serde::Deserialize, serde::Serialize)]
    #[candid_path("reference")]
    struct Record {
        /// Attributes belong to the Rust field; numeric labels own Candid identity.
        #[serde(rename = "a")]
        97 => amount: u64,
        98 => b: Option<String>,
        99 => c: Vec<bool>,
    }
}
crate::candid_variant! {
    #[derive(Debug, PartialEq, reference::CandidType, serde::Deserialize)]
    #[candid_path("reference")]
    enum Choice {
        /// A null payload.
        65 => A,
        66 => B(u64),
        67 => C { 97 => a: u64, 98 => b: String },
    }
}
#[derive(Debug, PartialEq)]
enum PreviousChoice {
    A,
    B(u64),
    C { a: u64, b: String },
}
crate::candid_variant!(PreviousChoice { 65 => A, 66 => B(u64), 67 => C { 97 => a: u64, 98 => b: String } });

#[test]
fn variants_preserve_existing_mapping_and_independent_bytes() {
    for (value, previous) in [
        (Choice::A, PreviousChoice::A),
        (Choice::B(u64::MAX), PreviousChoice::B(u64::MAX)),
        (
            Choice::C {
                a: 42,
                b: "🌍".into(),
            },
            PreviousChoice::C {
                a: 42,
                b: "🌍".into(),
            },
        ),
    ] {
        let bytes = encode_one(&value, 4096).unwrap();
        assert_eq!(bytes, encode_one(&previous, 4096).unwrap());
        assert_eq!(bytes, reference::encode_one(&value).unwrap());
        assert_eq!(decode_one::<Choice>(&bytes, limits()).unwrap(), value);
        assert_eq!(reference::decode_one::<Choice>(&bytes).unwrap(), value);
    }
    // Frozen protocol vector: variant {65:null;66:nat64;67:record{97:nat64;98:text}}.
    assert_eq!(
        encode_one(&Choice::A, 1024).unwrap(),
        b"DIDL\x02\x6b\x03\x41\x7f\x42\x78\x43\x01\x6c\x02\x61\x78\x62\x71\x01\0\0"
    );
}

#[test]
fn records_preserve_bytes_optional_admission_and_bounded_errors() {
    let previous = super::Record {
        a: 42,
        b: Some("memo".into()),
        c: vec![true],
    };
    let value = Record {
        amount: 42,
        b: Some("memo".into()),
        c: vec![true],
    };
    let bytes = encode_one(&value, 4096).unwrap();
    assert_eq!(bytes, encode_one(&previous, 4096).unwrap());
    assert_eq!(bytes, reference::encode_one(&value).unwrap());
    assert_eq!(serde_json::to_value(&value).unwrap()["a"], 42);
    let extended = reference::encode_one(super::Extension {
        a: 42,
        c: vec![true],
        extra: vec!["skip".into()],
    })
    .unwrap();
    assert_eq!(decode_one::<Record>(&extended, limits()).unwrap().b, None);
    assert_eq!(
        decode_one_with_skip_limit::<Record>(&extended, limits(), 0),
        Err(Error::Limit)
    );
    for len in 0..bytes.len() {
        assert_eq!(
            decode_one::<Record>(&bytes[..len], limits()).unwrap_err(),
            decode_one::<super::Record>(&bytes[..len], limits()).unwrap_err()
        );
    }
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert_eq!(
        decode_one::<Record>(&trailing, limits()),
        Err(Error::Trailing)
    );
    for budget in [
        crate::candid_codec::decoder::Limits {
            work: 0,
            ..limits()
        },
        crate::candid_codec::decoder::Limits {
            allocation: 0,
            ..limits()
        },
    ] {
        assert_eq!(decode_one::<Record>(&bytes, budget), Err(Error::Limit));
    }
}

crate::candid_record! {
    #[derive(Debug, PartialEq, reference::CandidType, serde::Deserialize)]
    #[candid_path("reference")]
    struct Index {
        596_483_356 => balance: Nat,
        3_331_539_157 => transactions: Vec<Nat>,
        3_582_638_454 => oldest_tx_id: Option<Nat>,
    }
}
crate::candid_variant! {
    #[derive(Debug, PartialEq, reference::CandidType, serde::Deserialize)]
    #[candid_path("reference")]
    enum Recursive {
        65 => A(Nat),
        66 => B(Vec<Recursive>),
    }
}

#[test]
fn unbounded_nats_and_recursive_shapes_match_oracle() {
    let big: Nat = "340282366920938463463374607431768211456".parse().unwrap();
    let index = Index {
        balance: big.clone(),
        transactions: vec![Nat::from(u128::MAX), big.clone()],
        oldest_tx_id: Some(big.clone()),
    };
    let bytes = encode_one(&index, 4096).unwrap();
    assert_eq!(bytes, reference::encode_one(&index).unwrap());
    assert_eq!(decode_one::<Index>(&bytes, limits()).unwrap(), index);
    let mut recursive = Recursive::B(vec![Recursive::A(big)]);
    let bytes = reference::encode_one(&recursive).unwrap();
    assert_eq!(encode_one(&recursive, 4096).unwrap(), bytes);
    assert_eq!(
        decode_one::<Recursive>(&bytes, limits()).unwrap(),
        recursive
    );
    for _ in 0..17 {
        recursive = Recursive::B(vec![recursive]);
    }
    assert_eq!(
        decode_one::<Recursive>(&encode_one(&recursive, 4096).unwrap(), limits()),
        Err(Error::Limit)
    );
}
