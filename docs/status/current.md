# Current handoff

The maintainer released 0.1.3 locally at `b22481d`; package metadata is 0.1.3.
The public `dragginzgame/ic-platform` remote exists. The publication-target work
has not uploaded a crate or created commits. Testkit, Query, Canic, Timers and Host sibling checkouts were
inspected read-only; their pending work is preserved. Storage and ic-memory remain
out of scope.

The Candid boundary now supplies the real Testkit/Query argument traits plus
`Nat`, `Int` and `Reserved`. The `candid` feature selects big-number support;
the optional native parser also enables it upstream. Runtime features never
select host parsing and defaults remain empty. The decoder and its six tests
still match Testkit's reviewed working-tree owner; behavior and limits are not
changed. Release 0.1.3 adds unconditional `ic_platform::Principal` with upstream
conversion, self-authenticating and Serde support. There is no principal feature;
the root and Candid contract exports share one upstream type identity. The package
version remains 0.1.3. Default builds now include Principal's dependencies while
remaining Candid/CDK-free; executor-only builds remain Candid-free.

Release 0.1.3 also adds optional `ic_platform::certification`: explicit upstream
certificate, delegation, hash-tree and label identities, lookup results, tree
constructors and `fork_hash`. It selects ic-certification 3.2.0 with Serde and is
independent of Candid and executor features. Host retains its strict bounded CBOR
decoder; trust roots, verification and application policy stay with consumers.
The migration contract records the reviewed Host, Query and Canic working trees
and exact import/manifest changes. Those sibling checkouts remain untouched.

Certification-only standalone consumer tests pass on Rust 1.99 and 1.88, with
Wasm compilation. All eight feature combinations pass native/Wasm graph checks;
all-feature tests, MSRV, Wasm, Clippy and Rustdoc pass. Host's six decoder tests
also pass on both compilers from a disposable source copy with only certification
imports changed. Logs are `target/certification-checks.log` and
`target/certification-host/checks.log`; copied source hashes are retained beside
the latter. These checks do not qualify complete sibling builds or authentication
flows. The next certification step is coordinated Host/Query/Canic adoption and
their certificate, receipt and signature verification tests against one resolved
upstream type identity. Native macOS and installed-canister qualification remain
outstanding; no Wasm-size reduction is claimed.

Pending 0.1.4 adds an optional native `candid-parser` feature and explicit
`ic_platform::candid_parser` APIs for parsing/checking, service/type equality,
constructor/textual arguments and JavaScript/TypeScript generation. This is a
compatible additive surface; package versions remain 0.1.3. The parser dependency
and module are absent on Wasm even with all features. Runtime features never
enable parsing. Host retains service inventories, admission bounds and aggregate
decoder accounting; consumers retain import, artifact, diagnostic and retry policy.
Upstream `check_prog` ignores imports, and pretty loaders can print diagnostics;
the migration contract records the required caller distinctions.

Parser-only tests and the separate Platform-only consumer pass on Rust 1.99 and
1.88. Host's four existing service-inspection tests pass from a disposable copy
with only imports changed, including import refusal and caller bounds. All-feature
native tests, MSRV, Wasm compilation, Clippy and Rustdoc pass. Graph admission
now checks all sixteen combinations plus actual defaults on native/Wasm and
enforces empty defaults from Cargo metadata. Negative fixtures reject nonempty
defaults, runtime-enabled parsing and an unconditional parser dependency leaking
into Wasm. Evidence is in `target/parser-tests.log`, `target/parser-checks.log`,
`target/parser-consumers.log`, `target/parser-host/` and `target/parser-negative/`;
Host source hashes are in `target/parser-before/host-source.sha256`.

The next parser step is actual Host/Canic adoption and their service, binding-byte,
argument-byte and protocol tests. Existing siblings remain read-only. These local
proofs do not qualify complete consumer builds, native macOS or installed canisters.

Principal qualification passes no-feature tests on Rust 1.99/1.88, no-feature
Wasm test compilation, all-feature tests and Wasm compilation, standalone Candid
consumer tests, feature graphs, Clippy and Rustdoc. Logs are retained in
`target/principal-checks.log` and `target/principal-final-checks.log`; the first
Clippy run caught missing test-crate documentation, corrected before the passing
rerun. The lockfile only adds the already-selected Principal dependency edge.

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
identity. No full CDK call/macro surface is implemented. The native parser
dependency boundary is now available; consumer parser implementation stays local.
Host aggregate byte debiting must remain caller-owned when its pending decoder
is consolidated. The next consequential step is real Testkit/Query host adoption
qualification; runtime work needs a standalone generated-entrypoint fixture and
installed mixed CDK/Timers trap-cleanup proof.

Focused Linux x86-64 validation on Rust 1.99.0 and 1.88.0 passes all six retained
decoder tests and the expanded DTO test. The separate Platform-only consumer
passes on both native compilers and compiles for Wasm; it covers recursive/renamed
DTOs, generic arguments, big-number wire vectors and typed source downcasting.
All-feature Wasm compilation, strict Clippy and warning-denied Rustdoc pass.
Earlier normal/build graphs established empty defaults, Candid-free native/Wasm executor
selection, and no host parser in that historical all-feature selection. Logs are retained
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

The sole library and all six dependencies serve production capabilities; no catalog
entry is tests/tooling-only. Global manifest ordering is retained; inline
dependency role labels do not enforce admission. Release metadata checks enforce the same order during
preparation, current checks and selected-commit checks. Disposable qualification
proves formatter idempotence, unsorted-input rejection, hook repair, and committed
source selection despite an unsorted working manifest. The hook is shipped and
qualified but is not activated in this checkout (`core.hooksPath` is unset).

Before the parser extension, the dependency gate checked all eight supported feature selections on Linux
x86-64 and Wasm, including transitive normal/build paths; forbidden parser and
Testkit packages are rejected in negative substitute checks. Tools, formatting,
snapshot and substitute release-command checks pass. Evidence, including failed
fixture attempts before correction, is retained under `target/adoption-0.24.0/`.
Snapshot adoption left lockfile and package versions unchanged. The certification
extension adds ic-certification and serde_bytes to the lockfile; package versions
remain unchanged. The native parser extension adds its upstream dependency graph
to the lockfile while preserving Wasm exclusions. No full CI or release was run.

Remaining enforcement boundary: comments do not classify future dependencies
automatically, publication does not run the graph gate, and this library has no
deployment entrypoint binding a final application's graph/build to an artifact
digest. Consuming applications must enforce that admission for their exact
deployment selections. Native macOS and installed IC behavior remain unqualified.
