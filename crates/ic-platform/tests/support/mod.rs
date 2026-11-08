//! Test-only decoding of retained, independently captured hexadecimal bytes.

/// Decode complete pairs, refusing malformed fixture data.
pub fn decode_hex(text: &str) -> Vec<u8> {
    let (pairs, remainder) = text.trim().as_bytes().as_chunks::<2>();
    assert!(remainder.is_empty(), "fixture hex must have complete bytes");
    pairs
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
