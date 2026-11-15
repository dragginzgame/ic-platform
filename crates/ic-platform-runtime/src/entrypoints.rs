//! Explicit guard, raw-byte admission and decoding for future entrypoint adapters.
//!
//! This slice does not generate exports, create tasks, select an encoder or issue
//! replies. Applications retain authorization policy and explicit codec choice.

use crate::system::{self, ArgumentTooLarge};

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
