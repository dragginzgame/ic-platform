use super::*;
use crate::candid_codec::{
    decoder::{Limits, decode_one},
    encode_one,
};
#[test]
fn reserved_preserves_serde_wire_and_validated_skipping() {
    let limits = Limits {
        bytes: 4096,
        header: 1024,
        types: 64,
        work: 10000,
        allocation: 10000,
    };
    assert_eq!(
        encode_one(&Reserved, 1024).unwrap(),
        candid::encode_one(candid::Reserved).unwrap()
    );
    assert_eq!(
        serde_json::to_vec(&Reserved).unwrap(),
        serde_json::to_vec(&candid::Reserved).unwrap()
    );
    assert_eq!(
        serde_cbor::to_vec(&Reserved).unwrap(),
        serde_cbor::to_vec(&candid::Reserved).unwrap()
    );
    for bytes in [
        candid::encode_one(42u64).unwrap(),
        candid::encode_one(vec![Some("text")]).unwrap(),
    ] {
        assert_eq!(decode_one::<Reserved>(&bytes, limits).unwrap(), Reserved);
        for end in 0..bytes.len() {
            assert!(decode_one::<Reserved>(&bytes[..end], limits).is_err());
        }
        assert!(decode_one::<Reserved>(&bytes, Limits { work: 0, ..limits }).is_err());
    }
    for text in ["null", "42", "{\"x\":[1,2]}"] {
        assert!(serde_json::from_str::<Reserved>(text).is_ok());
        assert!(serde_json::from_str::<candid::Reserved>(text).is_ok());
    }
}
