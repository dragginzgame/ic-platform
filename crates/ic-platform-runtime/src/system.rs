//! Small reviewed IC system operations, independent of codecs and authorization.

use crate::call::ic0;
pub use crate::executor::ReplyError;

/// Observes raw argument length without copying, for explicit pre-copy guards.
///
/// Valid only in argument-capable IC contexts; system context violations trap.
#[must_use]
pub fn argument_size() -> usize {
    ic0::argument_size()
}

/// The current message's caller, in the canonical Types identity.
///
/// Observe before awaiting when the original caller is needed. Context violations
/// or an invalid system Principal length trap. This does not authorize the caller.
#[must_use]
pub fn caller() -> candidate_types::Principal {
    ic0::principal(true)
}

/// This canister's identity, in the canonical Types identity.
#[must_use]
pub fn canister_self() -> candidate_types::Principal {
    ic0::principal(false)
}

/// IC-provided time in nanoseconds since the Unix epoch for this execution.
///
/// This is not a host clock or an application freshness policy. Query results
/// still require authentication before a remote consumer trusts the observation.
#[must_use]
pub fn time() -> u64 {
    ic0::system_time()
}

/// Whether the supplied Principal is a controller at this execution's state.
///
/// The caller owns the decision to use this observation for authorization.
#[must_use]
pub fn is_controller(principal: &candidate_types::Principal) -> bool {
    ic0::controller(principal)
}

/// Publishes exactly one 32-byte certification root in an allowed IC context.
///
/// This changes certified data only if the execution commits. IC context
/// violations trap; Runtime neither constructs witnesses nor chooses a hash.
pub fn set_certified_data(root: &[u8; 32]) {
    ic0::certified_root(root);
}

/// Sets the IC global timer to an absolute timestamp in nanoseconds since Unix epoch.
///
/// Returns the previous timer value. Zero deactivates the timer. The IC resets
/// the timer before invoking `canister_global_timer`; the export owner chooses
/// rearming and scheduling policy. State changes commit with the execution.
///
/// # Panics
/// The IC traps in contexts where setting the global timer is forbidden.
#[expect(
    clippy::must_use_candidate,
    reason = "arming or disarming is the operation; observing the previous deadline is optional"
)]
pub fn global_timer_set(timestamp: u64) -> u64 {
    ic0::set_global_timer(timestamp)
}

/// Observes the monotonically increasing IC canister version.
///
/// The IC increments this for state-changing executions, including updates,
/// timer invocations and upgrades. It is not constant within an installation.
/// Consumers may capture it during initialization as an incarnation marker;
/// reading it does not authenticate an external dispatch or a schema version.
///
/// # Panics
/// The IC traps in contexts where version observation is forbidden.
#[must_use]
pub fn canister_version() -> u64 {
    ic0::system_version()
}

/// An available certificate exceeds the caller's pre-allocation byte allowance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CertificateTooLarge {
    /// Exact certificate size reported by the IC.
    pub actual: usize,
    /// Inclusive caller-selected maximum.
    pub maximum: usize,
}

/// Copies the query certificate, if present, after checking its size.
///
/// Absence returns `Ok(None)` without reading size or copying. Presence does not
/// authenticate these bytes: verification belongs to the receiving consumer.
/// Replicated executions do not receive a query certificate.
///
/// # Errors
/// Returns [`CertificateTooLarge`] before allocation/copy when over the limit.
///
/// # Panics
/// An allocation failure or invalid IC execution context can trap.
pub fn data_certificate(maximum: usize) -> Result<Option<Vec<u8>>, CertificateTooLarge> {
    ic0::certificate(maximum)
}

/// Traps this execution with an explicit caller-supplied diagnostic.
///
/// The IC rolls back this execution's state changes; earlier messages and
/// remote effects are not rolled back. This is not a reject reply or retry.
pub fn trap(message: &str) -> ! {
    ic0::trap_message(message)
}

/// Sends exact raw reply bytes once from the current live update task.
///
/// The caller owns encoding. Empty bytes mean an empty raw reply, not encoded
/// unit. This operation appends and replies synchronously, retaining no borrow.
/// Success is conditional on the enclosing IC execution committing; it does not
/// terminate the task or guarantee delivery. Completing a task without calling
/// this function still does not issue a reply automatically.
///
/// This ordinary path shares one reply authority per task, including resumed
/// callbacks. It cannot police external raw IC0 calls or multiple manual roots
/// created for one export. Generated automatic updates share this reply authority.
///
/// # Errors
/// Refuses outside a live method task, in timer roots, during cleanup, or after this task already
/// replied. Refusal performs no IC0 reply imports.
///
/// # Panics
/// Invalid system contexts, oversized responses or an earlier raw reply can
/// trap. Installed reply/rollback qualification remains pending.
pub fn reply(bytes: &[u8]) -> Result<(), ReplyError> {
    crate::executor::claim_reply()?;
    ic0::reply(bytes);
    Ok(())
}

/// The current message exceeds the caller's raw argument-byte allowance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArgumentTooLarge {
    /// Exact message size reported by the system.
    pub actual: usize,
    /// Caller-selected maximum, inclusive.
    pub maximum: usize,
}

/// Copies the current message's raw arguments after checking their byte length.
///
/// Run identity-only guards before calling this operation. The maximum is
/// checked before allocation and copying; equality is admitted. Returned bytes
/// are owned and survive later callbacks. This operation does not decode bytes,
/// choose a codec, enforce authorization, submit a call or issue a reply.
/// The limit bounds this argument buffer, not the caller's total heap usage.
/// Available on wasm32 with `call-ic0` during argument-capable IC executions.
/// In a reply callback the system exposes that callback's reply, not the
/// original method arguments; capture method arguments before awaiting.
///
/// # Errors
/// Returns [`ArgumentTooLarge`] without copying when the size exceeds `maximum`.
///
/// # Panics
/// System context violations or an allocation failure can trap. This function
/// does not turn traps into successful admission or an encoded error response.
pub fn read_args(maximum: usize) -> Result<Vec<u8>, ArgumentTooLarge> {
    ic0::read_args(maximum)
}
