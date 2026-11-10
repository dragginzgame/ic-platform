# Executor implementation record

On 2026-11-10 a separate implementer context authored Runtime's executor module
from the [reviewed contract](executor-contract.md) and the reviewer's
[public observations](executor-api-observations.txt). This replaces the existing
Runtime implementation in place; it does not add a second executor.

## Inputs and separation

The existing repository inputs read for this component, relative to
`/home/adam/projects/ic-platform`, were only:

- `docs/design/executor-contract.md`
- `docs/design/executor-api-observations.txt`

Previously read `AGENTS.md`, `TOKO-TOOLING.md` and
`docs/principles/rust-code-hygiene.md` supplied policy. They were not reread.
The implementer did not read prior executor source or tests, provenance body
hashes, upstream implementation, Git history or patches. No observations or
source from other replacement components were used as executor evidence.

The reviewer supplied these additional contract/API clarifications without
previous implementation text:

- Final automatic context-exit reclamation occurs after leaving the context;
  destructors there cannot spawn. Explicit bulk cancellation inside a context
  retains that context and preserves destructor-spawned tasks.
- A retained query method keeps query eligibility during callbacks/recovery.
- `TaskHandle::current()` applies only during polling. Completion destruction
  runs after restoring the surrounding context and clearing the current task.
- Selected SlotMap APIs include private `new_key_type!` keys, `with_key`,
  `insert_with_key`, `get`, `get_mut`, `remove`, `len` and `is_empty`.
- MethodHandle and TaskHandle retain their existing Send/Sync auto-traits;
  no native cross-thread support or identity-token mechanism is requested.
- Removal should preserve the constant-time last-member case and avoid scanning
  a ready queue for a task that is not queued. The IC panic hook applies to the
  Wasm target family.

The reviewer separately assigned public lifecycle tests to another specification-
only context. This implementer did not delegate work or read that test author's
source; it retained private-state tests and the implementation record.

A process-name check before editing found no Cargo/rustc process. The initial
module and tests were written in full without reading their previous contents.
Subsequent edits affected only newly authored code, including review-requested
membership-removal and Wasm-gating adjustments.

## Authored files and lifecycle

- `crates/ic-platform-runtime/src/executor/runtime/mod.rs`
- `crates/ic-platform-runtime/src/executor/runtime/tests.rs`
- `docs/design/executor-implementation.md`

One thread-local executor owns generational SlotMap task/method records. Methods
retain Vec membership in creation order and their own FIFO ready queues;
migratory work has a separate FIFO queue. Protected eligibility is checked before
migratory work on every drain iteration. Query contexts exclude migratory work.
Each task caches one Arc-backed Waker keyed by its generation. Queue admission
coalesces duplicate wakes and remains available while the future is being polled.

Each active context holds one method retention. External handles add retentions;
callback/recovery entry consumes its handle as the active retention. Untracked
handles retain no method. Recovery runs its closure without draining and rejects
spawns. Root context exit clears execution state before releasing its retention.
An explicit final-handle drop within a different surrounding context runs the
cancelled futures' destructors in that surrounding context; the reviewer
confirmed that no temporary suppression is required for that case.

The reviewer later reported a behavioral difference from a captured public probe
of the prior implementation: dropping the final MethodHandle outside a context
previously left its pending task unreclaimed. The replacement drops that task
exactly once immediately, as required by the supplied lifetime contract. The
reviewer explicitly retained this cleanup correction. This is an intentional
semantic correction, not a claim of full observational equivalence with the
previous implementation.

The future is extracted before polling. A poll guard restores the surrounding
context and clears the current-task handle before completion/self-cancellation
destruction. Native unwind cleanup removes the interrupted task and restores
bookkeeping; this is not a replacement for the IC's message rollback and explicit
cleanup callback model.

Cancellation removes records and their queue/member references before running
future destructors. Bulk cancellation detaches the entire old membership and
ready queue first, then drops the batch in creation order. Destructor-spawned
work therefore enters a new live batch. Individual member removal preserves
survivor order, with a direct Vec pop for the last member; nonqueued removals do
not scan ready queues. No user future is intentionally polled or dropped under
an executor RefCell borrow.

Only Wasm installs the once-only panic hook. It reports payload and source
location through the existing IC0 debug-print and trap APIs. Native execution
does not install a hook or invoke IC0. No unsafe code, dependency, package,
feature, public telemetry or executor-construction/reset API was added.

Five private-state test functions cover repeated completion reclamation, stable
member removal and bulk destructor reentrancy, Pending after self-cancellation,
stale generation/waker refusal, and final method-retention release. They use
fresh native threads and inspect only the newly authored internals. The separate
public lifecycle and installed-runtime qualifications remain reviewer-owned.

## Validation and limitations

Formatting completed successfully with no output using:

```text
rustfmt +1.99.1 --edition 2024 --config-path /dev/null --config skip_children=true crates/ic-platform-runtime/src/executor/runtime/mod.rs crates/ic-platform-runtime/src/executor/runtime/tests.rs
```

The explicit empty configuration and `skip_children` restricted formatting to
these authored files. The implementer ran no Cargo command, Clippy, compilation,
tests, installed IC scenarios or performance measurements. Parent-owned review,
public test execution, matched artifact measurements and installed qualification
are separate evidence; none is claimed as an implementer-run pass here.

Parent integration owns removal of prior test files and applicable source guards,
licenses and provenance updates. No implementation or performance claim extends
beyond the recorded inputs and checks. This record does not provide legal
clearance, claim absence of model training exposure, or mathematically prove
independent authorship. Compatibility evidence alone cannot establish those
conclusions.
