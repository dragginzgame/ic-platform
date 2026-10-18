//! Exact original-owner type identities and public derivation bytes.
#![cfg(not(target_family = "wasm"))]

#[test]
fn original_types_and_derivation_bytes() {
    use ic_platform::{Principal, secp256k1::*};
    let _: fn(PublicKey) -> ic_secp256k1::PublicKey = |key| key;
    let _: fn(ic_secp256k1::PublicKey) -> PublicKey = |key| key;
    let _: fn(KeyDecodingError) -> ic_secp256k1::KeyDecodingError = |error| error;
    let _: fn(ic_secp256k1::KeyDecodingError) -> KeyDecodingError = |error| error;
    let _: fn(MasterPublicKeyId) -> ic_secp256k1::MasterPublicKeyId = |id| id;
    let _: fn(ic_secp256k1::MasterPublicKeyId) -> MasterPublicKeyId = |id| id;
    let _: fn(PocketIcMasterPublicKeyId) -> ic_secp256k1::PocketIcMasterPublicKeyId = |id| id;
    let _: fn(ic_secp256k1::PocketIcMasterPublicKeyId) -> PocketIcMasterPublicKeyId = |id| id;
    let root = Principal::from_text("5lnwm-ziaaa-aaaae-agtqa-cai").unwrap();
    let path = vec![b"platform-boundary".to_vec()];
    let (actual, chain) = PublicKey::derive_mainnet_key(MasterPublicKeyId::EcdsaKey1, &root, &path);
    let (original, original_chain) = ic_secp256k1::PublicKey::derive_mainnet_key(
        ic_secp256k1::MasterPublicKeyId::EcdsaKey1,
        &root,
        &path,
    );
    assert_eq!(actual.serialize_sec1(true), original.serialize_sec1(true));
    assert_eq!(chain, original_chain);
}
