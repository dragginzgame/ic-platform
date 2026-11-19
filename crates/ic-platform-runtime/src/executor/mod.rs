//! Update and global-timer root tasks; no migratory or child tasks.
//!
//! The manual update adapter is experimental. Export/context and installed
//! rollback qualification remain required before production adoption.

#[cfg(test)]
mod tests;
#[cfg(all(feature = "call-ic0", target_arch = "wasm32"))]
mod wasm;

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
#[cfg(all(feature = "call-ic0", target_arch = "wasm32"))]
pub(crate) use wasm::check_update_entry;
#[cfg(all(feature = "call-ic0", target_arch = "wasm32"))]
pub use wasm::{run_global_timer, run_update};

#[cfg(all(feature = "call-ic0", not(target_arch = "wasm32")))]
pub use crate::native::run_update;

pub use crate::execution_types::{EntryError, ReplyError, TaskHandle, TaskState};

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
    method_reply: bool,
}
struct Executor {
    last: u64,
    tasks: BTreeMap<u64, Task>,
    current: Option<TaskToken>,
    synchronous: bool,
}

thread_local! {
    static EXECUTOR: RefCell<Executor> = const { RefCell::new(Executor { last: 0, tasks: BTreeMap::new(), current: None, synchronous: false }) };
}

pub(crate) struct SynchronousEntry;

impl Drop for SynchronousEntry {
    fn drop(&mut self) {
        EXECUTOR.with(|executor| executor.borrow_mut().synchronous = false);
    }
}

pub(crate) fn enter_synchronous() -> Result<SynchronousEntry, EntryError> {
    EXECUTOR.with(|executor| {
        let mut executor = executor.borrow_mut();
        if executor.current.is_some() || executor.synchronous {
            return Err(EntryError::NestedEntry);
        }
        executor.synchronous = true;
        Ok(SynchronousEntry)
    })
}

fn start(future: UserFuture) -> Result<TaskHandle, EntryError> {
    start_root(future, true)
}

fn start_root(future: UserFuture, method_reply: bool) -> Result<TaskHandle, EntryError> {
    let status = Rc::new(Cell::new(TaskState::Running));
    let signal = Arc::new(Signal {
        ready: AtomicBool::new(true),
        live: AtomicBool::new(true),
    });
    // Admission must not drop a refused user future under an executor borrow.
    let token = EXECUTOR.with(|executor| {
        let mut executor = executor.borrow_mut();
        if executor.current.is_some() || executor.synchronous {
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
                method_reply,
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
        if task.stop || !task.method_reply {
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

/// Returns occupied executor entries for the installed accounting fixture.
#[cfg(ic_platform_qualification_accounting)]
#[doc(hidden)]
pub fn observed_tasks() -> usize {
    EXECUTOR.with(|executor| executor.borrow().tasks.len())
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
            if executor.current.is_some() || executor.synchronous {
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
