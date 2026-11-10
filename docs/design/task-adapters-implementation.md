# Independent task policy adapters

This author worked in a specification-only context. The task-specific input was
`docs/design/task-adapters-contract.md`; the previously admitted public executor
contract, observation transcript, `AGENTS.md`, `TOKO-TOOLING.md` and Rust hygiene
guide remained available. The same author previously wrote public executor
consumer tests from those specifications, without implementation exposure.

No old CDK adapter bodies or tests, supplier source, executor implementation,
Git history, diffs or other repository sources were inspected. The author wrote
new `cdk/futures/mod.rs`, `cdk/futures/tests.rs`, `cdk/futures/internals/mod.rs`,
this record, and a disposable alternative module for coordinator comparison.
The coordinator integrated the selected alternative into the maintained module.

## Selected implementation

Protected spawning wraps its inner future directly in a generic future with a
completion flag. The existing `pin-project-lite` supplier provides safe pin
projection and `PinnedDrop`; only the inner future is structurally pinned. The
wrapper records completion immediately when the inner poll returns Ready,
before any inner destructor can run. Its pinned destructor enforces the exact
protected-task cancellation panic unless completion or explicit trap recovery
permits destruction. This covers never-polled and Pending cancellation while
preserving the original panic if a completed inner future's destructor panics.

Weak and migratory tasks delegate directly, as do recovery queries and the two
named context aliases. No unsafe code was authored, and no extra allocation,
scheduler or public abstraction was added. Pin projection remains supplied by
`pin-project-lite`; it is not independently reimplemented here.

Fresh tests cover completed, never-polled, Pending and self-cancelled protected
tasks; exactly-once inner destruction and original completed-future destructor
panic preservation; weak cancellation; recovery cancellation; query and nested
context refusal; and self-waking non-Unpin futures with stable poll addresses
and public current-task access. Independent panic cases use fresh native
threads. The tests do not treat native unwinding as replica trap rollback.

## Superseded async experiment

The initial alternative constructed a completion guard before an async wrapper.
Coordinator execution exposed a disjoint-field capture defect despite passing
Clippy: accessing only the completion field could leave the guard outside the
async state, causing it to drop at spawn return. Borrowing the whole guard via a
completion method fixed capture. A separate ordering review replaced direct
`future.await` with standard `pin!` and `poll_fn`, marking completion on Ready
before a potentially panicking inner destructor. That corrected async candidate
passed qualification, but is superseded by the direct pinned-future wrapper.
The author did not execute either defective version.

## Validation custody

The author ran formatting only, using compiler 1.99.1, an empty configuration
and disabled child traversal. Compilation, Clippy, runtime execution, installed
qualification and artifact comparison belong to the integration coordinator.

The coordinator reports the selected implementation passed 39 Runtime unit
checks, 3 consumer tests, 13 public executor tests and 2 doctests, native and
Wasm Clippy, and all four installed profiles at `target/cdk-runtime.TDdGXt`.
Against the qualified async candidate, those measured profiles reduced complete
uncompressed Wasm size by 1,798–4,370 bytes and code-section size by
1,248–3,027 bytes. The coordinator owns the full qualification and comparison
records. These are reported artifact-size observations, not execution-speed
claims. Input separation and passing qualification do not establish legal
clearance.
