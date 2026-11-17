#![forbid(unsafe_code)]
//! Replay independent management metadata and refusals on 32-bit Wasm.
use ic_platform_protocol::{
    candid::{Error, Limits},
    management_snapshots as wire,
};
fn main() {
    let limits = Limits {
        bytes: 2 * 1024 * 1024,
        header: 65536,
        types: 64,
        work: 8 * 1024 * 1024,
        allocation: 2 * 1024 * 1024,
        vector: 1024 * 1024,
        skipping: 0,
    };
    let bytes = include_bytes!("../../fixtures/snapshot-metadata.bin");
    let value = wire::decode_snapshot_metadata(std::hint::black_box(bytes), limits).unwrap();
    assert_eq!(
        value.globals[2],
        Some(wire::Global::F32(wire::Float32Bits(0xffc12345)))
    );
    assert_eq!(
        value.globals[3],
        Some(wire::Global::F64(wire::Float64Bits(0xfff8123456789abc)))
    );
    assert_eq!(value.globals[4], Some(wire::Global::V128(u128::MAX)));
    assert_eq!(value.globals[5], None);
    assert_eq!(
        value.source,
        Some(wire::SnapshotSource::TakenFromCanister(wire::Reserved))
    );
    let encoded = wire::fixtures::snapshot_metadata(&value, limits.bytes).unwrap();
    assert_eq!(
        wire::decode_snapshot_metadata(&encoded, limits).unwrap(),
        value
    );
    assert_eq!(
        wire::fixtures::snapshot_metadata(&value, encoded.len()).unwrap(),
        encoded
    );
    assert_eq!(
        wire::fixtures::snapshot_metadata(&value, encoded.len() - 1),
        Err(wire::EncodeError::OutputLimit)
    );
    for bytes in [
        include_bytes!("../../fixtures/backup-status-stopped.bin"),
        include_bytes!("../../fixtures/backup-status-stopping.bin"),
        include_bytes!("../../fixtures/backup-status-running.bin"),
    ] {
        let status = wire::decode_capture_status(bytes, limits).unwrap();
        assert_eq!(status.settings.controllers.len(), 2);
        assert_eq!(
            wire::decode_canister_status(bytes, limits),
            Err(Error::Type)
        );
        let encoded = wire::fixtures::capture_status(&status, 1024).unwrap();
        assert_eq!(
            wire::decode_capture_status(&encoded, limits).unwrap(),
            status
        );
    }
    for end in 0..bytes.len() {
        assert!(wire::decode_snapshot_metadata(&bytes[..end], limits).is_err());
    }
    for select in 0..6 {
        let mut bound = limits;
        match select {
            0 => bound.bytes = bytes.len() - 1,
            1 => bound.header = 0,
            2 => bound.types = 0,
            3 => bound.work = 0,
            4 => bound.allocation = 0,
            _ => bound.vector = 0,
        }
        assert_eq!(
            wire::decode_snapshot_metadata(bytes, bound),
            Err(Error::Limit)
        );
    }
    let mut trailing = bytes.to_vec();
    trailing.push(0);
    assert_eq!(
        wire::decode_snapshot_metadata(&trailing, limits),
        Err(Error::Trailing)
    );
    assert_eq!(wire::decode_empty_response(b"DIDL\x00\x00", limits), Ok(()));
    assert_eq!(
        wire::decode_empty_response(b"DIDL\x00\x01\x7f", limits),
        Err(Error::Arity)
    );
}
