//! Compile a safe raw-system caller with only Platform's `ic0` feature selected.
//!
//! Native builds check the signature without executing an IC system call.

/// Forward the caller's exact marker bytes without formatting or an executor.
pub fn emit_marker(marker: &[u8]) {
    ic_platform::ic0::debug_print(marker);
}

/// Preserve the upstream replicated/query flag without normalizing its integer type.
pub fn replicated_execution() -> u32 {
    ic_platform::ic0::in_replicated_execution()
}

/// Pass the caller's existing Principal bytes to the upstream controller check.
pub fn controller(principal: &[u8]) -> u32 {
    ic_platform::ic0::is_controller(principal)
}

/// Append the caller's exact byte slices in order, then finalize the reply.
pub fn reply_in_parts(first: &[u8], second: &[u8]) {
    ic_platform::ic0::msg_reply_data_append(first);
    ic_platform::ic0::msg_reply_data_append(second);
    ic_platform::ic0::msg_reply();
}

/// Forward existing rejection bytes without formatting or retry policy.
pub fn reject(message: &[u8]) {
    ic_platform::ic0::msg_reject(message);
}
