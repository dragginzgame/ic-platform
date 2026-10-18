# Current handoff

The maintainer released 0.1.1 locally at `ba7202c`; package metadata is 0.1.1.
The public `dragginzgame/ic-platform` remote exists. The publication-target work
has not uploaded a crate or created commits. Testkit, Query, Canic, Timers and Host sibling checkouts were
inspected read-only; their pending work is preserved. Storage and ic-memory remain
out of scope.

The Candid boundary now supplies the real Testkit/Query argument traits plus
`Nat`, `Int` and `Reserved`. Big-number support is confined to the `candid` feature;
no default features or host parser are selected. The decoder and its six tests
still match Testkit's reviewed working-tree owner; behavior and limits are not
changed. Compatible publication tooling is recorded under pending 0.1.2 notes;
the package version remains 0.1.1.

`make publish` now uploads the current committed package version to crates.io
with locked dependencies and all features, requiring a clean checkout.
`make publish-check` permits dirty source and runs Cargo's dry run. The real 0.1.1
dry run packaged and verified successfully; Cargo aborted upload as requested.
An earlier sandboxed attempt failed on registry DNS access. Evidence remains under
`target/publication/`. Substitute checks verified exact arguments and failure
status retention; the real dirty-checkout guard refused before Cargo dispatch.
The selected snapshot does not include the shared publication runner; the local
adapter uses shared source/version checks and Cargo directly. Publication behavior
was preserved during snapshot adoption. Native macOS qualification remains outstanding. Release targets do
not publish crates; publication still requires that separate qualification.

See [the migration contract](../migration.md) for source identities, actual callers,
import/manifest/generator changes, tests and coordinated retirement of
`ic-testkit-candid`. Local host-side Candid integration is ready for consumer
qualification. Registry availability and real consumer adoption remain unproven.
Testkit's canister probe and Query's canister feature still require the unfinished
CDK boundary; the broader workspaces cannot yet claim full direct-dependency
isolation. External wire formats, persisted data and consumer policy are unchanged.

Runtime investigation traced Canic's generated absolute Candid references and raw
calls, CDK's entrypoint/callback/recovery contexts, and Timers' shared executor
identity. No full CDK call/macro surface or parser relocation is implemented.
Host aggregate byte debiting must remain caller-owned when its pending decoder
is consolidated. The next consequential step is real Testkit/Query host adoption
qualification; runtime work needs a standalone generated-entrypoint fixture and
installed mixed CDK/Timers trap-cleanup proof.

Focused Linux x86-64 validation on Rust 1.99.0 and 1.88.0 passes all six retained
decoder tests and the expanded DTO test. The separate Platform-only consumer
passes on both native compilers and compiles for Wasm; it covers recursive/renamed
DTOs, generic arguments, big-number wire vectors and typed source downcasting.
All-feature Wasm compilation, strict Clippy and warning-denied Rustdoc pass.
Normal/build graphs establish empty defaults, Candid-free native/Wasm executor
selection, and no host parser in current all-feature selection. Logs are retained
under `target/candid-boundary-checks.log`; disposable consumer evidence is under
`target/boundary-consumer.*`. These are local checks, not migrated sibling builds.
Native macOS Intel/Apple Silicon and installed-canister behavior are unqualified.
No full release gate or Wasm-size measurement was run. Direct dependency isolation
is not transitive Candid removal or evidence of reduced Wasm size.

Shared Tooling 0.24.0 is adopted at `cab12cd3dfa4fdb34846b25397f0d72881bb9ae0`
through its canonical exporter, including the previously missing shared hook.
All 96 snapshot payloads verify; none were hand-edited. The common Rust tools
were activated with the new two-tool selection. The migration document's older
Shared revision identifies its historical consumer audit, not the current pin.

The sole library and all three dependencies are labelled production; no catalog
entry is tests/tooling-only. Global manifest ordering is retained with inline
dependency labels. Release metadata checks now enforce that same order during
preparation, current checks and selected-commit checks. Disposable qualification
proves formatter idempotence, unsorted-input rejection, hook repair, and committed
source selection despite an unsorted working manifest. The hook is shipped and
qualified but is not activated in this checkout (`core.hooksPath` is unset).

The dependency gate now checks all four supported feature selections on Linux
x86-64 and Wasm, including transitive normal/build paths; forbidden parser and
Testkit packages are rejected in negative substitute checks. Tools, formatting,
snapshot and substitute release-command checks pass. Evidence, including failed
fixture attempts before correction, is retained under `target/adoption-0.24.0/`.
The lockfile and package versions are unchanged. No full CI or release was run.

Remaining enforcement boundary: comments do not classify future dependencies
automatically, publication does not run the graph gate, and this library has no
deployment entrypoint binding a final application's graph/build to an artifact
digest. Consuming applications must enforce that admission for their exact
deployment selections. Native macOS and installed IC behavior remain unqualified.
