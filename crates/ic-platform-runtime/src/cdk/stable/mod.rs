//! Stable-memory passthroughs over the registry IC0 supplier.
//!
//! IC0 owns bounds and traps; allocation, storage layout and reopening policy
//! remain with consumers.

use crate::ic0::upstream;

/// A failed request to grow stable memory.
#[derive(Debug, thiserror::Error)]
pub enum StableMemoryError {
    /// IC0 refused the growth request.
    #[error("Out of memory")]
    OutOfMemory,
}

fn growth_result(previous_pages: u64) -> Result<u64, StableMemoryError> {
    if previous_pages == u64::MAX {
        Err(StableMemoryError::OutOfMemory)
    } else {
        Ok(previous_pages)
    }
}

/// Grows stable memory by `new_pages`, returning its previous page count.
///
/// The IC0 failure sentinel becomes [`StableMemoryError::OutOfMemory`].
pub fn stable_grow(new_pages: u64) -> Result<u64, StableMemoryError> {
    growth_result(upstream::stable64_grow(new_pages))
}

/// Writes bytes at the 64-bit stable-memory offset, preserving IC0 bounds traps.
pub fn stable_write(offset: u64, buf: &[u8]) {
    upstream::stable64_write(buf, offset);
}

/// Reads bytes at the 64-bit stable-memory offset, preserving IC0 bounds traps.
pub fn stable_read(offset: u64, buf: &mut [u8]) {
    upstream::stable64_read(buf, offset);
}

#[cfg(test)]
mod tests;
