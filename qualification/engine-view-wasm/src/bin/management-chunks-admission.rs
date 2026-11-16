#![forbid(unsafe_code)]
//! Literal chunk replies and resource ceilings on 32-bit Wasm.
use ic_platform_protocol::{
    candid::{Error, Limits},
    management_chunks as chunks,
};
fn main() {
    let limits = Limits {
        bytes: 65_536,
        header: 4096,
        types: 256,
        work: 131_072,
        allocation: 262_144,
        vector: 65_536,
        skipping: 131_072,
    };
    // vec nat8; record { hash: table[0] }; one record argument.
    let mut reply = b"DIDL\x02\x6d\x7b\x6c\x01\xce\xdf\xa0\xa8\x04\x00\x01\x01\x20".to_vec();
    reply.extend_from_slice(&[255; 32]);
    assert_eq!(
        chunks::decode_upload_chunk(std::hint::black_box(&reply), limits)
            .unwrap()
            .hash,
        [255; 32]
    );
    for end in 0..reply.len() {
        assert!(chunks::decode_upload_chunk(&reply[..end], limits).is_err());
    }
    let mut bounded = limits;
    bounded.vector = 31;
    assert_eq!(
        chunks::decode_upload_chunk(&reply, bounded),
        Err(Error::Limit)
    );
    bounded = limits;
    bounded.allocation = 0;
    assert_eq!(
        chunks::decode_upload_chunk(&reply, bounded),
        Err(Error::Limit)
    );
    reply.push(0);
    assert_eq!(
        chunks::decode_upload_chunk(&reply, limits),
        Err(Error::Trailing)
    );
}
