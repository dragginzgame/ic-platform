use super::*;
use crate::candid_codec::{
    decoder::{Limits, decode_one},
    encode_one,
};

#[test]
fn signed_range_wire_serde_and_display_match_registry() {
    let limits = Limits {
        bytes: 8192,
        header: 1024,
        types: 32,
        work: 100_000,
        allocation: 32_768,
    };
    for bits in [0usize, 1, 7, 32, 64, 128, 256, 1024, 8192] {
        let boundary = BigInt::from(1u8) << bits;
        for magnitude in [&boundary - 1u8, boundary.clone(), &boundary + 1u8] {
            for value in [magnitude.clone(), -magnitude] {
                let owned = Int(value.clone());
                let external = value.to_string().parse::<candid::Int>().unwrap();
                let bytes = candid::encode_one(&external).unwrap();
                assert_eq!(encode_one(&owned, 8192).unwrap(), bytes);
                assert_eq!(decode_one::<Int>(&bytes, limits).unwrap(), owned);
                assert_eq!(
                    serde_json::to_vec(&owned).unwrap(),
                    serde_json::to_vec(&external).unwrap()
                );
                assert_eq!(
                    serde_cbor::to_vec(&owned).unwrap(),
                    serde_cbor::to_vec(&external).unwrap()
                );
                assert_eq!(
                    serde_json::from_slice::<Int>(&serde_json::to_vec(&owned).unwrap()).unwrap(),
                    owned
                );
                assert_eq!(
                    serde_cbor::from_slice::<Int>(&serde_cbor::to_vec(&owned).unwrap()).unwrap(),
                    owned
                );
                assert_eq!(owned.to_string(), external.to_string());
            }
        }
    }
}

#[test]
fn serde_sign_and_magnitude_pair_matches_bigint_and_refuses_malformed_input() {
    for input in [
        "[0,[]]",
        "[1,[1]]",
        "[-1,[4294967295,4294967295,1]]",
        "[0,[123]]",
        "[1,[]]",
        "[-1,[0]]",
        "[]",
        "[1]",
        "[1,[],0]",
        "[2,[1]]",
        "[-2,[1]]",
        "[null,[]]",
        "[1,1]",
        "[1,[-1]]",
        "[1,[4294967296]]",
        "[1,[1.5]]",
    ] {
        let expected = serde_json::from_str::<BigInt>(input);
        let actual = serde_json::from_str::<Int>(input).map(|value| value.0);
        assert_eq!(actual.is_ok(), expected.is_ok(), "{input}");
        if let (Ok(actual), Ok(expected)) = (&actual, &expected) {
            assert_eq!(actual, expected, "{input}");
        }
        let value: serde_json::Value = serde_json::from_str(input).unwrap();
        let cbor = serde_cbor::to_vec(&value).unwrap();
        let decoded = serde_cbor::from_slice::<Int>(&cbor).map(|value| value.0);
        assert_eq!(decoded.is_ok(), expected.is_ok(), "CBOR: {input}");
        if let (Ok(decoded), Ok(expected)) = (decoded, expected) {
            assert_eq!(decoded, expected, "CBOR: {input}");
        }
    }
}

#[test]
fn serde_admission_matches_registry() {
    for text in [
        "0",
        "-1",
        "9223372036854775807",
        "18446744073709551615",
        "1.5",
        "[]",
        "[0,1]",
        "\"-340282366920938463463374607431768211456\"",
        "\"1_000\"",
    ] {
        let own = serde_json::from_str::<Int>(text).map(|n| n.0);
        let reference = serde_json::from_str::<candid::Int>(text).map(|n| n.0);
        assert_eq!(own.is_ok(), reference.is_ok(), "{text}");
        if let (Ok(own), Ok(reference)) = (own, reference) {
            assert_eq!(own.to_string(), reference.to_string());
        }
    }
}

#[test]
fn tagged_buffers_preserve_signed_and_natural_admission() {
    struct Tagged(Vec<u8>);
    impl<'de> serde::Deserializer<'de> for Tagged {
        type Error = serde::de::value::Error;
        fn deserialize_any<V: serde::de::Visitor<'de>>(
            self,
            visitor: V,
        ) -> Result<V::Value, Self::Error> {
            visitor.visit_byte_buf(self.0)
        }
        serde::forward_to_deserialize_any! {
            bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
            byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct
            map struct enum identifier ignored_any
        }
    }
    assert!(Int::deserialize(Tagged(vec![])).is_err());
    for bytes in [
        vec![0],
        vec![1],
        vec![0, 255],
        vec![1, 255],
        vec![0, 0, 128],
        vec![2, 1],
    ] {
        let own = Int::deserialize(Tagged(bytes.clone())).map(|n| n.0);
        let reference = candid::Int::deserialize(Tagged(bytes)).map(|n| n.0);
        assert_eq!(own.is_ok(), reference.is_ok());
        if let (Ok(own), Ok(reference)) = (own, reference) {
            assert_eq!(own.to_string(), reference.to_string());
        }
    }
}
