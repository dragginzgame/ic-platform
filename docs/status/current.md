# Current handoff

## Pending 0.6.7

The [individual export audit](../audits/export-usage-2026-10-21.md) records 393
named exports/owned methods against 7,156 sibling Rust files. Generated Candid
and entrypoint support, Storage's substrate alias and receiver calls were reviewed
separately. Thirty-three names have only Platform fixture evidence or no resolved
sibling reference; this is not proof of safe removal. Public type contracts and
upstream inherited members retain the report's explicit scope limits. No API was
removed, no sibling was built/edited and existing inventory work was preserved.

The [IC inventory](../ic-inventory.md) maps public capability groups to upstream
or imported implementations, observed sibling usage and maintenance ownership.
Sibling observations are dated, include dirty-state caveats and are not released
adoption evidence. CI now checks all workspace feature/direct-dependency facts
and complete README feature coverage without network access or sibling scans.
API semantics and continuing sibling usage still need source review; this does
not replace dependency graphs, imported-source guards or runtime qualification.

Full Clippy passed before regression tests. Inventory checks and mutation tests,
formatting, documentation links, boundary/pin checks and snapshot integrity pass
on Linux. Evidence: `/tmp/platform-inventory-clippy.log` and
`/tmp/platform-inventory-checks.log`. No Rust, dependency, API or crypto changes;
no sibling edits. Native macOS and installed-canister checks were not rerun.

## Released 0.6.6

Adopted Shared Tooling 0.26.4 at
`dc1ced7e240e90f0c6daa89fb6248fcb074164eb` through its canonical exporter.
No library API, dependency selection, lockfile or package version changed.
The release gate runs Clippy before runtime fixtures; publication runs it before
packaging. Shared IC executables remain optional and require explicit selection.

Full Clippy, explicit tool setup/offline admission, snapshot integrity, formatting,
dependency boundaries/pins, documentation links and release-adapter/changelog
regressions pass on Linux. A substituted failing Clippy command stops publication
with its original status before packaging. Evidence: `/tmp/platform-tooling-*.log`
and `/tmp/platform-publish-lint.TnDMER`. Native macOS and complete release/runtime
qualification were not rerun. No release, publication or sibling edits occurred.

## Released 0.6.5

Compatible addition for [#45](https://github.com/dragginzgame/ic-platform/issues/45):
`stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory}`
re-exports the exact upstream 0.7.2 identities under the existing opt-in feature.
No new dependency, defaults, public removal, implementation, crypto or persisted
format changes. Memory retains allocation authority, exclusive leases, checked
layout reopening and growth preflight. Migration imports and supplying-version
requirements are documented in the migration contract; no sibling is edited.

Identity checks cover all three exports. The Platform-only consumer exercises
virtual region growth, separate byte ranges, reopening on original vector
backing and retained bytes after further growth. This isolated fixture does not
qualify production allocation governance or installed upgrade behavior.

Full Clippy passed before focused tests (`/tmp/platform045-clippy.log`). Consumer
native tests and Wasm checks pass on development Rust, 1.88 and 1.91; original
type-identity checks pass on 1.88. Evidence: `/tmp/platform045-consumer.log`,
`/tmp/platform045-191.log`, `target/boundary-consumer.JmMSGs`. Native/Wasm graphs
exclude the substrate by default; the selected graph contains only Platform,
stable-structures 0.7.2 and upstream Principal. Declaration/pin, formatting,
documentation-link and diff checks pass. Native macOS,
installed-canister qualification and published Persistence adoption are separate
acceptance steps. The maintainer integrated the implementation in release 0.6.5
at `14a670c`; registry publication and downstream adoption were not checked here.

## Released 0.6.4 and earlier qualification

0.6.4 includes PR #44's declared-development-compiler selection. The earlier
0.6.3 shell fix and regression are delivered: [native CI](https://github.com/dragginzgame/ic-platform/actions/runs/39580492029)
passed Linux, Apple Silicon and Intel macOS, with a separate successful
[installed-runtime run](https://github.com/dragginzgame/ic-platform/actions/runs/39580492011).
Producer issues #43, #24, #30 and #36 are closed. These checks qualify their
respective delivered behavior, not the new manager exports or downstream adoption.

Released 0.6.2 contains bounded Candid sequences/strict reserved helpers from
PR #26, exact-identity substrate exports from PR #32, and explicit external-Candid
composition proof/migration guidance. It also includes short feature-graph cache
keys, hygienic printing macro paths and Clippy-before-tests gate ordering. Its
original macOS failure remains historical; the 0.6.3 pass supersedes it for
current acceptance. Historical local evidence is `/tmp/platform062-*.log` and
`/tmp/platform043-*.log`; target artifacts may have been removed by maintainer
cleanup. Original proposal PRs are closed as integrated.

Further hard cuts remain deferred until Canic is live. Compiler retirement
[#38](https://github.com/dragginzgame/ic-platform/issues/38) depends on Canic #613
and Tooling #225. Agent copy optimization #39 was closed as not planned: retain
upstream Agent without a fork or claimed fix. Historical benchmark evidence is
unchanged. GitHub remains the follow-up tracker.

## Released 0.6.1

The secp256k1 standalone fixture now imports its test-only API inside the test,
avoiding unused-import warnings when Cargo also compiles it as a library.
Its unchanged derivation/encoding test passes on native Rust and MSRV 1.88;
both Wasm checks pass without warnings. Evidence:
`target/boundary-consumer.9knNxT`, `/tmp/platform-secp-consumer-warning.log`.
No cryptographic implementation or public API changed.

The local fix for [42](https://github.com/dragginzgame/ic-platform/issues/42)
updates only the runtime fixture lock's rustls-webpki 0.103.15 to 0.103.16 and
webpki-root-certs 1.0.9 to 1.0.10, including their checksums. A fresh RustSec
checkout at `4b587f00996ebe5c05a73eaef105ef241e876414` reports two vulnerabilities
before and zero after (`cargo audit --no-fetch --no-yanked`); existing `paste`
and `serde_cbor` unmaintained warnings remain. No library selection changes.

The rebuilt native driver tests, native/Wasm graph admission and all four
installed Rust 1.99.1/1.91.0 × Wire/external-Candid fixtures pass, including raw
awaits, callback traps, timer cancellation, upgrade and stable rollback.
Evidence: `target/cdk-runtime.9KcpTI`, `/tmp/platform-post06`. These are Linux
PocketIC results; native macOS and live IC qualification were not performed.
The maintainer subsequently integrated this fix and released 0.6.1 at `0df13fc`.
GitHub main was verified at that revision and #42 was closed as resolved.

Published registry 0.6.0 passes standalone executor+IC0, Principal and IC0
consumer tests/checks on native Rust, MSRV 1.88 and Wasm. Evidence is retained in
`target/boundary-consumer.00sBZ3`, `target/boundary-consumer.HS80XV` and
`target/boundary-consumer.aquYOL`. Timers and IcyDB already contain pending 0.6
adoption work; IcyDB's facade and schema select `principal` explicitly. These
dirty sibling checkouts were inspected read-only, not independently qualified
as complete products or claimed as released adoption.

Canic's live frontend still calls both Candid binding compilers. Its canonical
role unions still derive Candid types; their `wire` directory name does not
prove IC Wire schema availability. [Canic #613](https://github.com/dragginzgame/canic/issues/613)
must supply endpoint-derived schemas and the replacement artifact identity
before [38](https://github.com/dragginzgame/ic-platform/issues/38) can retire the
exports. The linked old Build repository is currently inaccessible; Build code
now lives in `ic-tooling` under `crates/ic-build`, with orchestration follow-up
[225](https://github.com/dragginzgame/ic-tooling/issues/225). No parser fork or removal is
justified by this audit, and no consumer/persisted format has changed.

## Released 0.6.0

The maintainer authorized issues [40](https://github.com/dragginzgame/ic-platform/issues/40)
and [41](https://github.com/dragginzgame/ic-platform/issues/41): explicit `principal`
feature ownership and lean signature-creation packaging. Runtime, Candid and
external client/DTO profiles imply `principal`; type-only consumers must add it.
Principal/Account bytes, crypto bodies, executor identity, callbacks and consumer
policies are unchanged. The original creation source is recovered exactly by
stripping only eight reviewed test/reference packaging attributes.

[Graph evidence](../benchmarks/0.6/README.md) records executor+IC0 at 26 to 6
packages and creation at 30 to 26, including Platform in both counts. Original
vectors pass in 42 focused Rust 1.88 tests; eight standalone consumer profiles
pass native/MSRV/Wasm. All 199 routine feature selections pass native/Wasm graph
admission. Source/mutation guards, strict focused Clippy, minimal rustdoc,
formatting, dependency boundaries, pins and documentation links pass.

The clean Timers 0.30.0 checkout at `41e1e2fb` compiles on Rust 1.91 native/Wasm
through an isolated root-patched consumer. Siblings remain read-only. This is
local integration evidence, not published adoption. Native macOS, installed
canisters/timers and live IC were not rerun; no Wasm-size/speed gain is claimed.
The maintainer subsequently released 0.6.0 at `73dda4a`; issues 40 and 41 are
closed with default-branch evidence. The earlier qualification preserved the
pre-existing Candid/derive 0.10.39 lock update. Working logs:
`/tmp/platform-06-work`. Published consumer evidence is recorded above.

Parser/compiler retirement remains blocked on Canic's live generator calls;
[38](https://github.com/dragginzgame/ic-platform/issues/38) tracks that migration.
Agent remains upstream-owned and unchanged.

## 0.5.1 (historical)

The native Agent request-copy experiment qualifies a one-line ownership change
in an isolated copy of registry ic-agent 0.49.2; production still uses the
unchanged registry package. Each nonempty request saves one allocation and its
body length in allocated bytes. Rust 1.88 byte/replay/status probes and real
loopback HTTP retry/response-limit checks pass before and after the patch.
[Evidence and reproduction](../benchmarks/0.5.1/agent-copy/README.md) distinguish
local timings from network performance and record unqualified paths.
[Platform #39](https://github.com/dragginzgame/ic-platform/issues/39) tracks the
distribution decision: a wrapper cannot fix this private clone, and a consumer
Cargo patch is not a published solution. No Agent fork or adoption is implied.

The message-buffer reader now uses pinned ic0's all-or-trap spare-capacity copy
and a function-local unsafe length update. Other API functions retain the deny
lint; submission, callback/cleanup and protected-spawn fingerprints are unchanged.
The installed fixture compares the old zero-filled allocation against the actual
production helper and checks exact frames and binary awaited replies.

Rust 1.88 passes 12 native runtime/library consumer tests and Wasm compilation;
strict runtime Clippy, source/mutation guards and native/Wasm runtime graphs pass.
Four driver tests qualify rejection classification and the new fixture budgets.
All four installed Rust 1.99.1/1.91.0 × Wire/external-Candid combinations pass,
including callback traps, cancellation, upgrades and stable rollback.
[Retained measurements](../benchmarks/0.5.1/README.md) show about 30% fewer
instructions for the isolated read at 1 KiB and 37% at 2 KiB, not whole-RPC gains.
Evidence: `target/cdk-runtime.UsGpFZ`, `/tmp/platform051-input-*.log`.
Native macOS and live IC qualification were not performed. No executor,
cryptographic code, consumer policy, package version or sibling checkout changed.

Starting from released 0.5.0 (`b840423`), remove the generator's unused
`syn/extra-traits` feature while retaining full syntax parsing. Ten Rust 1.88
generator tests and standalone CDK consumer native/MSRV/Wasm checks pass, with
no `extra-traits` in the generator's resolved feature graph. Evidence:
`/tmp/platform051-syn-*.log`, `target/boundary-consumer.dI5NLx`.
That feature reduction changed no Rust implementation or lockfile.
Native macOS and installed-canister execution were not rerun for this build-only
feature reduction.

The maintainer subsequently authorized deleting the unused, unexported lazy
root-public-key cache and Platform's direct `lazy_static` dependency. No internal
code reads the cache; external consumers cannot reach it through the private
signature module. The public DER constant and extraction function remain.
An exact before/after comparison verified that only the cache and its import
were removed from the fingerprinted source before updating its creation hash.
All cryptographic bodies, constants, vectors, verification source and licenses
are unchanged. Source and mutation-refusal guards pass.

Rust 1.88 passes 39 focused library/consumer tests, and standalone creation and
verification consumers pass native/MSRV/Wasm checks. Evidence:
`/tmp/platform051-lazy-*`, including the expected deletion-only source copy.
Creation graphs on native/Wasm no longer contain `lazy_static`; verification
still receives it from unchanged upstream `ic-verify-bls-signature`.
Package versions and thiserror remain unchanged. No sibling edits, commits or
publication occurred. Native macOS and installed-canister qualification were not
rerun for this unused-cache removal.

## 0.5 hard cut (historical)

The raw-identifier audit reproduced a proc-macro panic for `fn r#match`.
Generated wrapper/default export names now use the unescaped identifier, while
the handler call retains the original Rust identifier and explicit `name` values
remain verbatim. The standalone consumer also qualifies a disabled conditional
endpoint with unavailable types/codecs; Rust removes it correctly, so that
inspection found no additional conditional-compilation defect.
Ten generator tests, standalone native/MSRV/Wasm compilation, strict Clippy and
all four installed compiler/protocol combinations pass. The installed fixture
now implements its unchanged `scalar` method with a raw Rust handler identifier.
Evidence: `/tmp/platform05-final-audit-*.log` and `target/cdk-runtime.oL3rKY`.
No sibling edits, version changes, commits or publication occurred; native macOS
and live IC qualification remain outstanding.

The native export audit reproduced a duplicate-symbol compilation failure for
distinct endpoint names `native-name` and `native_name`. The generator's lossy
punctuation substitution is replaced with full UTF-8 hexadecimal encoding under
an `__ic_export_` prefix. Tests cover punctuation, Unicode and query/update
distinctions; the Wasm branch retains its exact IC export name.
Nine generator tests, standalone CDK native/MSRV 1.88/Wasm compilation, strict
Clippy, formatting and boundary checks pass. Evidence:
`/tmp/platform05-host-symbol-*.log`, `target/boundary-consumer.590IOE`.
Host symbol-inspection tools must update their expectations; ordinary consumer
imports and external wire names are unchanged. Native macOS qualification is
still outstanding; installed execution was not rerun for this native-only
symbol change. No runtime bodies, dependencies or sibling files changed.

The latest generator audit reproduced five compilation errors for valid handler
names, codec names and argument bindings before fixing local-variable hygiene.
Arguments now use positional mixed-site bindings; generated guard/input/result
locals cannot capture caller paths. Generated Result/String/Vec references are
absolute standard-library paths, qualified against shadowing consumer aliases.
The standalone CDK consumer includes both synchronous and asynchronous collision
cases, including a handler named exactly like a generated positional binding.
The generator also rejects previously ignored `composite` on non-query methods
and `manual_reply` on lifecycle hooks. Migration notes include IcyDB's handwritten
endpoint fixtures, which still use the separately retired `hidden` option.
Eight generator tests, the standalone native/MSRV/Wasm CDK consumer, strict
Clippy and all four rebuilt installed compiler/protocol combinations pass.
Evidence: `/tmp/platform05-hygiene-*.log` and `target/cdk-runtime.oL3rKY`.
The before-fix compile log is `/tmp/platform05-hygiene-before.log`.
No runtime/crypto bodies, dependency selections or sibling files changed in
this pass; native macOS and live IC qualification remain separate.

The subsequent generator/call cleanup retires `hidden` and `change_timeout`.
Entrypoint options now use Syn directly, with targeted retirement, duplicate and
unknown-option errors; ordered guards, explicit codec paths and boolean flags
remain supported. String-encoded booleans are rejected. Argument processing no
longer clones discarded type syntax. Darling 0.23 and its generator dependency
edge are removed, without changing unrelated dependency versions.

`with_wait_policy(WaitPolicy::{Bounded { timeout_seconds }, Unbounded})` replaces
the timeout mutator that panicked on unbounded calls. Constructor defaults and
stored/submitted timeout values are unchanged. Blob has three old-method users;
Testkit performance probes still supply `hidden`. Both migrations are documented,
and siblings remain untouched. Runtime fixtures exercise both mode transitions.

The callback ownership audit found one Arc allocation for the shared call state;
cloning the Arc for submission shares that allocation. Immediate submission
failure reclaims the raw reference; successful submission transfers it to the
callback/cleanup path. Trap rollback is essential to that ownership argument.
Callbacks release the state lock before waking the task. Removing synchronization
would also require checking native auto-trait behavior and executor assumptions,
not merely assuming single-threaded Wasm makes it redundant. No callback/lock
rewrite or measured speed improvement is claimed; source fingerprints remain
unchanged for submission, callback cleanup and protected spawning.

This cleanup passes seven generator tests and 22 focused runtime tests on Rust
1.88, standalone runtime/CDK native and Wasm checks, strict Clippy, source and
mutation guards, and all 180 routine feature selections on native/Wasm.
Evidence: `/tmp/platform05-macro-*.log`, `/tmp/platform05-wait-tests.log` and
`target/feature-graphs/attempt.ku8S0D`. The reused installed fixture
`target/cdk-runtime.oL3rKY` passes all four compiler/protocol combinations after
rebuilding its canisters with the new parser and wait-policy transitions;
`/tmp/platform05-macro-installed.log` records the run. Native macOS and live IC
qualification remain outstanding. No cryptographic source or executor changed.

The checkout started clean at released 0.4.1 (`30058e1`). The maintainer authorized
retiring unused surface and auditing frequently used IC paths, with siblings
read-only and cryptographic implementations unchanged. The checkpoint API has no
live calls in the searched sibling Rust trees outside Platform's own fixture.
Its public module, private stable IO abstraction/writer and whole-memory copy are
removed. Raw stable grow/read/write retain their signatures and ic0 behavior.
The unreachable OutOfBounds error variant is removed: raw bounds violations
already trap, while failed growth returns OutOfMemory.
The fixture explicitly owns its existing fixed Candid record and preserves
no-finalization restore; no production checkpoint replacement is introduced.

The usage audit included app/test canisters before deciding removals: Timers and
Canic use cycle burning, and Canic uses HTTP cost estimation, so those remain.
Common principal/call paths, clock/counter passthroughs and decoder policy were
reviewed. Caller/canister/subnet ID construction had a temporary Vec despite the
29-byte identity bound; it now uses stack buffers. Clock/counter wrappers already
delegate directly; callback/trap state-machine and cryptographic bodies remain
unchanged. Source reference counts are prioritization evidence, not runtime
profiling. Audit revisions and searches are retained at `/tmp/platform05-audit`.

The [0.5 migration contract](../migration.md#pending-05-retirement) documents
unknown downstream checkpoint users, preserved data policies and retirement.
Package versions remain 0.4.1; the pending changelog is 0.5.0. No dependency
elimination across the fleet or measured speed/Wasm saving is claimed. No sibling edits, commits,
pushes or publication were performed.

The expanded audit separates two independently consumed capabilities.
`candid-service` now owns service registration/export macros and their printer;
plain `candid` retains external DTO encoding, decoding, big numbers and reflection.
Canic and Blob still generate Candid declarations, so their macro/test owners must
select the new feature before adoption. `runtime` exposes the existing `cdk::*`
adapters without entrypoint macros; `cdk` includes it and keeps the generator.
Canic Core's raw-call use can select `runtime`, while Canic's entrypoint owner
keeps `cdk`. No sibling manifests were changed.

At the preceding feature-split stage, native/Wasm isolated graph comparisons removed four packages from plain Candid
(`pretty`, `typed-arena`, `arrayvec`, `unicode-width`) and six from runtime-only
versus CDK (`ic-platform-macros`, `darling`, `darling_core`, `darling_macro`,
`ident_case`, `strsim`). Other selected features can unify these back in.
That graph checker admitted all 180 routine selections on native and Wasm and rejected
an unselected printer or entrypoint generator. Evidence:
`target/feature-graphs/attempt.K9Wcl3`, `/tmp/platform05-expanded-graphs.log`.
Standalone Candid, service, runtime and CDK consumers pass development/MSRV 1.88
native tests and Wasm compilation; 26 focused MSRV tests pass. The runtime bodies,
executor identity and cryptographic code did not change in this expansion.
The installed qualification below predates these feature separations; neither
native macOS nor installed-canister execution was rerun for this expansion.

Rust 1.88 runtime/Candid tests, standalone CDK consumer native/MSRV/Wasm checks,
compile-fail retirement documentation, strict selected Clippy, source-fingerprint
and mutation guards, dependency graphs, formatting and changelog admission pass.
All four installed PocketIC 16.0.0 compiler/protocol combinations pass on the
candidate, retaining original fixture checkpoint bytes. Evidence:
`target/cdk-runtime.oL3rKY`, `target/boundary-consumer.tjAOD6` and
`/tmp/platform05-*.log`. Native macOS and live IC qualification of this candidate
remain separate.

Released 0.4.1's [installed-runtime CI](https://github.com/dragginzgame/ic-platform/actions/runs/39405021699)
now passes, resolving the earlier fresh-runner provisioning uncertainty. It does
not qualify these uncommitted changes. The older sections below are historical
qualification records, not current branch status.

## 0.4.1 Account formatting qualification (historical)

Account Display now encodes subaccount hexadecimal text into one fixed 64-byte
buffer rather than repeatedly invoking integer formatting. No public identity,
checksum, cryptographic body, persisted bytes or feature selection changes.
The [follow-up measurements](../benchmarks/0.4.1/README.md) put nonzero-subaccount
formatting at 234.7/289.6 ns for 10/29-byte owners, versus 377.7/491.5 ns for the
external implementation in the same run. Prior owned measurements were
540.7/613.0 ns in a separate run; these are native timings, not Wasm/cycle claims.

Rust 1.88 passes 14 library, three ledger differential and two generated parser
tests plus ledger-feature Wasm checking. Strict selected Clippy and formatting
pass. Evidence: `/tmp/platform041-{tests,wasm,clippy}.log`,
`target/type-benchmark.6cMHAN`, and the retained benchmark trials. The 0.4.1
changelog includes the preceding qualification work; package versions remain
0.4.0 for the maintainer's release tooling.

GitHub [CI run 39392859266](https://github.com/dragginzgame/ic-platform/actions/runs/39392859266)
passed Linux, macOS Intel and macOS ARM for released `26a07c5`. It does not qualify
the pending changes. The new installed-runtime workflow is still an untracked
local file and cannot be dispatched on GitHub before the maintainer pushes it.
Fresh-runner Testkit provisioning and native macOS qualification of this patch
therefore remain outstanding. No commit, push or publication was performed.

## 0.4 post-release qualification

That audit started at the maintainer's 0.4.0 release (`26a07c5`). The follow-up added a separate
Linux installed-runtime workflow using Testkit-owned setup/check, and an exact
registry mode for the installed fixture. Published mode rejects local facade or
generator replacements, retains its disposable resolved lock, and applies the
existing single-executor/ic0 graph admission. Local `--prepare` is the explicit
cache setup step; ordinary qualification does not provision a server.

Published 0.4.0 passes all four installed PocketIC 16.0.0 combinations (Rust 1.91
and 1.99.1; Wire-only and external Candid), including traps, timers, upgrades,
cycles, stable rollback and certificates. Evidence: `target/cdk-runtime.UaFe6A`
and `/tmp/platform-registry-installed-unrestricted.log`. The first sandboxed run
could not bind loopback; the admitted rerun passed. This is Linux local-simulator
evidence, not a completed GitHub workflow run, native macOS or live IC acceptance.

The [benchmark evidence](../benchmarks/0.4/README.md) now retains previous trials,
reproduction tooling and new Account/type-layout measurements. Account parsing
improves, but that initial run found a nonzero-subaccount formatting regression;
the pending fix above addresses it without cryptographic changes. Generated Unicode/boundary/round-trip parser
tests pass Rust 1.88. Coverage-guided fuzzing is a separate explicit tooling command.
The incoming rustls-webpki lock update is preserved; no release or publication
was performed.

Isolated sibling snapshots at `/tmp/platform-adoption-audit` passed Auth's 11
canister-signature tests (`24bbad5`, clean source), IcyDB's 17 Account stored-byte
and ordering tests (`07a5964` plus pending work), and Canic's four call-adapter
tests (`8c2dda4` plus pending work). A disposable test added only to the Wire
snapshot (`fe7ab95`, incoming lock changes) compares 120 Principal and 360 Account
encodings against registry external types and decodes the owned values, preserving
absent/explicit-zero fields. It passes against registry Platform 0.4.0.
Snapshot HEAD/status records and logs are retained alongside the copies. This
qualifies the selected tests, not all sibling endpoints or historical database
upgrades. No sibling checkout was edited.

Canic's frozen Wasm normal/build graph still includes Platform 0.2.2 through
`ic-auth-protocol-types 0.10.7` and `ic-timers 0.29.4`, alongside 0.4.0.
Executor 2.0.0 and ic0 1.2.0 remain single identities. This is not a complete
consumer cutover; dependency adoption belongs to those consumers. IcyDB Core
and Auth default graphs select Platform 0.4.0 without Candid. Retained graphs and
the Canic reverse-dependency trace are in the snapshot evidence directory.

The coverage-guided parser campaign passes 1,441,163 executions in 61 seconds
with AddressSanitizer and seed 42. Evidence: `target/parser-fuzz.WPlyho` and
`/tmp/platform-parser-fuzz-asan.log`. The first campaign completed 1,543,528
executions but LeakSanitizer failed at shutdown under sandbox process inspection;
the successful rerun set `ASAN_OPTIONS=detect_leaks=0`. Leak detection is not
qualified, and the initial empty crash artifact records that tooling failure.
Rust 1.88 generated parser tests and CDK/ledger Wasm checking, strict selected
Clippy, registry-admission refusal tests, source fingerprints, pins/boundaries,
formatting, shell/YAML parsing and documentation links pass. Full CI remains
release-owned; the new GitHub workflow itself has not yet run.

## Released 0.4 transport-only calls

The maintainer requested further justified breaking cuts in the same migration.
Raw Call/Response no longer implement Candid request/response methods, and the
combined call Error and string-only CandidDecodeFailed are retired. Use explicit
Candid encoders/decoders around raw bytes; encoding traps, decoding/arity limits,
diagnostics and retry decisions remain caller-owned. Existing oneway and unknown
reject-code error types are now explicitly nameable through the facade. The
borrowed `encode_args_ref` helper preserves the previous borrowed-argument use.
See [the transport migration](../migration.md#transport-and-codec-separation-04).

Canic's request builder already encodes explicitly; its response helpers and
obsolete combined-error mapping are the targeted migration points. No sibling
files were edited. Checkpoint implementation/bytes, system calls, executor identity,
callback/trap cleanup, timeouts and cycles remain unchanged. The maintainer released this work as 0.4.0 at `26a07c5`.

Linux validation passes: Rust 1.88 all-feature tests/Wasm, standalone CDK and Candid
native/MSRV/Wasm consumers, strict Clippy, warning-denied Rustdoc, source/license
and mutation guards, formatting and documentation checks. The new installed
fixture explicitly encodes management arguments and uses its original upstream
decoding policy. All four PocketIC 16.0.0 runs pass (Rust 1.91 and 1.99.1, Wire-only
and external-Candid), including original checkpoint bytes and upgrade/rollback.
Evidence: `target/cdk-runtime.oau1gn` and `/tmp/platform04-transport-*.log`.
Actual consumer migration, native macOS and live IC qualification remain separate.
No new feature, dependency, publication, measured size claim or crypto change.

## Released 0.4 runtime codes and signature packaging

The maintainer authorized both changes after the 0.3 audit. Runtime RejectCode
is owned, preserving codes 1–6, Display/Serde spelling and raw unknown-code
refusal. Agent/PocketIC enums remain external. Its Strum iteration and external
ErrorCode conversion are intentionally not retained. Signature creation/verifier
source is packaged inside Platform with unchanged upstream bodies and 29 tests;
only documented imports and the unused store declaration differ. Exact source
and license fingerprints plus mutation refusals guard the imported modules.
BLS, certificate and hash algorithms are unchanged. See the
[0.4 migration](../migration.md#runtime-codes-and-signature-packaging-04).

These public identity changes shipped in the maintainer’s 0.4.0 release. No commits, publication or sibling edits were made.
The prior handoff correction is preserved. The root lock removes the retired
packages and adds only the imported test helper; the runtime fixture lock removes
its obsolete runtime error-catalogue edge without changing other selections.

Linux qualification passes: Rust 1.88 all-feature tests and Wasm checking,
standalone CDK/creation/verifier native/MSRV/Wasm consumers, strict all-feature
Clippy and warning-denied Rustdoc. Eighteen native/Wasm production graphs confirm
signature-only selections exclude Candid/ic0 and both registry signature packages;
CDK-only excludes ic-error-types/Strum. Graph admission includes 71 regression
cases. Feature closures/selections, runtime source/mutation/lock checks, dependency
ownership/pins, formatting and documentation checks pass. Signature fingerprints
also retain the original tests and licenses. This is not a measured Wasm saving.

Four installed PocketIC 16.0.0 runs pass: Rust 1.91 and 1.99.1 Wasm, each Wire-only
and external-Candid mode, covering callback/trap cleanup, timer cancellation,
upgrade, cycles, stable rollback and certificates. Artifacts are retained at
`target/cdk-runtime.0tbMOx`; logs at `/tmp/platform04-installed-*.log`. Other focused
logs are `/tmp/platform04-{msrv,consumers,all-msrv,all-wasm,final-clippy,doc}.log`;
graphs are in `target/platform04-graphs`. Native macOS, live IC and actual Auth/Canic
migration or installed signature-policy qualification remain separate work.

## Released 0.3 owned account

The historical 0.3.0 release was `390644f`, following implementation commit
`80a81e8`. The maintainer reports it pushed; this pass does not independently
establish crates.io publication. Qualification below records the implementation
checks, not completed sibling adoption.

The unconditional `Account`, `Subaccount` and `AccountError` now accompany owned
Principal. Effective-zero equality, ordering and hashing match upstream; Serde
and conversions preserve absent versus explicit zero fields. Text parsing admits
at most 136 bytes, decoding subaccounts into a fixed array. Formatting preserves
the ledger CRC32/Base32/hex spelling with no intermediate strings. No cryptographic
implementation, dependency, feature or storage integration is added.

The existing external ledger DTO is unchanged, also named
`account::ExternalAccount` under `icrc-ledger-types`. Explicit conversions preserve
owner bytes and optional subaccount fields. Owned Account has no Candid trait.
Apache source/license provenance is retained under `src/account`. See the
[account migration](../migration.md#owned-account-migration-03).

Linux qualification passes: Rust 1.88 unit tests, Platform-only standalone native
and Wasm consumers on Rust 1.88 and the development toolchain, strict selected
Clippy, formatting, documentation links and six native/Wasm production graphs.
Differential coverage checks 23,040 account cases against upstream text and Candid
bytes, plus marker-free owned/borrowed Serde and effective-zero identity tests.
Logs: `/tmp/platform-account-{consumers,clippy}.log`; graphs: `target/account-graphs`.
These are source/codec checks, not sibling Wire integration, persisted-data,
native macOS or installed-canister qualification. No sibling files, release
versions, commits or publication changed; prior Principal work remains intact.

The follow-up account sanity review found no production defect. Expanded tests
cover all 64 trimmed hex lengths, 53,504 text mutations against upstream admission,
4,096 ordering comparisons, upstream hash parity, explicit-zero Serde tokens and
malformed array lengths. Rust 1.88's 14 unit/3 ledger integration tests, standalone
native/MSRV/Wasm consumer and strict selected Clippy pass. Evidence is retained
in `/tmp/platform-account-review-{tests,consumer,clippy}.log`.

## Released 0.3 owned Principal

The maintainer authorized an owned Principal inside Platform, retaining Apache
provenance and unchanged upstream cryptography. Source now exposes owned root
`Principal`/`PrincipalError`, with explicit registry
`principal::{ExternalPrincipal, ExternalPrincipalError}` for external DTO/client
boundaries. Candid's contract still exports the registry type. The owned type
has no Candid trait or marker handling and no generic range-step implementation.
Parsing admits at most 63 text bytes before work; parsing/Display use stack
buffers. Original bytes, padding and length-first ordering are preserved.
Self-authentication delegates to registry ic_principal; this is not dependency
removal. See [the migration](../migration.md#owned-principal-migration-03).

Workspace versions are now 0.3.0. The implementation pass preserved the incoming
webpki-root-certs lock update and added direct dependency edges and the Serde test
helper; the maintainer subsequently committed and released that work. It did not
qualify sibling migration.

Qualification passes on Linux: all-feature native tests and strict Clippy;
Rust 1.88 Principal/runtime/Candid tests; standalone Principal, Candid and CDK
native/MSRV/Wasm consumers; sixteen native/Wasm production graphs; warning-denied
Rustdoc; formatting, pins, boundary, documentation and runtime provenance checks.
Principal regressions compare 7,680 byte/text values with the upstream owner,
check both owned/borrowed Serde bytes and preserve self-authentication vectors.
A disposable Rust 1.88 allocation probe confirms zero allocation/reallocation
calls for parsing, oversized rejection and Display into a prepared destination;
`to_text` performs one result allocation. This is not a speed or Wasm-size claim.
Probe evidence: `/tmp/platform-principal-measure/run.log`.

Installed Testkit/PocketIC 16.0.0 qualification passes for Rust 1.91 and 1.99 Wasm,
each Wire-only and external-Candid mode: RPCs, raw awaits, traps, timer cancellation,
upgrade, cycles, stable rollback and certificates. External mode additionally
preserves management-call and checkpoint bytes. Sources/artifacts are retained in
`target/cdk-runtime.Jy8nvl`; logs are
`/tmp/platform-principal-installed-{1.91.0,1.99.0}-{wire,external-candid}.log`.

The maintained runtime fixture's older Testkit/Timers require registry Platform
0.1.18. Its previously rewritten lock omitted that compatible version; it is now
resolved explicitly beside the local candidate. The graph still has one registry
executor 2.0.0 and ic0 1.2.0. Release-lock regression permits only the necessary
local version-qualified dependency-reference updates, preserving registry nodes.
This fixture is compatibility evidence, not actual next-minor sibling adoption.
Native macOS, live IC and consumer migration qualification remain outstanding.

## Prior delivery evidence

Issue [#27](https://github.com/dragginzgame/ic-platform/issues/27)'s existing
[PR #28](https://github.com/dragginzgame/ic-platform/pull/28) implementation is now
applied locally on released 0.2.1 (`de1c469`), with pending 0.2.2 notes. It adds
eight original PocketIC root/REST identities, including effective-principal
routing, pending ingress and the management 0.8 chunked-install mode. It changes
no client behavior, server ownership or retries. The pre-existing Tokio lockfile
update is preserved unchanged and is not part of this fix.

Focused Linux validation passes: upstream identity coverage on Rust 1.88, all
three Platform-only consumer tests and Wasm checks on development Rust and Rust
1.88, eight native/Wasm dependency graphs, admission refusal tests, formatting,
pins, boundaries and documentation links. Evidence is retained at
`target/boundary-consumer.6tXBv4` and `target/issue27-graphs`.
This is local source qualification, not a supplying publication, native macOS or
installed consumer qualification. Keep #27 open through delivery; Testkit must
require the supplying release before Backup/Canic/Blob can adopt its forwarded
surface through ordinary registry resolution. Existing PR #28 remains the delivery
vehicle; no replacement PR, release or sibling edit was made in this pass.


The checkout at `c7da516` declares **0.2.0** for Platform and its generator and
contains the maintained non-cryptographic runtime introduced through
[PR #35](https://github.com/dragginzgame/ic-platform/pull/35). The earlier 0.1.18
candidate identity below describes historical qualification, not the current
package version. This documentation pass does not establish registry publication
or current GitHub CI status. Sibling checkouts remain read-only.

The protocol contract explicitly includes public, third-party callers: all owned
application endpoints use Wire. Candid is only an adapter for required IC service
interfaces, plus legacy persisted checkpoints. The [maintenance and recovery
contract](../migration.md#runtime-maintenance-and-recovery) defines remaining
upstream responsibilities, intentional runtime-change admission and recovery
acceptance. A clean-cache recovery drill and actual Canic consumer qualification
are not established by the existing fixture evidence. No new dependency fork,
cryptographic change, release or publication is part of this pass.

This contract pass passes formatting, documentation links, runtime source and
mutation checks, lock/graph regressions and all five generator tests on Rust 1.88.
It changes no runtime behavior and does not rerun installed or macOS qualification.

The prior runtime integration passed Rust 1.88 generator/runtime tests and Candid-free
Wasm compilation, 24 actual native/Wasm production graphs, 66 admission cases,
feature selection/closure regressions, formatting, runtime provenance/lock checks,
dependency pins/ownership and documentation links. The three remaining scaffolding
source/license checks pass at `target/upstream-scaffolding.dayzwB`. The original
pending work is backed up at `/tmp/platform-runtime-local-integration.msFBme`.
That integration reconciled pending tooling/docs and removed its superseded CDK
proposal branch. Temporary paths below are historical evidence locations, not
durable recovery artifacts.

## Runtime and protocol contract

`cdk` selects owned explicit-codec entrypoints, system adapters, raw calls and
protected spawning. Its normal/build graph contains neither Candid nor full
`ic-cdk`/`ic-cdk-macros`. Every controlled application endpoint uses Wire;
Platform has no Wire dependency and installs no default codec or resource policy.
Query/update macros require explicit decoding even for unit requests, and explicit
encoding unless manually replying. Argument-bearing lifecycle hooks also require
a decoder; no-argument IC lifecycle hooks retain raw no-input behavior.
Implicit Candid endpoint/service generation is retired.

The sole executor remains registry `ic-cdk-executor =2.0.0`; raw system calls
retain `ic0 =1.2.0`. Imported call submission, callback/trap cleanup, protected
spawn semantics and the upstream multithreading refusal are preserved. No executor
implementation is copied. Call/response/cost/performance/error/stable-error types
now have Platform identities. The 0.4 candidate also owns runtime RejectCode and
signature-key identity; ExternalPrincipal, external DTOs, certification and BLS
retain registry identities. See
[the migration contract](../migration.md#maintained-runtime-and-wire-entrypoint-adoption).

Raw call constructors begin with empty transport bytes. Every Wire call supplies
its request frame; required external no-argument Candid calls explicitly use
`candid::contract::encode_args(())` and supply the resulting raw bytes. Bounded calls still default to 300 seconds. Callers retain
cycles, deadlines, retries, budgets, authentication and application interpretation.
Select `candid` separately for management/system and standard ledger/NNS/SNS calls.
Existing Candid checkpoints remain an explicit legacy persisted-data exception,
with original bytes and restore behavior. No storage implementation, allocation,
retry policy, cryptographic primitive or trust root changes.

Imported maintained adapters retain the original Apache-2.0 license and provenance
under `crates/ic-platform/src/cdk/upstream/`; the combined library declares
MIT AND Apache-2.0. Reviewed unsafe callback/legacy byte-copy modules are narrowly
allowed under the workspace deny lint. The superseded CDK macro-gating patch and
obsolete Candid service fixture are retired. Pending crypto-scaffolding patches
remain local uncommitted work, separate from PR #35.

## Qualification evidence

Focused Linux checks pass at Rust 1.99 and MSRV 1.88: generator diagnostics/context
coverage, raw constructor and explicit external-unit tests, bounded Candid decoder
regressions, and the separate Platform-only native/Wasm consumer. This standalone
consumer declares no direct Candid/CDK dependencies. Evidence is in
`/tmp/platform-owned-runtime.kzhHt0/platform/target/boundary-consumer.P8ZOXK`.
Strict selected Clippy, warning-denied raw/external Rustdoc, formatting,
dependency pins/ownership, documentation links and runtime-lock checks pass.
Source fingerprints match the original CDK 0.20.3 callback/submission and protected
spawn regions; mutated cleanup predicates/diagnostics and licenses are refused.
Feature selection/closure/admission regressions and 34 focused actual native/Wasm
production graphs pass. The routine full matrix and local `make ci` are release-owned
and were not run for this task.

The actual committed Timers 0.29.2 source at
`41ebd6c3d29681e61be5b9b223333a6499930f87` was archived into
`/tmp/platform-runtime-timers-proof`. Both real runtime and size probes compile
for Wasm at Rust 1.91 with only a disposable Platform override and no source edits.
Their native/Wasm normal/build graphs are Candid-free and resolve the sole
qualified executor/ic0 identities. This capture selects Wire 0.4.6; the maintained
installed fixture selects Wire 0.4.7. Neither is a sibling migration or publication.

Installed Linux PocketIC **16.0.0** qualification passes for all four selections:
Wasm from Rust **1.91 and 1.99**, each pure Wire and explicit external Candid.
All application RPCs and installation/upgrade arguments use Wire in both modes.
The latter additionally calls management `raw_rand` with required Candid bytes
and checks original legacy checkpoint bytes across upgrade. Tests cover awaited
and composite calls, typed rejection/submission-pressure/insufficient-cycle
failures, guards, malformed/unit frames, all four Wire allowances, response-encode
and callback traps, protected/weak/migratory tasks, Timers await/trap/cancellation/
rearm, upgrade, cycles/refunds, stable bounds/rollback and certificate lookup.

Sources, frozen locks, metadata and four distinct Wasm artifacts are retained at
`/tmp/platform-owned-runtime.kzhHt0/platform/target/cdk-runtime.eNOQ6w`;
logs are `/tmp/platform-installed-{1.91.0,1.99.0}-{wire,external-candid}.log`.
Testkit owns the admitted server lifecycle and transport. These are installed
local simulator results, **not native macOS or live IC qualification**.
No measured Wasm-size reduction is claimed.

## Contributor tooling and adoption

Shared Tooling is the reviewed **0.26.4** snapshot at
`dc1ced7e240e90f0c6daa89fb6248fcb074164eb`.
Numbered pending headings also accept `- Unreleased`. The pinned IC Build 0.18.0
release editor and two dependency-role catalog sections remain in place.

Frozen CI consumers need one unfiltered `cargo fetch --locked` during explicit
setup. Native plus Wasm target filters still missed `wasm-bindgen-futures 0.4.79`
for the native-only PocketIC client. A disposable public cache reproduces the
omission and successful unfiltered preparation at
`/tmp/platform-all-target-cache.vyrGZl`. The focused repair is in
[PR #34](https://github.com/dragginzgame/ic-platform/pull/34), linked to
[#33](https://github.com/dragginzgame/ic-platform/issues/33); PR #35 includes the
same prerequisite. Consumer validation remains frozen, with no per-consumer
fetch loops or lock-resolution changes. Those PR-head observations are historical; this pass does not recheck GitHub
status. Native results must be tied to the actual supplying revision.

Consumer adoption requires the supplying next-minor registry release, actual
consumer generator/manifest/import migration, and its own wire/retry/storage
qualification. No compatibility re-export package or permanent duplicate owner is
introduced. Existing Testkit external Candid decoder/DTO responsibility remains
at Platform; Testkit owns transport/server policy. Parser, Agent and PocketIC stay
native opt-ins and are excluded from canister deployment graphs. Product artifact
admission remains consumer-owned; dependency-role labels do not enforce deployment
exclusions by themselves. Native macOS and live IC acceptance remain separate.

## Preserved pending scaffolding work

The prior scaffolding qualification started at `392032a` (released 0.1.18).
That pass left production dependencies, versions and locks unchanged. Its pending
notes now join 0.2.0. Production adoption awaits the pending
source-distribution decision: maintained source variants and explicit identity
migration, or continued registry identities with the unsupported trims deferred.
No source-admission policy was weakened and no crypto replacement was selected.

New packaging patches retain the signature store under a default-enabled feature,
disable it only on the verifier edge, and gate secp256k1 scaffolding behind a
default-enabled `full` feature. The CDK macro patch is now superseded by the
maintained runtime.
`make check-upstream-scaffolding` now checks the three remaining crypto-scaffolding
proposals. Its earlier run verified source/license preservation and prepared
separate vector builds at `target/upstream-scaffolding.wtHSSX`; a
deliberate hash-body mutation is refused (`/tmp/platform-scaffolding-refusal.log`).
Source comparison preserves every original secp source line after removing only
feature attributes; no cryptographic function body, constant or vector changed.

Disposable qualification at `/tmp/platform-scaffolding.mpkcl3` passes Rust 1.88
creation tests with the store disabled (17), verifier tests (12), and default
creation tests with its store retained (24). Separate original/reduced/full secp
builds match 64 derivations plus four malformed-key outputs; the reduced native
and Wasm graphs compile without rand, rand_chacha, getrandom, PEM or simple_asn1.
Inherited upstream deprecation/unused-import warnings remain in this proposal;
strict-lint qualification is not claimed. Standalone Platform creation, verifier
and CDK consumers pass native tests and Wasm checks/builds on Rust 1.99 and 1.88.

The superseded optional-macros CDK variant passed installed Linux PocketIC 16.0.0
tests with Rust 1.99/1.91 Wasm producers, including callbacks/traps, timer cancellation,
upgrade and rollback. Evidence is retained under the disposable Platform copy at
`platform/target/cdk-runtime.ImAZc8`; its refreshed test lock selects Testkit
0.54.1 and Timers 0.29.2. Executor =2.0.0 and ic0 =1.2.0 remain single registry
identities. This is source-copy qualification, not a supplying release, macOS
qualification or live-network qualification.

Thirteen frozen normal/build consumer graphs were inspected read-only: Auth default,
portable verification, chain-key verification and signature preparation; Timers;
Host certificate and interface/certificate; Query certificate-host; Testkit native
and Wasm; Canic Core native/Wasm and Host native. No further justified manifest-only
cut was found; signature preparation retains Candid through upstream packaging.
Canic Host inherits CDK through shared Core: removing that edge needs a Core
feature/API split rather than an import edit. Canic Core and Testkit Wasm graphs
exclude host/parser/testing clients (excluding the selected root itself).
Evidence: `/tmp/platform-consumer-scaffolding-graphs`.
The preserved local graph changes reject unintended RNG/entropy implementations
while permitting rand_core traits and selected native host requirements. Prior routine graph qualification
passes 145 selections per target with 222 queries at
`target/feature-graphs/attempt.pJssgw`, including the portable crypto triple.
Source checks, synthetic refusals, pins/boundaries, formatting and doc links pass.
Those proofs did not migrate siblings or publish source variants.

Maintainer constraint for dependency trimming: retain DFINITY-authored
cryptographic implementations and upstream identities. Do not replace public-key
derivation, signature verification, key encoding/hashing or the BLS backend with
Platform-owned implementations. Earlier replacement proposals are withdrawn;
existing crypto prototypes remain historical evidence, not adoption candidates.
Further trimming targets unused modules, duplicate generators and unnecessary
feature/dependency activation while preserving the selected cryptographic code.
