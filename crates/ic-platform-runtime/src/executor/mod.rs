//! One method-attached root task per update entry; no migratory or child tasks.
//!
//! The manual update adapter is experimental. Export/context and installed
//! rollback qualification remain required before production adoption.

use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Wake, Waker},
};

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
pub struct TaskHandle(Rc<Cell<TaskState>>);
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

/// Refusal before admitting a new update-entry task.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryError {
    /// Non-replicated execution cannot enter the update adapter.
    NonReplicated,
    /// A task is already being polled or destroyed in this entry.
    NestedEntry,
    /// The instance exhausted its non-recycled task identifiers.
    Capacity,
}

/// Refusal to issue a reply through the current executor task.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplyError {
    /// No live executor task owns the current execution, including cleanup.
    NoActiveTask,
    /// This task already used its ordinary reply authority.
    AlreadyReplied,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct TaskToken(u64);
type UserFuture = Pin<Box<dyn Future<Output = ()>>>;

struct Signal {
    ready: AtomicBool,
    live: AtomicBool,
}
impl Wake for Signal {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        if self.live.load(Ordering::SeqCst) {
            self.ready.store(true, Ordering::SeqCst);
        }
    }
}
struct Task {
    future: Option<UserFuture>,
    signal: Arc<Signal>,
    status: Rc<Cell<TaskState>>,
    stop: bool,
    replied: bool,
}
struct Executor {
    last: u64,
    tasks: BTreeMap<u64, Task>,
    current: Option<TaskToken>,
}
thread_local! {
    static EXECUTOR: RefCell<Executor> = const { RefCell::new(Executor { last: 0, tasks: BTreeMap::new(), current: None }) };
}

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
#[cfg(all(feature = "call-ic0", target_arch = "wasm32"))]
pub fn run_update(future: impl Future<Output = ()> + 'static) -> Result<TaskHandle, EntryError> {
    check_update_entry()?;
    start(Box::pin(future))
}

#[cfg(all(feature = "call-ic0", target_arch = "wasm32"))]
pub(crate) fn check_update_entry() -> Result<(), EntryError> {
    if !crate::call::replicated_execution() {
        return Err(EntryError::NonReplicated);
    }
    EXECUTOR.with(|executor| {
        let executor = executor.borrow();
        if executor.current.is_some() {
            Err(EntryError::NestedEntry)
        } else if executor.last == u64::MAX {
            Err(EntryError::Capacity)
        } else {
            Ok(())
        }
    })
}

fn start(future: UserFuture) -> Result<TaskHandle, EntryError> {
    let status = Rc::new(Cell::new(TaskState::Running));
    let signal = Arc::new(Signal {
        ready: AtomicBool::new(true),
        live: AtomicBool::new(true),
    });
    // Admission must not drop a refused user future under an executor borrow.
    let token = EXECUTOR.with(|executor| {
        let mut executor = executor.borrow_mut();
        if executor.current.is_some() {
            return Err(EntryError::NestedEntry);
        }
        let id = executor.last.checked_add(1).ok_or(EntryError::Capacity)?;
        executor.last = id;
        Ok(TaskToken(id))
    })?;
    let handle = TaskHandle(Rc::clone(&status));
    EXECUTOR.with(|executor| {
        let previous = executor.borrow_mut().tasks.insert(
            token.0,
            Task {
                future: Some(future),
                signal,
                status,
                stop: false,
                replied: false,
            },
        );
        assert!(previous.is_none(), "task id reused");
    });
    drive(token);
    Ok(handle)
}

pub(crate) fn current() -> Option<TaskToken> {
    EXECUTOR.with(|executor| executor.borrow().current)
}

pub(crate) fn claim_reply() -> Result<(), ReplyError> {
    EXECUTOR.with(|executor| {
        let mut executor = executor.borrow_mut();
        let token = executor.current.ok_or(ReplyError::NoActiveTask)?;
        let task = executor
            .tasks
            .get_mut(&token.0)
            .ok_or(ReplyError::NoActiveTask)?;
        if task.stop {
            return Err(ReplyError::NoActiveTask);
        }
        if task.replied {
            return Err(ReplyError::AlreadyReplied);
        }
        // This mutation and the subsequent IC reply commit together. A system
        // trap must roll both back; no native-unwind recovery is promised.
        task.replied = true;
        Ok(())
    })
}

struct ResetContext(Option<TaskToken>);
impl Drop for ResetContext {
    fn drop(&mut self) {
        EXECUTOR.with(|executor| executor.borrow_mut().current = self.0);
    }
}

pub(crate) fn drive(token: TaskToken) {
    loop {
        let work = EXECUTOR.with(|executor| {
            let mut executor = executor.borrow_mut();
            if executor.current.is_some() {
                return None;
            }
            let task = executor.tasks.get_mut(&token.0)?;
            if !task.signal.ready.swap(false, Ordering::SeqCst) {
                return None;
            }
            let future = task.future.take().expect("ready task owns its future");
            let signal = Arc::clone(&task.signal);
            executor.current = Some(token);
            Some((future, signal))
        });
        let Some((mut future, signal)) = work else {
            return;
        };
        let _context = ResetContext(None);
        let waker = Waker::from(signal);
        let completed = future
            .as_mut()
            .poll(&mut Context::from_waker(&waker))
            .is_ready();
        let mut future = Some(future);
        let retired = EXECUTOR.with(|executor| {
            let mut executor = executor.borrow_mut();
            let task = executor
                .tasks
                .get_mut(&token.0)
                .expect("polled task stays registered");
            if completed || task.stop {
                task.signal.live.store(false, Ordering::SeqCst);
                executor.tasks.remove(&token.0)
            } else {
                task.future = future.take();
                None
            }
        });
        if let Some(task) = retired {
            // No state transition promises successful destruction before it
            // returns. On the IC a trap rolls back this execution's changes.
            drop(future);
            task.status.set(if task.stop {
                TaskState::TerminatedByCleanup
            } else {
                TaskState::Completed
            });
        }
        // The context guard releases only after user future destruction.
    }
}

pub(crate) fn terminate(token: TaskToken) {
    let retired = EXECUTOR.with(|executor| {
        let mut executor = executor.borrow_mut();
        let task = executor.tasks.get_mut(&token.0)?;
        task.stop = true;
        task.signal.live.store(false, Ordering::SeqCst);
        // A reentrant request cannot destroy a future during its own poll.
        if task.future.is_none() {
            None
        } else {
            executor.tasks.remove(&token.0)
        }
    });
    if let Some(mut task) = retired {
        // Destruction has the same entry exclusion as polling. Preserve an
        // enclosing context if termination was requested reentrantly.
        let previous = EXECUTOR.with(|executor| executor.borrow_mut().current.replace(token));
        let _context = ResetContext(previous);
        drop(task.future.take());
        task.status.set(TaskState::TerminatedByCleanup);
    }
}

#[cfg(test)]
mod tests;
