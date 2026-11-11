# Current handoff — 2026-11-10

## Independent passive ICRC DTOs — qualified locally

The remaining metadata, transfer/memo, allowance, block/certificate and ICRC-21
declarations/helpers now come from the authorized specification-only context.
Public fields/variants/traits, numeric ranges, conversions, default handling,
diagnostics and Serde/Candid bytes remain. Field docs replace seven blanket lint
exceptions. Within ICRC, only the complete value/hash module remains imported;
its license, vectors and guards are retained. Other management/external-schema
source elsewhere in Platform remains separate work.

Native/Wasm Clippy and 35 focused tests pass. All 23 grouped JSON/CBOR/Candid
observations match the old owner exactly and now form a permanent fixture.
The existing 28-schema check now runs with ICRC alone and shares its comparison
helper with the other protocol tests. Retained source/license mutation checks
pass. Native/Wasm production graphs and Cargo.lock are unchanged. Evidence:
`target/icrc-dtos.O22GVg`; see [qualification](../design/icrc-dtos-qualification.md).
No speed/Wasm-size claim, installed IC run, macOS or sibling qualification.

First-party replacement licensing, installed Toko's IC0 admission and the separate
ICP derivation draft remain release blockers. Other dirty work is preserved.
No commit, push or publication.

## Independent ICRC Account/archive adapters — qualified locally

The two imported adapters now come from the authorized specification-only
context. External Account identity behavior delegates to Types; its distinct
text errors and sequence-shaped Serde remain. Parsing uses fixed scratch rather
than proportional padding/decoded buffers. Archive references move canonical
Principal/method fields directly, preserving fallible API signatures, generic
bounds and full query signature checks. Remaining ledger DTO/value/hash imports
keep their common license and guards; no blanket notice deletion occurred.

Native/Wasm Clippy and 27 focused tests pass. All 48 independently captured
old-owner text outcomes match exactly and now form a permanent regression
fixture. Frozen Account/archive Candid bytes, Serde distinctions, registry bridges
and retained hash/license guards pass. Native/Wasm normal/build dependency trees
and Cargo.lock are unchanged. Evidence: `target/icrc-adapters.yeW4NY`; see
[qualification](../design/icrc-adapters-qualification.md). No measured speed or
Wasm-size claim, installed IC run, macOS run or sibling qualification is made.

First-party license text/metadata, installed Toko's IC0 admission and the separate
ICP derivation draft remain release blockers. Concurrent/unrelated work is
preserved. No commit, push or publication.

## Independent entrypoint generator — qualified locally

The generator, dispatch and syntax helpers now come from an authorized
specification-only context, with separately authored public consumers. The old
export source/tests and macro license are retired together. The existing owned
external-Candid adapter only changes its two private generator references.
Public macros, export names, explicit codecs, guard/context order, inference and
the sole executor remain. No dependency or feature changed.

Macro/Runtime Clippy passes, as do 104 unit/public/link/doctests and all four
installed raw/prepared × Wire/external-Candid profiles at
`target/cdk-runtime.EpTK6Z` (status 0). A reconstructed same-path old/new generator
comparison at `target/generator-comparison.FQ01yz` passes eight installed runs
and exact Wasm import/export equality. Total Wasm changes range from −872 to
+51 bytes; code-section changes range from −451 to +38 bytes. No speed claim,
fresh macOS run or Scheduling-owned composition proof is made. See
[generator qualification](../design/entrypoint-generator-qualification.md).

No local Runtime or macro LICENSE/NOTICE/provenance files remain. Retained
management/ledger imports still carry their applicable records. First-party
license text/metadata, installed Toko's IC0 admission and the separate ICP
derivation draft remain release blockers. Other dirty work is preserved.
No commit, push or publication.

## Independent raw call transport — qualified locally

The remaining `cdk::upstream` call implementation is replaced through the
authorized specification-only context. A separate context authored public
contract tests before source exposure, then private lifecycle tests using only
the new implementation. Request data stays outside the owned callback state;
raw environment reclamation is explicit, and arbitrary Waker operations execute
outside the completion lock. Public identities, bytes/errors, cycles/waits,
callback rollback and cleanup behavior remain. The old CDK source directory and
its license/provenance/fingerprint scripts are retired. Management/ledger checks
remain; Make invokes the management guards directly. The generator was still
imported at this qualification; its later replacement is recorded above.
Other imported DTO/ledger source remains separate unfinished work.

Native/Wasm Clippy passes. Tests: 52 Runtime unit, seven public call, three CDK
consumer, 13 public executor and two doctests pass. All four installed profiles
pass at `target/cdk-runtime.G67DHY` (status 0), including the new two-call recovery
scenario. The matched old-owner baseline is `target/cdk-runtime.RAIlQV`.
Native/Wasm feature closures, inventory, doc links and retained management/ICRC
source/license mutation checks pass. No fresh macOS or Scheduling-owned
composition proof is claimed.

Matched Wasm totals change by +466–950 bytes for raw-call profiles and
−493–1,001 bytes for prepared-call profiles. Native future inline size grows from
8 to 112 bytes; unpolled futures no longer allocate callback state. Total heap use
and instruction speed were not measured. See
[call qualification](../design/call-transport-qualification.md) for exact results,
source separation and remaining limits.

No dependency or feature changed. First-party license text/metadata, installed
Toko's IC0 admission and the separate ICP derivation draft still block release.
Unrelated dirty work is preserved. No commit, push or publication.

## Independent System API, stable and task adapters — qualified locally

Separate specification-only contexts replaced `cdk::api`, `cdk::stable` and
`cdk::futures`, with fresh tests. Parent integration preserves explicit IC0
delegation, bounded Principal copying, initialized byte buffers, strict UTF-8,
full integer ranges, stable offsets/growth refusal and task cancellation policy.
The old adapter directories are removed. The retained call state machine remains
unchanged and Apache-attributed; its original fingerprint and mutation tests pass.
The generator and management/ledger imports retain their separate obligations.

Native/Wasm Runtime Clippy, 39 unit tests, three CDK consumer tests, 13 public
executor tests and two doctests pass. All four installed raw/prepared ×
Wire/external-Candid profiles pass at `target/cdk-runtime.TDdGXt` (status 0).
Feature-contract refusal fixtures and 17 bounded selections on native/Wasm pass
at `target/feature-graphs/attempt.3PsyRM`. No fresh macOS or Scheduling-owned
composition proof is claimed.

The selected direct pinned wrapper keeps `pin-project-lite`: removing it with a
standard-async wrapper passed correctness checks but added 1.8–4.4 KB of Wasm.
Against the pre-adapter baseline, final fixture totals shrink by 909–2,108 bytes;
code shrinks in three profiles and grows 69 bytes in prepared external Candid.
See [adapter qualification](../design/system-adapters-qualification.md) for exact
measurements and source-separation limits. No speed claim is made.

First-party license replacement, installed Toko's registry-IC0 admission and the
separate ICP derivation draft remain release blockers. Other dirty work is
preserved. No commit, push or publication was performed.

## Independent executor — qualified locally

The maintainer requested continued component-by-component source retirement.
The executor is now independently implemented from a reviewed behavior/API
contract by a separate context that did not read its old implementation/tests.
A second isolated context authored public tests. Parent review/integration
preserved the public owner and handle auto-traits, protected/migratory routing,
query/recovery behavior, generation checks, cached/coalesced wakes, stable task
membership and future destruction outside state borrows. Source records are
under `docs/design`; they do not claim legal clearance.

An intentional lifetime correction now destroys pending tasks when the final
method retention is dropped outside execution; the captured old implementation
left them allocated. Independent tests check exactly-once destruction and
reentrancy. This is documented as a correction, not total behavior equivalence.
The old executor source license/provenance and its fingerprint checker are
removed together. Imported CDK adapters, macros and management/ledger sources
retain their applicable notices and guards.

Native/Wasm Runtime Clippy passes. Nineteen unit tests (including five new
executor-state tests), 13 independently authored public executor tests, three CDK
consumer tests and two doctests pass. Public scheduling traces match the captured
baseline. Native/Wasm feature closures and actual Cargo singleton/coexistence
proofs pass (`target/runtime-singleton.fu8auQ`), as do retained-source guards and
mutation tests. All four installed raw/prepared × Wire/external-Candid profiles
pass with status 0 at `target/cdk-runtime.w76c08`, including await, callback traps,
cancellation, upgrades, cycles, stable rollback and certificates. No fresh macOS
or Scheduling-owned composition proof is claimed.

Matched uncompressed Cargo release fixtures shrink: raw Wire from 273,265 to
226,685 bytes (code 178,753 to 150,462); external Candid from 732,376 to 677,515
(code 509,018 to 473,762). Compiler, manifests, fixture sources, lock/config and
raw feature selections match the captured baseline `target/cdk-runtime.nhOywW`.
See [executor qualification](../design/executor-qualification.md) for hashes,
methodology and limits. No execution-speed or arbitrary application-size claim.

First-party licensing, installed Toko's IC0 admission and the separate ICP
derivation draft remain release blockers described below. The first-party MIT
files remain removed as requested; retained ICP Apache notices/provenance were
restored again because no reviewed independent replacement covers that draft.
Other dirty source/benchmark work is preserved. No commit, push or publication.

## First-party license replacement pending

The maintainer explicitly requested removal of the first-party MIT license files
because a new license is coming. Root/facade copies were already deleted; the
remaining Types, Runtime, guard, browser and owned strict-CBOR copies are now
removed. Do not restore them. No replacement text or SPDX identifier was invented;
manifest license metadata still requires the maintainer's new selection before
publication. Retained third-party Apache source licenses/notices remain in place.
This supersedes earlier instructions to restore first-party MIT files and the
identity qualification's description of MIT as the intended release license.

## Specification-only source retirement — identities qualified locally

After the maintainer confirmed the other writer had stopped, the separately
authorized implementation context completed base Principal and Account from
reviewed protocol/API contracts and black-box observations. It did not read the
old source/tests. RejectCode and unsigned Nat payload encoding were completed
through the same process earlier in this task. Records under `docs/design`
describe inputs and limits; they do not claim legal clearance. Parent integration
preserves Principal's historical binary-format text admission, Account's
effective-zero semantics, exact bytes and error behavior.

Types now inherits the workspace MIT license after its imported implementations
and tests were replaced. Optional self-authentication delegates to registry
`ic_principal` 0.1.5; only this capability brings the supplier's SHA-2 0.10.
Base identity selection remains crypto/Candid/Runtime/Agent-free. IC0 is already
a registry dependency, and copied Agent PEM fixtures were regenerated with
OpenSSL. No cryptographic primitive was implemented by this work.

Native/Wasm Types Clippy and all 26 tests pass. Published Types 0.19.0 public
comparisons match across 16,000 Principal and 3,900 Account cases, including
JSON/CBOR/Postcard bytes. Focused facade Clippy, 74 unit tests and three public
codec tests pass; three explicit benchmarks remain ignored. Runtime Clippy,
25 unit tests, three consumer tests and two doctests pass; one benchmark remains
ignored. Feature closures/refusal fixtures, inventory, codec admission, changed
Rust formatting and local documentation links pass.

Fresh installed Runtime evidence at `target/cdk-runtime.nhOywW` passes raw Wire
and external-Candid awaits, trap cleanup, upgrade, stable rollback, certificates
and management/endpoint contracts. Fresh Agent evidence at
`target/agent-installed.FyGDbv` passes verified queries, updates, read-state and
wrong-root refusal for Ed25519/delegated/P-256/secp256k1. No fresh macOS or Timers
composition proof is claimed. See
[identity qualification](../design/identity-qualification.md) for commands,
counts and evidence boundaries.

Retained CDK and ICP notices/provenance were restored after premature deletion;
ICRC notices were restored earlier. Runtime and management source guards pass.
The existing ICP derivation draft still fails its source guard; it is not an
approved independent replacement. Executor, CDK/macros and retained management/
ledger imports still require their attribution and further work. Their bodies
were not rewritten by the separated identity context.

The narrow Toko Runtime→IC0 checker correction is now present in committed
supplier `3ede2fc` (its manifest selects pending 0.39.0). It has not been adopted
here, and installed 0.38.2 still fails `make check-boundary`. This supersedes the
earlier note describing an uncommitted 0.38.4 supplier draft. No snapshot payload
was edited. The other session's signature implementation/benchmarks and unrelated
dirty selections remain preserved. No commit, push, release or publication was
performed; the combined tree is not release-ready.

## Owned canister-signature protocol — implementation and benchmarks prepared

The maintainer approved owning protocol composition while retaining external
BLS/curve, SHA-256 and certified-tree primitives. The existing creation and
verification public modules now have independent implementations using canonical
Types Principal. A separate agent worked from the reviewed protocol/API contract
without upstream signature bodies or old tests in its context. Public DER/OID/root
key constants were explicitly permitted. See
[the contract](../design/canister-signatures/contract.md) for context custody and
[the comparison report](../benchmarks/canister-signatures-2026-11-10.md) for evidence.
No signature store, timer, new trust/freshness policy or primitive was added.

Both production signature profiles exclude Candid, registry Principal, the old
signature packages, IC0 and an executor on native and Wasm. The old signature
packages are dev-only comparison references. The all-feature production Wasm
closure also has no Candid; native Agent/transport and development qualification
still have their separate dependencies. Public features and key bytes remain;
strict DER, root/delegated identity refusal, owned range decoding/order and
signing-certificate-only time extraction pass. Authentication retains the decoded
certificate for time extraction. Some malformed certificate/shard shapes now
fail closed; this is not universal acceptance equivalence with the raw supplier.

Focused validation: 34 unit/public/differential tests pass, one timing test is
normally ignored and passed separately in release mode; native/Wasm capability
Clippy, all four size-probe Clippy selections, 143 graph mutation/refusal cases,
feature contracts/selections, runtime oracle rejection, Candid/CBOR policy,
manifest inventory, formatting and ShellCheck pass. The CBOR allowance follows
the already-moved Principal tests.rs path; no Principal implementation was edited.
Existing unrelated source-retirement work remains outside this qualification.

Native key operations take 43–81% less time than the raw DFINITY public APIs in
this run. Verification takes about 15% more time, including Platform's stricter
identity admission. The matched shrunk Wasm creation probe is 6.0% larger and
verification 1.0% larger; dependency removal is not a binary-size win here.
Raw rows, compiler/methodology and input hashes are retained in the report;
Wasm binaries are reclaimable artifacts in target/evidence/signature-size.dYBDJz.
Installed PocketIC 16.1 measurements now cover 156 successful workload assertions.
With root/delegated identity admission on both sides, owned and DFINITY-based
verification differ by less than 0.01% in instructions. Owned warm whole-update
debits are 252.63 million cycles direct and 498.26–498.29 million delegated;
the first verification adds about 13.6 million. Raw DFINITY omits those checks
and uses about 16% fewer instructions. Key encoding improves, while 4 KiB hashing
uses 5.1% more Wasm instructions. The report retains cold/warm rows, matched
no-op methodology, hashes and three-arm Clippy/identity evidence under
target/evidence/signature-instructions.Fknhmz. These are local simulator results.
No live-network qualification, release, commit, push, sibling adoption or
publication is claimed. Existing dirty work and the installed Toko snapshot are
preserved.

The follow-up size refresh saves 494,495 total and 445,938 code-section bytes
in the management codec probe versus the former upstream adapter (83.6% total).
Both arms passed Clippy/build and source stability checks; evidence is retained
at target/evidence/candid-removal.EXDO1s and in the codec matrix report. The
installed signature benchmark saves 122,999 total and 122,396 code bytes versus
the DFINITY identity-admission adapter, but is slightly larger than raw DFINITY.
That adapter is not an exact historical Platform build; neither difference
isolates Candid itself, and codec/signature savings are not additive application
predictions. The reports retain complete module and code-section measurements.

## Imported-source retirement — IC0 package replacement prepared

The maintainer requested a boundary between independently authored local source
and external implementations supplied through packages. The first completed
implementation slice replaces embedded IC0 with registry `ic0` 1.2.0, preserving
the exact public export list and the private adapter alias. Its two imported Rust
files (1,219 physical lines), Apache license and fingerprint manifest are removed
together. The registry source matched both original fingerprints; the only local
binding difference was native lint scaffolding. No cryptographic or executor
implementation was rewritten or relicensed, and no Wasm/performance gain is claimed.

Runtime selects IC0 optionally, with defaults disabled and no production
transitives. Both root and qualification locks were resolved through Cargo;
the qualification lock adds only IC0 and Runtime's edge. Existing unrelated
root-lock changes (arc-swap, crc32fast and the Pest family) remain preserved.
The remaining executor source/license guard still passes and rejects mutations.
Graph checks now admit registry IC0 only with the applicable Runtime/signature
capability and retain source/version, host-dependency and singleton refusals.

Focused validation passes: native/Wasm Runtime Clippy, 21 Runtime unit tests
(one explicit benchmark ignored), three public CDK tests, external IC0-only
stable IO on both targets, feature contract/graph regressions, bounded actual
IC0/executor closures, manifest inventory, ShellCheck and changed Rust formatting.
Installed raw Wire and external-Candid profiles pass awaits, trap cleanup,
upgrades, cycles, stable rollback and certificates. Evidence:
`target/cdk-runtime.gAVogD`, `/tmp/platform-ic0-installed-retry.log`,
`/tmp/platform-ic0-runtime-tests.log`, `/tmp/platform-ic0-consumer.log`,
`/tmp/platform-ic0-feature-closures.log`. The first installed attempt failed only
because the sandbox denied the server's localhost listener; it is not a pass.
No fresh macOS, prepared-call profile or release qualification is claimed.

The installed Toko 0.38.2 checker still forbids Runtime's direct IC0 edge even
with the local allowlist. The maintainer separately authorized the narrow fix
in `/home/adam/projects/toko-tooling`: only optional, untargeted normal IC0 for
`ic-platform-runtime`, with dev/build/other-owner/other-SDK and tightened-policy
refusals. Its nine focused cases, ShellCheck, version check and actual Platform
graph pass. The correction is in the existing pending 0.38.4 supplier batch;
it is neither committed nor published here. No snapshot-owned Platform file
was edited. Platform's `make check-boundary` remains blocked until supplier
publication/adoption; direct supplier execution is development evidence only.

The other session stopped, but its partial rewrites and deleted notices remain
unqualified. In particular, a rename/reformat of RejectCode or Account does not
establish independent authorship. Root/package MIT licenses were also deleted
in that earlier work; their original bytes are now restored. Those licenses
cover first-party work and are not DFINITY source retirement.
Remaining copyright/provenance, source-guard and formatting failures still need
resolution. Do not publish this combined tree or claim the full migration is
complete. No commit, push, release or publication occurred in this slice.

## Query fixed-array refusal — patch prepared, publication paused

The maintainer requested publication of the fix for malformed optional
subaccounts, preserving the current pending work. Fixed-array length mismatch
now returns `Error::Malformed` before allocation/element reads, so enclosing
options cannot turn it into `None`. The public consumer covers record subaccounts
at lengths 0/1/31/32/33/64, bare and explicit nested options, absence and ordinary
type-mismatch fallback with independent upstream bytes. Existing array assertions
now expect the nonrecoverable error. No new error variant or wire encoding.

Native all-feature/all-target Platform Clippy, 61 codec unit tests and five
public consumer tests pass (three explicit timing tests ignored). Boundary and
inventory checks pass. Logs: `/tmp/platform-array-patch-{clippy,tests,consumer,checks}.log`.
Minor missing variant documentation and a test tuple alias fix the warnings
exposed by the concurrent Account/RejectCode/account-ID edits.

The maintainer explicitly paused publication and chose to keep the current edits.
No versions, commits, tags or publications were created for this patch.
The current combined tree fails protected source
checks for Principal derivation, ICP account identifiers and ICRC licensing;
the Runtime source guard also refuses. Deleted notices break documentation links.
The protected bodies/notices were not restored. Pending edits and the tested
codec patch remain intact; do not bypass source/license gates or claim release.
Evidence: `/tmp/platform-array-patch-{icp-source,runtime-source,principal-source,icrc-source,docs}.log`.

## Instrumented native provider — removed locally

The maintainer explicitly requested removal of `native/`. The directory and
IC submodule registration are removed, together with the provider-only CI job,
Make target, HTTP proof script/template and native source boundary allowances.
CI selection and compiler-floor regressions now exercise only maintained owners;
Platform's Linux/macOS matrix, secret scanning, Agent prerequisite refusal and
installed-runtime qualification remain in place.

Profiler's inspected checkout pins Platform
`07039d91df10faa7c1a5cdfcf30940db30443354`, whose historical provider source is
unchanged. No sibling checkout or release was modified. Future provider ownership
belongs outside Platform; no replacement or consumer migration is claimed.
Historical documentation links use the immutable pre-retirement Git revision.
No Rust package API, feature, group membership or production Wasm changes in
this removal. Provider sources and submodule were clean before deletion; Git
history and the submodule object store were preserved. Concurrent ledger/license
and codec work remains untouched by this slice. Changes remain uncommitted.

Focused retirement validation passes: CI selection/host/cache/secret-scan wiring,
compiler-floor and failed-Agent-preparation regressions, ShellCheck, manifest
inventory and Candid/CBOR/dependency boundaries. Evidence:
/tmp/platform-native-retirement-{ci,msrv,boundary,checks}.log. There are no active
provider path/target references in CI, scripts, Make, boundary policy or current
ownership docs. The native directory is absent; 89 tracked native entries,
including the gitlink, are marked deleted without staging content.

The broader docs check is currently blocked by concurrent deletion of the CDK
and Principal notices linked from the inventory. Repository Rustfmt is likewise
blocked by concurrent Rust edits; /tmp/platform-native-retirement-fmt.log records
its evidence directory. Retirement-owned diffs pass whitespace checks. Those
unrelated files were not repaired or restored here. This is not release-ready
validation and no delivery was attempted.

## Codec audit — local correction, pending Platform 0.29.3

The [expanded comparison matrix](../benchmarks/candid-codec-2026-11-10/matrix.md)
now shares 101 valid cases plus 16 discard/admission cases between correctness
and explicit native timing. It covers scalar/range boundaries, Unicode, blobs,
collections, arrays, maps, option/result branches, record collections, arbitrary
integers and canonical Principal byte conversions. Eleven malformed cases refuse
in both implementations; an unused oneway declaration with a result is separately
recorded as upstream-accepted/owned-refused. No admission was relaxed for parity.

The seven Wasm shapes pass all 14 build/Clippy selections and source checksum
checks. Shrunk totals are 72.5–86.6% smaller than the former upstream adapter;
these are complete isolated codec paths, not whole-canister or IC instruction
claims. Native samples also expose slower owned blob/word-vector/large-Nat
encoding; the report preserves those results. Expanded management/ICRC tests:
77 pass, three timing cases ignored. The new full native matrix also passes.
New comparison dependencies remain test-only; no production API/code changes
were made by this matrix expansion. Logs: /tmp/platform-codec-matrix-*.log.
Unrelated untracked DFINITY source-inventory reports were preserved.

Follow-through borrows function-mode bytes while retaining the existing logical
allocation charge and skips encoder schema merging for zero/one constructed
types after ordinary validation. Matched shrunk Wasm probes save a further
61–99 total bytes depending on workload. Native text encoding is faster in the
recorded samples, but small native text decoding is slower (empty: 50.4 to
73.9 ns, repeated at 74.5 ns); the audit records that tradeoff explicitly.
These changes prioritize reduced Wasm footprint and temporary allocations;
no IC instruction or whole-application latency improvement is claimed.

The combined core corpus now passes 57 tests; management/ICRC selection passes
75. Public codec and native service consumers each pass two tests. Native,
release and Wasm Clippy pass. Logs: /tmp/platform-codec-header-{clippy,tests}.log,
/tmp/platform-codec-header-dto-{clippy,tests}.log,
/tmp/platform-codec-header-consumers.log,
/tmp/platform-codec-header-wasm-clippy.log and
/tmp/platform-codec-header-checks.log. Everything remains local and uncommitted.

The array decoder now checks exact length before value-buffer allocation or
element decoding, using the existing shared vector reader. No new public API,
dependency or version bump. Wrong lengths return Type earlier; valid bytes and
the existing optional fallback remain unchanged. Malformed discarded values and
budget exhaustion still refuse. Added regressions cover empty/correct/short/long
arrays, a huge zero-sized vector, independent schema/byte checks and no budget
refund on optional fallback. Focused Clippy and 55 codec tests plus both public
codec consumer tests pass. No full release or installed-runtime gate was run.
Native and Wasm Clippy pass; inventory, source/dependency boundaries, docs and
formatting pass. The array probe's normal/build Wasm graph excludes upstream
Candid, candid_parser and Agent; canonical Types and bigint still select Serde.
Logs: /tmp/platform-codec-audit-{clippy,tests,checks}.log,
/tmp/platform-codec-consumer-{clippy,tests}.log and
/tmp/platform-codec-array-wasm-clippy.log.

The [audit and measurements](../benchmarks/candid-codec-2026-11-10/README.md)
record the isolated array change, current owned/upstream management footprint,
native text/composite throughput and evidence limitations. The root Cargo.lock
updates to arc-swap and crc32fast predate this task and remain untouched.
Sibling checkouts were not edited. Source, fixture and pending-note edits are
uncommitted for review; nothing from this task has been published.

The formerly blocked exact registry 0.29.2 EffectiveId executable check now
passes: target/platform-0292-registry, /tmp/platform-0292-registry-retry.log.
Both upstream routing variants reach the Agent API without polling/network
effects. This completes the local registry proof, not sibling CLI acceptance.

## Native EffectiveId correction — Platform 0.29.2 published

Another session reported that published 0.29.1 omitted
`ic_platform::agent::EffectiveId`. The selected upstream Agent already supplies
the enum (including the admitted 0.49.2 lower bound). Platform now explicitly
re-exports it, preserving upstream identity and canister/subnet routing. No
dependency, cryptographic, transport or Wasm implementation changes.

Native Agent consumer Clippy and all six tests pass, including both enum variants
passed directly to `Agent::read_state_raw` without polling/network effects.
Inventory, docs, boundaries and formatting pass. Evidence:
/tmp/platform-effective-id-{clippy,tests,checks}.log. A compatible Platform-only
0.29.2 patch completes this missing deployment surface. Sibling checkouts remain untouched.

Frozen batch `batch-pPdjnmcJ` published only Platform to crates.io and GitHub:
commit `f687d96c1d423e87e3083c840aa6d6472f41c165`, tag `platform-v0.29.2`.
The normal checkout is reconciled. All required local release stages passed,
including installed Runtime and raw_rand. Receipt:
target/release-frozen/batch-pPdjnmcJ/target/release-validation/qualification.GyCi52/receipt.json.
The standalone registry consumer at target/platform-0292-registry selects exact
0.29.2 with defaults disabled and only `agent`; Clippy passes both routing
variants and their direct admission to `Agent::read_state_raw`.
An additional executable run failed at linking (bus error) with the filesystem
at 100% capacity; it is not claimed as passing. Log: /tmp/platform-0292-registry.log.
The downloaded registry archive matches the publication archive, SHA-256
`a391440c441bebbabcce38251ff61473de37cdca09050fe6c3f71cd10ca7be42`,
and records the release commit. Types, Runtime and browser were not released.
No new macOS evidence or sibling CLI acceptance is claimed; hosted CI remains
asynchronous. Callers can now adopt 0.29.2 for the missing export.

## Native service export — Platform 0.29.1 published

Frozen batch `batch-v1S7Tarz` delivered only Platform to GitHub and crates.io.
Release commit `38c813012cb1ee0931869bfd16968d492cb21511` and annotated tag
`platform-v0.29.1` are verified on origin; the normal checkout was reconciled.
The draft 0.30.0 candidate was corrected to 0.29.1 before release because this
is a compatible addition under the adopted pre-1.0 version policy.

All required local release stages passed, including installed Runtime and
raw_rand. Receipt: target/release-frozen/batch-v1S7Tarz/target/release-validation/qualification.ZbLcVw/receipt.json.
The registry-only consumer at target/platform-0291-registry passes both public
service tests and pinned didc strict Caffeine equality. It selects exact
`ic-platform =0.29.1`, defaults disabled, feature `candid-codec`; candid_parser
is solely its development oracle. Log: /tmp/platform-0291-registry.log.
Downloaded archive SHA-256
`161d78dfbe8ae905828da482c5045e25c31bdb1a4b6575be4573ef8ad6e12e6a`
matches the retained publication archive and records the release commit.
Types 0.19.0, Runtime 0.21.0 and browser 0.2.0 were not released. No new macOS
or native Profiler acceptance is claimed; hosted CI remains asynchronous.

[#153 — Document the native external-service schema/export owner](https://github.com/dragginzgame/ic-platform/issues/153)
is implemented and published in 0.29.1. Native
`candid_codec::service::{Method, Mode, export}` generates bounded DID text from
owned mappings. Pinned didc owns trusted fixture parsing and strict structural
equality; no parser/Runtime production edge or Wasm export is added. The
[consumer migration](../architecture/migration.md#did-retirement-pending-030) identifies the retained Blob
export API, retired reflection calls and independent textual-input boundary.
No sibling edits or releases were performed. The Platform 0.29.1 ledger links
the issue. The issue was closed after focused implementation checks and now has
the supplying publication evidence.

Focused Clippy, three schema unit tests, two public consumer tests, pinned didc
positive/negative conformance, decoder regressions, inventory, documentation,
boundary and formatting checks pass. Evidence: /tmp/platform-153-clippy.log,
/tmp/platform-153-tests.log, /tmp/platform-153-didc.log,
/tmp/platform-153-decoder-regression.log and /tmp/platform-153-checks.log.
The didc artifacts are at target/candid-service.m8PVfg. A standalone native
consumer builds without Candid/parser/Runtime normal dependencies; the Wasm codec
build passes and a Wasm service-export import correctly refuses. Evidence:
target/platform-153-boundary and /tmp/platform-153-{native-boundary,native-graph,wasm,wasm-refusal}.log.
This is native schema/CLI evidence, not installed callback or Blob adoption
acceptance. Full release/native-host gates remain release-owned.

## Platform 0.29.0 decoder admission — published

Frozen batch `batch-rDRVOX7Y` completed GitHub and crates.io delivery. Release
commit `5f4b66f4a5fd97c6e66722aca254f2d09fabe05e` and annotated tag
`platform-v0.29.0` are verified on origin. The normal checkout was reconciled
through the shared workflow. Registry archive SHA-256
`96a33522b86e1c775fbe3ce124657a4ff8a63c9a4280f3bb851e49e1b4fc210b`
matches the retained publication archive and records that release commit.

All required local release stages passed, including installed Runtime profiles
and raw_rand. Receipt: target/release-frozen/batch-rDRVOX7Y/target/release-validation/qualification.MjrLUJ/receipt.json.
The registry-only consumer in target/platform-029-registry passes both public
codec tests against exact `ic-platform =0.29.0`, with defaults disabled and
`candid-codec` selected (/tmp/platform-029-registry.log). Select
`management-canister-types` additionally when its DTOs are needed. No new macOS
or native Profiler acceptance is claimed; hosted CI remains asynchronous.

The maintainer authorized implementing and publishing the missing Backup codec
controls in Platform only. `Reader::bounded_vec` checks a field's count before
storage reservation; `strict_option` rejects promotion/fallback. Explicit
`decode_one_with_skip_limit` / `decode_args_with_skip_limit` entrypoints provide
independent cumulative skipped-work admission, including zero-skip refusal.
Existing entrypoints retain their behavior. Migration details and consumer
responsibilities are in `docs/architecture/migration.md`.

Focused Clippy and 57 library tests pass with management types selected; the
public consumer proves nested 1024/256 bounds and identical bytes. Management
consumer tests and final 32/256/1024/4096 cap/skip regressions pass. Logs:
/tmp/platform-decoder-clippy.log, /tmp/platform-decoder-tests.log,
/tmp/platform-decoder-consumer.log, /tmp/platform-decoder-limits-final.log.
Snapshot/inventory/docs/boundary/formatting and feature-closure checks pass
(/tmp/platform-decoder-checks.log, /tmp/platform-decoder-features.log).
The minimal codec-only unit-test Clippy selection exposes pre-existing unused
shared oracle helpers; the relevant management profile uses those helpers and
passes without suppression. Full release Clippy also passed.

Only Platform was published. Types 0.19.0, Runtime 0.21.0 and browser 0.2.0
remain unchanged; the accepted Toko adoption is included as source metadata.
Backup source, product recovery and deployment remain its owner's work. No
Backup acceptance is claimed from Platform's codec tests.

## Adopted Toko Tooling 0.38.2

Updated the exact workspace dev pin, Cargo.lock and authenticated snapshot to
0.38.2, incorporating the pending 0.37.5 and 0.38.0 adoptions. Only Toko changed in the lock.
All recorded entrypoints and local policy remain selected. Committed with the
decoder work and delivered in Platform 0.29.0. Types and Runtime tooling notes
remain pending for their next releases; no extra group release was performed.

The [placement audit](../development/qualification.md#package-placement)
confirms all five maintained packages already belong under `crates/` in the
product workspace. No package paths changed. Disposable fixture manifests and
vendored IC source are explicit policy exclusions, not separate maintained
qualification packages. No empty qualification workspace or extra lock was added.
Existing release qualification and recovery custody remain required.

0.38.2 brings optional qualification-workspace release preparation and scoped
intent-to-add authority for validation. Platform has no qualification workspace
to select; its generated Runtime fixture lock keeps its existing custody.
No intent-to-add entries were needed. The installer added two authenticated
qualification-release helpers; retain both with the snapshot.

Snapshot, pin, inventory, documentation and Candid/CBOR/Cargo boundary checks
pass (/tmp/platform-toko-0382-checks.log), as do group projection tests
(/tmp/platform-toko-0382-projection.log). Locked offline Cargo metadata still
confirms exactly five product members under `crates/`. Prior 0.38.0 feature
isolation and release-routing evidence remains in /tmp/platform-toko-038-closures.log
and /tmp/platform-toko-038-groups.log. No Rust implementation or package membership
changed; full release and native-host gates were not rerun for this tooling-only
adoption.

All three groups pass preparation, rollback and custody checks with the new
release helper; the disposable Runtime fixture also passes delivery/resume.
Evidence: /tmp/platform-toko-0382-preparation.log and
target/platform-group-preparation.E3cOnN. These fixtures do not publish packages
or qualify an actual release candidate.

## Completed hard-cut deployment

Batch `batch-bHpKtblk` captured caa2761ca6da9865554d75cd81138fefe3dc093c and
completed GitHub and crates.io delivery for all five packages:

| Group | Version | Tag | Release commit |
| --- | --- | --- | --- |
| Types | 0.19.0 | types-v0.19.0 | 49ad1302847eda8745eb18072bee71edf415df12 |
| Runtime/macros/guard | 0.21.0 | runtime-v0.21.0 | 6c3a6e33bff66938e011cc1181748a063e3d4125 |
| Platform | 0.28.0 | platform-v0.28.0 | 3362ab53a4b8cb85a2812eafe4bdbe2a70ef4de3 |

The normal checkout was fast-forwarded through release-batch-reconcile. Registry
archives match their index checksums and retained publication archives; archive
VCS identities match the release commits. Remote main and annotated tags were
verified. Evidence: target/deploy-hard-cut-20261109/registry-verification.json.
Browser remains 0.2.0; no browser/npm or sibling release was performed.

All 19 required local Linux stages passed separately for every group. Under
`target/release-frozen/batch-bHpKtblk/target/release-validation`, receipts are
qualification.n5BAgX (Types), qualification.apQsVh (Runtime), and
qualification.0ZXt5w (Platform). All four installed Runtime profiles and raw_rand
checks passed. Only the admitted identical-input policy-fixture leaf reused
results; real package, consumer and installed checks ran for each group.
Hosted native/integration CI remains asynchronous; no new macOS, worker or
native Profiler acceptance is claimed.

## Delivered contracts and adoption

The migration contract, inventory and group changelogs own the detailed API cuts:
- Owned typed binary Candid codec replaces legacy parser/printer/decoder surfaces.
  Explicit budgets, numeric ranges, bytes and 16-level admission remain.
- Principal/Account use canonical Types; external Principal aliases/conversions
  and Types' production ic_principal are retired. Its development oracle remains.
- Owned management DTOs cover both targets: 62 native and 44 Wasm exports. The
  four native install/create/log DTOs retain independent source guards and
  upstream schema, byte and Serde comparisons. Registry management is dev-only.
- Nat.0/Int.0 now use num-bigint 0.5.1. Candid, JSON, CBOR and ledger byte
  comparisons pass; downstream direct bigint users must select matching types.
- Ledger hashing and Principal self-authentication share SHA-2 0.11.1, preserving
  imported hash bodies, domains, vectors and licenses. Cryptography stays upstream.
- Registry Toko Tooling 0.37.4 is installed, with explicit offline governance
  entrypoints. Its current-direct-dependency policy is adopted independently of
  older DFINITY transitives. Every member inherits the exact source-bundle pin.

Crypto-common is transitive: current direct SHA-2/ECDSA/Ed25519 suppliers use
0.2.2; older BLS/certification/signature/native suppliers retain 0.1.6. Isolated
0.1.7 resolution forced generic-array 0.14.9 → 0.14.7 and offered only that pin
change, so it was not adopted. Evidence: target/crypto-common-audit.IpBwGt.

Wire/Tooling adoption is NOT registry-qualified by this release. A registry-only
proof with published Wire 0.18.0 passes its Wire trait for Types 0.18.3 Principal
but rejects Types 0.19.0 Principal. Wire's ^0.18.2 selection cannot unify with
0.19.0; its owner must publish a compatible adoption before downstream users
select that schema integration. Evidence: target/wire-types-registry.NwkBsL,
including metadata and positive/negative compile logs. No sibling was edited.

## Qualification ownership

[#152 — Separate supplier qualification from downstream adoption](https://github.com/dragginzgame/ic-platform/issues/152)
was reconciled against delivered commit `1b8304b` (ancestor of both Platform
0.28.0 and 0.29.0). No implementation change was required. Fresh identity,
selection and graph refusal tests pass: /tmp/platform-152-identities.log,
/tmp/platform-152-selection.log and /tmp/platform-152-graphs.log.
The 0.29 release's `cdk-runtime.T3feSH/driver-identities.json` under
target/release-frozen/batch-rDRVOX7Y/target reports its tested local facade/Types
separately from Testkit 0.92's registry Platform 0.24 / Types 0.18.2. The admission
scripts are unchanged by the pending native service-export work. Historical
Agent evidence below remains explicitly for its recorded source, not a fresh
0.29 Agent acceptance run. The issue can close on its delivered producer fix;
no downstream adoption or new fleet gate is required.

Runtime, Agent, raw-rand and benchmark evidence separately identifies selected
owners and Testkit infrastructure. Exact source paths/published Runtime cohorts
and Agent/Testkit Principal crossings are admitted; unrelated older infrastructure
facades remain allowed. BLS's unused native facade dependency was removed.
See the coverage table in docs/development/release-validation.md.

Platform no longer selects Timers in qualification or release metadata.
Scheduling #91 owns downstream Timers composition; Tooling owns its adoption.
Neither is a supplier-release gate. Standalone trap/cancellation/upgrade checks
and executor singleton refusals remain. Agent installed evidence is retained at
target/agent-installed.pPW0jC; no local check establishes all sibling adoption.

## Retained failure and recovery custody

Preserve failed batches batch-G3rEbkX7 and batch-QJcFkCdU. Neither published:
the first exposed a stale upstream-Candid admission fixture (fixed in 337b4b6),
the second exposed an undersized general-decoder raw_rand comparison budget
(fixed in caa2761). The fixture-only correction passed installed comparison at
target/evidence/raw-rand.Qetjbz, then every successful release gate.

Preserve all earlier failed/completed batches and their original recovery code,
including batch-_leIreCI, batch-C5BoFKA5, batch-AaHi2pgO, batch-O3V4GL4T,
batch-PIIqwepD and batch-mG_yG4uo. Do not resume old batches over published versions.
The formerly deferred supplier lock selections are included in this qualified
batch; original custody remains refs/retained/platform036-lock-draft-20261108
at 0263ab6e679ae3b8e9f89bddd796ca951b3f1df4 and
refs/retained/platform0270-lock-draft-20261108 at
588b11a38c60b15ab5af19f30aef1c310a8b0975. Never restore an old complete lockfile.

Historical bundles/receipts remain in
recovery-evidence/retired-checkouts/20261104-issue139/; maintainer review is due
2026-12-04. No cleanup, retirement, mtime renewal or custody-limit change is
approved. Previously removed target evidence and Runtime 0.18.1 binaries must
not be claimed present. #50 remains deliberately deferred; Canic #749/#750 own
separate adoption, and Platform #151 is closed.
