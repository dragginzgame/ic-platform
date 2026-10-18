# Current handoff

Local initial 0.1.0 candidate; no commit, remote or publication yet. The maintainer
authorized creation of `../ic-platform` as the fleet boundary. Existing sibling
repositories remain read-only. Storage work is deferred.

The initial crate contains Testkit's bounded Candid decoder and an independent
executor capability. See [the migration contract](../migration.md). Full CDK
entrypoints/calls and host parser relocation are not implemented; current consumers
still select their existing packages. Source and check evidence do not establish
registry availability or adoption. The root changelog records the candidate.

Shared source `4eabe7f771907ccebfebceafffe10657b911c0eb` combines released 0.23.0
with the reviewed Cargo source-evidence contribution. The snapshot records exact
selected payloads; it is not a claim of current upstream main. Setup and release
commands use the pinned export, never a moving sibling at execution time.

Initial Linux qualification passes six imported decoder tests and one DTO consumer
test on Rust 1.99 and 1.88, strict Clippy, warning-denied Rustdoc, all-feature Wasm
compilation, empty-default and Candid-free executor graphs. A separate consumer
with only Platform declared passes its native test and Wasm compilation. Snapshot,
documentation links, formatting, dependency boundaries, shell checks and substitute
release-command routing pass. Pin checks used an isolated temporary index to admit
the new lockfile; the real index remains empty. Native macOS and installed IC runtime
behavior are not qualified. No full release gate or publication was executed.
