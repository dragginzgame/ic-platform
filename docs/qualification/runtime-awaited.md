# Experimental awaited IC0 transport

Runtime's wasm32 `call-ic0` feature implements `Call::execute` and
`CallFuture`. This is a transport increment, not the completed Runtime executor.
Applications must not treat it as a qualified replacement for method/task
retention, query/migratory context admission or installed trap recovery.

Construction is inert. First poll checks cost/balance, registers callback custody
and submits once. A synchronous refusal removes the registration and preserves
the raw submission result. Success keeps an owned completion registration
independent of the future and all request borrows. Dropping the future retires
local interest; a later callback still removes its physical registration.

## Identity and borrow discipline

A private safe registry allocates nonzero u32 handles monotonically. Removed
handles are never recycled, and exhaustion refuses with `CallbackCapacity`
before submission. The callback environment is an integer, not an Arc/Rc or
borrowed pointer. Native unit tests cover zero, duplicate/stale IDs and exhaustion
without wrapping. Registry values leave their borrow before destruction.

Reply/reject captures exact bytes or strict UTF-8 text and refund observations
before publication or waking. Refund is unavailable (`None`) for non-replicated
callbacks; zero stays an observed zero. Waker clone, replacement destruction and
wake happen outside the completion lock. The experimental executor resumes only
the task captured at first poll. Cost/balance admission and IC0 assembly are
shared with notify.

From 0.23.1, each future also retains its first polling context locally. A later
poll from another executor task, or a crossing between manual polling and task
polling, traps before waker replacement or response consumption. The callback's
method context cannot follow a moved Rust future. An unpolled future remains
freely movable into its initial task; manual polling remains an advanced path
whose caller owns IC method-context correctness.

## Cleanup boundary

The response callback removes its registration before reading the response.
Replica rollback must restore that registration if the callback traps. Cleanup
then removes it, discards the wake interest and marks the transport cleaned up.
No remote rejection is fabricated. A manually polled surviving future remains
pending after cleanup and cannot resubmit. For an executor-owned call, cleanup
also removes the owning task and drops its future outside executor borrows.
Termination becomes observable only after destruction returns successfully.
Cleanup traps cannot establish committed termination.

The approved IC-only unsafe scope remains confined to private imports/wrappers.
Callback functions, registry and future state transitions are safe Rust. Static
`extern "C" fn(u32)` callback pointers supply the required Wasm table signature;
all input pointers are used only for synchronous IC0 copies. Output buffers are
initialized and read only after the matching copy; malformed UTF-8 traps rather
than silently normalizing rejection text.

## Experimental method-attached executor

`executor::run_update` admits one root future from a manual update export.
It rejects non-replicated and nested entry; the flag alone cannot distinguish
replicated queries from update exports. Generated entrypoint admission remains
pending. No child or migratory tasks, automatic replies or public method-context
handles are supplied by this slice.

Task IDs are monotonic and never recycled. A task-owned wake flag coalesces wakes;
response callbacks drive only their registered task. Dead wake signals cannot
grant authority to another task. Dropping a lifecycle handle does not cancel
the future. Cleanup removes the task before running user destruction, keeps
nested-entry exclusion active through destruction, and publishes termination
afterward. A reentrant termination during polling defers destruction until the
poll returns. Native scope guards are not evidence of IC trap recovery.

Six native regressions cover self-wake coalescing, owner-only polling, stale
wakes, handle drop, cleanup/destructor reentry, deferred destruction and ID
exhaustion. Three additional Wasm tasks exercise actual callback-driven
completion, cleanup termination, a subsequent task after handle drop, nested
entry refusal during polling/destruction, and non-replicated admission refusal.

Four further tasks qualify interleaving through the actual Wasm callback table:
two roots receive replies out of order, the first root submits a second call
from its resumed callback, and each completion preserves its own bytes/refund.
A root with two physical calls is then terminated by cleanup of one call.
The other callback still retires its registration exactly once without polling
the terminated root or a separate live task. That live task subsequently
completes normally. Poll counts, lifecycle handles and a destructor count are
checked between host deliveries, not just at the end.

## Executable evidence

`make qualify-runtime-awaited` compiles a separate `runtime-awaited` binary in
the existing private consumer package and exports its function table solely for
the host fixture. No fixture export or host control API enters product code.
The Node harness invokes actual response/cleanup functions between guest entries
and admits exactly the expected IC0 imports.

The corpus covers unpolled drop, repeated pending polls, exact owned arguments,
bounded and unbounded wait assembly, reply bytes, high-word refund capture,
non-replicated refund absence, SYS_UNKNOWN and unknown rejection codes, delivery
after future drop, cleanup retirement, raw refusal, and a subsequent valid call.
Separate fresh instances check stale response handles, duplicate cleanup and
malformed UTF-8 trapping. Notify's independent harness still forbids callback,
cleanup and wait imports in its final artifact.

The host deliberately does not simulate rollback by restoring memory: that
would not prove replica behavior. Direct cleanup invocation checks transport
retirement only. Real callback traps, cleanup traps, method/task lifetimes,
destructor/waker reentrancy in the installed backend and the singleton executor
remain required installed-canister evidence. The native reentrancy model is
supporting evidence, not a substitute for those checks.

## GitHub snapshot 0.23.0 — 2026-11-15

Rust 1.100.0 / Linux x86-64 and Node 24.21.0 passed 22 native Runtime tests,
strict Clippy/Rustdoc, Wasm test compilation, the fifteen-submission awaited corpus
with seven executor tasks and four trap cases, the independent 28-import notify/cost corpus, graph
admission, formatting and workflow lint. CI includes both ABI targets on its
supported hosts; hosted/macOS results for this increment remain pending.

| Artifact | SHA-256 |
| --- | --- |
| Awaited callback/executor fixture, exported table | `28b14b14084baf8f28c2a21f7a21924b84c96b73d9b081acf7b8251d87becbcc` |
| Notify/cost fixture, no callback imports | `312585dc04078dae7ca34dce320eaa643b199d4c9c41c6bb77f53ccbe3efe1e4` |

## Next development version 0.23.1

The local 0.23.1 increment adds synchronous submission refusal inside a root
task. The task receives the exact raw refusal and completes; a subsequent task
submits and completes normally. Attempting delivery through the refused call's
former callback ID traps, proving that synchronous refusal removed its authority.

The cross-task polling regression first failed against the previous code:
the second task replaced the first task's waker without trapping. With the
ownership check it traps before that replacement and submits no additional call.
The fixture does not treat the trapped Node instance as replica rollback evidence.

`make qualify-runtime-call qualify-runtime-awaited qualify-runtime-ic0` passes
22 native tests, strict Clippy/Rustdoc, Wasm compilation, the expanded corpus of
18 submissions, 11 executor task admissions (including the trapped transfer),
six negative cases, notify/cost checks and dependency graphs on the same
Linux/Rust/Node toolchain. This is local development evidence, not hosted CI or
installed-canister rollback proof.

| Artifact | SHA-256 |
| --- | --- |
| Awaited callback/executor fixture | `4c993ee904d6a724c2b2d22c2d20a8fe30843a53b49fcbc321904e25469a02c1` |
| Notify/cost fixture | `2fe710d19540364ca8687bb86365e2aa796f45a47e7114b7aab533c606325d1a` |
