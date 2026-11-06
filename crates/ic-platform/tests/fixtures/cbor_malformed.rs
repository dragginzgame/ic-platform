//! Shared malformed-input exercise for deterministic checks and libFuzzer.
use ic_platform::certification::cbor::{self, strict};
use ic_platform::ic_protocol;

pub fn exercise(bytes: &[u8]) {
    let limits = strict::DecodeLimits { bytes: bytes.len() };
    if let Ok(tree) = strict::decode_hash_tree(bytes, limits) {
        let encoded = cbor::encode_hash_tree(&tree, usize::MAX).unwrap();
        assert_eq!(
            tree,
            strict::decode_hash_tree(
                &encoded,
                strict::DecodeLimits {
                    bytes: encoded.len()
                }
            )
            .unwrap()
        );
    }
    if let Ok(certificate) = strict::decode_certificate(bytes, limits) {
        assert_eq!(
            certificate,
            cbor::decode_certificate(bytes, bytes.len()).unwrap()
        );
    }
    let _ = cbor::decode_canister_signature(bytes, bytes.len());
    let _ = ic_protocol::decode_envelope(bytes, bytes.len());
    let _ = ic_protocol::decode_call_content_strict(bytes, bytes.len());
    let _ = ic_protocol::decode_call_response(bytes, bytes.len());
    let _ = ic_protocol::decode_read_state_response(bytes, bytes.len());
    let _ = ic_protocol::decode_subnet_metrics(bytes, bytes.len());
    let _ = ic_protocol::decode_controllers(bytes, bytes.len());
    if let Some(maximum) = bytes.len().checked_sub(1) {
        assert!(matches!(
            strict::decode_hash_tree(bytes, strict::DecodeLimits { bytes: maximum }),
            Err(strict::DecodeError::ByteLimit { .. })
        ));
        assert!(matches!(
            cbor::decode_certificate(bytes, maximum),
            Err(cbor::CborError::InputTooLarge)
        ));
        assert!(matches!(
            ic_protocol::decode_subnet_metrics(bytes, maximum),
            Err(cbor::CborError::InputTooLarge)
        ));
    }
}
