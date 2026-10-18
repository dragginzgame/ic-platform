# Current handoff

The maintainer released 0.1.4 locally at `9934d8d`; package metadata is 0.1.4.
The public `dragginzgame/ic-platform` remote exists. The publication-target work
has not uploaded a crate or created commits. Testkit, Query, Canic, Timers and Host sibling checkouts were
inspected read-only; their pending work is preserved. Storage and ic-memory remain
out of scope.

Pending 0.1.5 now also adds optional `management-canister-types`, exposing the
seven upstream HTTP request/result/header/method/transform identities through
`ic_platform::management_canister_types` from exactly `=0.11.0`. This compatible
addition leaves package versions at 0.1.4. Canic's existing runtime probe supplied
the actual request use; its checkout remains read-only. The feature supports
native/Wasm, brings transitive Candid, and selects no Platform codec, executor,
parser or Agent feature. Select `candid` separately for Platform encoding/decoding.
Consumers retain byte ceilings, pricing/replication, cycles, transforms and retries.
Other upstream management DTOs and call helpers remain outside this explicit API.

Management qualification compares the probe request with the original upstream
encoder's bytes and exercises all six HTTP methods, headers/body, transform
references/context, result roundtrips and caller byte refusal. The standalone
consumer declares only Platform and passes native Rust 1.99/1.88 tests and Wasm
compilation. Feature admission covers all 128 native/Wasm combinations plus
actual defaults. Negative fixtures reject widening the pin and an executor that
selects management DTOs. Evidence is under `target/management-upstream/`,
`target/management-negative/`, `target/management-before/` and
`target/management-qualification.log`; the separate consumer is
`target/boundary-consumer.f0a8rG/`. The first strict Clippy run required replacing
hex-vector chunk iteration. Final focused tests, standalone consumers, Rust 1.88,
Wasm compilation/build, strict Clippy/Rustdoc, formatting, declaration and snapshot
checks pass in `target/management-final-qualification.log` and
`target/management-wasm-build.log`. The 128-combination graph pass is in the
earlier qualification log. Packaging without verification passes from the cached
registry after the online attempt failed DNS; the normalized package preserves
the exact optional constraint and includes the module. See
`target/management-package-offline.log`; nothing was uploaded. Full CI/release
was not run.
Next adoption needs publication and Canic's import/manifest migration plus its
existing CDK/runtime qualification. Native macOS and installed-canister behavior
remain unqualified; no transitive removal or measured Wasm reduction is claimed.

The current issue repair batch extends pending 0.1.5 with explicit upstream
staged decoding/configuration, Candid service registration/export/reflection,
encoding macros and unconditional PrincipalError. The existing finite-limit
decoders are unchanged. Candid-only standalone qualification preserves fixed DID,
wire bytes, optional/default quotas, typed stage failures and generator paths.
Certification-only tests now cover all subtree outcomes and a fixed labelled
commitment/pruned witness. The Candid feature selects upstream printer support
for service export without enabling parser or executor features.

Executor and raw-system requirements are now exact `=2.0.0` and `=1.2.0`, with
documented compatibility exceptions. The independent `ic0` feature directly
exports Timers' observed safe/unsafe system calls; it owns no callback, timer,
scheduler or storage. A Platform-only resolver fixture rejects executor 2.0.1.
A disposable Timers source copy removes both direct declarations and changes raw
paths: 160 native tests pass at Timers' Rust 1.91 floor, and its production Wasm
build resolves one executor/system identity without Candid/CDK frontend/parser.
These are not installed callback/trap/upgrade results or sibling edits.

Current remote CI at `9934d8d` fails all three hosts during offline unfiltered
consumer metadata, requesting uncached windows-link. The local fix filters to
the compiler's native host. A disposable cache omitting that crate reproduces
the original failure and passes filtered metadata plus the consumer test. See
[the CI failure](https://github.com/dragginzgame/ic-platform/issues/10) and
[native run](https://github.com/dragginzgame/ic-platform/actions/runs/39111096632).
The remote run has not been rerun. Required native macOS qualification of the
new tree remains outstanding.

The previous issue batch's focused tests, Rust 1.88 checks, standalone consumers, all sixty-four native/Wasm
feature combinations, all-feature Wasm build, Clippy, Rustdoc and declaration
checks pass. Evidence is in `target/issues-final-qualification.log`,
`target/issues-cache/`, `target/issues-executor/` and `target/issues-timers/`;
`target/issues-before/` preserves incoming Agent work for attribution. No full CI,
commit, push, release or publication was performed. Existing CDK macro/call/spawn
and published Testkit identity exceptions are explicit in the packaged README.
Next delivery needs the maintainer's commit/release flow and a fresh required CI
matrix, followed by registry-only consumer and installed runtime qualification.

The Candid boundary now supplies the real Testkit/Query argument traits plus
`Nat`, `Int` and `Reserved`. The `candid` feature selects big-number support;
the optional native parser also enables it upstream. Runtime features never
select host parsing and defaults remain empty. The decoder and its six tests
still match Testkit's reviewed working-tree owner; behavior and limits are not
changed. Release 0.1.3 adds unconditional `ic_platform::Principal` with upstream
conversion, self-authenticating and Serde support. There is no principal feature;
the root and Candid contract exports share one upstream type identity. The package
version remains 0.1.4. Default builds now include Principal's dependencies while
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

Release 0.1.4 adds an optional native `candid-parser` feature and explicit
`ic_platform::candid_parser` APIs for parsing/checking, service/type equality,
constructor/textual arguments and JavaScript/TypeScript generation. This is a
compatible additive surface; package versions remain 0.1.4. The parser dependency
and module are absent on Wasm even with all features. Runtime features never
enable parsing. Host retains service inventories, admission bounds and aggregate
decoder accounting; consumers retain import, artifact, diagnostic and retry policy.
Upstream `check_prog` ignores imports, and pretty loaders can print diagnostics;
the migration contract records the required caller distinctions.

Parser-only tests and the separate Platform-only consumer pass on Rust 1.99 and
1.88. Host's four existing service-inspection tests pass from a disposable copy
with only imports changed, including import refusal and caller bounds. All-feature
native tests, MSRV, Wasm compilation, Clippy and Rustdoc pass. Graph admission
checked all sixteen combinations plus actual defaults on native/Wasm and
enforces empty defaults from Cargo metadata. Negative fixtures reject nonempty
defaults, runtime-enabled parsing and an unconditional parser dependency leaking
into Wasm. Evidence is in `target/parser-tests.log`, `target/parser-checks.log`,
`target/parser-consumers.log`, `target/parser-host/` and `target/parser-negative/`;
Host source hashes are in `target/parser-before/host-source.sha256`.

The next parser step is actual Host/Canic adoption and their service, binding-byte,
argument-byte and protocol tests. Existing siblings remain read-only. These local
proofs do not qualify complete consumer builds, native macOS or installed canisters.

Pending 0.1.5 adds optional native `agent` and explicit `ic_platform::agent`
exports from ic-agent 0.49.2, including PEM identities, builders, typed errors,
signed updates, request IDs and lookup. This is a compatible additive extension;
package versions remain 0.1.4. Certification owns certificate/hash-tree identities
and now exposes observed subtree lookup results and labelled hashing. Agent does
not enable other Platform features; its upstream Candid/certification dependencies
remain transitive. Agent and its module are absent on Wasm even with all features.
No runtime feature enables transport. Consumers retain endpoint/TLS admission,
identity, trust roots, body limits, verification, expiry, retry and reconciliation.

Agent/certification tests pass on Rust 1.99 and 1.88 for offline signing, raw bytes,
effective-ID routing, canonical request identity, mismatch refusals, PEM/error
types and certificate identity. All-feature tests, MSRV, Clippy, Rustdoc and Wasm
build also pass. Query's five certificate tests and Host's six decoder tests pass
on both compilers from a disposable combined copy with only
imports adapted. Source hashes, metadata, locks and logs are retained under
`target/agent-query/`; initial test compilation and corrected qualification are in
`target/agent-tests.log` and `target/agent-tests-2.log`. Separate consumer evidence
is in `target/agent-consumers.log`. Agent qualification checked all thirty-two
combinations plus actual defaults on native/Wasm; negative fixtures under
`target/agent-negative/` reject runtime-enabled Agent and an Agent dependency
leaking into Wasm. Full CI/release was not run.

The next Agent step is actual Query/Canic migration and their bounded HTTP,
authentication, signed-request, retry and effect-reconciliation tests. Existing
siblings remain untouched. These proofs do not qualify complete consumer builds,
native macOS, live gateways or installed-canister behavior, and establish no
transitive dependency removal or measured Wasm-size reduction.

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

The sole library and all catalog dependencies serve production capabilities; no catalog
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
