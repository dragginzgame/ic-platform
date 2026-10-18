//! Compile a safe raw-system caller with only Platform's `ic0` feature selected.
//!
//! Native builds check the signature without executing an IC system call.

/// Forward the caller's exact marker bytes without formatting or an executor.
pub fn emit_marker(marker: &[u8]) {
    ic_platform::ic0::debug_print(marker);
}
