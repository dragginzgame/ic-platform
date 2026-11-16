//! Reviewed wasm32 IC0 imports. No raw ownership pointers cross this boundary.

#[expect(
    unsafe_code,
    reason = "IC0 foreign declaration at the approved IC-specific boundary"
)]
#[link(wasm_import_module = "ic0")]
unsafe extern "C" {
    fn cost_call(method_name_size: u64, payload_size: u64, dst: u32);
    fn canister_liquid_cycle_balance128(dst: u32);
    fn call_new(
        callee_src: u32,
        callee_size: u32,
        name_src: u32,
        name_size: u32,
        reply_fun: u32,
        reply_env: u32,
        reject_fun: u32,
        reject_env: u32,
    );
    fn call_data_append(src: u32, size: u32);
    fn call_cycles_add128(high: u64, low: u64);
    fn call_perform() -> u32;
    fn call_on_cleanup(function: u32, environment: u32);
    fn call_with_best_effort_response(seconds: u32);
    fn msg_reject_code() -> u32;
    fn msg_reject_msg_size() -> u32;
    fn msg_reject_msg_copy(dst: u32, offset: u32, size: u32);
    fn msg_arg_data_size() -> u32;
    fn msg_arg_data_copy(dst: u32, offset: u32, size: u32);
    fn msg_cycles_refunded128(dst: u32);
    fn in_replicated_execution() -> u32;
    fn msg_reply_data_append(src: u32, size: u32);
    fn msg_reply();
    fn msg_caller_size() -> u32;
    fn msg_caller_copy(dst: u32, offset: u32, size: u32);
    fn canister_self_size() -> u32;
    fn canister_self_copy(dst: u32, offset: u32, size: u32);
    fn time() -> u64;
    fn is_controller(src: u32, size: u32) -> u32;
    fn certified_data_set(src: u32, size: u32);
    fn data_certificate_present() -> u32;
    fn data_certificate_size() -> u32;
    fn data_certificate_copy(dst: u32, offset: u32, size: u32);
    fn trap(src: u32, size: u32) -> !;
}

#[expect(
    unsafe_code,
    reason = "IC identity copies use initialized, size-checked Principal storage"
)]
pub(crate) fn principal(caller: bool) -> candidate_types::Principal {
    let mut bytes = [0_u8; 29];
    // SAFETY: Both IC sizes are checked before copying into initialized memory.
    // No user code runs between observation and synchronous copy.
    let size = unsafe {
        if caller {
            msg_caller_size()
        } else {
            canister_self_size()
        }
    };
    assert!(size <= 29, "IC returned an oversized Principal");
    if size != 0 {
        unsafe {
            if caller {
                msg_caller_copy(bytes.as_mut_ptr() as u32, 0, size);
            } else {
                canister_self_copy(bytes.as_mut_ptr() as u32, 0, size);
            }
        }
    }
    candidate_types::Principal::from_slice(&bytes[..size as usize])
}

#[expect(unsafe_code, reason = "IC time observation takes no pointers")]
pub(crate) fn system_time() -> u64 {
    // SAFETY: Scalar-only system observation; context violations trap.
    unsafe { time() }
}

#[expect(unsafe_code, reason = "IC argument-size observation takes no pointers")]
pub(crate) fn argument_size() -> usize {
    // SAFETY: Scalar-only observation; context violations trap.
    unsafe { msg_arg_data_size() as usize }
}

#[expect(
    unsafe_code,
    reason = "IC controller observation synchronously reads a valid Principal"
)]
pub(crate) fn controller(principal: &candidate_types::Principal) -> bool {
    // SAFETY: Valid bounded Principal bytes stay readable throughout the call.
    unsafe { is_controller(principal.as_slice().as_ptr() as u32, principal.len() as u32) != 0 }
}

#[expect(
    unsafe_code,
    reason = "IC certified-data publication synchronously reads a fixed-size root"
)]
pub(crate) fn certified_root(root: &[u8; 32]) {
    // SAFETY: Exactly 32 initialized readable bytes; no pointer is retained.
    unsafe {
        certified_data_set(root.as_ptr() as u32, 32);
    }
}

#[expect(
    unsafe_code,
    reason = "IC certificate reads use initialized bounded owned memory"
)]
pub(crate) fn certificate(
    maximum: usize,
) -> Result<Option<Vec<u8>>, crate::system::CertificateTooLarge> {
    // SAFETY: Check presence before size, and the limit before allocating/copying.
    // The destination remains writable through the synchronous copy.
    unsafe {
        if data_certificate_present() == 0 {
            return Ok(None);
        }
        let actual = data_certificate_size() as usize;
        if actual > maximum {
            return Err(crate::system::CertificateTooLarge { actual, maximum });
        }
        let mut bytes = vec![0; actual];
        if actual != 0 {
            data_certificate_copy(bytes.as_mut_ptr() as u32, 0, actual as u32);
        }
        Ok(Some(bytes))
    }
}

#[expect(
    unsafe_code,
    reason = "IC trap synchronously reads the diagnostic and never returns"
)]
pub(crate) fn trap_message(message: &str) -> ! {
    // SAFETY: wasm32 slice address/length fit u32 and remain readable until trap.
    unsafe { trap(message.as_ptr() as u32, message.len() as u32) }
}

#[expect(
    unsafe_code,
    reason = "IC0 synchronously copies the live reply slice and issues its reply"
)]
pub(crate) fn reply(bytes: &[u8]) {
    // SAFETY: wasm32 pointers/lengths fit u32. The borrowed slice stays readable
    // until the synchronous copy completes and the system retains no pointer.
    // No user code executes between append and reply. System violations trap.
    unsafe {
        if !bytes.is_empty() {
            msg_reply_data_append(bytes.as_ptr() as u32, bytes.len() as u32);
        }
        msg_reply();
    }
}

#[expect(
    unsafe_code,
    reason = "IC0 context observation at the approved import boundary"
)]
pub(crate) fn replicated_execution() -> bool {
    // SAFETY: This context observation takes no pointers and retains no state.
    unsafe { in_replicated_execution() != 0 }
}

#[expect(
    unsafe_code,
    reason = "IC0 argument reads use initialized bounded output memory"
)]
pub(crate) fn read_args(maximum: usize) -> Result<Vec<u8>, crate::system::ArgumentTooLarge> {
    // SAFETY: These imports synchronously observe the current message. No user
    // code runs between size and copy. wasm32 lengths fit the import's u32;
    // the initialized destination stays writable for the entire copy.
    unsafe {
        let actual = msg_arg_data_size() as usize;
        if actual > maximum {
            return Err(crate::system::ArgumentTooLarge { actual, maximum });
        }
        let mut bytes = vec![0; actual];
        if actual != 0 {
            msg_arg_data_copy(bytes.as_mut_ptr() as u32, 0, actual as u32);
        }
        Ok(bytes)
    }
}

#[expect(
    unsafe_code,
    reason = "IC0 synchronously writes a u128 to initialized local Wasm memory"
)]
pub(super) fn liquid_balance() -> u128 {
    let mut output = [0_u8; 16];
    // SAFETY: The writable sixteen-byte local stays live for the synchronous
    // copy; the system retains no pointer. Context violations trap.
    unsafe {
        canister_liquid_cycle_balance128(output.as_mut_ptr() as u32);
    }
    u128::from_le_bytes(output)
}

pub(super) fn enqueue_oneway(request: &super::Call<'_, '_>) -> u32 {
    assemble(request, [u32::MAX; 4], None)
}

pub(super) fn enqueue(
    request: &super::Call<'_, '_>,
    id: u32,
    response: extern "C" fn(u32),
    cleanup: extern "C" fn(u32),
) -> u32 {
    assemble(
        request,
        [response as usize as u32, id, response as usize as u32, id],
        Some(cleanup as usize as u32),
    )
}

#[expect(
    unsafe_code,
    reason = "Reviewed IC0 assembly with static callbacks and integer-only environment handles"
)]
fn assemble(request: &super::Call<'_, '_>, callbacks: [u32; 4], cleanup: Option<u32>) -> u32 {
    let target = request.target();
    let method = request.method().as_bytes();
    let args = request.raw_args();
    // SAFETY: All source slices remain readable for these synchronous copies.
    // wasm32 lengths/pointers fit u32. Callback entries come only from static
    // extern C fn(u32) functions, or the non-existent MAX one-way sentinel.
    // Environments are registry ids, never ownership or borrowed pointers.
    // No user code runs between assembly and perform. System context/limits
    // remain enforced by IC0 traps. Cycles are split high/low without truncation.
    unsafe {
        call_new(
            target.as_slice().as_ptr() as u32,
            target.len() as u32,
            method.as_ptr() as u32,
            method.len() as u32,
            callbacks[0],
            callbacks[1],
            callbacks[2],
            callbacks[3],
        );
        if let Some(cleanup) = cleanup {
            call_on_cleanup(cleanup, callbacks[1]);
            if let super::Wait::Bounded { seconds } = request.wait() {
                call_with_best_effort_response(seconds);
            }
        }
        if !args.is_empty() {
            call_data_append(args.as_ptr() as u32, args.len() as u32);
        }
        let cycles = request.attached_cycles();
        if cycles != 0 {
            call_cycles_add128((cycles >> 64) as u64, cycles as u64);
        }
        call_perform()
    }
}

#[expect(
    unsafe_code,
    reason = "IC0 callback reads synchronously initialize bounded live output buffers"
)]
pub(super) fn capture() -> super::Completion {
    // SAFETY: Invoked only from our registered response callback. All destination
    // buffers are initialized, sized from the matching IC0 size query, and live
    // until the synchronous copies return. There is no caller code/reentrancy
    // between size and copy. Refund reads are excluded in non-replicated query
    // callbacks, where that system observation is unavailable.
    unsafe {
        let refund = if replicated_execution() {
            let mut bytes = [0_u8; 16];
            msg_cycles_refunded128(bytes.as_mut_ptr() as u32);
            Some(u128::from_le_bytes(bytes))
        } else {
            None
        };
        if let Some(code) = std::num::NonZeroU32::new(msg_reject_code()) {
            let size = msg_reject_msg_size();
            let mut bytes = vec![0; size as usize];
            msg_reject_msg_copy(bytes.as_mut_ptr() as u32, 0, size);
            let message = String::from_utf8(bytes).expect("IC rejection text must be UTF-8");
            super::Completion::from_rejection(code, message, refund)
        } else {
            let size = msg_arg_data_size();
            let mut bytes = vec![0; size as usize];
            msg_arg_data_copy(bytes.as_mut_ptr() as u32, 0, size);
            super::Completion::from_reply(bytes, refund)
        }
    }
}

#[expect(
    unsafe_code,
    reason = "IC0 writes exactly sixteen bytes to initialized local Wasm memory"
)]
pub(super) fn cost(method_bytes: usize, argument_bytes: usize) -> u128 {
    // This module is compiled only for wasm32: pointers/lengths fit the ABI.
    // Initialize the complete buffer; never change a Vec's initialized length.
    let mut output = [0_u8; 16];
    // SAFETY: output is writable for sixteen bytes and lives through the import.
    // IC0 synchronously copies the little-endian u128 to dst and retains no
    // pointer. Import-context violations trap rather than return invalid data.
    // No user callback or host reentrancy is permitted by the IC0 cost contract.
    unsafe {
        cost_call(
            method_bytes as u64,
            argument_bytes as u64,
            output.as_mut_ptr() as u32,
        );
    }
    u128::from_le_bytes(output)
}
