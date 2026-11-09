use super::*;
use crate::candid_codec::{
    decoder::{Limits, decode_one},
    encode_one,
};

#[test]
fn natural_range_wire_and_serde_match_registry() {
    let limits = Limits {
        bytes: 8192,
        header: 1024,
        types: 32,
        work: 100_000,
        allocation: 32_768,
    };
    for bits in [0usize, 1, 7, 32, 64, 128, 256, 1024, 8192] {
        let value = BigUint::from(1u8) << bits;
        for magnitude in [value.clone(), &value - 1u8, &value + 1u8] {
            let owned = Nat(magnitude.clone());
            let external = magnitude.to_string().parse::<candid::Nat>().unwrap();
            let bytes = candid::encode_one(&external).unwrap();
            assert_eq!(encode_one(&owned, 8192).unwrap(), bytes);
            assert_eq!(decode_one::<Nat>(&bytes, limits).unwrap(), owned);
            let json = serde_json::to_vec(&owned).unwrap();
            assert_eq!(json, serde_json::to_vec(&external).unwrap());
            assert_eq!(serde_json::from_slice::<Nat>(&json).unwrap(), owned);
            let cbor = serde_cbor::to_vec(&owned).unwrap();
            assert_eq!(cbor, serde_cbor::to_vec(&external).unwrap());
            assert_eq!(serde_cbor::from_slice::<Nat>(&cbor).unwrap(), owned);
            assert_eq!(owned.to_string(), external.to_string());
        }
    }
}
#[test]
fn serde_admission_and_conversions_preserve_ranges() {
    for text in [
        "0",
        "1",
        "9223372036854775807",
        "18446744073709551615",
        "-1",
        "1.5",
        "[]",
        "[0,1]",
        "[4294967296]",
        "\"340282366920938463463374607431768211456\"",
        "\"1_000\"",
        "\"-1\"",
    ] {
        let own = serde_json::from_str::<Nat>(text).map(|n| n.0);
        let reference = serde_json::from_str::<candid::Nat>(text).map(|n| n.0);
        assert_eq!(own.is_ok(), reference.is_ok(), "{text}");
        if let (Ok(own), Ok(reference)) = (own, reference) {
            assert_eq!(own.to_string(), reference.to_string());
        }
    }
    assert!(Nat::from(u128::MAX) > u64::MAX);
    let owned = Nat::from(u128::MAX);
    let registry: candid::Nat = owned.clone().into();
    assert_eq!(Nat::from(registry), owned);
}

#[test]
fn tagged_serde_buffers_reject_empty_and_wrong_tags() {
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
    for bytes in [vec![], vec![0], vec![2, 1]] {
        assert!(Nat::deserialize(Tagged(bytes)).is_err());
    }
    for bytes in [vec![1], vec![1, 0], vec![1, 255, 255]] {
        let own = Nat::deserialize(Tagged(bytes.clone())).unwrap();
        let reference = candid::Nat::deserialize(Tagged(bytes)).unwrap();
        assert_eq!(own.to_string(), reference.to_string());
    }
}
