// Imported from ic-cdk 0.20.3; Apache-2.0. See ../NOTICE.md.
//! Reviewed non-cryptographic System API adapters.

use crate::Principal;

/// Gets the message argument data.
pub fn msg_arg_data() -> Vec<u8> {
    let len = ic0::msg_arg_data_size();
    let mut buf = vec![0u8; len];
    ic0::msg_arg_data_copy(&mut buf, 0);
    buf
}

/// Gets the identity of the caller, which may be a canister id or a user id.
///
/// During canister installation or upgrade, this is the id of the user or canister requesting the installation or upgrade.
/// During a system task (heartbeat or global timer), this is the id of the management canister.
pub fn msg_caller() -> Principal {
    let len = ic0::msg_caller_size();
    let mut buf = vec![0u8; len];
    ic0::msg_caller_copy(&mut buf, 0);
    // Trust that the system always returns a valid principal.
    Principal::try_from(&buf).unwrap()
}

/// Returns the reject code, if the current function is invoked as a reject callback.
pub fn msg_reject_code() -> u32 {
    ic0::msg_reject_code()
}

/// Gets the reject message.
///
/// This function can only be called in the reject callback.
///
/// Traps if:
/// - There is no reject message (i.e. if `reject_code` is 0).
/// - The message is not valid UTF-8.
pub fn msg_reject_msg() -> String {
    let len = ic0::msg_reject_msg_size();
    let mut buf = vec![0u8; len];
    ic0::msg_reject_msg_copy(&mut buf, 0);
    String::from_utf8(buf).expect("reject message is not valid UTF-8")
}

/// Replies to the sender with the data.
pub fn msg_reply<T: AsRef<[u8]>>(data: T) {
    let buf = data.as_ref();
    if !buf.is_empty() {
        ic0::msg_reply_data_append(buf);
    }
    ic0::msg_reply();
}

/// Rejects the call with a diagnostic message.
pub fn msg_reject<T: AsRef<str>>(message: T) {
    let message = message.as_ref();
    ic0::msg_reject(message.as_bytes());
}

/// Gets the number of cycles transferred by the caller of the current call, still available in this message.
pub fn msg_cycles_available() -> u128 {
    ic0::msg_cycles_available128()
}

/// Gets the amount of cycles that came back with the response as a refund
///
/// This function can only be used in a callback handler (reply or reject).
/// The refund has already been added to the canister balance automatically.
pub fn msg_cycles_refunded() -> u128 {
    ic0::msg_cycles_refunded128()
}

/// Moves cycles from the call to the canister balance.
///
/// The actual amount moved will be returned.
pub fn msg_cycles_accept(max_amount: u128) -> u128 {
    ic0::msg_cycles_accept128(max_amount)
}

/// Burns cycles from the canister.
///
/// Returns the amount of cycles that were actually burned.
pub fn cycles_burn(amount: u128) -> u128 {
    ic0::cycles_burn128(amount)
}

/// Gets canister's own identity.
pub fn canister_self() -> Principal {
    let len = ic0::canister_self_size();
    let mut buf = vec![0u8; len];
    ic0::canister_self_copy(&mut buf, 0);
    // Trust that the system always returns a valid principal.
    Principal::try_from(&buf).unwrap()
}

/// Gets the current cycle balance of the canister.
pub fn canister_cycle_balance() -> u128 {
    ic0::canister_cycle_balance128()
}

/// Gets the current amount of cycles that is available for spending in calls and execution.
pub fn canister_liquid_cycle_balance() -> u128 {
    ic0::canister_liquid_cycle_balance128()
}

/// Gets the canister version.
///
/// See [Canister version](https://internetcomputer.org/docs/current/references/ic-interface-spec/#system-api-canister-version).
pub fn canister_version() -> u64 {
    ic0::canister_version()
}

/// Gets the ID of the subnet on which the canister is running.
pub fn subnet_self() -> Principal {
    let len = ic0::subnet_self_size();
    let mut buf = vec![0u8; len];
    ic0::subnet_self_copy(&mut buf, 0);
    // Trust that the system always returns a valid principal.
    Principal::try_from(&buf).unwrap()
}

/// Gets the name of the method to be inspected.
///
/// This function is only available in the `canister_inspect_message` context.
///
/// Traps if the method name is not valid UTF-8.
pub fn msg_method_name() -> String {
    let len = ic0::msg_method_name_size();
    let mut buf = vec![0u8; len];
    ic0::msg_method_name_copy(&mut buf, 0);
    String::from_utf8(buf).expect("msg_method_name is not valid UTF-8")
}

/// Accepts the message in `canister_inspect_message`.
///
/// This function is only available in the `canister_inspect_message` context.
/// This function traps if invoked twice.
pub fn accept_message() {
    ic0::accept_message();
}

/// Gets the current size of the stable memory (in WebAssembly pages).
///
/// One WebAssembly page is 64KiB.
pub fn stable_size() -> u64 {
    ic0::stable64_size()
}

/// Gets the public key (a DER-encoded BLS key) of the root key of this instance of the Internet Computer Protocol.
///
/// # Note
///
/// This traps in non-replicated mode.
pub fn root_key() -> Vec<u8> {
    let len = ic0::root_key_size();
    let mut buf = vec![0u8; len];
    ic0::root_key_copy(&mut buf, 0);
    buf
}

/// Sets the certified data of this canister.
///
/// Canisters can store up to 32 bytes of data that is certified by
/// the system on a regular basis.  One can call [`data_certificate`]
/// function from a query call to get a certificate authenticating the
/// value set by calling this function.
///
/// This function can only be called from the following contexts:
/// - `canister_init`, `canister_pre_upgrade` and `canister_post_upgrade`
///   hooks.
/// - `canister_update` calls.
/// - reply or reject callbacks.
///
/// # Panics
///
/// - This function traps if `data.len() > 32`.
/// - This function traps if it's called from an illegal context
///   (e.g., from a query call).
pub fn certified_data_set<T: AsRef<[u8]>>(data: T) {
    let buf = data.as_ref();
    ic0::certified_data_set(buf);
}

/// When called from a query call, returns the data certificate authenticating
/// certified data set by this canister.
///
/// Returns `None` if called not from a query call.
pub fn data_certificate() -> Option<Vec<u8>> {
    if ic0::data_certificate_present() == 0 {
        return None;
    }
    let n = ic0::data_certificate_size();
    let mut buf = vec![0u8; n];
    ic0::data_certificate_copy(&mut buf, 0);
    Some(buf)
}

/// Gets current timestamp, in nanoseconds since the epoch (1970-01-01)
pub fn time() -> u64 {
    ic0::time()
}

/// Sets global timer.
///
/// The canister can set a global timer to make the system
/// schedule a call to the exported `canister_global_timer`
/// Wasm method after the specified time.
/// The time must be provided as nanoseconds since 1970-01-01.
///
/// The function returns the previous value of the timer.
/// If no timer is set before invoking the function, then the function returns zero.
///
/// Passing zero as an argument to the function deactivates the timer and thus
/// prevents the system from scheduling calls to the canister's `canister_global_timer` Wasm method.
pub fn global_timer_set(timestamp: u64) -> u64 {
    ic0::global_timer_set(timestamp)
}

/// Gets the value of specified performance counter.
///
/// See [`PerformanceCounterType`] for available counter types.
#[inline]
pub fn performance_counter(counter_type: impl Into<PerformanceCounterType>) -> u64 {
    let counter_type: u32 = counter_type.into().into();
    ic0::performance_counter(counter_type)
}

/// The type of performance counter.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
pub enum PerformanceCounterType {
    /// Current execution instruction counter.
    ///
    /// The number of WebAssembly instructions the canister has executed
    /// since the beginning of the current Message execution.
    InstructionCounter,
    /// Call context instruction counter
    ///
    /// The number of WebAssembly instructions the canister has executed
    /// within the call context of the current Message execution
    /// since Call context creation.
    /// The counter monotonically increases across all message executions
    /// in the call context until the corresponding call context is removed.
    CallContextInstructionCounter,
    /// A performance counter type that is not recognized by this library.
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
    fn from(value: PerformanceCounterType) -> Self {
        match value {
            PerformanceCounterType::InstructionCounter => 0,
            PerformanceCounterType::CallContextInstructionCounter => 1,
            PerformanceCounterType::Unrecognized(value) => value,
        }
    }
}

impl PartialEq<u32> for PerformanceCounterType {
    fn eq(&self, other: &u32) -> bool {
        let self_as_u32: u32 = (*self).into();
        self_as_u32 == *other
    }
}

/// Returns the number of instructions that the canister executed since the last [entry
/// point](https://internetcomputer.org/docs/current/references/ic-interface-spec/#entry-points).
#[inline]
pub fn instruction_counter() -> u64 {
    performance_counter(0)
}

/// Returns the number of WebAssembly instructions the canister has executed
/// within the call context of the current Message execution since
/// Call context creation.
///
/// The counter monotonically increases across all message executions
/// in the call context until the corresponding call context is removed.
#[inline]
pub fn call_context_instruction_counter() -> u64 {
    performance_counter(1)
}

/// Determines if a Principal is a controller of the canister.
pub fn is_controller(principal: &Principal) -> bool {
    let slice = principal.as_slice();
    match ic0::is_controller(slice) {
        0 => false,
        1 => true,
        n => panic!("unexpected return value from is_controller: {n}"),
    }
}

/// Checks if in replicated execution.
///
/// The canister can check whether it is currently running in replicated or non replicated execution.
pub fn in_replicated_execution() -> bool {
    match ic0::in_replicated_execution() {
        0 => false,
        1 => true,
        n => panic!("unexpected return value from in_replicated_execution: {n}"),
    }
}

/// Gets the amount of cycles that a canister needs to be above the freezing threshold in order to successfully make an inter-canister call.
pub fn cost_call(method_name_size: u64, payload_size: u64) -> u128 {
    ic0::cost_call(method_name_size, payload_size)
}

/// Gets the cycle cost of the Management canister method [`create_canister`](https://internetcomputer.org/docs/references/ic-interface-spec#ic-create_canister).
pub fn cost_create_canister() -> u128 {
    ic0::cost_create_canister()
}

/// Gets the cycle cost of the Management canister method [`http_request`](https://internetcomputer.org/docs/references/ic-interface-spec#ic-http_request).
pub fn cost_http_request(request_size: u64, max_res_bytes: u64) -> u128 {
    ic0::cost_http_request(request_size, max_res_bytes)
}

/// The error type for the reviewed `cost_sign_with_ecdsa` adapter.
#[derive(thiserror::Error, Debug, Clone)]
pub enum SignCostError {
    /// The ECDSA/vetKD curve or Schnorr algorithm is invalid.
    #[error("invalid curve or algorithm")]
    InvalidCurveOrAlgorithm,

    /// The key name is invalid for the provided curve or algorithm.
    #[error("invalid key name")]
    InvalidKeyName,
    /// Unrecognized error.
    ///
    /// This error is returned when the System API returns an unrecognized error code.
    /// Please report to ic-cdk maintainers.
    #[error("unrecognized error: {0}")]
    UnrecognizedError(u32),
}

/// Helper function to handle the result of a signature cost function.
fn sign_cost_result(dst: u128, code: u32) -> Result<u128, SignCostError> {
    match code {
        0 => Ok(dst),
        1 => Err(SignCostError::InvalidCurveOrAlgorithm),
        2 => Err(SignCostError::InvalidKeyName),
        _ => Err(SignCostError::UnrecognizedError(code)),
    }
}

/// Gets the cycle cost of the Management canister method [`sign_with_ecdsa`](https://internetcomputer.org/docs/references/ic-interface-spec#ic-sign_with_ecdsa).
///
/// # Errors
///
/// This function will return an error if the `key_name` or the `ecdsa_curve` is invalid.
/// The error type [`SignCostError`] provides more information about the reason of the error.
pub fn cost_sign_with_ecdsa<T: AsRef<str>>(
    key_name: T,
    ecdsa_curve: u32,
) -> Result<u128, SignCostError> {
    let key_name = key_name.as_ref();
    let (cost, code) = ic0::cost_sign_with_ecdsa(key_name, ecdsa_curve);
    sign_cost_result(cost, code)
}

/// Emits textual trace messages.
///
/// On the "real" network, these do not do anything.
///
/// When executing in an environment that supports debugging, this copies out the data
/// and logs, prints or stores it in an environment-appropriate way.
pub fn debug_print<T: AsRef<str>>(data: T) {
    let buf = data.as_ref();
    ic0::debug_print(buf.as_bytes());
}

/// Traps with the given message.
///
/// The environment may copy out the data and log, print or store it in an environment-appropriate way,
/// or include it in system-generated reject messages where appropriate.
pub fn trap<T: AsRef<str>>(data: T) -> ! {
    let buf = data.as_ref();
    ic0::trap(buf.as_bytes());
}
