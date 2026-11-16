//! Independent upstream management schemas and hostile input/budget checks.
#![forbid(unsafe_code)]
#![cfg(all(feature = "management-chunks", not(target_family = "wasm")))]
use candidate_types::Principal;
use ic_management_canister_types::{CanisterIdRecord, ChunkHash, UploadChunkArgs};
use ic_platform_protocol::{
    candid::{Error, Limits},
    management_chunks as chunks,
};

fn limits() -> Limits {
    Limits {
        bytes: 65_536,
        header: 4096,
        types: 256,
        work: 131_072,
        allocation: 262_144,
        vector: 65_536,
        skipping: 131_072,
    }
}

#[test]
fn exact_management_requests_and_preallocation_limits() {
    for id in [vec![], vec![4], vec![255; 29]] {
        let principal = Principal::from_slice(&id);
        let bytes = chunks::encode_stored_chunks(principal, 1024).unwrap();
        let parsed: CanisterIdRecord = candid::decode_one(&bytes).unwrap();
        assert_eq!(parsed.canister_id.as_slice(), id);
        assert_eq!(
            chunks::encode_stored_chunks(principal, bytes.len()).unwrap(),
            bytes
        );
        assert!(chunks::encode_stored_chunks(principal, bytes.len() - 1).is_err());
        for length in [0, 1, 127, 128, 16_383, 16_384, 1_048_576] {
            let chunk: Vec<u8> = (0..length).map(|i| i as u8).collect();
            let bytes = chunks::encode_upload_chunk(principal, &chunk, length + 128).unwrap();
            let parsed: UploadChunkArgs = candid::decode_one(&bytes).unwrap();
            assert_eq!(parsed.canister_id.as_slice(), id);
            assert_eq!(parsed.chunk, chunk);
            assert_eq!(
                chunks::encode_upload_chunk(principal, &chunk, bytes.len()).unwrap(),
                bytes
            );
            assert!(chunks::encode_upload_chunk(principal, &chunk, bytes.len() - 1).is_err());
        }
    }
}

#[test]
fn external_replies_and_refusals() {
    for count in [0, 1, 127, 128] {
        let hashes: Vec<ChunkHash> = (0..count)
            .map(|i| ChunkHash {
                hash: vec![i as u8; 32],
            })
            .collect();
        let bytes = candid::encode_one(&hashes).unwrap();
        let decoded = chunks::decode_stored_chunks(&bytes, limits()).unwrap();
        assert_eq!(
            decoded.iter().map(|v| &v.hash).collect::<Vec<_>>(),
            hashes.iter().map(|v| &v.hash).collect::<Vec<_>>()
        );
        for end in 0..bytes.len() {
            assert!(chunks::decode_stored_chunks(&bytes[..end], limits()).is_err());
        }
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert_eq!(
            chunks::decode_stored_chunks(&trailing, limits()),
            Err(Error::Trailing)
        );
        for select in 0..6 {
            let mut restricted = limits();
            match select {
                0 => restricted.bytes = bytes.len() - 1,
                1 => restricted.header = 0,
                2 => restricted.types = 0,
                3 => restricted.work = 0,
                4 => restricted.allocation = 0,
                _ => restricted.vector = 0,
            }
            if count != 0 || select != 5 {
                assert_eq!(
                    chunks::decode_stored_chunks(&bytes, restricted),
                    Err(Error::Limit)
                );
            }
        }
    }
    for hash in [vec![], vec![0; 32], vec![255; 32]] {
        let bytes = candid::encode_one(ChunkHash { hash: hash.clone() }).unwrap();
        assert_eq!(
            chunks::decode_upload_chunk(&bytes, limits()).unwrap().hash,
            hash
        );
        for end in 0..bytes.len() {
            assert!(chunks::decode_upload_chunk(&bytes[..end], limits()).is_err());
        }
    }
    let wrong = candid::encode_one(vec![0u8; 32]).unwrap();
    assert!(chunks::decode_upload_chunk(&wrong, limits()).is_err());
    assert!(chunks::decode_stored_chunks(&wrong, limits()).is_err());
    let surplus = candid::encode_args((ChunkHash { hash: vec![1; 32] }, 0u8)).unwrap();
    assert_eq!(
        chunks::decode_upload_chunk(&surplus, limits()),
        Err(Error::Arity)
    );
}

#[test]
fn skipped_fields_keep_shared_work_and_allocation_bounds() {
    #[derive(candid::CandidType)]
    struct Extended {
        hash: Vec<u8>,
        unknown: Vec<String>,
    }
    let bytes = candid::encode_one(Extended {
        hash: vec![7; 32],
        unknown: vec!["kept opaque".into(); 50],
    })
    .unwrap();
    assert_eq!(
        chunks::decode_upload_chunk(&bytes, limits()).unwrap().hash,
        [7; 32]
    );
    let mut restricted = limits();
    restricted.skipping = 0;
    assert_eq!(
        chunks::decode_upload_chunk(&bytes, restricted),
        Err(Error::Limit)
    );
    restricted = limits();
    restricted.vector = 49;
    assert_eq!(
        chunks::decode_upload_chunk(&bytes, restricted),
        Err(Error::Limit)
    );
    for end in 0..bytes.len() {
        assert!(chunks::decode_upload_chunk(&bytes[..end], limits()).is_err());
    }
}
