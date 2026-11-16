use super::{Error, decode, encode};
#[test]
fn canonical_profile_preserves_all_seed_bytes_and_exact_bounds() {
    assert_eq!(encode(6).unwrap(), b"DIDL\0\0");
    assert_eq!(encode(5), Err(Error::ByteLimit));
    for seed in [[0; 32], [255; 32], std::array::from_fn(|i| i as u8)] {
        let mut frame = b"DIDL\x01\x6d\x7b\x01\x00\x20".to_vec();
        frame.extend_from_slice(&seed);
        assert_eq!(decode(&frame, 42), Ok(seed));
        assert_eq!(decode(&frame, 41), Err(Error::ByteLimit));
        for end in 0..frame.len() {
            assert!(decode(&frame[..end], 42).is_err());
        }
        for i in 0..10 {
            let mut wrong = frame.clone();
            wrong[i] ^= 1;
            assert!(decode(&wrong, 42).is_err());
        }
        frame.push(0);
        assert_eq!(decode(&frame, 43), Err(Error::Length));
    }
}
