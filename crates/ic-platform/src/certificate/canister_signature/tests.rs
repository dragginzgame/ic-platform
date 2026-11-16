use super::*;
const FRAME: &[u8] = b"\xd9\xd9\xf7\xa2\x6bcertificate\x41\xff\x64tree\x81\x00";
fn decode(bytes: &[u8]) -> Result<DecodedCanisterSignature<'_>, CanisterSignatureDecodeError> {
    decode_canister_signature(bytes, CertificateDecodeLimits { bytes: bytes.len() })
}
#[test]
fn retains_original_and_opaque_certificate_without_authentication() {
    let value = decode(FRAME).unwrap();
    assert!(std::ptr::eq(value.original_bytes(), FRAME));
    assert_eq!(value.certificate_bytes(), [255]);
    assert_eq!(value.unverified_witness(), &ic_certification::empty());
    // The outer envelope can be valid even though the certificate is not.
    assert!(
        super::super::decode_certificate(
            value.certificate_bytes(),
            CertificateDecodeLimits { bytes: 1 }
        )
        .is_err()
    );
}
#[test]
fn bounds_precede_tag_and_grammar_checks() {
    assert!(matches!(
        decode_canister_signature(&[255], CertificateDecodeLimits { bytes: 0 }),
        Err(CanisterSignatureDecodeError::InputLimit { .. })
    ));
    assert!(matches!(
        decode(&FRAME[3..]),
        Err(CanisterSignatureDecodeError::MissingTag)
    ));
    assert!(
        decode_canister_signature(
            FRAME,
            CertificateDecodeLimits {
                bytes: FRAME.len() - 1
            }
        )
        .is_err()
    );
    for end in 0..FRAME.len() {
        assert!(decode(&FRAME[..end]).is_err());
    }
    let mut trailing = FRAME.to_vec();
    trailing.push(0);
    assert!(decode(&trailing).is_err());
}
#[test]
fn duplicate_unknown_and_wrong_payload_kinds_refuse() {
    assert!(decode(b"\xd9\xd9\xf7\x82\x40\x81\x00").is_err());
    for suffix in [
        b"\x6bcertificate\x40".as_slice(),
        b"\x64tree\x81\x00",
        b"\x61x\x00",
    ] {
        let mut changed = FRAME.to_vec();
        changed[3] = 0xa3;
        changed.extend_from_slice(suffix);
        assert!(decode(&changed).is_err());
    }
    for payload in [b"\x61x".as_slice(), b"\x81\x00", b"\xf6"] {
        let mut changed = b"\xd9\xd9\xf7\xa2\x6bcertificate".to_vec();
        changed.extend_from_slice(payload);
        changed.extend_from_slice(b"\x64tree\x81\x00");
        assert!(decode(&changed).is_err());
    }
    for tree in [
        b"\x82\x00\x00".as_slice(),
        b"\x81\x05",
        b"\x82\x04\x40",
        b"\x82\x03\x60",
    ] {
        let mut changed = b"\xd9\xd9\xf7\xa2\x6bcertificate\x40\x64tree".to_vec();
        changed.extend_from_slice(tree);
        assert!(decode(&changed).is_err());
    }
}
