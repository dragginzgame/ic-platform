//! Explicit guard, raw-byte admission and decoding for future entrypoint adapters.
//!
//! The manual update adapter creates one task after successful admission. This
//! slice does not generate exports, select an encoder or issue replies.
//! Applications retain authorization policy and explicit codec choice.

use crate::executor::{self, EntryError, TaskHandle};
use crate::system::{self, ArgumentTooLarge};
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
