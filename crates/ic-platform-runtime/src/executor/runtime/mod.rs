//! Runtime's thread-local executor and retained method contexts.
//!
//! Protected tasks remain with their originating method. Migratory tasks are
//! admitted only by update execution; neither mode owns application scheduling.

use slotmap::{SlotMap, new_key_type};
use std::{
    cell::RefCell,
    collections::VecDeque,
    future::Future,
    mem::DropGuard,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
};

new_key_type! {
    struct MethodId;
    struct TaskId;
}

type UserFuture = Pin<Box<dyn Future<Output = ()>>>;

#[derive(Clone, Copy)]
struct ExecutionContext {
    method: Option<MethodId>,
    query: bool,
    recovering: bool,
}

struct Method {
    retentions: usize,
    query: bool,
    members: Vec<TaskId>,
    ready: VecDeque<TaskId>,
}

struct Task {
    owner: Option<MethodId>,
    future: Option<UserFuture>,
    waker: Waker,
    queued: bool,
}

struct Executor {
    methods: SlotMap<MethodId, Method>,
    tasks: SlotMap<TaskId, Task>,
    migratory_ready: VecDeque<TaskId>,
    context: Option<ExecutionContext>,
    current_task: Option<TaskId>,
}

impl Executor {
    fn new() -> Self {
        Self {
            methods: SlotMap::with_key(),
            tasks: SlotMap::with_key(),
            migratory_ready: VecDeque::new(),
            context: None,
            current_task: None,
        }
    }

    fn enqueue(&mut self, id: TaskId) {
        let Some(task) = self.tasks.get_mut(id) else {
            return;
        };
        if task.queued {
            return;
        }
        task.queued = true;
        match task.owner {
            Some(owner) => {
                self.methods
                    .get_mut(owner)
                    .expect("a protected task has a live owner")
                    .ready
                    .push_back(id);
            }
            None => self.migratory_ready.push_back(id),
        }
    }

    fn remove_task(&mut self, id: TaskId) -> Option<Task> {
        let task = self.tasks.remove(id)?;
        match task.owner {
            Some(owner) => {
                if let Some(method) = self.methods.get_mut(owner) {
                    if method.members.last() == Some(&id) {
                        method.members.pop();
                    } else {
                        method.members.retain(|member| *member != id);
                    }
                    if task.queued {
                        method.ready.retain(|queued| *queued != id);
                    }
                }
            }
            None => {
                if task.queued {
                    self.migratory_ready.retain(|queued| *queued != id);
                }
            }
        }
        Some(task)
    }

    fn release_method(&mut self, id: MethodId) -> Vec<Task> {
        let Some(method) = self.methods.get_mut(id) else {
            return Vec::new();
        };
        method.retentions -= 1;
        if method.retentions != 0 {
            return Vec::new();
        }
        let method = self.methods.remove(id).expect("the method was present");
        let mut removed = Vec::with_capacity(method.members.len());
        for member in method.members {
            if let Some(task) = self.tasks.remove(member) {
                removed.push(task);
            }
        }
        removed
    }

    fn next_poll(&mut self) -> Option<PollWork> {
        let surrounding = self.context.expect("draining requires a context");
        let protected = surrounding.method.and_then(|id| {
            self.methods
                .get_mut(id)
                .expect("the active method is retained")
                .ready
                .pop_front()
        });
        let id = protected.or_else(|| {
            if surrounding.query {
                None
            } else {
                self.migratory_ready.pop_front()
            }
        })?;
        let task = self.tasks.get_mut(id).expect("queued tasks remain live");
        task.queued = false;
        let future = task
            .future
            .take()
            .expect("a queued task is not being polled");
        let waker = task.waker.clone();
        let method = task.owner;
        let query = method.is_some_and(|id| {
            self.methods
                .get(id)
                .expect("the task owner is retained")
                .query
        });
        self.context = Some(ExecutionContext {
            method,
            query,
            recovering: false,
        });
        self.current_task = Some(id);
        Some(PollWork {
            id,
            future,
            waker,
            surrounding,
        })
    }
}

thread_local! {
    static EXECUTOR: RefCell<Executor> = RefCell::new(Executor::new());
}

struct TaskWake {
    id: TaskId,
}

impl TaskWake {
    fn enqueue(&self) {
        let _ = EXECUTOR.try_with(|executor| {
            executor.borrow_mut().enqueue(self.id);
        });
    }
}

impl Wake for TaskWake {
    fn wake(self: Arc<Self>) {
        self.enqueue();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.enqueue();
    }
}

/// Retains a tracked method, or the untracked context of a migratory poll.
///
/// Dropping the last retention outside execution cancels that method's remaining
/// protected tasks. Execution state is local to each native thread.
#[derive(Debug)]
pub struct MethodHandle {
    method: Option<MethodId>,
}

impl MethodHandle {
    fn new(method: Option<MethodId>) -> Self {
        Self { method }
    }
}

impl Drop for MethodHandle {
    fn drop(&mut self) {
        let Some(id) = self.method else {
            return;
        };
        let removed = EXECUTOR
            .try_with(|executor| executor.borrow_mut().release_method(id))
            .unwrap_or_default();
        // All table borrows ended before invoking any future destructor.
        for task in removed {
            drop(task);
        }
    }
}

/// Identifies one task generation for cancellation without retaining its method.
#[derive(Debug)]
pub struct TaskHandle {
    id: TaskId,
}

impl TaskHandle {
    fn new(id: TaskId) -> Self {
        Self { id }
    }

    /// Returns a handle only while a task's Future is being polled.
    #[must_use]
    pub fn current() -> Option<Self> {
        EXECUTOR.with(|executor| executor.borrow().current_task.map(Self::new))
    }
}

struct PollWork {
    id: TaskId,
    future: UserFuture,
    waker: Waker,
    surrounding: ExecutionContext,
}

struct PollGuard {
    id: TaskId,
    surrounding: ExecutionContext,
    finished: bool,
}

impl PollGuard {
    fn finish(&mut self, future: UserFuture, result: Poll<()>) {
        let mut future = Some(future);
        let removed = EXECUTOR.with(|executor| {
            let mut executor = executor.borrow_mut();
            executor.context = Some(self.surrounding);
            executor.current_task = None;
            match result {
                Poll::Ready(()) => executor.remove_task(self.id),
                Poll::Pending => {
                    if let Some(task) = executor.tasks.get_mut(self.id) {
                        task.future = future.take();
                    }
                    None
                }
            }
        });
        self.finished = true;
        drop(removed);
        // Completion and cancellation during poll both leave the local future
        // here. Its Drop observes the surrounding context and no current task.
        drop(future);
    }
}

impl Drop for PollGuard {
    fn drop(&mut self) {
        if !self.finished {
            // Native unwinding restores bookkeeping. IC traps instead roll back
            // the message and enter the separately requested recovery callback.
            let removed = EXECUTOR.with(|executor| {
                let mut executor = executor.borrow_mut();
                executor.context = Some(self.surrounding);
                executor.current_task = None;
                executor.remove_task(self.id)
            });
            drop(removed);
        }
    }
}

fn drain() {
    loop {
        let Some(mut work) = EXECUTOR.with(|executor| executor.borrow_mut().next_poll()) else {
            return;
        };
        let mut guard = PollGuard {
            id: work.id,
            surrounding: work.surrounding,
            finished: false,
        };
        let result = work
            .future
            .as_mut()
            .poll(&mut Context::from_waker(&work.waker));
        guard.finish(work.future, result);
    }
}

fn require_no_context() {
    EXECUTOR.with(|executor| {
        assert!(
            executor.borrow().context.is_none(),
            "executor contexts cannot nest"
        );
    });
}

fn run_context<R>(method: MethodHandle, recovering: bool, f: impl FnOnce() -> R) -> R {
    install_panic_hook();
    require_no_context();
    EXECUTOR.with(|executor| {
        let mut executor = executor.borrow_mut();
        let query = method.method.is_some_and(|id| {
            executor
                .methods
                .get(id)
                .expect("a retained method is live")
                .query
        });
        executor.context = Some(ExecutionContext {
            method: method.method,
            query,
            recovering,
        });
    });
    let guard = DropGuard::new(method, |method| {
        EXECUTOR.with(|executor| {
            let mut executor = executor.borrow_mut();
            executor.context = None;
            executor.current_task = None;
        });
        // Final automatic reclamation deliberately occurs outside the context.
        drop(method);
    });
    let result = f();
    if !recovering {
        drain();
    }
    drop(guard);
    result
}

fn track<R>(query: bool, f: impl FnOnce() -> R) -> R {
    install_panic_hook();
    require_no_context();
    let id = EXECUTOR.with(|executor| {
        executor.borrow_mut().methods.insert(Method {
            retentions: 1,
            query,
            members: Vec::new(),
            ready: VecDeque::new(),
        })
    });
    run_context(MethodHandle::new(Some(id)), false, f)
}

/// Executes an update closure in a new tracked method and drains eligible tasks.
///
/// # Panics
/// Panics if an executor context is already active.
pub fn in_tracking_executor_context<R>(f: impl FnOnce() -> R) -> R {
    track(false, f)
}

/// Executes a query closure and drains only its protected tasks.
///
/// # Panics
/// Panics if an executor context is already active.
pub fn in_tracking_query_executor_context<R>(f: impl FnOnce() -> R) -> R {
    track(true, f)
}

/// Resumes the retained method, retaining its update/query eligibility.
///
/// # Panics
/// Panics if an executor context is already active.
pub fn in_callback_executor_context_for<R>(method: MethodHandle, f: impl FnOnce() -> R) -> R {
    run_context(method, false, f)
}

/// Enters explicit recovery without draining wakeups; the caller owns cancellation.
///
/// # Panics
/// Panics if an executor context is already active.
pub fn in_trap_recovery_context_for<R>(method: MethodHandle, f: impl FnOnce() -> R) -> R {
    run_context(method, true, f)
}

/// Retains the current tracked method or current untracked migratory context.
///
/// # Panics
/// Panics outside an executor context.
#[must_use]
pub fn extend_current_method_context() -> MethodHandle {
    let method = EXECUTOR.with(|executor| {
        let mut executor = executor.borrow_mut();
        let method = executor
            .context
            .expect("method retention requires a context")
            .method;
        if let Some(id) = method {
            executor
                .methods
                .get_mut(id)
                .expect("the active method is live")
                .retentions += 1;
        }
        method
    });
    MethodHandle::new(method)
}

/// Reports whether explicit trap recovery is active on this native thread.
#[must_use]
pub fn is_recovering_from_trap() -> bool {
    EXECUTOR.with(|executor| {
        executor
            .borrow()
            .context
            .is_some_and(|context| context.recovering)
    })
}

/// Cancels one task generation, including during its own poll.
///
/// Repeated cancellation and handles for already completed tasks have no effect.
pub fn cancel_task(handle: &TaskHandle) {
    let removed = EXECUTOR.with(|executor| executor.borrow_mut().remove_task(handle.id));
    drop(removed);
}

/// Cancels the current method's protected tasks in their creation order.
///
/// The old batch is detached before destructors run, so destructor-spawned tasks
/// survive this operation. An untracked context has no protected tasks.
///
/// # Panics
/// Panics outside an executor context.
pub fn cancel_all_tasks_attached_to_current_method() {
    let removed = EXECUTOR.with(|executor| {
        let mut executor = executor.borrow_mut();
        let context = executor
            .context
            .expect("bulk cancellation requires a context");
        let Some(owner) = context.method else {
            return Vec::new();
        };
        let method = executor
            .methods
            .get_mut(owner)
            .expect("the active method is live");
        let members = std::mem::take(&mut method.members);
        method.ready.clear();
        let mut removed = Vec::with_capacity(members.len());
        for id in members {
            if let Some(task) = executor.tasks.remove(id) {
                removed.push(task);
            }
        }
        removed
    });
    for task in removed {
        drop(task);
    }
}

fn spawn(future: UserFuture, protected: bool) -> TaskHandle {
    let owner = EXECUTOR.with(|executor| {
        let executor = executor.borrow();
        let context = executor.context.expect("spawning requires a context");
        assert!(
            !context.recovering,
            "spawning is forbidden during trap recovery"
        );
        if protected {
            Some(
                context
                    .method
                    .expect("protected tasks require a tracked method"),
            )
        } else {
            assert!(!context.query, "queries cannot spawn migratory tasks");
            None
        }
    });
    let id = EXECUTOR.with(|executor| {
        let mut executor = executor.borrow_mut();
        let id = executor.tasks.insert_with_key(|id| Task {
            owner,
            future: Some(future),
            waker: Waker::from(Arc::new(TaskWake { id })),
            queued: false,
        });
        if let Some(owner) = owner {
            executor
                .methods
                .get_mut(owner)
                .expect("the owner is retained")
                .members
                .push(id);
        }
        executor.enqueue(id);
        id
    });
    TaskHandle::new(id)
}

/// Queues a task that remains attached to its originating tracked method.
///
/// # Panics
/// Panics without a tracked context or during explicit trap recovery.
pub fn spawn_protected(f: impl Future<Output = ()> + 'static) -> TaskHandle {
    spawn(Box::pin(f), true)
}

/// Queues a task that can survive its spawning method and run in later updates.
///
/// # Panics
/// Panics outside execution, in a query, or during explicit trap recovery.
pub fn spawn_migratory(f: impl Future<Output = ()> + 'static) -> TaskHandle {
    spawn(Box::pin(f), false)
}

#[cfg(target_family = "wasm")]
fn install_panic_hook() {
    static INSTALL: std::sync::Once = std::sync::Once::new();
    INSTALL.call_once(|| {
        std::panic::set_hook(Box::new(|info| {
            let payload = if let Some(text) = info.payload().downcast_ref::<&str>() {
                *text
            } else if let Some(text) = info.payload().downcast_ref::<String>() {
                text.as_str()
            } else {
                "non-string panic payload"
            };
            let report = match info.location() {
                Some(location) => format!(
                    "{payload} at {}:{}:{}",
                    location.file(),
                    location.line(),
                    location.column()
                ),
                None => format!("{payload} at an unknown source location"),
            };
            crate::ic0::upstream::debug_print(report.as_bytes());
            crate::ic0::upstream::trap(report.as_bytes());
        }));
    });
}

#[cfg(not(target_family = "wasm"))]
fn install_panic_hook() {}

#[cfg(test)]
mod tests;
