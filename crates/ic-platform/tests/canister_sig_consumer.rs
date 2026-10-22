//! Platform-only public-key/DER/hash qualification; no direct crypto or Candid dependency.
use ic_platform::canister_sig_creation::{
    CanisterSigPublicKey, IC_ROOT_PK_DER, IC_ROOT_PK_DER_PREFIX, extract_raw_root_pk_from_der,
    hash_bytes, hash_with_domain,
};

/// Preserve the original owner's key format, domain bytes and error identities.
pub fn roundtrip() {
    let issuer =
        ic_platform::principal::ExternalPrincipal::from_text("rwlgt-iiaaa-aaaaa-aaaaa-cai")
            .unwrap();
    let key = CanisterSigPublicKey::new(issuer, vec![42, 72, 44]);
    // Original 1.3.1 public-key vector, independent of Platform's exports.
    let der = b"\x30\x1f\x30\x0c\x06\x0a\x2b\x06\x01\x04\x01\x83\xb8\x43\x01\x02\x03\x0f\x00\x0a\x00\x00\x00\x00\x00\x00\x00\x00\x01\x01\x2a\x48\x2c";
    assert_eq!(key.to_der(), der);
    assert_eq!(CanisterSigPublicKey::try_from(der.as_slice()).unwrap(), key);
    assert_eq!(
        CanisterSigPublicKey::try_from_raw(&key.to_raw()).unwrap(),
        key
    );
    assert_eq!(key.canister_id, issuer);
    assert_eq!(key.seed, [42, 72, 44]);
    let error: String = CanisterSigPublicKey::try_from_raw(&[]).unwrap_err();
    assert_eq!(error, "empty raw canister sig pk");
    assert!(CanisterSigPublicKey::try_from_raw(&[2, 1]).is_err());
    let mut malformed = der.to_vec();
    malformed[2] ^= 1;
    assert!(CanisterSigPublicKey::try_from(malformed.as_slice()).is_err());
    assert_eq!(
        extract_raw_root_pk_from_der(IC_ROOT_PK_DER).unwrap(),
        IC_ROOT_PK_DER[37..]
    );
    assert_eq!(&IC_ROOT_PK_DER[..37], IC_ROOT_PK_DER_PREFIX);
    assert_eq!(
        extract_raw_root_pk_from_der(&[]),
        Err("invalid root pk length".into())
    );
    let mut root = *IC_ROOT_PK_DER;
    root[0] ^= 1;
    assert_eq!(
        extract_raw_root_pk_from_der(&root),
        Err("invalid OID".into())
    );
    assert_eq!(
        hash_bytes(b""),
        [
            0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14, 0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f,
            0xb9, 0x24, 0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c, 0xa4, 0x95, 0x99, 0x1b,
            0x78, 0x52, 0xb8, 0x55,
        ]
    );
    // Independent SHA-256 of the length byte, domain and payload.
    assert_eq!(
        hash_with_domain(b"sig", b"payload"),
        [
            0xa8, 0xd3, 0xc3, 0x51, 0xc1, 0xde, 0x69, 0xd1, 0x22, 0x79, 0x60, 0x54, 0x78, 0x88,
            0xfb, 0x33, 0x8f, 0xce, 0xb9, 0xcb, 0x91, 0xa5, 0x0c, 0x65, 0x86, 0xd7, 0xba, 0x22,
            0x36, 0x73, 0x16, 0x17,
        ]
    );
    assert_ne!(
        hash_with_domain(b"sig", b"payload"),
        hash_bytes(b"sigpayload")
    );
}

#[test]
fn key_and_hash_contracts_preserve_external_bytes() {
    roundtrip();
}

#[test]
fn short_der_panic_is_characterized_without_changing_crypto() {
    // Native unwinding characterizes the inherited bug; it is not a Wasm guard.
    // The imported parser reads the 14-byte OID at offset 2 before checking length.
    for length in 0..16 {
        let bytes = vec![0; length];
        assert!(
            std::panic::catch_unwind(|| CanisterSigPublicKey::try_from(bytes.as_slice())).is_err()
        );
    }
    // Once that slice is accessible, invalid OIDs return the upstream error.
    for length in 16..=20 {
        let bytes = vec![0; length];
        assert!(CanisterSigPublicKey::try_from(bytes.as_slice()).is_err());
    }
}
