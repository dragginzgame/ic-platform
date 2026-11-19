//! Public observations shared by IC entrypoints and their typed callbacks.
use std::{cell::Cell, rc::Rc};

/// Current local task observation, committed only with the enclosing execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskState {
    /// No completion or successful cleanup has committed for this task.
    ///
    /// A trapping cleanup can roll back to this state even after the IC has
    /// finished handling the response. It does not guarantee another poll.
    Running,
    /// The future completed and its destruction returned successfully.
    Completed,
    /// Cleanup retired the task and its destruction returned successfully.
    TerminatedByCleanup,
}

/// Read-only task observation. Dropping a handle does not cancel a task.
#[derive(Clone)]
pub struct TaskHandle(pub(crate) Rc<Cell<TaskState>>);
impl TaskHandle {
    /// Observes current local state, not remote effects or final settlement.
    ///
    /// `Running` can survive a cleanup trap; it is not a liveness signal or
    /// permission to retry the remote operation.
    #[must_use]
    pub fn state(&self) -> TaskState {
        self.0.get()
    }
}

/// Refusal before admitting a new root task.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryError {
    /// Non-replicated execution cannot enter a root-task adapter.
    NonReplicated,
    /// A task is already being polled or destroyed in this entry.
    NestedEntry,
    /// The instance exhausted its non-recycled task identifiers.
    Capacity,
}

/// Refusal to issue a reply through the current executor task.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplyError {
    /// No live method task owns reply authority, including timer roots and cleanup.
    NoActiveTask,
    /// This task already used its ordinary reply authority.
    AlreadyReplied,
}

#[cfg(feature = "call-ic0")]
/// The current message exceeds the caller's raw argument-byte allowance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArgumentTooLarge {
    /// Exact message size reported by the system.
    pub actual: usize,
    /// Caller-selected maximum, inclusive.
    pub maximum: usize,
}
