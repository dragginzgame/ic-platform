//! Explicit guard, bounded input and caller-selected codecs for IC entrypoints.
//!
//! Updates share one executor; synchronous queries and lifecycle handlers exclude
//! nested Runtime entry. Automatic replies require an explicit encoder.
//! Applications retain authorization policy and explicit codec choice.

use crate::{
    executor::{self, EntryError, TaskHandle},
    system::{self, ArgumentTooLarge},
};
use std::future::Future;

/// The stage that refused entrypoint input, preserving its original error value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdmissionError<G, D> {
    /// The caller's pre-copy guard refused; Runtime did not read arguments.
    Guard(G),
    /// Raw arguments exceeded the explicit byte allowance; decoding did not run.
    Arguments(ArgumentTooLarge),
    /// The selected decoder refused captured bytes.
    Decode(D),
}

/// Runs a pre-copy guard, bounded argument capture, then an explicit decoder.
///
/// Runtime runs each stage at most once and stops at the first error. A refused
/// guard causes no Runtime argument-size query or copy. Guards that use only
/// caller identity belong here; checks requiring decoded fields belong after
/// this function. User closures remain responsible for their own side effects.
///
/// The result owns any data needed beyond argument capture: it cannot borrow
/// from the temporary argument buffer. Capture before awaiting, since the system
/// exposes a reply callback's bytes rather than original arguments in callbacks.
/// No implicit codec, response encoding, reply or executor context is selected.
/// The raw-byte limit does not bound a custom decoder's allocations or work.
///
/// # Errors
/// Returns the exact guard/decoder error in [`AdmissionError`], or an argument
/// size refusal. No error is converted into a remote rejection or retry policy.
///
/// # Panics
/// User closures may panic. Argument capture may trap for an invalid IC context
/// or allocation failure; see [`system::read_args`].
pub fn prepare<T, G, D>(
    maximum: usize,
    guard: impl FnOnce() -> Result<(), G>,
    decode: impl FnOnce(&[u8]) -> Result<T, D>,
) -> Result<T, AdmissionError<G, D>> {
    guard().map_err(AdmissionError::Guard)?;
    let bytes = system::read_args(maximum).map_err(AdmissionError::Arguments)?;
    decode(&bytes).map_err(AdmissionError::Decode)
}

/// Refusal before the manual update handler starts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateError<G, D> {
    /// The update executor refused the execution context or task admission.
    Entry(EntryError),
    /// The guard, argument bound or selected decoder refused the input.
    Admission(AdmissionError<G, D>),
}

/// A generated automatic-response endpoint refused at this stage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EndpointError<G, D, E> {
    /// Execution context or root-task admission refused.
    Entry(EntryError),
    /// Pre-copy guard, argument bound or explicit decoding refused.
    Admission(AdmissionError<G, D>),
    /// The caller-selected response encoder refused; no reply bytes were appended.
    Encode(E),
    /// A handler already consumed the update task's reply authority.
    Reply(system::ReplyError),
}

/// Runs one synchronous query with explicit input and response policies.
///
/// Call only from a query export. No future or callback is registered. Nested
/// Runtime entries and manual Runtime replies refuse while this scope owns the
/// response. The guard precedes argument observation; encoding succeeds before
/// any response append. Both replicated and non-replicated queries are supported.
///
/// # Errors
/// Preserves entry, input and encoder refusals without issuing a reply.
///
/// # Panics
/// User code or IC imports may trap; the IC governs query rollback.
pub fn query<T, R, G, D, E>(
    maximum: usize,
    guard: impl FnOnce() -> Result<(), G>,
    decode: impl FnOnce(&[u8]) -> Result<T, D>,
    handler: impl FnOnce(T) -> R,
    encode: impl FnOnce(R) -> Result<Vec<u8>, E>,
) -> Result<(), EndpointError<G, D, E>> {
    let _scope = executor::enter_synchronous().map_err(EndpointError::Entry)?;
    let input = prepare(maximum, guard, decode).map_err(EndpointError::Admission)?;
    let bytes = encode(handler(input)).map_err(EndpointError::Encode)?;
    crate::call::reply(&bytes);
    Ok(())
}

/// Runs one synchronous lifecycle handler without reply or async task authority.
///
/// Use only from the matching generated init, pre-upgrade or post-upgrade export.
/// No persistence, migration policy or automatic upgrade guard is introduced.
/// Pre-upgrade has no arguments: its generated adapter supplies an empty input
/// to the explicit decoder instead of using IC argument imports.
///
/// # Errors
/// Preserves nested-entry, guard, size and decoder refusals.
///
/// # Panics
/// User code and system context violations may trap.
pub fn lifecycle<T, G, D>(
    maximum: usize,
    has_arguments: bool,
    guard: impl FnOnce() -> Result<(), G>,
    decode: impl FnOnce(&[u8]) -> Result<T, D>,
    handler: impl FnOnce(T),
) -> Result<(), UpdateError<G, D>> {
    let _scope = executor::enter_synchronous().map_err(UpdateError::Entry)?;
    let input = if has_arguments {
        prepare(maximum, guard, decode).map_err(UpdateError::Admission)?
    } else {
        guard().map_err(|e| UpdateError::Admission(AdmissionError::Guard(e)))?;
        decode(&[]).map_err(|e| UpdateError::Admission(AdmissionError::Decode(e)))?
    };
    handler(input);
    Ok(())
}

/// Starts one update task and encodes/replies after the handler completes.
///
/// The same executor and single reply authority as manual updates are used.
/// Encoder refusal invokes the explicit non-returning error callback before any
/// append. A manual reply from the handler makes the later automatic reply refuse.
/// The callback also handles refusals after suspension; Runtime chooses no codec
/// or remote error representation.
///
/// # Errors
/// Returns entry/input refusals before starting the task.
///
/// # Panics
/// User code, its error callback or an IC operation may trap.
pub fn update<T, R, G, D, E, F>(
    maximum: usize,
    guard: impl FnOnce() -> Result<(), G>,
    decode: impl FnOnce(&[u8]) -> Result<T, D>,
    handler: impl FnOnce(T) -> F + 'static,
    encode: impl FnOnce(R) -> Result<Vec<u8>, E> + 'static,
    on_error: fn(EndpointError<G, D, E>) -> !,
) -> Result<TaskHandle, EndpointError<G, D, E>>
where
    T: 'static,
    G: 'static,
    D: 'static,
    E: 'static,
    F: Future<Output = R> + 'static,
{
    executor::check_update_entry().map_err(EndpointError::Entry)?;
    let input = prepare(maximum, guard, decode).map_err(EndpointError::Admission)?;
    executor::run_update(async move {
        let result = handler(input).await;
        let bytes = match encode(result) {
            Ok(bytes) => bytes,
            Err(error) => on_error(EndpointError::Encode(error)),
        };
        if let Err(error) = system::reply(&bytes) {
            on_error(EndpointError::Reply(error));
        }
    })
    .map_err(EndpointError::Entry)
}

/// Admits input and starts one update task with caller-owned manual reply policy.
///
/// Call from an update export only. Execution-context checks precede the guard,
/// bounded capture and explicit decoder. The handler is invoked only after task
/// admission, receives owned decoded input, and runs in that task's context.
/// No task is started on admission failure. Input and handler may contain borrows
/// only when those borrows are static; they must survive callback suspension.
///
/// The executor checks admission again after user guard/decoder code, which may
/// have side effects. This adapter does not reserve a method context across such
/// code. It neither generates exports nor distinguishes replicated queries from
/// update exports. It supplies no automatic reply or encoding; completing the
/// handler is not evidence that a reply was issued.
///
/// # Errors
/// Returns [`UpdateError`] without invoking the handler when execution-context,
/// task-capacity or input admission fails. No error reply is sent automatically.
///
/// # Panics
/// User code and IC system operations may trap. Initial execution traps can
/// prevent returning a handle; installed rollback qualification remains pending.
pub fn update_manual<T, G, D, F>(
    maximum: usize,
    guard: impl FnOnce() -> Result<(), G>,
    decode: impl FnOnce(&[u8]) -> Result<T, D>,
    handler: impl FnOnce(T) -> F + 'static,
) -> Result<TaskHandle, UpdateError<G, D>>
where
    T: 'static,
    F: Future<Output = ()> + 'static,
{
    executor::check_update_entry().map_err(UpdateError::Entry)?;
    let input = prepare(maximum, guard, decode).map_err(UpdateError::Admission)?;
    executor::run_update(async move { handler(input).await }).map_err(UpdateError::Entry)
}
