use super::{CertificateDecodeLimits, DecodeError, decode_certificate};

// Literal named-map certificate: empty tree and one-byte (unverified) signature.
const DIRECT: &[u8] = b"\xa2\x64tree\x81\x00\x69signature\x41\x01";

fn decode(bytes: &[u8]) -> Result<super::DecodedCertificate<'_>, DecodeError> {
    decode_certificate(bytes, CertificateDecodeLimits { bytes: bytes.len() })
}

#[test]
fn original_bytes_and_unverified_fields_survive_without_normalization() {
    for tagged in [false, true] {
        let mut bytes = if tagged {
            vec![0xd9, 0xd9, 0xf7]
        } else {
            vec![]
        };
        bytes.extend_from_slice(DIRECT);
        let decoded = decode(&bytes).unwrap();
        assert!(std::ptr::eq(
            decoded.original_bytes().as_ptr(),
            bytes.as_ptr()
        ));
        assert_eq!(decoded.original_bytes(), bytes);
        assert_eq!(decoded.unverified().signature, [1]);
        assert!(decoded.unverified().delegation.is_none());
        assert_eq!(decoded.unverified().tree, ic_certification::empty());
    }
}

#[test]
fn delegation_is_retained_as_opaque_bytes_not_authenticated() {
    let mut bytes = DIRECT.to_vec();
    bytes[0] = 0xa3;
    bytes.extend_from_slice(b"\x6adelegation\xa2\x69subnet_id\x41\x04\x6bcertificate\x41\xff");
    let decoded = decode(&bytes).unwrap();
    let delegation = decoded.unverified().delegation.as_ref().unwrap();
    assert_eq!(delegation.subnet_id, [4]);
    // Invalid as an embedded certificate: this stage must not imply verification.
    assert_eq!(delegation.certificate, [255]);
    assert_eq!(decoded.original_bytes(), bytes);
}

#[test]
fn byte_admission_precedes_parsing_and_trailing_bytes_refuse() {
    assert!(matches!(
        decode_certificate(&[0xff], CertificateDecodeLimits { bytes: 0 }),
        Err(DecodeError::InputLimit {
            actual: 1,
            maximum: 0
        })
    ));
    assert!(decode(DIRECT).is_ok());
    assert!(matches!(
        decode_certificate(
            DIRECT,
            CertificateDecodeLimits {
                bytes: DIRECT.len() - 1
            }
        ),
        Err(DecodeError::InputLimit { .. })
    ));
    for end in 0..DIRECT.len() {
        assert!(decode(&DIRECT[..end]).is_err());
    }
    let mut extra = DIRECT.to_vec();
    extra.push(0);
    assert!(decode(&extra).is_err());
}

#[test]
fn duplicate_unknown_missing_and_wrongly_typed_fields_refuse() {
    for extra in [
        b"\x64tree\x81\x00".as_slice(),
        b"\x69signature\x40",
        b"\x67unknown\xf6",
    ] {
        let mut bytes = DIRECT.to_vec();
        bytes[0] = 0xa3;
        bytes.extend_from_slice(extra);
        assert!(decode(&bytes).is_err());
    }
    for bad in [
        b"\xa0".as_slice(),
        b"\xa1\x64tree\x81\x00",
        b"\xa2\x64tree\x81\x00\x69signature\x81\x01", // array, not bytes
        b"\xa2\x64tree\x81\x00\x69signature\x61x", // text, not bytes
        b"\xa3\x64tree\x81\x00\x69signature\x40\x6adelegation\xa0",
        b"\xa3\x64tree\x81\x00\x69signature\x40\x6adelegation\xa2\x69subnet_id\x81\x04\x6bcertificate\x40",
        b"\xa4\x64tree\x81\x00\x69signature\x40\x6adelegation\xf6\x6adelegation\xf6",
    ] { assert!(decode(bad).is_err(), "accepted {bad:?}"); }
}

fn tree_frame(tree: &[u8]) -> Vec<u8> {
    let mut frame = b"\xa2\x64tree".to_vec();
    frame.extend_from_slice(tree);
    frame.extend_from_slice(b"\x69signature\x40");
    frame
}

#[test]
fn tree_tags_arities_and_byte_payloads_are_strict() {
    for tree in [
        b"\x81\x00".as_slice(),
        b"\x83\x01\x81\x00\x81\x00",
        b"\x83\x02\x41a\x82\x03\x41b",
        b"\x9f\x00\xff",
    ] {
        assert!(decode(&tree_frame(tree)).is_ok());
    }
    for tree in [
        b"\x80".as_slice(),
        b"\x81\x05",
        b"\x82\x00\x00",
        b"\x82\x01\x81\x00",
        b"\x82\x03\x61a",
        b"\x83\x02\x81\x00\x81\x00",
        b"\x82\x04\x40",
        b"\x9f\x00\x00\xff",
    ] {
        assert!(decode(&tree_frame(tree)).is_err(), "accepted {tree:?}");
    }
    for length in [31_u8, 32, 33] {
        let mut tree = vec![0x82, 4, 0x58, length];
        tree.extend(std::iter::repeat_n(0, usize::from(length)));
        assert_eq!(decode(&tree_frame(&tree)).is_ok(), length == 32);
    }
}

#[test]
fn upstream_recursion_limit_remains_enabled() {
    let mut tree = vec![];
    for _ in 0..160 {
        tree.extend_from_slice(b"\x83\x02\x40");
    }
    tree.extend_from_slice(b"\x81\x00");
    assert!(decode(&tree_frame(&tree)).is_err());
}
