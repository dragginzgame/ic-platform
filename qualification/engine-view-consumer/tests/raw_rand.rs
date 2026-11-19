//! Independent upstream Candid producer checks for the IC management raw_rand profile.
#![forbid(unsafe_code)]
#![cfg(all(feature = "raw-rand", not(target_family = "wasm")))]

use ic_platform_protocol::raw_rand;

#[test]
fn canonical_management_schema_and_non_profile_refusals() {
    assert_eq!(
        candid::encode_args(()).unwrap(),
        raw_rand::encode(6).unwrap()
    );
    for seed in [[0u8; 32], [255; 32], std::array::from_fn(|i| i as u8)] {
        let bytes = candid::encode_one(seed.to_vec()).unwrap();
        assert_eq!(raw_rand::decode(&bytes, bytes.len()), Ok(seed));
    }
    for bytes in [
        candid::encode_one(vec![0u8; 31]).unwrap(),
        candid::encode_one(vec![0u8; 33]).unwrap(),
        candid::encode_one(vec![0u16; 32]).unwrap(),
        candid::encode_args((vec![0u8; 32], 0u8)).unwrap(),
    ] {
        assert!(raw_rand::decode(&bytes, 4096).is_err());
    }
}
