use crate::candid_codec::{
    decoder::{Error, Limits, decode_one},
    encode_one,
};
use candid as reference;
mod declarations;
fn limits() -> Limits {
    Limits {
        bytes: 1_000_000,
        header: 100_000,
        types: 1024,
        work: 10_000_000,
        allocation: 4_000_000,
    }
}

#[derive(Debug, PartialEq, reference::CandidType, serde::Deserialize)]
#[candid_path("reference")]
struct Record {
    a: u64,
    b: Option<String>,
    c: Vec<bool>,
}
crate::candid_record!(Record { 97 => a: u64, 98 => b: Option<String>, 99 => c: Vec<bool> });

#[derive(Debug, PartialEq, reference::CandidType, serde::Deserialize)]
#[candid_path("reference")]
enum Status {
    A(()),
    B(String),
    C(Record),
}
crate::candid_variant!(Status { 65 => A(()), 66 => B(String), 67 => C(Record) });

#[derive(reference::CandidType)]
#[candid_path("reference")]
struct Extension {
    a: u64,
    c: Vec<bool>,
    extra: Vec<String>,
}

#[test]
fn named_records_and_payload_variants_cross_decode() {
    for value in [
        Status::A(()),
        Status::B("message 🌍".to_owned()),
        Status::C(Record {
            a: 42,
            b: None,
            c: vec![true, false],
        }),
    ] {
        let expected = reference::encode_one(&value).unwrap();
        assert_eq!(decode_one::<Status>(&expected, limits()).unwrap(), value);
        let bytes = encode_one(&value, 1000).unwrap();
        assert_eq!(
            reference::decode_args_with_config::<(Status,)>(
                &bytes,
                &reference::DecoderConfig::new()
            )
            .unwrap()
            .0,
            value
        );
        for n in 0..expected.len() {
            assert!(decode_one::<Status>(&expected[..n], limits()).is_err());
        }
    }
}
#[test]
fn record_extensions_and_missing_fields() {
    let bytes = reference::encode_one(Extension {
        a: 10,
        c: vec![true],
        extra: vec!["ignored".to_owned()],
    })
    .unwrap();
    let decoded = decode_one::<Record>(&bytes, limits()).unwrap();
    let expected =
        reference::decode_args_with_config::<(Record,)>(&bytes, &reference::DecoderConfig::new())
            .unwrap()
            .0;
    assert_eq!(decoded, expected);
    assert_eq!(decoded.b, None);
    assert_eq!(
        decode_one::<Record>(&reference::encode_one(()).unwrap(), limits()),
        Err(Error::Type)
    );
}
#[test]
fn wrong_variant_shape_is_recoverable_only_for_options() {
    // A null payload has no index byte; shape must be checked before reading one.
    let bytes = reference::encode_one(()).unwrap();
    assert_eq!(decode_one::<Status>(&bytes, limits()), Err(Error::Type));
    assert_eq!(decode_one::<Option<Status>>(&bytes, limits()), Ok(None));
    let bytes = reference::encode_one(false).unwrap();
    assert_eq!(decode_one::<Option<Status>>(&bytes, limits()), Ok(None));
}

#[test]
#[cfg(feature = "principal")]
fn principal_preserves_canonical_identity_and_bytes() {
    for count in [0, 1, 28, 29] {
        let raw = vec![42; count];
        let owned = crate::Principal::try_from_slice(&raw).unwrap();
        let external = reference::Principal::from_slice(&raw);
        let bytes = reference::encode_one(external).unwrap();
        assert_eq!(encode_one(&owned, 1000).unwrap(), bytes);
        let decoded: crate::Principal = decode_one(&bytes, limits()).unwrap();
        assert_eq!(decoded, owned);
    }
    let mut invalid = b"DIDL\0\x01\x68\x01\x1e".to_vec();
    invalid.extend_from_slice(&[0; 30]);
    assert_eq!(
        decode_one::<crate::Principal>(&invalid, limits()),
        Err(Error::Malformed)
    );
    assert_eq!(
        decode_one::<crate::Principal>(b"DIDL\0\x01\x68\0", limits()),
        Err(Error::Unsupported)
    );
}

#[derive(Debug, PartialEq, reference::CandidType, serde::Deserialize)]
#[candid_path("reference")]
enum Recursive {
    A(u64),
    B(Vec<Recursive>),
    C(std::collections::BTreeMap<String, Recursive>),
}
crate::candid_variant!(Recursive {
    65 => A(u64), 66 => B(Vec<Recursive>),
    67 => C(std::collections::BTreeMap<String, Recursive>)
});

#[test]
fn recursive_container_mappings_cross_decode() {
    let value = Recursive::C(std::collections::BTreeMap::from([
        ("leaf".to_owned(), Recursive::A(42)),
        ("branch".to_owned(), Recursive::B(vec![Recursive::A(7)])),
    ]));
    let expected = reference::encode_one(&value).unwrap();
    assert_eq!(decode_one::<Recursive>(&expected, limits()).unwrap(), value);
    let bytes = encode_one(&value, 10000).unwrap();
    assert_eq!(candid::decode_one::<Recursive>(&bytes).unwrap(), value);
    for n in 0..bytes.len() {
        assert!(decode_one::<Recursive>(&bytes[..n], limits()).is_err());
    }
}

#[test]
fn map_order_duplicates_arrays_and_blob_match_registry() {
    use std::collections::BTreeMap;
    let entries = vec![
        ("z".to_owned(), 1u64),
        ("a".to_owned(), 2),
        ("z".to_owned(), 3),
    ];
    let bytes = reference::encode_one(entries).unwrap();
    let expected = candid::decode_one::<BTreeMap<String, u64>>(&bytes).unwrap();
    let owned = decode_one::<BTreeMap<String, u64>>(&bytes, limits()).unwrap();
    assert_eq!(owned, expected);
    assert_eq!(
        encode_one(&owned, 1024).unwrap(),
        reference::encode_one(&expected).unwrap()
    );
    let array = [255u8; 32];
    let bytes = reference::encode_one(array).unwrap();
    assert_eq!(encode_one(&array, 1024).unwrap(), bytes);
    assert_eq!(decode_one::<[u8; 32]>(&bytes, limits()).unwrap(), array);
    assert_eq!(
        decode_one::<[u8; 31]>(&bytes, limits()),
        Err(Error::Malformed)
    );
    assert_eq!(
        decode_one::<[u8; 33]>(&bytes, limits()),
        Err(Error::Malformed)
    );
    let mut budget = limits();
    budget.allocation = 0;
    assert_eq!(decode_one::<[u8; 32]>(&bytes, budget), Err(Error::Limit));
    #[cfg(feature = "icrc-ledger-types")]
    {
        let blob = serde_bytes::ByteBuf::from(array.to_vec());
        assert_eq!(encode_one(&blob, 1024).unwrap(), bytes);
        assert_eq!(
            decode_one::<serde_bytes::ByteBuf>(&bytes, limits()).unwrap(),
            blob
        );
    }
}

#[test]
fn recursive_values_keep_depth_and_budget_refusals() {
    let mut value = Recursive::A(1);
    for _ in 0..20 {
        value = Recursive::B(vec![value]);
    }
    let bytes = encode_one(&value, 10000).unwrap();
    assert_eq!(decode_one::<Recursive>(&bytes, limits()), Err(Error::Limit));
    let value = Recursive::C(std::collections::BTreeMap::from([(
        "x".to_owned(),
        Recursive::A(1),
    )]));
    let bytes = encode_one(&value, 10000).unwrap();
    let mut budget = limits();
    budget.work = 0;
    assert_eq!(decode_one::<Recursive>(&bytes, budget), Err(Error::Limit));
    budget = limits();
    budget.allocation = 0;
    assert_eq!(decode_one::<Recursive>(&bytes, budget), Err(Error::Limit));
}

#[test]
fn deferred_schema_cycles_fail_closed() {
    use crate::candid_codec::{Encode, Type, Writer};
    static LOOP: Type = Type::Deferred(|| &LOOP);
    struct Invalid;
    impl Encode for Invalid {
        const TYPE: &'static Type = &LOOP;
        fn write(&self, _: &mut Writer) -> Result<(), crate::candid_codec::Error> {
            Ok(())
        }
    }
    assert_eq!(
        encode_one(&Invalid, 1024),
        Err(crate::candid_codec::Error::Schema)
    );
}
