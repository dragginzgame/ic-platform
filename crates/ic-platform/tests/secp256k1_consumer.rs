//! Platform-only native public derivation, encoding and malformed-key contract.
#![cfg(not(target_family = "wasm"))]

use ic_platform::{
    principal::ExternalPrincipal as Principal,
    secp256k1::{KeyDecodingError, MasterPublicKeyId, PocketIcMasterPublicKeyId, PublicKey},
};

#[test]
fn public_derivation_and_encoding() {
    let root = Principal::from_text("5lnwm-ziaaa-aaaae-agtqa-cai").unwrap();
    let path = vec![b"platform-boundary".to_vec()];
    let (mainnet, chain) =
        PublicKey::derive_mainnet_key(MasterPublicKeyId::EcdsaKey1, &root, &path);
    let encoded = mainnet.serialize_sec1(true);
    assert_eq!(encoded.len(), 33);
    assert_eq!(
        PublicKey::deserialize_sec1(&encoded)
            .unwrap()
            .serialize_sec1(true),
        encoded
    );
    assert_eq!(mainnet.serialize_sec1(false).len(), 65);
    let (changed, changed_chain) =
        PublicKey::derive_mainnet_key(MasterPublicKeyId::EcdsaKey1, &root, &[]);
    assert_ne!(changed.serialize_sec1(true), encoded);
    assert_ne!(changed_chain, chain);
    for id in [
        PocketIcMasterPublicKeyId::EcdsaKey1,
        PocketIcMasterPublicKeyId::EcdsaTestKey1,
        PocketIcMasterPublicKeyId::EcdsaDfxTestKey,
    ] {
        let (local, _) = PublicKey::derive_pocketic_key(id, &root, &path);
        assert_ne!(local.serialize_sec1(true), encoded);
    }
    let invalid: Result<PublicKey, KeyDecodingError> = PublicKey::deserialize_sec1(&[0; 33]);
    assert!(invalid.is_err());
}
