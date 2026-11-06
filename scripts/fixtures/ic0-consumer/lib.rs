//! External IC0-only consumer; native qualification type-checks without executing IC calls.

/// Preserve the safe signatures, including data-before-offset argument order.
pub fn stable_io(pages: u64, offset: u64, written: &[u8], read: &mut [u8]) -> (u64, u64) {
    let before = ic_platform_runtime::ic0::stable64_size();
    let growth = ic_platform_runtime::ic0::stable64_grow(pages);
    ic_platform_runtime::ic0::stable64_write(written, offset);
    ic_platform_runtime::ic0::stable64_read(read, offset);
    (before, growth)
}

/// Fill an uninitialized destination through the same safe IC0-only boundary.
pub fn stable_read_uninit(offset: u64, read: &mut [std::mem::MaybeUninit<u8>]) {
    ic_platform_runtime::ic0::stable64_read_uninit(read, offset);
}
