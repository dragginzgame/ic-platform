//! Compare independent upstream and feature-gated builds without feature unification.
use ic_secp256k1::{CanisterId, MasterPublicKeyId, PocketIcMasterPublicKeyId, PublicKey};

fn main() {
    let principals = [
        vec![],
        vec![4],
        vec![0, 0, 0, 0, 0, 0, 0, 1, 1, 1],
        vec![1; 29],
    ];
    let paths = [
        vec![],
        vec![vec![]],
        vec![vec![0, 255], vec![8; 32]],
        vec![vec![7; 64]; 255],
    ];
    for bytes in principals {
        let principal = CanisterId::from_slice(&bytes);
        for path in &paths {
            let keys = [
                PublicKey::derive_mainnet_key(MasterPublicKeyId::EcdsaKey1, &principal, path),
                PublicKey::derive_pocketic_key(
                    PocketIcMasterPublicKeyId::EcdsaKey1,
                    &principal,
                    path,
                ),
                PublicKey::derive_pocketic_key(
                    PocketIcMasterPublicKeyId::EcdsaTestKey1,
                    &principal,
                    path,
                ),
                PublicKey::derive_pocketic_key(
                    PocketIcMasterPublicKeyId::EcdsaDfxTestKey,
                    &principal,
                    path,
                ),
            ];
            for (key, chain_code) in keys {
                println!(
                    "{:?}|{:?}|{:?}",
                    key.serialize_sec1(true),
                    key.serialize_sec1(false),
                    chain_code
                );
            }
        }
    }
    for bytes in [vec![], vec![0; 33], vec![255; 33], vec![4; 65]] {
        println!("{:?}", PublicKey::deserialize_sec1(&bytes));
    }
}
