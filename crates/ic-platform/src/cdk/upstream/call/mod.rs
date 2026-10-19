// Imported from ic-cdk 0.20.3; Apache-2.0. See ../NOTICE.md.
//! Raw inter-canister calls with the imported CDK 0.20.3 callback state machine.
//!
//! Callers supply explicit Wire frames. Optional Candid adapters serve required
//! external management/system interfaces. The registry executor remains shared.

use super::api::{cost_call, msg_arg_data, msg_reject_code, msg_reject_msg};
use super::{api::trap, futures::is_recovering_from_trap};
use crate::Principal;
#[cfg(feature = "candid")]
use candid::utils::{ArgumentDecoder, ArgumentEncoder, encode_args_ref};
#[cfg(feature = "candid")]
use candid::{CandidType, Deserialize, decode_args, decode_one, encode_one};
use ic_cdk_executor::{MethodHandle, TaskHandle};
use std::borrow::Cow;
use std::future::IntoFuture;
use std::mem;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::task::{Context, Poll, Waker};
use thiserror::Error;

pub use ic_error_types::RejectCode;

/// Raw inter-canister call builder.
///
/// Bounded waits default to 300 seconds. Constructors start with empty transport
/// bytes; supply an explicit Wire request or external Candid arguments. Callers
/// retain timeout, cycle and retry policy. Callback traps retain the imported
/// task-cancellation behavior of the shared executor.
#[derive(Debug, Clone)]
pub struct Call<'m, 'a> {
    canister_id: Principal,
    method: &'m str,
    cycles: u128,
    timeout_seconds: Option<u32>,
    encoded_args: Cow<'a, [u8]>,
}

// Constructors
impl<'m> Call<'m, '_> {
    /// Constructs a [`Call`] which will **boundedly** wait for response.
    ///
    /// # Note
    ///
    /// The bounded waiting is set with a default 300-second timeout.
    /// It aligns with the `MAX_CALL_TIMEOUT` constant in the current IC implementation.
    /// The timeout can be changed using the [`change_timeout`][Self::change_timeout] method.
    ///
    /// To unboundedly wait for response, use the [`Call::unbounded_wait`] constructor instead.
    #[must_use]
    pub fn bounded_wait(canister_id: Principal, method: &'m str) -> Self {
        Self {
            canister_id,
            method,
            cycles: 0,
            // Default to 300-second timeout.
            timeout_seconds: Some(300),
            // Raw transport starts empty; supply an explicit Wire or external Candid payload.
            encoded_args: Cow::Borrowed(&[]),
        }
    }

    /// Constructs a [`Call`] which will **unboundedly** wait for response.
    ///
    /// To boundedly wait for response, use the  [`Call::bounded_wait`] constructor instead.
    #[must_use]
    pub fn unbounded_wait(canister_id: Principal, method: &'m str) -> Self {
        Self {
            canister_id,
            method,
            cycles: 0,
            timeout_seconds: None,
            // Raw transport starts empty; supply an explicit Wire or external Candid payload.
            encoded_args: Cow::Borrowed(&[]),
        }
    }
}

// Configuration
impl<'a> Call<'_, 'a> {
    /// Sets the argument for the call.
    ///
    /// The argument must implement [`CandidType`].
    #[must_use]
    #[cfg(feature = "candid")]
    pub fn with_arg<A: CandidType>(self, arg: A) -> Self {
        Self {
            encoded_args: Cow::Owned(encode_one(&arg).unwrap_or_else(panic_when_encode_fails)),
            ..self
        }
    }

    /// Sets the arguments for the call.
    ///
    /// The arguments are a tuple of types, each implementing [`CandidType`].
    #[must_use]
    #[cfg(feature = "candid")]
    pub fn with_args<A: ArgumentEncoder>(self, args: &A) -> Self {
        Self {
            encoded_args: Cow::Owned(encode_args_ref(args).unwrap_or_else(panic_when_encode_fails)),
            ..self
        }
    }

    /// Sets the arguments for the call as raw bytes.
    ///
    /// # Note
    ///
    /// This method just borrows the bytes, so it is useful when making multiple calls with the same argument data.
    ///
    /// The `Call` object will be tied to the lifetime of the argument bytes,
    /// which may prevent storing the call in collections or returning it from functions
    /// if the arguments don't live long enough.
    ///
    /// For cases where you need to transfer ownership of the arguments bytes consider using [`Self::take_raw_args`] instead.
    #[must_use]
    pub fn with_raw_args(self, raw_args: &'a [u8]) -> Self {
        Self {
            encoded_args: Cow::Borrowed(raw_args),
            ..self
        }
    }

    /// Sets the arguments for the call as raw bytes and consumes the bytes.
    ///
    /// # Note
    ///
    /// This method takes ownership of the arguments bytes, so it is useful
    /// when you want to store the call in collections or return a `Call` from functions.
    ///
    /// For cases where you want to make multiple calls with the same argument data,
    /// consider using [`Self::with_raw_args`] instead to avoid unnecessary cloning.
    #[must_use]
    pub fn take_raw_args(self, raw_args: Vec<u8>) -> Self {
        Self {
            encoded_args: Cow::Owned(raw_args),
            ..self
        }
    }

    /// Sets the cycles payment for the call.
    ///
    /// # Note
    ///
    /// The behavior of this method when invoked multiple times is as follows:
    /// - Overrides any previously set cycle value
    /// - Last invocation determines the final cycles amount
    /// - Does not accumulate cycles across multiple invocations
    #[must_use]
    pub fn with_cycles(mut self, cycles: u128) -> Self {
        self.cycles = cycles;
        self
    }

    /// Changes the timeout for bounded response waiting.
    ///
    /// If invoked multiple times, the last value takes effect.
    ///
    /// The timeout value is silently capped by the `MAX_CALL_TIMEOUT` constant which is currently set to 300 seconds.
    /// Therefore, setting a timeout greater than 300 seconds will actually result in a 300-second timeout.
    ///
    /// # Panics
    ///
    /// This method will panic if invoked on an unbounded response waiting call constructed by [`Call::unbounded_wait`] .
    ///
    /// # Note
    ///
    /// A timeout of 0 second **DOES NOT** mean unbounded response waiting.
    /// The call would most likely time out (result in a [`SysUnknown`](RejectCode::SysUnknown) reject).
    /// Unless it's a call to the canister on the same subnet,
    /// and the execution manages to schedule both the request and the response in the same round.
    ///
    /// To unboundedly wait for response, use the [`Call::unbounded_wait`] constructor instead.
    #[must_use]
    pub fn change_timeout(mut self, timeout_seconds: u32) -> Self {
        match self.timeout_seconds {
            Some(_) => self.timeout_seconds = Some(timeout_seconds),
            None => {
                panic!("Cannot set a timeout for an instance created with Call::unbounded_wait")
            }
        }
        self
    }

    /// Returns the amount of cycles a canister needs to be above the freezing threshold in order to
    /// successfully perform this call. Takes into account the attached cycles ([`with_cycles`](Self::with_cycles))
    /// as well as
    /// - the method name byte length
    /// - the payload length
    /// - the cost of transmitting the request
    /// - the cost for the reservation of response transmission (may be partially refunded)
    /// - the cost for the reservation of callback execution (may be partially refunded).
    #[must_use]
    pub fn get_cost(&self) -> u128 {
        self.cycles.saturating_add(cost_call(
            self.method.len() as u64,
            self.encoded_args.len() as u64,
        ))
    }
}

/// Response of a successful call.
#[derive(Debug)]
pub struct Response(Vec<u8>);

impl Response {
    /// Gets the raw bytes of the response.
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }

    /// Decodes the response as a single Candid type.
    #[cfg(feature = "candid")]
    pub fn candid<R>(&self) -> Result<R, CandidDecodeFailed>
    where
        R: CandidType + for<'de> Deserialize<'de>,
    {
        decode_one(&self.0).map_err(|e| CandidDecodeFailed {
            type_name: std::any::type_name::<R>().to_string(),
            candid_error: e.to_string(),
        })
    }

    /// Decodes the response as a tuple of Candid types.
    #[cfg(feature = "candid")]
    pub fn candid_tuple<R>(&self) -> Result<R, CandidDecodeFailed>
    where
        R: for<'de> ArgumentDecoder<'de>,
    {
        decode_args(&self.0).map_err(|e| CandidDecodeFailed {
            type_name: std::any::type_name::<R>().to_string(),
            candid_error: e.to_string(),
        })
    }
}

impl PartialEq<&[u8]> for Response {
    fn eq(&self, other: &&[u8]) -> bool {
        self.0 == *other
    }
}

impl PartialEq<Vec<u8>> for Response {
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.0 == *other
    }
}

impl PartialEq for Response {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl std::ops::Deref for Response {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[u8]> for Response {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl std::borrow::Borrow<[u8]> for Response {
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}

// Errors ---------------------------------------------------------------------

/// Represents errors that can occur during inter-canister calls.
///
/// This is the top-level error type for the inter-canister call API.
///
/// This encapsulates all possible errors that can arise, including:
/// - Insufficient liquid cycle balance.
/// - `ic0.call_perform` failed.
/// - Asynchronously rejected.
/// - Candid decoding of the response failed.
#[derive(Error, Debug, Clone)]
pub enum Error {
    /// The liquid cycle balance is insufficient to perform the call.
    #[error(transparent)]
    InsufficientLiquidCycleBalance(#[from] InsufficientLiquidCycleBalance),

    /// The `ic0.call_perform` operation failed.
    #[error(transparent)]
    CallPerformFailed(#[from] CallPerformFailed),

    /// The inter-canister call is rejected.
    #[error(transparent)]
    CallRejected(#[from] CallRejected),

    /// The response from the inter-canister call could not be decoded as Candid.
    ///
    /// This variant wraps errors that occur when attempting to decode the response
    /// into the expected Candid type.
    #[error(transparent)]
    CandidDecodeFailed(#[from] CandidDecodeFailed),
}

/// The error type when awaiting a `CallFuture`.
///
/// This encapsulates all possible [`enum@Error`] except for the `CandidDecodeFailed` variant.
#[derive(Error, Debug, Clone)]
pub enum CallFailed {
    /// The liquid cycle balance is insufficient to perform the call.
    #[error(transparent)]
    InsufficientLiquidCycleBalance(#[from] InsufficientLiquidCycleBalance),

    /// The `ic0.call_perform` operation failed.
    #[error(transparent)]
    CallPerformFailed(#[from] CallPerformFailed),

    /// The inter-canister call is rejected.
    #[error(transparent)]
    CallRejected(#[from] CallRejected),
}

/// The error type of [`Call::oneway`].
///
/// This encapsulates all possible errors that can occur when sending a oneway call.
/// Therefore, it includes the [`InsufficientLiquidCycleBalance`] and [`CallPerformFailed`] variants.
#[derive(Error, Debug, Clone)]
pub enum OnewayError {
    /// The liquid cycle balance is insufficient to perform the call.
    #[error(transparent)]
    InsufficientLiquidCycleBalance(#[from] InsufficientLiquidCycleBalance),
    /// The `ic0.call_perform` operation failed.
    #[error(transparent)]
    CallPerformFailed(#[from] CallPerformFailed),
}

impl From<OnewayError> for Error {
    fn from(e: OnewayError) -> Self {
        match e {
            OnewayError::InsufficientLiquidCycleBalance(e) => {
                Error::InsufficientLiquidCycleBalance(e)
            }
            OnewayError::CallPerformFailed(e) => Error::CallPerformFailed(e),
        }
    }
}

impl From<CallFailed> for Error {
    fn from(e: CallFailed) -> Self {
        match e {
            CallFailed::InsufficientLiquidCycleBalance(e) => {
                Error::InsufficientLiquidCycleBalance(e)
            }
            CallFailed::CallPerformFailed(e) => Error::CallPerformFailed(e),
            CallFailed::CallRejected(e) => Error::CallRejected(e),
        }
    }
}

/// Represents an error that occurs when the liquid cycle balance is insufficient to perform the call.
///
/// The liquid cycle balance is determined by [`canister_liquid_cycle_balance`](super::api::canister_liquid_cycle_balance).
/// The cost of the call is determined by [`Call::get_cost`].
///
/// The call won't be performed if the former is less than the latter.
#[derive(Error, Debug, Clone)]
#[error("insufficient liquid cycles balance, available: {available}, required: {required}")]
pub struct InsufficientLiquidCycleBalance {
    /// The liquid cycle balance available in the canister.
    pub available: u128,
    /// The required cycles to perform the call.
    pub required: u128,
}

/// Represents an error that occurs when the `ic0.call_perform` operation fails.
///
/// This error type indicates that the underlying `ic0.call_perform` operation
/// returned a non-zero code, signaling a failure.
#[derive(Error, Debug, Clone)]
#[error("call perform failed")]
pub struct CallPerformFailed;

/// Represents an error that occurs when an inter-canister call is rejected.
///
/// The [`reject_code`][`Self::reject_code`] and [`reject_message`][`Self::reject_message`]
/// are exposed to provide details of the rejection.
///
/// This is wrapped by the [`CallFailed::CallRejected`] variant.
#[derive(Error, Debug, Clone)]
#[error("call rejected: {raw_reject_code} - {reject_message}")]
pub struct CallRejected {
    /// All fields are private so we will be able to change the implementation without breaking the API.
    /// Once we have `ic0.msg_error_code` system API, we will only store the `error_code` in this struct.
    /// It will still be possible to get the [`RejectCode`] using the public getter,
    /// because every `error_code` can map to a [`RejectCode`].
    raw_reject_code: u32,
    reject_message: String,
}

/// The error type for when an unrecognized reject code is encountered.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[error("unrecognized reject code: {0}")]
pub struct UnrecognizedRejectCode(u32);

impl CallRejected {
    /// Constructs a [`CallRejected`] instance with the reject code and message.
    ///
    /// # Note
    ///
    /// This constructor is primarily intended for testing scenarios where you need to simulate
    /// rejected inter-canister calls. In production code, instances of this error are typically
    /// created by the system when actual rejections occur during inter-canister communication.
    /// Use this constructor with caution outside of test environments.
    pub fn with_rejection(raw_reject_code: u32, reject_message: String) -> Self {
        Self {
            raw_reject_code,
            reject_message,
        }
    }

    /// Gets the [`RejectCode`].
    ///
    /// The value is converted from `ic0.msg_reject_code`.
    ///
    /// # Errors
    ///
    /// If the raw reject code is not recognized, this method will return an `UnrecognizedRejectCode` error.
    /// This can happen if the IC produces a new reject code that hasn't been included in [`ic_error_types::RejectCode`].
    /// Please check if your `ic-error-types` dependency is up-to-date.
    /// If the latest version of `ic-error-types` doesn't include the new reject code, please report it to the `ic-cdk` maintainers.
    pub fn reject_code(&self) -> Result<RejectCode, UnrecognizedRejectCode> {
        RejectCode::try_from(u64::from(self.raw_reject_code))
            .map_err(|_| UnrecognizedRejectCode(self.raw_reject_code))
    }

    /// Gets the raw numeric [`RejectCode`] value.
    ///
    /// This is a "never-fail" version of [`reject_code`](Self::reject_code) that returns the raw numeric value.
    pub fn raw_reject_code(&self) -> u32 {
        self.raw_reject_code
    }

    /// Retrieves the reject message associated with the call.
    ///
    /// This message is obtained from `ic0.msg_reject_msg`.
    pub fn reject_message(&self) -> &str {
        &self.reject_message
    }
}

/// Represents an error that occurs when the response from an inter-canister call
/// cannot be decoded as Candid.
///
/// This error type provides details about the Candid decoding failure, including
/// the type that was being decoded and the specific Candid error that occurred.
///
/// This is the error returned by the optional external Candid response adapters.
///
/// It is wrapped by the top-level [`Error::CandidDecodeFailed`] variant.
#[derive(Error, Debug, Clone)]
#[error("candid decode failed for type: {type_name}, candid error: {candid_error}")]
pub struct CandidDecodeFailed {
    type_name: String,
    candid_error: String,
}

// Errors END -----------------------------------------------------------------

impl<'m, 'a> IntoFuture for Call<'m, 'a> {
    type Output = Result<Response, CallFailed>;
    type IntoFuture = CallFuture<'m, 'a>;

    fn into_future(self) -> Self::IntoFuture {
        CallFuture {
            state: Arc::new(RwLock::new(CallFutureState::Prepared { call: self })),
        }
    }
}

// Execution
impl Call<'_, '_> {
    /// Sends the call and ignores the reply.
    pub fn oneway(&self) -> Result<(), OnewayError> {
        self.check_liquid_cycle_balance_sufficient()?;
        match self.perform(None) {
            0 => Ok(()),
            _ => Err(CallPerformFailed.into()),
        }
    }

    /// Checks if the liquid cycle balance is sufficient to perform the call.
    fn check_liquid_cycle_balance_sufficient(&self) -> Result<(), InsufficientLiquidCycleBalance> {
        let required = self.get_cost();
        let available = super::api::canister_liquid_cycle_balance();
        if available >= required {
            Ok(())
        } else {
            Err(InsufficientLiquidCycleBalance {
                available,
                required,
            })
        }
    }

    /// Performs the call.
    ///
    /// This is an internal helper function only for [`Self::oneway`] and [`CallFuture::poll`].
    ///
    /// # Arguments
    ///
    /// - `state_ptr`: An optional pointer to the internal state of the `CallFuture`.
    ///   - If `Some`, the call will be prepared for asynchronous execution:
    ///     - `ic0.call_new` will be invoked with [`callback`] and state pointer.
    ///     - `ic0.call_on_cleanup` will be invoked with [`cleanup`].
    ///   - If `None`, the call will be prepared for oneway execution:
    ///     - `ic0.call_new` will be invoked with invalid callback functions.
    ///     - `ic0.call_on_cleanup` won't be invoked.
    ///
    /// # Returns
    ///
    /// The return value of `ic0.call_perform`.
    fn perform(&self, state_opt: Option<Arc<RwLock<CallFutureState<'_, '_>>>>) -> u32 {
        let callee = self.canister_id.as_slice();
        let method = self.method;
        let arg = match &self.encoded_args {
            Cow::Owned(vec) => vec,
            Cow::Borrowed(r) => *r,
        };
        let state_ptr_opt = state_opt.map(Arc::<RwLock<CallFutureState<'_, '_>>>::into_raw);
        match state_ptr_opt {
            Some(state_ptr) => {
                // asynchronous execution
                //
                // # SAFETY:
                // - `callback` is intended as an entrypoint and therefore can be called as both reply and reject fn
                //      for ic0.call_new.
                // - `cleanup` is intended as an entrypoint and therefore can be called as cleanup fn for ic0.call_on_cleanup.
                // - `state_ptr` is a pointer created via Arc::<RwLock<CallFutureState>>::into_raw, and can therefore be passed as the userdata for
                //      `callback` and `cleanup`.
                // - if-and-only-if ic0.call_perform returns 0, exactly one(‡) of `callback` or `cleanup`
                //      receive ownership of `state_ptr`
                // - both functions deallocate `state_ptr`, and this enclosing function deallocates `state_ptr` if ic0.call_perform
                //      returns !=0, and therefore `state_ptr`'s ownership can be passed to FFI without leaking memory.
                //
                // ‡ The flow from outside the WASM runtime is that the callback runs, it traps, state is rolled back,
                //   and the cleanup callback runs afterwards. Inside the runtime, there is no difference between
                //   'state is rolled back to before the callback was called' and 'the callback was never called'.
                //   So from the code's perspective, exactly one function is called.
                unsafe {
                    ic0::call_new(
                        callee,
                        method,
                        callback,
                        state_ptr as usize,
                        callback,
                        state_ptr as usize,
                    );
                    ic0::call_on_cleanup(cleanup, state_ptr as usize);
                }
            }

            None => {
                ic0::call_new_oneway(callee, method);
                // There is no `call_on_cleanup` invocation because:
                // - the callback does not exist, and so cannot trap to require cleanup
                // - under the current behavior of the IC, this produces an error,
                //   which would unconditionally call the cleanup callback
            }
        }
        if !arg.is_empty() {
            ic0::call_data_append(arg);
        }
        if self.cycles > 0 {
            ic0::call_cycles_add128(self.cycles);
        }
        if let Some(timeout_seconds) = self.timeout_seconds {
            ic0::call_with_best_effort_response(timeout_seconds);
        }
        let res = ic0::call_perform();
        if res != 0
            && let Some(state_ptr) = state_ptr_opt
        {
            // SAFETY:
            // - `state_ptr_opt` is `Some` if-and-only-if ic0.call_new was called with ownership of `state`
            // - by returning !=0, ic0.call_new relinquishes ownership of `state_ptr`; it will never be passed
            //      to any functions
            // therefore, there is an outstanding handle to `state`, which it is safe to deallocate
            unsafe {
                Arc::from_raw(state_ptr);
            }
        }
        res
    }
}

// # Internal =================================================================

/// Internal state for the Future when sending a call.
#[derive(Debug, Default)]
enum CallFutureState<'m, 'a> {
    /// The future has been constructed, and the call has not yet been performed.
    /// Needed because futures are supposed to do nothing unless polled.
    /// Polling will attempt to fire off the request. Success returns `Pending` and transitions to `Executing`,
    /// failure returns `Ready` and transitions to `PostComplete.`
    Prepared { call: Call<'m, 'a> },
    /// The call has been performed and the message is in flight. Neither callback has been called. Polling will return `Pending`.
    /// This state will transition to `Trapped` if the future is canceled because of a trap in another future.
    Executing {
        waker: Waker,
        method: MethodHandle,
        task: Option<TaskHandle>,
    },
    /// `callback` has been called, so the call has been completed. This completion state has not yet been read by the user.
    /// Polling will return `Ready` and transition to `PostComplete`.
    Complete {
        result: Result<Response, CallFailed>,
    },
    /// The completion state of `Complete` has been returned from `poll` as `Poll::Ready`. Polling again will trap.
    #[default]
    PostComplete,
    /// The future (*not* the state) was canceled because of a trap in another future during `Executing`. Polling will trap.
    Trapped,
}

/// Represents a future that resolves to the result of an inter-canister call.
///
/// This type is returned by [`IntoFuture::into_future`] when called on a [`Call`].
/// The [`Call`] type implements the [`IntoFuture`] trait, allowing it to be converted
/// into a `CallFuture`. The future can be awaited to retrieve the result of the call.
#[derive(Debug)]
pub struct CallFuture<'m, 'a> {
    state: Arc<RwLock<CallFutureState<'m, 'a>>>,
}

impl std::future::Future for CallFuture<'_, '_> {
    type Output = Result<Response, CallFailed>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let self_ref = Pin::into_inner(self);
        let mut state = self_ref.state.write().unwrap();
        match mem::take(&mut *state) {
            CallFutureState::Prepared { call } => {
                if let Err(e) = call.check_liquid_cycle_balance_sufficient() {
                    *state = CallFutureState::PostComplete;
                    Poll::Ready(Err(e.into()))
                } else {
                    match call.perform(Some(self_ref.state.clone())) {
                        0 => {
                            // call_perform returns 0 means the call was successfully enqueued.
                            *state = CallFutureState::Executing {
                                waker: context.waker().clone(),
                                method: ic_cdk_executor::extend_current_method_context(),
                                task: TaskHandle::current(),
                            };
                            Poll::Pending
                        }
                        _ => {
                            *state = CallFutureState::PostComplete;
                            Poll::Ready(Err(CallPerformFailed.into()))
                        }
                    }
                }
            }
            CallFutureState::Executing { method, task, .. } => {
                *state = CallFutureState::Executing {
                    waker: context.waker().clone(),
                    method,
                    task,
                };
                Poll::Pending
            }
            CallFutureState::Complete { result } => {
                *state = CallFutureState::PostComplete;
                Poll::Ready(result)
            }
            CallFutureState::Trapped => trap("Call already trapped"),
            CallFutureState::PostComplete => trap("CallFuture polled after completing"),
        }
    }
}

impl Drop for CallFuture<'_, '_> {
    fn drop(&mut self) {
        // If this future is dropped while is_recovering_from_trap is true,
        // then it has been canceled due to a trap in another future.
        if is_recovering_from_trap() {
            *self.state.write().unwrap() = CallFutureState::Trapped;
        }
    }
}

/// The reply/reject callback for `ic0.call_new`.
///
/// It dereferences the future from a raw pointer, assigns the result and calls the waker.
/// We cannot use a closure here because we pass raw pointers to the System and back.
///
/// # Safety
///
/// This function must only be passed to the IC with a pointer from `Arc::<RwLock<CallFutureState>>::into_raw` as userdata.
unsafe extern "C" fn callback(env: usize) {
    let state_ptr = env as *const RwLock<CallFutureState<'_, '_>>;
    // SAFETY: This function is only ever called by the IC, and we only ever pass an Arc as userdata.
    let state = unsafe { Arc::from_raw(state_ptr) };
    let completed_state = CallFutureState::Complete {
        result: match msg_reject_code() {
            0 => Ok(Response(msg_arg_data())),
            code => {
                // The conversion is safe because the code is not 0.
                Err(CallFailed::CallRejected(CallRejected {
                    raw_reject_code: code,
                    reject_message: msg_reject_msg(),
                }))
            }
        },
    };
    let (waker, method) = match mem::replace(&mut *state.write().unwrap(), completed_state) {
        CallFutureState::Executing { waker, method, .. } => (waker, method),
        // This future has already been cancelled and waking it will do nothing.
        // All that's left is to explicitly trap in case this is the last call being multiplexed,
        // to replace an automatic trap from not replying.
        CallFutureState::Trapped => trap("Call already trapped"),
        _ => {
            unreachable!(
                "CallFutureState for in-flight calls should only be Executing or Trapped (callback)"
            )
        }
    };
    ic_cdk_executor::in_callback_executor_context_for(method, || {
        waker.wake();
    });
}

/// The cleanup callback for `ic0.call_on_cleanup`.
///
/// This function is called when [`callback`] was just called with the same parameter, and trapped.
/// We can't guarantee internal consistency at this point, but we can at least e.g. drop mutex guards.
/// Waker is a very opaque API, so the best we can do is set a global flag and proceed normally.
///
/// # Safety
///
/// This function must only be passed to the IC with a pointer from `Arc::<RwLock<CallFutureState>>::into_raw` as userdata.
unsafe extern "C" fn cleanup(env: usize) {
    let state_ptr = env as *const RwLock<CallFutureState<'_, '_>>;
    // SAFETY: This function is only ever called by the IC, and we only ever pass a Arc as userdata.
    let state = unsafe { Arc::from_raw(state_ptr) };
    // We set the call result, even though it won't be read on the
    // default executor, because we can't guarantee it was called on
    // our executor. However, we are not allowed to inspect
    // reject_code() inside of a cleanup callback, so always set the
    // result to a reject.
    //
    // Borrowing does not trap - the rollback from the
    // previous trap ensures that the RwLock can be borrowed again.
    let err_state = CallFutureState::Complete {
        result: Err(CallFailed::CallRejected(CallRejected {
            raw_reject_code: RejectCode::CanisterReject as u32,
            reject_message: "cleanup".into(),
        })),
    };
    let (method, task) = match mem::replace(&mut *state.write().unwrap(), err_state) {
        CallFutureState::Executing { method, task, .. } => (method, task),
        CallFutureState::Trapped => {
            // The future has already been canceled and dropped. There is nothing
            // more to clean up except for the CallFutureState.
            return;
        }
        _ => {
            unreachable!(
                "CallFutureState for in-flight calls should only be Executing or Trapped (cleanup)"
            )
        }
    };
    ic_cdk_executor::in_trap_recovery_context_for(method, || {
        ic_cdk_executor::cancel_all_tasks_attached_to_current_method();
        if let Some(task) = task {
            ic_cdk_executor::cancel_task(&task);
        }
    });
}

// # Internal END =============================================================

/// Panics with an informative message when argument encoding fails.
///
/// Currently, Candid encoding only fails when heap memory is exhausted,
/// in which case execution would trap before reaching the unwrap.
///
/// However, since future implementations might introduce other failure cases,
/// we provide an informative panic message for better debuggability.
#[cfg(feature = "candid")]
fn panic_when_encode_fails(err: candid::error::Error) -> Vec<u8> {
    panic!("failed to encode args: {err}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_constructors_do_not_choose_a_codec() {
        for call in [
            Call::bounded_wait(Principal::anonymous(), "probe"),
            Call::unbounded_wait(Principal::anonymous(), "probe"),
        ] {
            assert!(call.encoded_args.is_empty());
            assert_eq!(call.with_raw_args(b"raw").encoded_args.as_ref(), b"raw");
        }
    }

    #[cfg(feature = "candid")]
    #[test]
    fn external_unit_arguments_are_explicit() {
        let call =
            Call::unbounded_wait(Principal::management_canister(), "raw_rand").with_args(&());
        assert_eq!(call.encoded_args.as_ref(), candid::encode_args(()).unwrap());
    }
}
