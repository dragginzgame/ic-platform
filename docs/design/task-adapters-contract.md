# Task policy adapter replacement contract

Separate specification-only context: do not read existing CDK adapters, their
old tests, supplier code, history or diffs. Public executor APIs are already
specified by docs/design/executor-contract.md; use that surface without reading
implementation. Write ONLY new crates/ic-platform-runtime/src/cdk/futures/mod.rs,
futures/tests.rs, futures/internals/mod.rs and
`docs/design/task-adapters-implementation.md`. Parent integrates modules/guards.

Preserve four public functions:
- spawn<F: 'static + Future<Output=()>>(future: F) -> ()
- spawn_weak<F: 'static + Future<Output=()>>(future: F) -> ()
- spawn_migratory<F: 'static + Future<Output=()>>(future: F) -> ()
- is_recovering_from_trap() -> bool

Both protected/weak use crate::executor::spawn_protected. Migratory uses that
executor's spawn_migratory. Recovery query delegates directly. Discard task
handles: wrappers do not return cancellation authority or own a scheduler.

Weak tasks may be cancelled before Ready and simply drop their inner future.
Protected spawn adds this policy: if its inner future has not returned Ready(())
when it is destroyed, panic unless executor::is_recovering_from_trap() is true.
The panic reason is exactly:
`protected task outlived its canister method (did you mean to use spawn_weak or spawn_migratory?)`
This includes never-polled futures and cancellation during poll. Mark completion
only after inner Ready, never on Pending or before polling. Recovery cancellation
must not produce a second panic. Inner future must drop exactly once, remain
properly pinned even if !Unpin, and use the same executor/waker/context as other
tasks. No Candid, transport, retry, budget or implicit context may be added.

Implement without unsafe, with no extra box/allocation beyond the executor's
existing task allocation. Prefer a small completion guard with ordinary Rust
async machinery if it preserves never-polled drop semantics and pinning. The
existing pin-project-lite dependency may be used if necessary, but parent will
remove it if no remaining source needs it. No additional dependencies.

Public internals module has only these functions, backed by the corresponding
executor context API:
- in_executor_context<R>(f: impl FnOnce()->R)->R: tracking update
- in_query_executor_context<R>(f: impl FnOnce()->R)->R: tracking query

Explicit named aliases/re-exports of those functions are fine; no blanket export
or second context implementation. Preserve nested-context refusal and polling.

Fresh tests must cover completed protected tasks, never-polled and Pending
protected task cancellation outside recovery, weak cancellation, recovery
cancellation of unfinished protected task, query context/migratory refusal,
self-wake/current context, and !Unpin futures. Avoid double-panic-abort test
patterns: protected misuse outside any context may panic during construction and
again during destructor unwinding, so test policy under a valid context or a
private policy helper. Use fresh threads for independent panic cases; no unsafe
pin construction or test hooks in the executor. Tests may use public task handles
and retention APIs. Do not read existing implementation/tests. Run formatting
only; parent owns compilation and installed callback/trap tests. Record input
separation/limits without legal-clearance claims.
