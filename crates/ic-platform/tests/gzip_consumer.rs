//! Native gzip facade contract without unrelated Platform features.
#![cfg(not(target_family = "wasm"))]

use ic_platform::gzip::{GzipError, decode_gzip, encode_gzip};

#[test]
fn bounded_bundle_codec_is_available_without_other_platform_features() {
    let input = b"profile bundle profile bundle profile bundle";
    let mut compressed = Vec::new();
    encode_gzip(input, &mut compressed, 1, 1024).unwrap();
    assert_eq!(&compressed[..4], &[0x1f, 0x8b, 8, 0]);
    assert_eq!(&compressed[4..8], &[0; 4]);
    assert_eq!(decode_gzip(&compressed, 1024, input.len()).unwrap(), input);
    assert!(matches!(
        decode_gzip(&compressed, 0, input.len()),
        Err(GzipError::InputLimit { .. })
    ));
    assert!(decode_gzip(&compressed, 1024, input.len() - 1).is_err());
    assert!(encode_gzip(input, Vec::new(), 1, 1).is_err());
    compressed.push(0);
    assert!(matches!(
        decode_gzip(&compressed, 1024, input.len()),
        Err(GzipError::TrailingData)
    ));
}
