# Call test inputs and validation custody

## Original public-test phase

The public-test author read `docs/design/call-transport-contract.md` and used
previously admitted policy. Earlier tasks in this same context admitted
`AGENTS.md`, `TOKO-TOOLING.md`, the Rust hygiene guide, executor contract and
observation transcript, and task-adapter contract. The author previously wrote
independent executor public tests and task adapters; neither old nor replacement
call implementations or their private tests were exposed during this phase.

The coordinator supplied factual public API details absent from the call
contract: `ic_platform_runtime::Principal::{anonymous, management_canister}`
and the six `RejectCode` variant names and mappings. No supplier bodies, source
fingerprints, Git history, diffs or additional repository sources were read.

The original deliverables were
`crates/ic-platform-runtime/tests/call_consumer.rs` and this record. Seven tests
check public trait identities, borrowed/owned lazy builders, cloning and
unpolled destruction, bounded timeout extrema, repeated builder selection,
exact error displays, From variants, transparent source delegation, all six
known rejection mappings, and representative unknown raw u32 values. Rejection
text includes empty, Unicode, newline and NUL cases. They also check Response
byte-access trait signatures without constructing one. The coordinator reports
these original seven tests passed unchanged against both call owners.

The public tests do not call `get_cost`, `oneway`, or poll a call future. They
do not qualify submission, callbacks, transmitted bytes, builder field values,
timeout application, cycle charging or recovery. Public builders have no field
getters; admission tests do not prove last-selection semantics or absence of
internal copying. Unknown-code coverage includes range boundaries, not all u32
values. Send/Sync checks do not authorize cross-thread execution.

## Subsequent replacement-source-informed lifecycle phase

The coordinator later explicitly authorized reading only the newly written
`cdk/call/mod.rs`, `cdk/call/future/mod.rs` and `cdk/call/system/mod.rs`, plus the
contract. Those new implementation files were read. No old call source, old
private tests, supplier implementation, source fingerprint, history or diff was
exposed. The independently authored system-seam owner supplied its test API and
ownership protocol through coordination messages; its source was not read.

This phase authored `cdk/call/future/tests.rs`. These private tests are informed
by the replacement source, not specification-only independent observations.
They cover failed submission reclamation, ordinary pending destruction and
method retention, latest-Waker replacement, outside-lock wake/destruction,
reentrant submission, multiple-call recovery, cancelled cleanup preserving
later tasks, cancelled delivery refusal, external synthetic cleanup, original
migratory-task cancellation after another task repolls, and raw rejection text.
A test-only Completion destructor counter measures reclamation without changing
production code. Safe `Wake` implementations exercise wake and final-drop
reentrancy; arbitrary custom RawWaker clone behavior is not executed.

Each successful mock environment is consumed once. Tests first upgrade an
independently saved Weak reference, then reclaim the queued raw reference using
the replacement's Environment guard, and invoke ordinary delivery/cleanup
helpers with the retained Arc. This qualifies the guard and safe lifecycle
helpers without adding test unsafe, unwinding through extern functions, or
reusing consumed pointers to imitate replica rollback. Installed qualification
remains responsible for the actual callback ABI and replica rollback behavior.

The coordinator also supplied the observed public requirement that CallFuture
implements Debug. After new-source exposure, the author added that trait bound
to the existing public test; the seven original public cases were not otherwise
changed. This additional assertion is a coordinator-observed interface check,
not an assertion derived from the initial contract alone.

## Validation limits

The author ran formatting only with compiler 1.99.1, an empty configuration and
child traversal disabled. The coordinator owns manifest registration, Clippy,
native execution, and installed raw/prepared-call qualification. New private
checks and the additional Debug bound await coordinator execution at handoff.
No performance or legal-clearance claim follows from this record.
