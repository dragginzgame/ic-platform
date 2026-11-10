//! Explicit System API adapters over the registry IC0 supplier.
//!
//! These functions preserve IC0 effects and traps; callers own budgets, trust,
//! scheduling and application policy.

use crate::{Principal, ic0::upstream};
use std::mem::MaybeUninit;

pub use crate::ic0::upstream::{
    accept_message, canister_cycle_balance128 as canister_cycle_balance,
    canister_liquid_cycle_balance128 as canister_liquid_cycle_balance, canister_version, cost_call,
    cost_create_canister, cost_http_request, cycles_burn128 as cycles_burn, global_timer_set,
    msg_cycles_accept128 as msg_cycles_accept, msg_cycles_available128 as msg_cycles_available,
    msg_cycles_refunded128 as msg_cycles_refunded, msg_reject_code, stable64_size as stable_size,
    time,
};

fn system_bytes(size: usize, copy: impl FnOnce(&mut [MaybeUninit<u8>], usize)) -> Vec<u8> {
    let mut bytes = Vec::<u8>::with_capacity(size);
    copy(&mut bytes.spare_capacity_mut()[..size], 0);
    // SAFETY: The selected IC0 copy initializes the entire supplied slice or
    // traps. Length is published only after that copy returns successfully.
    #[expect(
        unsafe_code,
        reason = "IC0's all-or-trap copy initializes this exact spare-capacity slice"
    )]
    unsafe {
        bytes.set_len(size);
    }
    bytes
}

fn system_principal(size: usize, copy: impl FnOnce(&mut [u8], usize)) -> Principal {
    assert!(
        size <= Principal::MAX_LENGTH_IN_BYTES,
        "System API Principal exceeds 29 bytes"
    );
    let mut bytes = [0_u8; Principal::MAX_LENGTH_IN_BYTES];
    copy(&mut bytes[..size], 0);
    Principal::from_slice(&bytes[..size])
}

fn system_text(bytes: Vec<u8>, reason: &str) -> String {
    String::from_utf8(bytes).unwrap_or_else(|_| panic!("{reason}"))
}

fn system_bool(value: u32, operation: &str) -> bool {
    match value {
        0 => false,
        1 => true,
        _ => panic!("unexpected return value from {operation}: {value}"),
    }
}

/// Returns the message's argument bytes without decoding or imposing a budget.
pub fn msg_arg_data() -> Vec<u8> {
    system_bytes(
        upstream::msg_arg_data_size(),
        upstream::msg_arg_data_copy_uninit,
    )
}

/// Returns the current caller's Principal.
///
/// # Panics
/// Panics if the supplier reports more than 29 Principal bytes.
pub fn msg_caller() -> Principal {
    system_principal(upstream::msg_caller_size(), upstream::msg_caller_copy)
}

/// Returns this canister's Principal.
///
/// # Panics
/// Panics if the supplier reports more than 29 Principal bytes.
pub fn canister_self() -> Principal {
    system_principal(upstream::canister_self_size(), upstream::canister_self_copy)
}

/// Returns this subnet's Principal.
///
/// # Panics
/// Panics if the supplier reports more than 29 Principal bytes.
pub fn subnet_self() -> Principal {
    system_principal(upstream::subnet_self_size(), upstream::subnet_self_copy)
}

/// Returns the rejection message as strict UTF-8.
///
/// # Panics
/// Panics with `reject message is not valid UTF-8` for invalid text.
pub fn msg_reject_msg() -> String {
    system_text(
        system_bytes(
            upstream::msg_reject_msg_size(),
            upstream::msg_reject_msg_copy_uninit,
        ),
        "reject message is not valid UTF-8",
    )
}

/// Returns the message's method name as strict UTF-8.
///
/// # Panics
/// Panics with `msg_method_name is not valid UTF-8` for invalid text.
pub fn msg_method_name() -> String {
    system_text(
        system_bytes(
            upstream::msg_method_name_size(),
            upstream::msg_method_name_copy_uninit,
        ),
        "msg_method_name is not valid UTF-8",
    )
}

/// Appends nonempty reply bytes and completes the reply.
pub fn msg_reply<T: AsRef<[u8]>>(data: T) {
    let bytes = data.as_ref();
    if !bytes.is_empty() {
        upstream::msg_reply_data_append(bytes);
    }
    upstream::msg_reply();
}

/// Rejects the message using the supplied UTF-8 text bytes.
pub fn msg_reject<T: AsRef<str>>(message: T) {
    upstream::msg_reject(message.as_ref().as_bytes());
}

/// Returns the opaque root-key bytes; no cryptographic parsing is performed.
pub fn root_key() -> Vec<u8> {
    system_bytes(upstream::root_key_size(), upstream::root_key_copy_uninit)
}

/// Sets certified data, leaving size and execution-context traps to IC0.
pub fn certified_data_set<T: AsRef<[u8]>>(data: T) {
    upstream::certified_data_set(data.as_ref());
}

fn certificate_bytes(
    present: u32,
    size: impl FnOnce() -> usize,
    copy: impl FnOnce(&mut [MaybeUninit<u8>], usize),
) -> Option<Vec<u8>> {
    if present == 0 {
        None
    } else {
        Some(system_bytes(size(), copy))
    }
}

/// Returns an available data certificate, including an empty present certificate.
///
/// Absence performs no size query or copy. Certificate interpretation is
/// consumer-owned.
pub fn data_certificate() -> Option<Vec<u8>> {
    certificate_bytes(
        upstream::data_certificate_present(),
        upstream::data_certificate_size,
        upstream::data_certificate_copy_uninit,
    )
}

/// Selects an IC performance counter while preserving unknown numeric values.
#[repr(u32)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PerformanceCounterType {
    /// Instructions executed in the current message.
    InstructionCounter = 0,
    /// Instructions executed in the current call context.
    CallContextInstructionCounter = 1,
    /// A counter number not named by this adapter.
    Unrecognized(u32),
}

impl From<u32> for PerformanceCounterType {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::InstructionCounter,
            1 => Self::CallContextInstructionCounter,
            _ => Self::Unrecognized(value),
        }
    }
}

impl From<PerformanceCounterType> for u32 {
    fn from(counter: PerformanceCounterType) -> Self {
        match counter {
            PerformanceCounterType::InstructionCounter => 0,
            PerformanceCounterType::CallContextInstructionCounter => 1,
            PerformanceCounterType::Unrecognized(value) => value,
        }
    }
}

impl PartialEq<u32> for PerformanceCounterType {
    fn eq(&self, value: &u32) -> bool {
        u32::from(*self) == *value
    }
}

/// Reads the selected performance counter, forwarding its full numeric value.
#[inline]
pub fn performance_counter(counter_type: impl Into<PerformanceCounterType>) -> u64 {
    upstream::performance_counter(u32::from(counter_type.into()))
}

/// Reads performance counter zero.
#[inline]
pub fn instruction_counter() -> u64 {
    upstream::performance_counter(0)
}

/// Reads performance counter one.
#[inline]
pub fn call_context_instruction_counter() -> u64 {
    upstream::performance_counter(1)
}

/// Reports whether the Principal controls this canister.
///
/// # Panics
/// Panics if IC0 returns a value other than zero or one.
pub fn is_controller(principal: &Principal) -> bool {
    system_bool(
        upstream::is_controller(principal.as_slice()),
        "is_controller",
    )
}

/// Reports whether execution is replicated.
///
/// # Panics
/// Panics if IC0 returns a value other than zero or one.
pub fn in_replicated_execution() -> bool {
    system_bool(
        upstream::in_replicated_execution(),
        "in_replicated_execution",
    )
}

/// A signing-cost query failure reported by the IC.
#[derive(Debug, Clone, thiserror::Error)]
pub enum SignCostError {
    /// The requested curve or algorithm is invalid.
    #[error("invalid curve or algorithm")]
    InvalidCurveOrAlgorithm,
    /// The requested key name is invalid.
    #[error("invalid key name")]
    InvalidKeyName,
    /// An unknown supplier error code, retained without truncation.
    #[error("unrecognized error: {0}")]
    UnrecognizedError(u32),
}

fn sign_cost_result(amount: u128, code: u32) -> Result<u128, SignCostError> {
    match code {
        0 => Ok(amount),
        1 => Err(SignCostError::InvalidCurveOrAlgorithm),
        2 => Err(SignCostError::InvalidKeyName),
        value => Err(SignCostError::UnrecognizedError(value)),
    }
}

/// Queries signing cost while preserving the full amount or supplier error code.
pub fn cost_sign_with_ecdsa<T: AsRef<str>>(
    key_name: T,
    ecdsa_curve: u32,
) -> Result<u128, SignCostError> {
    let (amount, code) = upstream::cost_sign_with_ecdsa(key_name.as_ref(), ecdsa_curve);
    sign_cost_result(amount, code)
}

/// Prints the supplied UTF-8 text through IC0.
pub fn debug_print<T: AsRef<str>>(data: T) {
    upstream::debug_print(data.as_ref().as_bytes());
}

/// Traps execution with the supplied UTF-8 text through IC0.
pub fn trap<T: AsRef<str>>(data: T) -> ! {
    upstream::trap(data.as_ref().as_bytes())
}

#[cfg(test)]
mod tests;
