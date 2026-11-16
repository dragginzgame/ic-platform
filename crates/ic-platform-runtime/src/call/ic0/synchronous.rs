//! Synchronous IC system operations for caller-owned Wasm exports.
//!
//! Available only on `wasm32` with `system-raw`. This feature has no dependency,
//! executor, callback registry, heap allocation or initialization requirement. The
//! caller owns exports, authorization, byte protocols and reply sequencing.
//!
//! These are raw operations: the IC enforces execution contexts, source ranges,
//! valid controller identities and one reply per call. Do not mix raw replies
//! with automatic entrypoints or executor-managed reply ownership for one call.
//! Traps roll back the current message, not earlier messages or remote effects.

#[expect(
    unsafe_code,
    reason = "IC system declarations within the approved IC0 boundary"
)]
#[link(wasm_import_module = "ic0")]
unsafe extern "C" {
    #[link_name = "msg_arg_data_size"]
    fn argument_size() -> u32;
    #[link_name = "msg_arg_data_copy"]
    fn argument_copy(dst: u32, offset: u32, size: u32);
    #[link_name = "msg_caller_size"]
    fn caller_size() -> u32;
    #[link_name = "msg_caller_copy"]
    fn caller_copy(dst: u32, offset: u32, size: u32);
    #[link_name = "is_controller"]
    fn controller(src: u32, size: u32) -> u32;
    #[link_name = "msg_reply_data_append"]
    fn append(src: u32, size: u32);
    #[link_name = "msg_reply"]
    fn complete();
    #[link_name = "trap"]
    fn abort(src: u32, size: u32) -> !;
    fn performance_counter(counter_type: u32) -> u64;
    #[link_name = "in_replicated_execution"]
    fn replicated_execution() -> u32;
}

/// Observe the IC's replicated-execution flag without allocating, submitting a
/// call, entering an executor or modifying source custody.
///
/// Ordinary non-replicated queries return false. Updates and replicated queries
/// follow the IC's true observation. This does not authenticate an export kind,
/// prove update reply authority, authorize a caller or make a query an update.
///
/// # Panics
/// IC execution-context restrictions still apply; there is no native fallback.
#[must_use]
#[expect(
    unsafe_code,
    reason = "Scalar IC context observation with no memory pointers"
)]
pub fn in_replicated_execution() -> bool {
    // SAFETY: The scalar import retains no pointer and has no application effects.
    unsafe { replicated_execution() != 0 }
}

/// Returns the current raw message argument length without allocating or copying.
///
/// # Panics
/// The IC traps outside argument-bearing contexts (for example pre-upgrade).
#[must_use]
#[expect(unsafe_code, reason = "Scalar IC observation with no memory pointers")]
pub fn msg_arg_data_size() -> usize {
    // SAFETY: Scalar observation; IC checks the execution context.
    unsafe { argument_size() as usize }
}

/// Copies `destination.len()` argument bytes starting at `offset`.
///
/// The caller supplies initialized writable storage; no allocation occurs.
/// Zero-length copies still invoke the IC and obey its context/range rules.
///
/// # Panics
/// The IC traps if the source range exceeds argument bytes or the context is
/// invalid. No offset arithmetic is truncated: all wasm32 `usize` values fit u32.
#[expect(
    unsafe_code,
    reason = "IC synchronously copies into the caller's valid mutable slice"
)]
pub fn msg_arg_data_copy(destination: &mut [u8], offset: usize) {
    // SAFETY: The destination is initialized and writable for its complete length;
    // the IC validates its own source range. No pointer survives this operation.
    unsafe {
        argument_copy(
            destination.as_mut_ptr() as u32,
            offset as u32,
            destination.len() as u32,
        );
    }
}

/// Returns the current caller's binary Principal length.
///
/// # Panics
/// The IC traps outside caller-bearing contexts. This observation grants no authority.
#[must_use]
#[expect(unsafe_code, reason = "Scalar IC caller-size observation")]
pub fn msg_caller_size() -> usize {
    // SAFETY: Scalar observation; IC checks the execution context.
    unsafe { caller_size() as usize }
}

/// Copies `destination.len()` caller bytes starting at `offset`, without allocation.
///
/// # Panics
/// The IC traps for an invalid context or source range, including invalid offsets
/// on empty copies. The destination is an initialized caller-owned mutable slice.
#[expect(
    unsafe_code,
    reason = "IC caller copy writes only the supplied mutable slice"
)]
pub fn msg_caller_copy(destination: &mut [u8], offset: usize) {
    // SAFETY: Slice pointer/length fit wasm32 and stay writable during the copy.
    unsafe {
        caller_copy(
            destination.as_mut_ptr() as u32,
            offset as u32,
            destination.len() as u32,
        );
    }
}

/// Returns 1 if these binary Principal bytes identify a current controller, else 0.
///
/// Every call observes current IC controller state; no membership is cached.
/// The consumer owns any authorization decision. No Rust Principal type crosses
/// this byte API, so the feature needs no Types dependency.
///
/// # Panics
/// The IC traps if the bytes do not encode a valid Principal.
#[must_use]
#[expect(
    unsafe_code,
    reason = "IC controller check synchronously reads a live byte slice"
)]
pub fn is_controller(principal: &[u8]) -> u32 {
    // SAFETY: Slice is readable throughout the call; IC validates the identity.
    unsafe { controller(principal.as_ptr() as u32, principal.len() as u32) }
}

/// Appends exact bytes to this call's reply buffer, without completing the reply.
///
/// Repeated appends concatenate bytes; an empty slice is a raw empty append.
/// This does not claim executor reply authority or encode a value.
///
/// # Panics
/// The IC traps in non-reply contexts, after reply completion, or on reply limits.
#[expect(unsafe_code, reason = "IC synchronously copies the live reply slice")]
pub fn msg_reply_data_append(bytes: &[u8]) {
    // SAFETY: Readable wasm32 slice; the IC does not retain the pointer.
    unsafe {
        append(bytes.as_ptr() as u32, bytes.len() as u32);
    }
}

/// Completes this call's raw reply, including an empty reply with no appends.
///
/// It does not terminate the handler. The reply is committed only if execution
/// succeeds; a subsequent trap rolls it back with this message's state changes.
///
/// # Panics
/// The IC traps outside reply contexts or if this call already replied/rejected.
#[expect(
    unsafe_code,
    reason = "Scalar IC reply completion, with caller-owned sequencing"
)]
pub fn msg_reply() {
    // SAFETY: No pointers; context and duplicate-reply checks belong to the IC.
    unsafe {
        complete();
    }
}

/// Traps with the supplied diagnostic bytes, rolling back the current message.
///
/// Bytes are forwarded unchanged; no UTF-8 conversion or allocation is performed.
/// This never returns. Earlier committed messages and remote effects remain.
#[expect(
    unsafe_code,
    reason = "IC trap reads a valid diagnostic slice and never returns"
)]
pub fn trap(message: &[u8]) -> ! {
    // SAFETY: Live readable bytes remain valid until the non-returning system call.
    unsafe { abort(message.as_ptr() as u32, message.len() as u32) }
}

/// Observes IC performance counter 0 for the current message execution.
///
/// This deterministic instruction observation is not wall-clock time or a stable
/// cross-version cost guarantee. It introduces no reset, accumulation or state.
#[must_use]
#[expect(
    unsafe_code,
    reason = "Scalar IC instruction observation with fixed supported selector zero"
)]
pub fn instruction_counter() -> u64 {
    // SAFETY: No pointers and a fixed supported counter type.
    unsafe { performance_counter(0) }
}

#[expect(
    unsafe_code,
    reason = "Stable memory and cycle imports at the approved IC0 boundary"
)]
#[link(wasm_import_module = "ic0")]
unsafe extern "C" {
    #[link_name = "stable64_size"]
    fn stable_size() -> u64;
    #[link_name = "stable64_grow"]
    fn stable_grow(pages: u64) -> i64;
    #[link_name = "stable64_read"]
    fn stable_read(dst: u64, offset: u64, size: u64);
    #[link_name = "stable64_write"]
    fn stable_write(offset: u64, src: u64, size: u64);
    fn canister_liquid_cycle_balance128(dst: u32);
    fn cycles_burn128(high: u64, low: u64, dst: u32);
    fn cost_http_request(request_size: u64, maximum_response_size: u64, dst: u32);
}

/// Observes stable memory size in 64 KiB pages; this grants no allocation ownership.
///
/// # Panics
/// IC context violations trap. Stable allocation governance belongs to Storage.
#[must_use]
#[expect(unsafe_code, reason = "Scalar stable-memory observation")]
pub fn stable64_size() -> u64 {
    // SAFETY: No pointers; the IC enforces permitted contexts.
    unsafe { stable_size() }
}

/// Attempts to append zero-filled stable pages, returning the previous page count.
/// `None` preserves the IC growth-failure sentinel. No allocator or retry is added.
///
/// # Panics
/// IC context violations trap; successful changes roll back if this message traps.
#[must_use]
#[expect(
    unsafe_code,
    reason = "Scalar stable-memory growth with explicit failure sentinel"
)]
pub fn stable64_grow(pages: u64) -> Option<u64> {
    // SAFETY: Scalar-only ABI, without truncating the requested 64-bit page count.
    let previous = unsafe { stable_grow(pages) };
    if previous == -1 {
        None
    } else {
        Some(previous as u64)
    }
}

/// Copies stable bytes into initialized caller-owned storage without allocating.
/// The offset remains 64-bit even on wasm32. Empty reads are forwarded to the IC;
/// they access no bytes and do not establish that the offset is in bounds.
///
/// # Panics
/// The IC traps on invalid context, overflow or a range outside stable memory.
#[expect(
    unsafe_code,
    reason = "Synchronous stable read writes only a live initialized slice"
)]
pub fn stable64_read(offset: u64, destination: &mut [u8]) {
    // SAFETY: The initialized slice is writable throughout the synchronous copy.
    // Widening wasm32 pointer/length to u64 is exact; IC validates stable bounds.
    unsafe {
        stable_read(
            destination.as_mut_ptr() as u64,
            offset,
            destination.len() as u64,
        )
    }
}

/// Copies exact caller bytes into stable memory; no pointer or borrow is retained.
/// The offset remains 64-bit. Empty writes are forwarded to the IC and access no
/// bytes; they do not establish that the offset is in bounds.
///
/// # Panics
/// The IC traps on invalid context, overflow or a range outside stable memory.
/// Writes commit only with their enclosing message, including lifecycle messages.
#[expect(
    unsafe_code,
    reason = "Synchronous stable write reads only a live initialized slice"
)]
pub fn stable64_write(offset: u64, source: &[u8]) {
    // SAFETY: Readable slice stays live; IC validates destination offset and length.
    unsafe { stable_write(offset, source.as_ptr() as u64, source.len() as u64) }
}

/// Observes the IC's u128 liquid cycle balance, respecting reservations/freezing.
/// This is an observation, not spending permission or a promised future balance.
///
/// # Panics
/// IC execution-context violations trap.
#[must_use]
#[expect(
    unsafe_code,
    reason = "IC writes one u128 into initialized local storage"
)]
pub fn liquid_cycle_balance() -> u128 {
    let mut bytes = [0; 16];
    // SAFETY: IC synchronously writes exactly sixteen bytes and retains no pointer.
    unsafe { canister_liquid_cycle_balance128(bytes.as_mut_ptr() as u32) }
    u128::from_le_bytes(bytes)
}

/// Requests burning up to `amount`, returning the exact actual u128 amount burned.
/// The IC limits burning by liquid balance and freezing/reservation constraints.
/// Repeated calls spend again; the caller owns spending policy. A later message
/// trap rolls back this burn together with the message's other state changes.
///
/// # Panics
/// Invalid IC execution contexts trap. No automatic funding or retry is performed.
#[must_use]
#[expect(
    unsafe_code,
    reason = "IC burn uses exact high/low scalars and initialized u128 output"
)]
pub fn cycles_burn(amount: u128) -> u128 {
    let mut bytes = [0; 16];
    // SAFETY: Exact u128 split, live writable sixteen-byte output, no retained pointer.
    unsafe {
        cycles_burn128(
            (amount >> 64) as u64,
            amount as u64,
            bytes.as_mut_ptr() as u32,
        )
    }
    u128::from_le_bytes(bytes)
}

/// Quotes IC HTTP outcall cycles for explicitly selected pricing version 1.
/// `request_bytes` is the IC's chargeable request-size measure, not Candid length.
/// This does not submit an HTTP call, attach cycles or select a pricing policy.
///
/// # Panics
/// Invalid IC execution contexts or unsupported system parameters trap.
#[must_use]
#[expect(
    unsafe_code,
    reason = "IC cost query writes a u128 into initialized local storage"
)]
pub fn cost_http_request_v1(request_bytes: u64, maximum_response_bytes: u64) -> u128 {
    let mut bytes = [0; 16];
    // SAFETY: Full-width scalar inputs and a live initialized sixteen-byte output.
    unsafe {
        cost_http_request(
            request_bytes,
            maximum_response_bytes,
            bytes.as_mut_ptr() as u32,
        )
    }
    u128::from_le_bytes(bytes)
}
