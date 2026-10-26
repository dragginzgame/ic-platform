//! Standalone-compatible consumer of the three explicit upstream identities.
use ic_platform::blake3::{Hash, Hasher, hash};

/// Exercise one-shot and streaming hashing in ordinary native/Wasm consumer code.
pub fn hashes(bytes: &[u8]) -> (Hash, Hash) {
    let mut streaming = Hasher::new();
    for chunk in bytes.chunks(63) {
        streaming.update(chunk);
    }
    (hash(bytes), streaming.finalize())
}

#[test]
fn upstream_hash_and_streaming_agree() {
    let empty: Hash = hashes(b"").0;
    assert_eq!(
        empty.to_hex().as_str(),
        "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
    );
    let bytes: Vec<u8> = (0..4097).map(|index| (index % 251) as u8).collect();
    let (one_shot, streaming) = hashes(&bytes);
    assert_eq!(streaming, one_shot);
    assert_eq!(empty.as_bytes().len(), 32);
}
