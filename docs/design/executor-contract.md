# Executor replacement contract

Specification for a separate implementation context. Read this contract and the
public observation transcript only, plus previously admitted policy. Do not read
the existing executor source, old tests, provenance body hashes, supplier source,
Git history or patches. This is one replacement of Runtime's existing owner, not
an additional executor, scheduler, timer, transport or public abstraction.

## Public API

Implement these names in the existing private executor/runtime module; its parent
exports them explicitly. Keep only the current public surface:

- `in_tracking_executor_context<R>(f: impl FnOnce() -> R) -> R`
- `in_tracking_query_executor_context<R>(f: impl FnOnce() -> R) -> R`
- `in_callback_executor_context_for<R>(method: MethodHandle, f: impl FnOnce() -> R) -> R`
- `in_trap_recovery_context_for<R>(method: MethodHandle, f: impl FnOnce() -> R) -> R`
- `extend_current_method_context() -> MethodHandle`
- `is_recovering_from_trap() -> bool`
- `cancel_all_tasks_attached_to_current_method()`
- `cancel_task(handle: &TaskHandle)`
- `spawn_protected(f: impl Future<Output = ()> + 'static) -> TaskHandle`
- `spawn_migratory(f: impl Future<Output = ()> + 'static) -> TaskHandle`
- opaque public `MethodHandle` and `TaskHandle`, both Debug, neither Clone/Copy;
  `TaskHandle::current() -> Option<Self>`.

Both existing handle types have Send + Sync auto-traits; preserve those traits.
This does not promise meaningful cross-thread execution or wake behavior.

No external module relies on private executor data structures. Method/task handles
retain Runtime identity. New private structures are your choice subject to the
invariants below. No public telemetry, executor constructor, reset or poll API.

## Observable scheduling contract

Each native thread has independent executor state; canisters are single-threaded.
A tracking update/query context establishes a fresh method, executes its closure,
then drains eligible wakeups before returning the closure's original value.
A callback consumes a retained MethodHandle and resumes its original method.
Contexts cannot nest, including invocation inside a polled future: panic on
entry. No new cross-thread wake semantics or multithreading support is needed.

A protected task belongs to its originating method for its whole lifetime.
It is initially queued but not polled immediately within spawn. Wakeups from
another method do not migrate it: it runs only when its owner resumes. This must
remain true for wakeups issued during the task's own poll. A migratory task is
not owned by a method; it survives its spawning method and can be polled at a
later update context. Migratory polling has no tracked method: protected spawning
there must panic, and a handle retained there carries that untracked context.
Such a handle may resume an update-style callback without creating a tracked
owner. Queries cannot create migratory tasks or drain existing migratory wakeups.

Eligible protected wakeups have priority over migratory wakeups. Within either
queue, preserve FIFO admission. Drain until there is no eligible wakeup, including
wakes/spawns caused by polling. Waking never recursively polls. Multiple wakes
before the next poll coalesce; a wake during polling schedules a later poll
without losing it. Reuse the task's Waker identity (`will_wake`) across polls.
A finished/cancelled task or stale Waker must never address a later task whose
slot is reused. Retain the existing SlotMap generational dependency and Vec task
membership decision; do not add SmallVec/TinyVec or another dependency.

`TaskHandle::current()` is Some only during the poll of that task. A current
handle may cancel its own task. If a future returns Pending after self-cancel,
that future must be dropped exactly once and never polled again. Completion,
cancellation and stale handles must not leave growing live task/membership state.

## Lifetime, cancellation and recovery

A method stays available through retained MethodHandles. Extending the current
context produces a fresh retention handle. On ordinary context exit, when no
handles retain that method, all its protected tasks are cancelled and method
bookkeeping is reclaimed. Migratory tasks are unaffected. Polling a protected
task uses its original method, so retention inside the task retains that method.
Automatic final method reclamation runs after leaving the active context: task
destructors there observe no active context, so spawning must panic. This differs
from explicit bulk cancellation inside a context. A retained query handle
continues to enforce query eligibility in subsequent callbacks/recovery.

Individual cancellation is idempotent and works inside or outside a context.
Bulk cancellation requires a context, cancels its protected tasks, and preserves
stable creation order among surviving members. Futures can execute arbitrary
code in Drop, including spawning work. Never poll or drop a user future while
holding a RefCell borrow of any executor table/queue. Clear the old bulk-cancel
work before running destructors: destructor-spawned tasks survive that bulk
operation and their new wakeups are not erased. Explicit recovery forbids all
spawns, including from destructors. Recovery enters the retained method with
`is_recovering_from_trap() == true`, executes the closure and does not drain
wakeups. Its caller controls cancellation. Normal execution reports false.

Method extension, both spawn functions and bulk cancellation require an active
context; protected spawn additionally requires a tracked method. Migratory spawn
requires update/untracked execution, never query. Misuse panics rather than
silently changing task policy. Exact panic text is not a protocol requirement.
Native tests may use fresh threads for misuse cases; do not rely on unwinding
as the IC trap model. On IC traps the replica rolls back the message, and the
separate cleanup callback enters explicit recovery. Preserve that lifecycle.

## IC panic adapter

Only Wasm installs a process panic hook, once. Delegate to the existing internal
IC0 alias APIs `crate::ic0::upstream::debug_print(&[u8])` and `trap(&[u8]) -> !`.
Report payload plus source location, print it then trap; native execution must
not replace the process's panic hook or invoke IC0. Add no unsafe code. Format
and string plumbing may be independently written; cryptography is not involved.

## Test and source custody

Write executor/runtime/mod.rs and a fresh executor/runtime/tests.rs only; the
parent removes the old tests directory after integration. Do not read old tests.
Write docs/design/executor-implementation.md with inputs and validation limits.
Use existing SlotMap, std Future/Wake/Waker/collections and Rust 2024; no new
package, feature or dependency. New internals/tests may be organized with private
helpers inside these ordinary module files. Keep scope narrow.

Tests should independently exercise foreign-method wake routing, protected/query
self-wakes, duplicate wakes, stale generation reuse, current handles/self-cancel,
method retention/final release, migratory survival/query isolation, stable drop
ordering and destructor reentrancy/bulk-spawn survival. Add bounded state-stress
checks for completed/cancelled task reclamation using your own private internals.
Do not implement a second reference executor. Parent owns Clippy, tests and
installed raw/prepared-call callback-trap/upgrade/cancellation qualification.
Formatting may be run with skip_children on only authored files. No Cargo while
parent is capturing existing behavior. Record no performance claim without
measurement. This input separation does not by itself establish legal clearance.
