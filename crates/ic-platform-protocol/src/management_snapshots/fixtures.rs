//! Narrow fixture encoders. These create wire observations, never trusted evidence.
use super::{
    CanisterStatus, CaptureStatus, Snapshot, SnapshotAllocation, SnapshotData, SnapshotMetadata,
};
use crate::candid::encode::{self, EncodeError};

/// Encode a canister status reply, allowing deliberately invalid policy values.
/// # Errors
/// Refuses output beyond the inclusive byte bound or failed output allocation.
pub fn canister_status(
    value: &CanisterStatus,
    maximum_bytes: usize,
) -> Result<Vec<u8>, EncodeError> {
    encode::encode(value, maximum_bytes)
}

/// Encode a capture status reply, allowing deliberately invalid policy values.
/// # Errors
/// Refuses output beyond the inclusive byte bound or failed output allocation.
pub fn capture_status(value: &CaptureStatus, maximum_bytes: usize) -> Result<Vec<u8>, EncodeError> {
    encode::encode(value, maximum_bytes)
}

/// Encode a take snapshot reply, allowing deliberately invalid policy values.
/// # Errors
/// Refuses output beyond the inclusive byte bound or failed output allocation.
pub fn take_snapshot(value: &Snapshot, maximum_bytes: usize) -> Result<Vec<u8>, EncodeError> {
    encode::encode(value, maximum_bytes)
}

/// Encode a list snapshots reply, allowing deliberately invalid policy values.
/// # Errors
/// Refuses output beyond the inclusive byte bound or failed output allocation.
pub fn list_snapshots(value: &Vec<Snapshot>, maximum_bytes: usize) -> Result<Vec<u8>, EncodeError> {
    encode::encode(value, maximum_bytes)
}

/// Encode a snapshot metadata reply, allowing deliberately invalid policy values.
/// # Errors
/// Refuses output beyond the inclusive byte bound or failed output allocation.
pub fn snapshot_metadata(
    value: &SnapshotMetadata,
    maximum_bytes: usize,
) -> Result<Vec<u8>, EncodeError> {
    encode::encode(value, maximum_bytes)
}

/// Encode a snapshot data reply, allowing deliberately invalid policy values.
/// # Errors
/// Refuses output beyond the inclusive byte bound or failed output allocation.
pub fn snapshot_data(value: &SnapshotData, maximum_bytes: usize) -> Result<Vec<u8>, EncodeError> {
    encode::encode(value, maximum_bytes)
}

/// Encode a snapshot allocation reply, allowing deliberately invalid policy values.
/// # Errors
/// Refuses output beyond the inclusive byte bound or failed output allocation.
pub fn snapshot_allocation(
    value: &SnapshotAllocation,
    maximum_bytes: usize,
) -> Result<Vec<u8>, EncodeError> {
    encode::encode(value, maximum_bytes)
}

/// Encode the exact zero-argument acknowledgement, not a one-unit result.
/// # Errors
/// Refuses a bound smaller than the complete six-byte frame or failed allocation.
pub fn empty_response(maximum_bytes: usize) -> Result<Vec<u8>, EncodeError> {
    const BYTES: &[u8] = b"DIDL\0\0";
    if maximum_bytes < BYTES.len() {
        return Err(EncodeError::OutputLimit);
    }
    let mut result = Vec::new();
    result
        .try_reserve_exact(BYTES.len())
        .map_err(|_| EncodeError::Allocation)?;
    result.extend_from_slice(BYTES);
    Ok(result)
}
