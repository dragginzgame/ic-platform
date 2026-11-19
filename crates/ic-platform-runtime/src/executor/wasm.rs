//! Entry admission for replicated Wasm callbacks and synchronous exports.

use super::{EXECUTOR, EntryError, TaskHandle, start, start_root};
use crate::call::ic0;
use std::future::Future;

/// Enters one method-attached root future from a manual update entrypoint.
///
/// Call only from a canister update export. Non-replicated and nested entries
/// refuse; replicated-query/export validation still belongs to the entrypoint
/// layer and is not established by the replicated-execution flag alone. This
/// adapter emits no reply automatically and exposes no method-context handle.
///
/// # Panics
/// User polling or destruction may trap. Initial traps can prevent returning a
/// handle; callback rollback and cleanup behavior require installed qualification.
pub fn run_update(future: impl Future<Output = ()> + 'static) -> Result<TaskHandle, EntryError> {
    check_update_entry()?;
    start(Box::pin(future))
}

/// Enters a root future from the caller-owned `canister_global_timer` export.
///
/// The task uses the same executor and callback custody as updates, but has no
/// ordinary reply authority, including in resumed callbacks. It is attached to
/// this timer invocation; it is neither a child task nor migratory work. Call
/// once per root from that export, never from an update, query or lifecycle hook.
/// The replicated-execution flag cannot authenticate the export kind; the export
/// owner must enforce this entry contract. Runtime does not arm or rearm timers.
///
/// # Errors
/// Refuses non-replicated execution, nested polling/destruction and exhausted
/// task identifiers before admission. Dropping the handle does not cancel work.
///
/// # Panics
/// User polling/destruction may trap. Initial traps can prevent returning a
/// handle. A failed callback cleanup leaves `Running` as an uncertain observation,
/// not permission to retry. The consumer owns its committed dispatch record.
pub fn run_global_timer(
    future: impl Future<Output = ()> + 'static,
) -> Result<TaskHandle, EntryError> {
    check_update_entry()?;
    start_root(Box::pin(future), false)
}

pub(crate) fn check_update_entry() -> Result<(), EntryError> {
    if !ic0::replicated_execution() {
        return Err(EntryError::NonReplicated);
    }
    EXECUTOR.with(|executor| {
        let executor = executor.borrow();
        if executor.current.is_some() || executor.synchronous {
            Err(EntryError::NestedEntry)
        } else if executor.last == u64::MAX {
            Err(EntryError::Capacity)
        } else {
            Ok(())
        }
    })
}
