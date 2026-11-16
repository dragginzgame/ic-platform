#![forbid(unsafe_code)]

//! Executed Wasm admission corpus, separate from the fixed size probe.

#[cfg(any(
    all(feature = "candidate", feature = "published"),
    not(any(feature = "candidate", feature = "published"))
))]
compile_error!("select exactly one of candidate or published");

#[cfg(feature = "candidate")]
use ic_platform_protocol::{
    candid::{Error, Limits},
    cloud_engine as engine,
};
#[cfg(feature = "published")]
use legacy_platform::{
    candid_codec::decoder::{Error, Limits},
    cloud_engine_types as engine,
};

fn limits() -> Limits {
    Limits {
        bytes: 512 * 1024,
        header: 64 * 1024,
        types: 256,
        work: 2_000_000,
        allocation: 4 * 1024 * 1024,
        #[cfg(feature = "candidate")]
        vector: 1024,
        #[cfg(feature = "candidate")]
        skipping: 128 * 1024,
    }
}

fn decode(bytes: &[u8], cap: Limits) -> Result<Vec<engine::EngineView>, Error> {
    let bytes = std::hint::black_box(bytes);
    #[cfg(feature = "candidate")]
    let result = engine::decode_get_engine_views_response(bytes, cap);
    #[cfg(feature = "published")]
    let result = engine::decode_get_engine_views_response_with_skip_limit(bytes, cap, 128 * 1024);
    std::hint::black_box(result)
}

fn main() {
    // Independently specified vec record {id:text; deleted_at:opt int64;
    // node_ids:vec principal}, followed by one record's id and Some tag.
    let prefix = b"DIDL\x04\x6d\x01\x6c\x03\xdb\xb7\x01\x71\xd9\xe5\xd5\xd6\x0a\x02\xbb\xf8\xfd\xed\x0f\x03\x6e\x74\x6d\x68\x01\x00\x01\x01x\x01";
    for timestamp in [i64::MIN, -1, 0, i64::MAX] {
        let mut bytes = prefix.to_vec();
        bytes.extend_from_slice(&timestamp.to_le_bytes());
        // One anonymous Principal: transparent tag, byte length, principal bytes.
        bytes.extend_from_slice(&[1, 1, 1, 4]);
        let views = decode(&bytes, limits()).unwrap();
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].id, "x");
        assert_eq!(views[0].deleted_at, Some(timestamp));
        assert_eq!(views[0].node_ids.len(), 1);
        assert_eq!(views[0].node_ids[0].as_slice(), &[4]);
        assert!(views[0].subnet.is_none());
        assert!(views[0].subnet_id.is_none());
        assert!(views[0].engine_operator_id.is_none());

        for end in 0..bytes.len() {
            assert_eq!(
                decode(&bytes[..end], limits()).unwrap_err(),
                Error::Malformed
            );
        }
        for budget in 0..5 {
            let mut cap = limits();
            match budget {
                0 => cap.bytes = bytes.len() - 1,
                1 => cap.header = 4,
                2 => cap.types = 3,
                3 => cap.work = 1,
                4 => cap.allocation = 0,
                _ => unreachable!(),
            }
            assert_eq!(decode(&bytes, cap).unwrap_err(), Error::Limit);
        }
        bytes.push(0);
        assert_eq!(decode(&bytes, limits()).unwrap_err(), Error::Trailing);
    }
    assert_eq!(decode(b"DIDL\0\0", limits()).unwrap_err(), Error::Arity);
    // Published value-based admission: empty elements are not decoded.
    assert!(
        decode(b"DIDL\x01\x6d\x7e\x01\0\0", limits())
            .unwrap()
            .is_empty()
    );
}
