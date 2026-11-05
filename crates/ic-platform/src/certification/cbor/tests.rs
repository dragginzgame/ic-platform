use super::*;
use crate::certification::{Delegation, labeled, leaf};

fn certificate() -> Certificate {
    Certificate {
        tree: labeled(b"time", leaf(vec![1, 2, 3])),
        signature: vec![4; 48],
        // Certificate/delegation blobs must never be normalized by the codec.
        delegation: Some(Delegation {
            subnet_id: vec![5, 6],
            certificate: vec![0xd9, 0xd9, 0xf7, 0xa0],
        }),
    }
}

fn upstream_cbor(value: &impl Serialize) -> Vec<u8> {
    let mut serializer = serde_cbor::Serializer::new(Vec::new());
    serializer.self_describe().unwrap();
    value.serialize(&mut serializer).unwrap();
    serializer.into_inner()
}

#[test]
fn certificate_preserves_upstream_bytes_at_exact_budget() {
    let cert = certificate();
    let expected = upstream_cbor(&cert);
    assert_eq!(encode_certificate(&cert, expected.len()).unwrap(), expected);
    let decoded = decode_certificate(&expected, expected.len()).unwrap();
    assert_eq!(decoded.tree, cert.tree);
    assert_eq!(decoded.signature, cert.signature);
    assert_eq!(decoded.delegation, cert.delegation);
    assert!(matches!(
        encode_certificate(&cert, expected.len() - 1),
        Err(CborError::OutputTooLarge)
    ));
    assert!(matches!(
        decode_certificate(&expected, expected.len() - 1),
        Err(CborError::InputTooLarge)
    ));
}

#[test]
fn signature_preserves_standard_framing_certificate_and_witness() {
    #[derive(Serialize)]
    struct Envelope<'a> {
        #[serde(with = "serde_bytes")]
        certificate: &'a [u8],
        tree: &'a HashTree,
    }
    let cert = encode_certificate(&certificate(), 1024).unwrap();
    let tree = labeled(b"sig", leaf(Vec::new()));
    let value = Envelope {
        certificate: &cert,
        tree: &tree,
    };
    let expected = upstream_cbor(&value);
    let signature = encode_canister_signature(&cert, &tree, expected.len()).unwrap();
    assert_eq!(signature, expected);
    assert_eq!(&signature[..3], &[0xd9, 0xd9, 0xf7]);
    for encoded in [
        signature.clone(),
        serde_cbor::ser::to_vec_packed(&value).unwrap(),
        upstream_cbor(&(serde_bytes::Bytes::new(&cert), &tree)),
    ] {
        let decoded = decode_canister_signature(&encoded, encoded.len()).unwrap();
        assert_eq!(decoded.certificate, cert);
        assert_eq!(decoded.tree, tree);
    }
    assert!(matches!(
        encode_canister_signature(&cert, &tree, signature.len() - 1),
        Err(CborError::OutputTooLarge)
    ));
    assert!(matches!(
        decode_canister_signature(&signature, signature.len() - 1),
        Err(CborError::InputTooLarge)
    ));
}

#[test]
fn malformed_trailing_and_zero_budget_inputs_fail_closed() {
    for invalid in [
        vec![],
        vec![0xff],
        vec![0xa0],
        vec![0x80],
        vec![0xd9, 0xd9, 0xf7, 0xff],
    ] {
        assert!(matches!(
            decode_certificate(&invalid, 1024),
            Err(CborError::Decode(_))
        ));
        assert!(matches!(
            decode_canister_signature(&invalid, 1024),
            Err(CborError::Decode(_))
        ));
    }
    // Length admission takes precedence over malformed CBOR admission.
    assert!(matches!(
        decode_certificate(&[0xff], 0),
        Err(CborError::InputTooLarge)
    ));
    assert!(matches!(
        decode_canister_signature(&[0xff], 0),
        Err(CborError::InputTooLarge)
    ));
    let cert = certificate();
    let mut bytes = encode_certificate(&cert, 1024).unwrap();
    bytes.push(0);
    assert!(matches!(
        decode_certificate(&bytes, 1024),
        Err(CborError::Decode(_))
    ));
    let mut signature = encode_canister_signature(&bytes, &cert.tree, 2048).unwrap();
    signature.push(0);
    assert!(matches!(
        decode_canister_signature(&signature, 2048),
        Err(CborError::Decode(_))
    ));
    assert!(matches!(
        encode_certificate(&cert, 0),
        Err(CborError::OutputTooLarge)
    ));
    assert!(matches!(
        encode_canister_signature(&bytes, &cert.tree, 0),
        Err(CborError::OutputTooLarge)
    ));
}

#[test]
fn upstream_recursion_admission_is_retained() {
    let mut cert = certificate();
    for _ in 0..160 {
        cert.tree = labeled(b"nested", cert.tree);
    }
    let bytes = encode_certificate(&cert, 16 * 1024).unwrap();
    assert!(matches!(
        decode_certificate(&bytes, bytes.len()),
        Err(CborError::Decode(_))
    ));
    let signature = encode_canister_signature(&bytes, &cert.tree, 32 * 1024).unwrap();
    assert!(matches!(
        decode_canister_signature(&signature, signature.len()),
        Err(CborError::Decode(_))
    ));
}

#[test]
fn output_refuses_before_growing_past_the_host_budget() {
    let mut output = Output {
        bytes: Vec::new(),
        maximum: 17,
    };
    output.write_all(b"abc").unwrap();
    output.write_all(&[7; 14]).unwrap();
    assert_eq!(output.bytes.len(), 17);
    assert!(output.bytes.capacity() <= 17);
    let before = output.bytes.clone();
    let capacity = output.bytes.capacity();
    assert_eq!(
        output.write_all(&[0; 4096]).unwrap_err().kind(),
        io::ErrorKind::WriteZero
    );
    assert_eq!(output.bytes, before);
    assert_eq!(output.bytes.capacity(), capacity);
    output.write_all(&[]).unwrap();
}
