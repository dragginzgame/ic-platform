# Raw call transport implementation record

On 2026-11-10 a separate implementer context authored the raw transport from
[call-transport-contract.md](call-transport-contract.md), reusing the previously
admitted public [executor contract](executor-contract.md). Existing call source,
old tests, source fingerprints, supplier call implementations and Git history
were excluded.

## Inputs and separation

The only existing repository file newly read for this component was
`docs/design/call-transport-contract.md`, relative to
`/home/adam/projects/ic-platform`. Previously read `docs/design/executor-contract.md`,
`AGENTS.md`, `TOKO-TOOLING.md` and `docs/principles/rust-code-hygiene.md` supplied
public lifecycle and policy context without another read. No other existing
module source was inspected.

The coordinator supplied these supplemental facts without old implementation:

- Executor APIs are reached through `crate::executor`; MethodHandle and
  TaskHandle preserve Send/Sync auto-traits.
- Preflight insufficiency and nonzero perform results require no active method.
  Registration retention is armed only after a zero perform result; callbacks
  cannot run before perform returns, and execution is not multithreaded.
- The associated future retains Debug. Constructors, builder setters and
  `get_cost` retain `#[must_use]`.
- Native unwind must not simulate replica rollback or consume one callback
  pointer twice. Tests should cover multiple outstanding calls, already-cancelled
  cleanup, migratory task custody and externally held futures.

No source files were written before the coordinator's old-owner installed
baseline hold was released. Production files were then authored in full without
reading their previous contents. Later changes followed review of only the new
implementation: documentation/trait annotations and two lint corrections. The
coordinator owns wiring and retirement of the previous owner.

The coordinator separately assigned `future/tests.rs` to another specification-
only test author, who was permitted to read the new implementation and contract.
The implementer coordinated the private seam API but did not read that author's
test source or delegate additional work. No old implementation or tests were
shared between the contexts.

## Authored files

This implementation context wrote:

- `crates/ic-platform-runtime/src/cdk/call/mod.rs`
- `crates/ic-platform-runtime/src/cdk/call/future/mod.rs`
- `crates/ic-platform-runtime/src/cdk/call/system/mod.rs`
- `crates/ic-platform-runtime/src/cdk/call/system/tests.rs`
- `crates/ic-platform-runtime/src/cdk/call/tests.rs`
- `docs/design/call-transport-implementation.md`

The separately authored `call/future/tests.rs` is not claimed as this context's
work. The old owner, manifests, provenance and unrelated modules were untouched
by this implementer.

## Ownership and lock boundaries

Call retains a borrowed method and borrowed/owned raw payload. Its future is
lazy; preflight uses saturated u128 cost, and failed perform remains a typed
error. Oneway builds no callback environment. Request data is separate from the
owned/static completion state crossing the callback ABI.

An Arc-backed completion begins unarmed. One additional Arc reference becomes a
raw callback environment. A local guard owns that reference until successful
perform; refusal reclaims it immediately. Zero perform transfers that ownership
to exactly one committed reply/reject or cleanup. Reply/reject share the same
completion entrypoint. Raw Arc reclamation is centralized and each unsafe ABI
operation has a local ownership argument and lint expectation. Callback-trap
rollback restores the environment for cleanup rather than creating a second
live ownership path.

After successful submission, registration owns the current method, optional
current task and Waker. Pending polls replace only the Waker. State transitions
move displaced Wakers, method handles and results out of the lock before Drop;
Waker cloning also occurs before locking. Delivery stores its result before
resuming the retained method and waking. Recovery cancellation marks completion
cancelled. Cleanup reads no message APIs, stores the synthetic rejection, enters
executor recovery, cancels the method's protected tasks and then the saved task.
Already-cancelled cleanup releases its ownership without recancelling new work.

Normal future destruction retains committed callback ownership. Native unwind
is not treated as IC rollback. No additional executor, scheduling policy, codec,
cryptography, retry permission or dependency was introduced.

## Tests, validation and limits

The private test seam records only supplier-boundary effects and scripted scalar
results/messages; it is not a replica or executor. Its registration declarations
retain the production unsafe function signatures but do not dereference pointers
or invoke callbacks. Eight tests authored here cover borrowed/owned builders,
selection precedence, UTF-8 byte lengths, saturated and wide costs, oneway
ordering/omissions/refusal, lazy unpolled destruction, required traits, response
bytes and public errors. The other author owns completion-lifecycle tests.

The coordinator requested safe Wake wake/drop reentrancy checks and independent
review of clone-before-lock placement. No test-only RawWaker vtable or unsafe
allocator instrumentation was added. Direct injection of arbitrary Waker clone
callbacks is therefore an explicit test limitation; code review checks that
boundary instead.

Rustfmt 1.99.1 completed successfully on authored Rust files using
`--edition 2024 --config-path /dev/null --config skip_children=true` in scoped
batches. This implementer ran no Cargo command, Clippy, compilation, tests,
installed scenarios or size measurements. Coordinator-owned qualifications and
measurements are separate evidence, not implementer-run passes here. No size or
speed improvement is claimed; the owned completion split may alter future and
artifact size.

This record describes one separated context's allowed inputs and actions. It
does not provide legal clearance, claim absence of model training exposure, or
mathematically prove independent authorship. Passing compatibility checks alone
would not establish those conclusions.
