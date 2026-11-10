# System API and stable adapter replacement contract

Input for a separate specification-only context. Do not read existing adapters,
old tests, supplier implementations, history or diffs. Write new
`crates/ic-platform-runtime/src/cdk/api/mod.rs`, `api/tests.rs`,
`stable/mod.rs`, `stable/tests.rs`, and a short
`docs/design/system-adapters-implementation.md` input/validation record. Parent
will replace the previous private owner and retain callback machinery separately.
No new public API, dependency, feature, limits, storage manager or crypto logic.

## Supplier boundary and public functions

All IC operations delegate to `crate::ic0::upstream`, the registry IC0 1.2.0
owner. Canonical identity is `crate::Principal`, supporting MAX_LENGTH_IN_BYTES=29,
from_slice(&[u8]) and as_slice(). Use ordinary modules. Explicit re-exports of
individual IC0 functions are permitted when their signatures exactly match the
public surface below; no blanket exports. Existing unused IC0 functions stay hidden.

Public api module:

| Public function | Supplier operation / behavior |
|---|---|
| msg_arg_data() -> Vec<u8> | msg_arg_data_size() -> usize and msg_arg_data_copy_uninit(&mut [MaybeUninit<u8>], usize) |
| msg_caller() -> Principal | msg_caller_size() -> usize, msg_caller_copy(&mut [u8], usize) |
| canister_self() -> Principal | canister_self_size(), canister_self_copy, same signatures |
| subnet_self() -> Principal | subnet_self_size(), subnet_self_copy, same signatures |
| msg_reject_code() -> u32 | same-named IC0 function |
| msg_reject_msg() -> String | msg_reject_msg_size/copy_uninit; strict UTF-8; invalid panic reason `reject message is not valid UTF-8` |
| msg_method_name() -> String | msg_method_name_size/copy_uninit; strict UTF-8; invalid panic reason `msg_method_name is not valid UTF-8` |
| msg_reply<T: AsRef<[u8]>>(data: T) | append nonempty bytes with msg_reply_data_append(&[u8]), then call msg_reply(); skip append for empty bytes |
| msg_reject<T: AsRef<str>>(message: T) | msg_reject(message.as_ref().as_bytes()) |
| msg_cycles_available() -> u128 | msg_cycles_available128() |
| msg_cycles_refunded() -> u128 | msg_cycles_refunded128() |
| msg_cycles_accept(max_amount: u128) -> u128 | msg_cycles_accept128(max_amount) |
| cycles_burn(amount: u128) -> u128 | cycles_burn128(amount) |
| canister_cycle_balance() -> u128 | canister_cycle_balance128() |
| canister_liquid_cycle_balance() -> u128 | canister_liquid_cycle_balance128() |
| canister_version() -> u64 | same-named function |
| accept_message() | same-named function |
| stable_size() -> u64 | stable64_size() |
| root_key() -> Vec<u8> | root_key_size/copy_uninit; opaque bytes, no parse/crypto |
| certified_data_set<T: AsRef<[u8]>>(data: T) | certified_data_set(&[u8]); leave size/context traps upstream |
| data_certificate() -> Option<Vec<u8>> | data_certificate_present() -> u32 equal zero means None without size/copy; otherwise data_certificate_size/copy_uninit supplies Some, including empty bytes |
| time() -> u64 | same-named function |
| global_timer_set(timestamp: u64) -> u64 | same-named function; no scheduler policy |
| performance_counter(counter_type: impl Into<PerformanceCounterType>) -> u64 | performance_counter(u32) after conversion below |
| instruction_counter() -> u64 | performance_counter(0) |
| call_context_instruction_counter() -> u64 | performance_counter(1) |
| is_controller(principal: &Principal) -> bool | is_controller(principal.as_slice()) -> u32: 0=false, 1=true, all others panic with `unexpected return value from is_controller: {n}` |
| in_replicated_execution() -> bool | same-named supplier -> u32, 0/1 mapping; other values panic `unexpected return value from in_replicated_execution: {n}` |
| cost_call(method_name_size: u64, payload_size: u64) -> u128 | same-named function |
| cost_create_canister() -> u128 | same-named function |
| cost_http_request(request_size: u64, max_res_bytes: u64) -> u128 | same-named function |
| cost_sign_with_ecdsa<T: AsRef<str>>(key_name: T, ecdsa_curve: u32) -> Result<u128, SignCostError> | cost_sign_with_ecdsa(&str,u32) -> (u128,u32); result code below |
| debug_print<T: AsRef<str>>(data: T) | debug_print(data.as_ref().as_bytes()) |
| trap<T: AsRef<str>>(data: T) -> ! | trap(data.as_ref().as_bytes()) |

PerformanceCounterType retains repr(u32), Debug/PartialEq/Eq/Clone/Copy, with
InstructionCounter, CallContextInstructionCounter, Unrecognized(u32).
From<u32>: 0->InstructionCounter, 1->CallContextInstructionCounter, others retain
payload. From<PerformanceCounterType> for u32 is the reverse numeric projection,
including manually constructed Unrecognized(0/1). PartialEq<u32> compares numeric
projection; enum-to-enum PartialEq still distinguishes variants. No reverse
PartialEq<PerformanceCounterType> for u32 or extra traits needed.

SignCostError retains Debug/Clone/std::error::Error/Display, variants:
InvalidCurveOrAlgorithm (`invalid curve or algorithm`), InvalidKeyName
(`invalid key name`), UnrecognizedError(u32) (`unrecognized error: {0}`). Cost
result code 0 returns full u128; 1/2 map those named errors; all others retain
full u32. Source is None. Existing thiserror is available; add no dependency.

## Copy and text invariants

All identity reads use a bounded 29-byte stack buffer; supplier lengths above
29 must panic before copy, never truncate or allocate. Offset is always zero.
All five Vec-returning readers (arguments, reject text, method text, root key,
certificate) use the supplier's all-or-trap uninitialized-byte copy, avoiding
zero-fill. Supplier guarantee: the exact supplied slice is fully initialized
or execution traps. Only publish vector length after successful copy. Keep a
small private helper; unsafe is allowed only for the reviewed set_len boundary,
with a local expect(unsafe_code) reason and safety comment. The rest stays safe.
Do not introduce unsafe allocator instrumentation. Use strict UTF-8 validation,
without including rejected-byte Debug dumps in panic messages. Keep size/copy
ordering and zero-length behavior; callers retain allocation/budget admission.

## Public stable module

StableMemoryError: Debug/std::error::Error/Display, sole variant OutOfMemory,
display `Out of memory`, source None. No Eq/Clone/new variants required.
`stable_grow(new_pages: u64) -> Result<u64, StableMemoryError>` delegates to
stable64_grow(u64)->u64: u64::MAX is Err; every other value is Ok(previous_pages).
`stable_write(offset: u64, buf: &[u8])` delegates stable64_write(buf, offset).
`stable_read(offset: u64, buf: &mut [u8])` delegates stable64_read(buf, offset).
No added preflight bounds, 32-bit truncation, caps or allocation. IC0 owns traps.

## Tests and custody

Write fresh unit tests from this contract only: private byte helper initialized
contents/zero length/copy failure; valid/invalid UTF-8; bounded Principal reads;
performance counter exact variants/projections/u32 limits; error formatting and
u128 cost preservation; growth sentinel mapping. Small private pure helpers may
make these tests meaningful; avoid a full fake IC0 layer or testing the supplier.
Parent supplies real installed IC qualification and native/Wasm Clippy. No Cargo
or excluded-source inspection. Format only authored files with skip_children.
Record exact allowed inputs and limitations; no legal-clearance claim.
