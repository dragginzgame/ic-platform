# Independent public executor tests

The specification-only test author read these admitted inputs:

- `AGENTS.md`
- `TOKO-TOOLING.md`
- `docs/principles/rust-code-hygiene.md`
- `docs/design/executor-contract.md`
- `docs/design/executor-api-observations.txt`

The integration coordinator additionally supplied the public factual requirement
that `MethodHandle` and `TaskHandle` retain their `Send + Sync` auto-traits.
This does not authorize cross-thread execution or wake semantics.

The author did not inspect existing implementations, supplier bodies, old tests,
Git history, diffs, source fingerprints, manifests or other repository sources.
Only the fresh `crates/ic-platform-runtime/tests/executor_consumer.rs` and this
record were authored. The coordinator owns manifest registration and integration.

The tests use only the public Runtime executor API and standard-library consumer
futures. They cover protected routing, query wake coalescing and waker identity,
queue priority/FIFO, stale handle/waker generation churn, self-cancellation,
retention, stable bulk destruction, destructor reentrancy, migratory lifetime,
retained query restrictions, recovery and invalid context entry. Panic cases run
on fresh native threads and assert refusal without matching panic text. Native
unwinding is not used as a model for replica trap rollback.

The author did not run Cargo, Clippy or executable tests. Formatting is limited
to the authored integration test with compiler 1.99.1 and child traversal
disabled. The coordinator owns focused compilation, linting, runtime execution,
private-state reclamation checks and installed IC qualification. Public churn
cannot directly prove private allocation reclamation or force a particular slot
allocation strategy. No performance or legal-clearance claim is made.
