# Manual update admission — Runtime 0.23.3

`entrypoints::update_manual` composes the existing guarded input path with the
single method-attached executor. It is intended for a manually declared update
export; it generates no export, selects no encoder and sends no reply.

The order is context/capacity preflight, caller guard, bounded raw capture,
selected decoder, final executor admission, then handler invocation and polling.
Input and handler must survive callback suspension. The handler is invoked from
inside the admitted async task, so its construction code observes that task's
context. Rejected admission never invokes the handler.

`UpdateError::Entry` retains executor refusal and `UpdateError::Admission`
retains the exact guard, size or decoder error. No failure becomes an encoded
reply or retry decision. Final admission repeats the preflight because guard
and decoder code may have side effects; no context reservation spans that user
code. User closures remain responsible for their own behavior and destruction.

Non-replicated and nested entry are refused before invoking the guard or reading
arguments. The replicated flag still cannot distinguish a replicated query from
an update export. Correct export binding, single reply ownership, unsupported
async/query modes and installed rollback remain outstanding generator/qualification
obligations. A completed task does not establish that the caller received a reply.

## Executable evidence

The private `runtime-entrypoints` binary uses the public API. Its host accepts
only the context flag and argument size/copy imports, verifies initialized copy
buffers and asserts the complete import sequence. Six cases cover guard refusal,
oversize refusal, decoder refusal, successful owned bytes, nested entry and
non-replicated entry. Refusal handlers panic if invoked; the successful handler
runs exactly once and attempts nested entry during its construction to verify
that construction is already task-owned. Three size reads and two copies occur.

`make qualify-runtime-call qualify-runtime-ic0 qualify-runtime-awaited` passes
24 native Runtime tests, strict Clippy/Rustdoc, Wasm compilation, the new corpus,
the ten argument-admission cases, existing notify/callback checks and dependency
graphs on Linux with Rust 1.100.0 and Node 24.21.0.

The awaited fixture also exercises three tasks through `update_manual` itself.
The host supplies method input at entry, then distinct reply bytes during the
callback. The handler retains decoded input across suspension and checks exact
reply/refund observations. Guard, decoder and handler counters must each advance
once per admitted task. Cleanup of the second task drops its suspended handler
once without advancing the completion counter; a third update subsequently
completes. Each input is copied once and is never reread in the callback.

The expanded awaited corpus passes with 21 submissions, 14 task admissions and
six negative cases. Cleanup is invoked directly by the host, not as a simulated
replica rollback. Installed export/context and rollback proof remains pending.

## Explicit manual replies

`system::reply(bytes)` claims the live task's ordinary reply authority, releases
the executor borrow, then appends the exact bytes and issues the system reply.
Empty bytes skip append but still issue a reply. The operation retains no input
borrow and selects no codec. A successful return depends on the enclosing IC
execution committing; it is not a delivery guarantee or task cancellation.

`ReplyError::NoActiveTask` refuses outside a live task and during cleanup or
retirement. `AlreadyReplied` refuses a second ordinary reply from the same task,
including after callback suspension. Errors invoke neither reply import. The
authority flag and system effects must roll back together if execution traps;
the host does not prove that replica property. External raw IC0 or creating
multiple manual roots for one export can bypass this ordinary-path ownership;
generated entrypoints must still establish one root/reply owner per method.

The immediate adapter fixture checks binary and empty replies, no reply outside
a task and duplicate refusal. The awaited fixture replies with retained input
after each successful update callback and rejects a second attempt. Handler
destruction also attempts a reply: ordinary completion sees `AlreadyReplied`,
while cleanup sees `NoActiveTask`. Exact host traces contain only the intended
two callback replies and no cleanup/duplicate imports. A native regression checks
fresh authority in separate tasks and refusal after retirement, bringing native
Runtime coverage to 25 tests. Strict Clippy/Rustdoc, all Runtime ABI corpora and
dependency checks pass on the local Linux toolchain.

## Compiler-enforced input contracts

`make qualify-runtime-input-ui` compiles maintained fixture files for wasm32
through a renamed `renamed-runtime` dependency. Owned decoded input with a static
handler compiles. Invalid decoder arguments fail with E0631; decoded input
borrowing entry-local storage fails with E0597; handler construction borrowing
entry-local state fails with E0373. The driver checks structured Cargo/rustc
diagnostics for the expected code and primary fixture span, plus build status.
The positive control prevents missing imports or dependencies from counting as
negative-contract proof. Temporary manifests reference the maintained source
files; no Rust source is generated or patched. All probes run offline, are
selected by CI and preserve qualification's unsafe-code prohibition.

| Artifact | SHA-256 |
| --- | --- |
| Manual update fixture | `ae3c1792e96e644e422c7923d2cdd3829fd4c6e52f8141a956813505da61ae64` |
| Arguments fixture | `dd3bee510d3e9baee3fcdac4bd8cf8f716ea3c3fae102a50b2508346193ddec6` |
| Awaited fixture | `ca222a381ea6c63f5945298ec7e956880f76968d42c6183497f452daf9e9e24b` |
| Notify/cost fixture | `23ba40a8171693d6e1626aa1ab1cced269a0bdd19f28cc5e11fe15a139524c27` |
