# Current handoff

Pending 0.1.1 notes, based on local 0.1.0 commit `9ada261`. The public
`dragginzgame/ic-platform` remote now exists. This task has not committed, pushed
or published changes. Testkit, Query, Canic, Timers and Host sibling checkouts were
inspected read-only; their pending work is preserved. Storage and ic-memory remain
out of scope.

The Candid boundary now supplies the real Testkit/Query argument traits plus
`Nat`, `Int` and `Reserved`. Big-number support is confined to the `candid` feature;
no default features or host parser are selected. The decoder and its six tests
still match Testkit's reviewed working-tree owner; behavior and limits are not
changed. The maintainer selected a 0.1.1 changelog for these compatible additions;
the package and lockfile version remain 0.1.0 pending release preparation.

The requested `make publish` failed before dispatch: no publication target exists.
No upload was attempted. The README requires clean committed source and native
qualification before initial publication; the current changes remain uncommitted
and macOS qualification is outstanding. Release targets do not publish crates.

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

Shared Tooling remains pinned at `4eabe7f771907ccebfebceafffe10657b911c0eb`.
Snapshot payloads remain unchanged. The migration contract retains the original
Testkit source hashes; the snapshot records shared payload provenance separately.
