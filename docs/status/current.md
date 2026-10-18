# Current handoff

The feature-gate optimization starts from clean release `07e3835` (0.1.13).
The local 0.1.14 draft changes validation only; package versions are unchanged.
Wasm keys omit Agent/parser/secp256k1 leaves only after checking their exact
single optional native-dependency contracts. Changed feature edges, target
conditions, optionality, aliases or duplicate dependency declarations fail closed.
Defaults remain distinct. Every original selection is still admitted against its
Cargo graph, now by one AWK process per target rather than repeated subprocesses
and graph copies. Attempts retain target-specific TSV maps and canonical graphs.

Full selection enumeration confirms 8,193 selections per target and 4,609 native
plus 577 Wasm keys: 5,186 Cargo queries versus 9,218 before (43.7% fewer), with
all 16,386 checks retained. Forty-six adversarial/control cases agree with the
previous validator; shared-key reuse and missing graphs fail as intended. Seven
changed native-leaf contracts are refused. Sixty fresh native/Wasm observations
across default-free, executor, CDK, verifier, management and ledger selections
with individual/combined host features pass old/new admission and key-equivalence
comparison. Evidence is under `target/feature-optimization-proof/`.

The previous turn's exhaustive artifacts are absent from this checkout. A fresh
full-matrix trial was stopped before completion to limit package-cache contention;
its partial outputs are not a pass. Full matrix/full CI remain release-owned.
No measured wall-clock speedup, new Rust compilation, installed-canister or native
macOS qualification is claimed. Shell syntax, formatting, documentation links,
pins, direct boundaries and unchanged snapshot checks pass. No shared snapshot,
sibling, release version, staging, commit, remote or publication was changed.

## Previous secp256k1 qualification

The secp256k1 addition starts from clean `a6bbc2d` (the committed 0.1.13 draft;
package metadata is still 0.1.12). It adds the maintainer's exact
`ic-secp256k1 =0.3.0` behind independent native-only `secp256k1`, with explicit
public key, master-key identifiers and decoding-error exports. Canic's read-only
`ops/config/chain_key` consumer establishes the scope: offline mainnet/PocketIC
public derivation. Consumers retain allowed key IDs, root/path admission and
configuration policy. Upstream RNG dependencies are not made available to Wasm;
no entropy backend, private-key owner or cryptographic implementation is added.
See [the migration contract](../migration.md#native-secp256k1-public-derivation).

The Platform-only consumer passes native Rust 1.99/1.88 tests and Wasm checks
(`target/boundary-consumer.ZSDBka`). Native original-owner type/byte identity
passes at 1.88. Six fresh production graphs under `target/secp256k1-proof` verify
default/executor isolation and native-only 0.3.0 selection without Candid/CDK or
host transport. The feature matrix admits the new optional native feature and
rejects unselected/non-registry/wrong-version instances; full enumeration remains
release-owned. Combined runtime graphs still pass with the unchanged maintained
lock (`target/cdk-runtime.AHxztv`). Clippy, Rustdoc, pins, boundaries, doc links,
runtime graph rejection cases and unchanged shared snapshot checks pass.

The root lock adds only secp256k1 and its missing PEM/base64/ASN.1 dependencies;
existing package versions are unchanged. This is local source qualification, not
publication, full Canic adoption, native macOS or installed-canister acceptance.
No staging, commit, remote, publication or sibling edit was performed. Earlier
issue/registry qualification records below describe the already committed work.

## Previous issue qualification

This pass started clean at `338b14b` (release 0.1.12); package versions remain
0.1.12. The undated 0.1.13 changelog describes the new local enforcement fix.
No staging, commit, publication, remote or sibling checkout changes were made.

Review of the open producer issues found no additional requested API missing
from the current source. Canic now reports
[published 0.1.12 adoption](https://github.com/dragginzgame/ic-platform/issues/22#issuecomment-6251010946),
resolving #22's import blocker; its Memory/Storage failures remain separate.
Query and Timers report adoption of Testkit 0.54's canonical limits in
[#2](https://github.com/dragginzgame/ic-platform/issues/2), and Query/Blob report
removal of direct CDK dependencies in
[#1](https://github.com/dragginzgame/ic-platform/issues/1). These are consumer
reports, not new sibling qualification performed here. No open PR overlaps this
pass. The [0.1.12 CI run](https://github.com/dragginzgame/ic-platform/actions/runs/39135409911)
was still in progress when inspected; it does not yet establish producer closure.

The combined CDK/Timers fixture previously checked executor/system identities
without rejecting native parser, Agent or Testkit reachability. It now rejects
those packages and unqualified CDK runtime/macros in both native/Wasm canister
normal/build graphs. The host driver remains outside that selection. Twelve
adversarial rejection cases and a passing control are part of
`make check-runtime-lock`, which is included in CI. `--check-graph` runs the real
locked graph admission without requiring compilation or a server.

Fresh Linux native/Wasm graphs pass at `target/cdk-runtime.tgeJv0`; the regression
cases, release-lock checks, release adapters, formatting, documentation links and
unchanged shared snapshot checks pass. This change affects qualification scripts only, with no Rust API,
protocol, storage, decoder policy or runtime implementation change. Full CI,
Rust/MSRV compilation and installed-canister execution were not repeated for
this script-only change. Native macOS and live IC acceptance remain separate.
The next consequential step is the completed 0.1.12 producer CI evidence and
published-selection consumer qualification, rather than further speculative APIs.

The subsequent issue pass preserved the pending graph changes and added
`check-platform-consumer.sh --published X.Y.Z`. Its exact registry selection
rejects facade/generator path or Git replacements before compilation. Published
0.1.12 now passes all three standalone CDK tests and linked Wasm on Rust 1.99
and 1.88 with only Platform declared, retaining the existing DID fixture and
system/error import checks. Both selected packages are registry 0.1.12. Evidence
is under `target/boundary-consumer.2qKcg1` (metadata, lock and both target graphs).
Those graphs pass the runtime identity and host-dependency exclusions. Four
injected facade/generator path/Git replacements and five malformed version
arguments are rejected; shell syntax, formatting and document links pass.
No Rust implementation or sibling was changed, and no new installed test was run.

The open-issue refresh also shows #19 closed; Auth reports published adoption of
the signature/tree APIs in #16–#18. This pass did not change issue status. Release
CI still had all three native hosts validating when inspected. Auth/Wire's own
published-runtime application acceptance remains separate from this standalone
archive proof. The README now correctly describes the nonempty tooling catalog.

## 0.1.12 implementation record

The following report predates the maintainer's 0.1.12 release. Its temporary
artifact paths are historical and are not current proof files in this checkout.

This task started clean at `ffc36b6` (0.1.11). Package versions remain 0.1.11;
`CHANGELOG.md` contains the undated 0.1.12 draft. No staging, commit, publication,
remote or sibling checkout change was made. The shared snapshot is unchanged.

The local CDK extension addresses the reviewed Platform surface requested in
[#19](https://github.com/dragginzgame/ic-platform/issues/19),
[#20](https://github.com/dragginzgame/ic-platform/issues/20),
[#21](https://github.com/dragginzgame/ic-platform/issues/21) and
[#22](https://github.com/dragginzgame/ic-platform/issues/22): checkpoint/stable
passthroughs, certificate publication, typed call errors and system/cost helpers.
All are explicit original upstream identities under the existing `cdk` feature.
No storage implementation, protocol, retry policy or executor is replaced.
See [the adoption contract](../migration.md#remaining-consumer-cdk-helpers).

The maintained runtime lock had retained 0.1.10 local identities after release
0.1.11. It now matches the workspace. Release preparation updates only those two
local identities and includes this lock in release files; normal validation
rejects drift. A disposable regression proves wrong-base refusal, unchanged
external selections and idempotent retry. Dependency refresh remains explicit.

Focused Linux evidence: standalone Platform-only CDK tests and linked Wasm pass
on Rust 1.99/1.88 (`target/boundary-consumer.pOWkHD`); exact function/error identity
tests pass on both compilers. The expanded maintained PocketIC 16.1 fixture passes
for Rust 1.99/1.91 optimized canisters (`target/cdk-runtime.x7mkc4`), preserving
checkpoint bytes across upgrade, stable write/growth trap rollback, certified
root lookup, cycle burning and costs, alongside previous callback/timer/refund
regressions. The initial attempt caught a test using `msg_method_name` outside
`inspect_message`; the fixture was corrected without changing upstream behavior.

Captured dirty Auth, Timers and Wire source copies compile natively and to Wasm
on Rust 1.91 with their direct CDK declarations/imports removed. Archives, base
revisions, source status, metadata and logs are under `target/cdk-completion/`.
Canic's captured copy fails on `ic_memory::ic_stable_structures` imports and
associated Memory/Storage generic errors; its full adoption is not qualified.
These are captured-source checks, not changes to or acceptance of sibling trees.
Auth must review its runtime-bundle dependency allowance while retaining Wire
codecs; Canic must resolve its separate Memory/Storage migration. Existing
consumer generators and installed policies remain their owners' responsibility.

The next delivery step is the maintainer's paired facade/generator release,
followed by published consumer adoption and consumer-owned installed tests.
No transitive CDK/Candid removal or Wasm-size reduction is claimed. Full CI,
native macOS and live installed-IC qualification have not been run for this draft.

Strict CDK fixture/identity and runtime-driver Clippy, CDK Rustdoc, formatting,
pins, direct boundaries, document links and unchanged-snapshot checks pass.
Six focused native/Wasm production graphs confirm Candid-free defaults/executor,
no host parser/Agent/Testkit and one CDK executor 2.0.0/ic0 1.2.0 identity.
The combined canister's deployment graph also excludes those host dependencies.
A disposable 0.1.11-to-0.1.12 metadata preparation passes coordinated package,
manifest ordering and runtime-lock checks (`target/release-lock-proof.orzQX6`).

## Previous qualification record

The following records describe earlier work and its then-current evidence.
Temporary evidence directories cited below are absent from this checkout; their
paths are historical, not newly verified artifacts. The maintained sources above
replace reliance on temporary runtime fixtures.

[The 0.1.9 native CI run](https://github.com/dragginzgame/ic-platform/actions/runs/39122716825)
passes Linux x86_64, macOS 15 Intel and Apple Silicon. Delivered DTO, feature-graph
admission and Rust 1.88 Wasm enforcement issues are closed with that evidence.
Earlier cancelled runs are historical, not additional gates to rerun. This run
does not qualify the then-uncommitted 0.1.10 draft. Full local CI was not run.

The draft adds independent `bls`, `canister-sig-creation` and
`signature-verification` features and the missing certification `AsHashTree`,
`RbTree` and `Hash` exports. All are explicit upstream identities, with no store,
trust/freshness policy, clock, root publication, wrapper or custom cryptography.
Defaults remain empty. BLS selects alloc without RNG defaults and stays free of
creation, certification, Candid, ic0, executor and host tooling. Creation and the
full verifier retain upstream Candid/ic0; direct isolation does not remove them.
See [the signature contract](../migration.md#canister-signature-and-bls-extension).

Four Platform-only consumers pass native and Wasm checks on Rust 1.99/1.88,
preserving original DER/hash/BLS/certificate bytes, typed failures and nested
mutable tree/witness operations. Separate Rust 1.88 native/Wasm function-pointer
proofs establish bidirectional key/signature/error/tree and trait identity.
The four focused library subjects, existing decoder cases with signatures enabled,
all-feature Wasm checks and actual optimized Wasm library compilation pass at
1.88. Strict all-feature Clippy/Rustdoc and formatting/pin/direct-boundary/docs/
unchanged-snapshot checks pass. Clippy's initial fixture-loop lint and its MSRV-
compatible correction are retained; no production behavior changed in that fix.

A captured copy of dirty Auth at base `2da4dc0e` changes only dependency selection
and imports. All 49 signature-store, verifier and certified-proof tests pass on
Rust 1.91, with combined-feature Wasm library checking. Auth's implementation,
tests and lock inputs remain unchanged by this task; its root manifest changed
concurrently. These results qualify the captured copy, not its evolving checkout,
installed qualification canister, upgrade or ingress workflows.

Before the CDK addition, the signature feature gate covered all 2,048
combinations plus defaults on native/Wasm and preserved reviewed runtime/crypto
identities. Those 4,098 graph observations passed, followed by default-native and
executor-Wasm compilation.
Nine adversarial admission replays fail with an unchanged passing control.
Four aliased inactive-target direct-dependency cases likewise fail with a passing
Platform-only control. These replays qualify enforcement, not cryptographic or
canister execution. Focused sources, locks, vectors, graph/identity/rejection
proofs and logs are under `target/auth-boundary/`; incoming before-images are
under `target/auth-boundary-before/`. Native macOS qualification of this draft
requires its own delivered CI result.

`patches/canister-sig-principal.patch` proposes the canonical Principal-only
dependency correction at upstream `3e5786bb`. It applies to exact reviewed main
preimages. All 24 upstream library tests pass on Rust 1.99/1.88, plus Wasm at 1.88;
disposable downstream creation/verifier probes preserve fixed vectors and lose
Candid on native/Wasm only with the candidate selected. ic0 remains 1.2.0.
The patch is not submitted or selected in production. Existing package versions
stay selected; the lock adds only creation 1.3.1, verifier 0.3.0 and their
representation-independent-hash 3.2.0 dependency. No Wasm-size claim is made.

Fresh registry-only `=0.1.9` standalone Candid, management and raw-system
consumers pass native tests and Wasm checks on Rust 1.99/1.88. Candid preserves
configured/default/staged decoding, derives and external service/wire formats.
Management imports all 24 Backup snapshot/status identities plus Canic's
`DeleteCanisterSnapshotArgs`; fourteen original-owner snapshot/status/delete
vectors preserve bytes, exact arity, truncation, byte and skipping refusals.
Raw-system checks compile caller code without executing IC system calls on a
native host. Each production graph excludes CDK frontends, executor, Agent,
parser and storage; raw-system selection also excludes Candid.

A separate management-only registry proof, without Platform's Candid feature,
preserves bidirectional Rust identity with original management types 0.11.0 on
native/Wasm Rust 1.88. A registry-only executor consumer also compiles on both
targets at 1.88; its graph retains only executor 2.0.0 and ic0 1.2.0 among runtime
packages, with no Candid, frontend, Agent, parser or storage. Evidence is under
`target/registry-019/`: fresh index records, fixture sources/manifests/locks,
metadata, native/Wasm graphs and qualification logs. These are retained 0.1.9
publication proofs, separate from the new draft's source-based qualification.
The Shared snapshot remains unchanged.

Canic reports [local adoption of the published delete DTO](https://github.com/dragginzgame/ic-platform/issues/13#issuecomment-6249299659),
removing its obsolete local request type while preserving unbounded wait,
target/method, IDs and authority/recovery behavior. Its type/byte/cross-decoding
proof and Wasm checks pass; this is not an installed snapshot deletion result.
The original Backup surface was already accepted against published 0.1.8.
The migration contract now reflects delivery and preserves intentional partial
response projections and all consumer-owned limits, retries and recovery.

The new optional `cdk` feature supplies Platform-owned entrypoint/service macros
and explicit upstream call/system/future exports. The maintained helper is an
Apache-2.0 fork of published macros 0.20.3 at
`624606f7fd1ecd6668e9608594946486aec29167`; runtime `ic-cdk =0.20.3` remains
published upstream code, with one executor 2.0.0 and ic0 1.2.0 identity. CDK
activates the exact executor/raw capabilities in downstream resolution.
Executor-only selection remains Candid-free, and no runtime feature enables the
host parser or Agent. The old CDK contribution patch is retired; this generator
has one maintained local owner and needs no upstream acceptance. The independent
crypto proposal and incoming signature implementation remain unchanged.

The helper defaults generated runtime/Candid paths to Platform and retains parsed
overrides for renamed dependencies. It preserves upstream quotas, zero-argument
handling, guards, tuple/unit replies, custom/manual/hidden methods, contexts and
protected spawning. Explicit helpers preserve upstream allocating argument/caller
Principal construction and controller 0/1/panic behavior. No unchecked raw flag
comparison, copied runtime, automatic limit or alternative executor is introduced.
Consumers retain admission, call wait mode, cycles, deadlines, retries and policy.

Fourteen generator tests and all-feature facade/helper tests pass on Rust
1.99/1.88, with all-feature Wasm library checking at 1.88. A standalone consumer
with only Platform declared preserves the original CDK's exact DID and compiles
to optimized Wasm at 1.88. A renamed-dependency proof passes native/MSRV and
release Wasm with explicit runtime/Candid/derive/Serde/service paths. Strict
all-feature Clippy/Rustdoc pass. The upstream-derived Darling expansion needs
one narrowly documented redundant-field lint allowance in the generator module;
initial failure/correction logs are retained.

Linux/PocketIC 16.1.0 comparisons use published Testkit 0.54 for byte admission,
managed startup and teardown. Original CDK and Platform-only canisters each pass
20 matching observations: scalar/tuple/unit/DTO/raw replies, composite queries,
controller helper, guards, malformed/arity/skipping traps, bounded/unbounded
awaits, original call rejection, callback trap cleanup and subsequent success,
protected-task outliving panic, migratory work and init/post-upgrade arguments.
Original-CDK and Platform fixtures combined with published Timers 0.28 each pass
25 observations, adding timer callback await/trap cleanup, cancellation, later
scheduling and post-upgrade runtime reinitialization. Timers owns the sole timer
export; final native/Wasm graphs resolve one executor/system identity. This is
focused simulator evidence, not full Timers, Query or Canic adoption and not live
IC qualification. Early fixture tuple/diagnostic expectation failures and their
fresh-attempt corrections are retained; generator behavior was not altered to
make those expectations pass.

The pre-optimization feature matrix had 4,096 combinations plus defaults (8,194
native/Wasm observations). This continuation ran 30 real focused graph
observations, default/executor compilation and 14 adversarial admission replays
with a passing control. A separate CDK-only resolver control retains the qualified
versions and refuses synthetic executor 2.0.1 and ic0 1.2.1 candidates. The
complete expanded matrix remains a release gate.
It admits runtime/generator packages only with CDK selected, rejects additional
or alternate-source executor/system/frontend/generator identities and preserves
host/Wasm exclusions. The full release CI was deliberately not run locally.

Workspace checks now cover the helper. Coordinated metadata preparation/checks
pass in a disposable copy from 0.1.9 to 0.1.10, including both package versions
and the compatible helper declaration. Metadata was 0.1.9 at that earlier qualification.
`make publish-check` successfully packages/verifies both packages with all
features and explicitly aborts both uploads as a dry run. Cargo owns dependency
ordering; publication intent records both identities. Offline workspace
verification triggers Cargo 1.99's temporary-registry checksum error (also on
1.99.1); the normal index-enabled package/dry run passes. No offline publication
support or upload success is claimed. Reconcile both registry identities after
an interrupted dispatch; partial publication is not transactional.

Focused inputs, source copies, locks, graphs, macro tests, packaging, release and
simulator logs are under `target/cdk-generator/`; incoming content is retained
in `target/cdk-generator-before/source.tar`. The migration contract gives
imports/manifests/generator/test changes. Runtime adoption still requires the
paired published release, actual consumer wire/task/timer qualification, native
macOS CI and installed-IC acceptance. See
[the runtime boundary](https://github.com/dragginzgame/ic-platform/issues/1).

Testkit 0.54.0 is now non-yanked and published at
`ff88087d6757121ac92e0dec4705ecaccfd8ebed`, archive checksum
`787c2ea232d8dd36586d7bd2fb72d78d098a2b892cedd1a319450b2ecea7347d`.
The index/archive agree; published calls/errors directly use Platform's canonical
identities without the former decoder dependency. Its
[release CI](https://github.com/dragginzgame/ic-testkit/actions/runs/39124256235)
passes all Linux x86_64, macOS 15 Intel and Apple Silicon jobs at this inspection. Query's dirty manifest/driver already select
0.54 and Platform limits, but no new Governance simulator run was performed here.
Qualify actual published-selection drivers before retiring their old declaration;
consumers still selecting Testkit 0.52/0.53 retain the narrow native exception.
See [producer coordination](https://github.com/dragginzgame/ic-platform/issues/2)
and [the migration contract](../migration.md).

Native macOS producer CI, Linux simulator results, Wasm compilation and live IC
are separate qualification categories. The new runtime fixtures were installed only in local PocketIC; callback/trap
comparisons above do not qualify live IC or snapshot operations. Direct dependency isolation does not prove
transitive removal or measured Wasm-size reduction. Deployment admission remains
consumer-owned. Shared Tooling stays pinned to 0.24.0 at `cab12cd`; no snapshot
payload, runtime policy, external wire bytes or persisted data changed.
