#![forbid(unsafe_code)]

//! Literal management frames on the Wasm pointer-width boundary.

use ic_platform_protocol::raw_rand::{Error, decode, encode};

fn main() {
    assert_eq!(encode(6).unwrap(), b"DIDL\0\0");
    assert_eq!(encode(5), Err(Error::ByteLimit));
    let mut frame = [0u8; 42];
    frame[..10].copy_from_slice(b"DIDL\x01\x6d\x7b\x01\x00\x20");
    for seed in [[0u8; 32], [255; 32], std::array::from_fn(|i| i as u8)] {
        frame[10..].copy_from_slice(&seed);
        assert_eq!(decode(std::hint::black_box(&frame), 42), Ok(seed));
        assert_eq!(decode(&frame, 41), Err(Error::ByteLimit));
        for end in 0..42 {
            assert!(decode(&frame[..end], 42).is_err());
        }
        for i in 0..10 {
            let mut wrong = frame;
            wrong[i] ^= 1;
            assert!(decode(&wrong, 42).is_err());
        }
    }
    let mut trailing = frame.to_vec();
    trailing.push(0);
    assert_eq!(decode(&trailing, 43), Err(Error::Length));
}
