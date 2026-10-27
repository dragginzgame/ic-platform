# Current handoff

## Current optimization batch (pending 0.7.1)

Further Candid experiments remain isolated: neither shared exact-one setup nor
runtime-valued sequence visitor limits were adopted. Both reduced multi-type/
multi-limit Wasm probes, and all 780 differential value/error cases matched,
but setup sharing slowed the native scalar/blob controls and visitor sharing
added 132 bytes (`z`) / 150 (`3`) in the single-limit control. The original
production implementations are restored byte-for-byte. The extra admitted-huge-
hint regression remains; it protects the existing no-reservation contract.
Opt-in benchmark profiles, source patches/drivers and size/native data are
recorded under `docs/benchmarks/0.7.1-candid-*` and in the migration contract.

All prototypes passed full Clippy first, focused native/MSRV Candid suites and
standalone differential checks; the combined case was qualified independently.
Separate build targets avoid stale host artifacts between same-dependency source
copies. Root Clippy was restored to the actual working source after identifying
shared-target reuse. Wasm sweeps used normalized source/probe paths and checked
input hashes before/after. Evidence: `/tmp/platform-candid-experiments`.
No IC instruction, installed-canister, application-size or native macOS result
is claimed. Earlier qualified optimizations below remain intact.
The retained implementation passes full Clippy, native/MSRV Candid suites (13
each), standalone native/MSRV/Wasm consumer proofs, formatting, snapshot,
inventory, doc-link, ShellCheck and changelog checks. Saved prototype patches
pass isolated dry-run application. Cargo.lock and the real index remain unchanged.

Follow-up attribution against the same owned baseline isolates the parser-body
change as −35,561 bytes of the text/JSON probe at `z-fat-1` (−2,288 at `3-fat-1`).
Serialization contributes −310 bytes and Account formatting −137 in isolated
`z` trials; error derives alone are size-neutral. These are interacting compiler
results, not additive or DFINITY-versus-owned comparisons. The raw binary control
is unchanged at 7,631 bytes (`z`) / 8,157 (`3`). Data and interpretation are in the
migration contract and `docs/benchmarks/0.7.1-{attribution,controls}.tsv`.

The additional owned Principal wrapper now passes a slice to the unchanged
upstream self-authentication function, canonicalizing only its generic input
type. Focused optimized Wasm falls by 799 bytes (`z`) / 824 (`3`); text/JSON
sizes stay unchanged. No crypto body, hash/domain/key encoding or vector changed.
Native and Rust 1.91 type tests pass (12 each), including Vec/slice equivalence
and exactly-once custom AsRef evaluation; full Clippy passed first. New controls
compile at Rust 1.91 for Wasm, with source/crypto/runtime provenance guards intact.
Evidence: `/tmp/platform-size-followup`. The manual harness adds an optional
recipe subset and identifies archive inputs without borrowing a parent Git HEAD.

Implemented against clean baseline `e918d23`: one non-generic Principal parser
behind the existing generic API, stack-buffer human-readable serialization,
simpler Account formatting and mechanical `thiserror` error implementations.
Messages, source chains, identities, CRC/hash inputs and binary Serde stay intact.
`principal` now selects the already optional `thiserror`; inventory is updated.
Default/executor-only graphs remain application/Candid/substrate-free.

Runtime System API text keeps UTF-8 validation and panic reasons but drops
rejected-byte Debug dumps. The private shared adapter has valid/invalid-text
regression coverage. Submission/callback/trap cleanup, executor identity and all
cryptographic bodies/vectors/license fingerprints remain unchanged.

Kept automatic encoder Vec inference compatible. Standalone consumers now prove
the existing manual-reply path accepts fixed arrays/borrowed slices and automatic
generic encoders remain supported; no new generator mode or public API was added.
Feature contracts now use one jq pass with the same named policies, tested
against changed selections, missing dependencies and missing/ambiguous packages.
Normal feature-selection checks own this regression coverage.

Added the manual `scripts/benchmarks/wasm-size.sh` sweep and its standalone probe;
the existing native type runner now creates its target parent on clean checkouts.
Normalized source/probe paths before final size comparison, so panic-location
strings do not create fake gains. Full raw/code/optimized evidence and source/
artifact/tool identities live under `/tmp/platform-size-work/normalized-{before,after}`;
provisional unnormalized trials and failed setup attempts remain retained but
are not the reported results. The migration contract and repository TSV data
record all five recipes/four capabilities. Same-recipe optimized changes include
Principal/Account −2,578 bytes (`3-fat-1`) and −36,003 (`z-fat-1`), raw-call runtime
−1,561 (`z-fat-1`), external Candid and signature verification 0. These fixture
deltas are not additive or claims about application Wasm. Native Account timings
use two alternating before/after process pairs; formatting medians are lower and
parsing roughly unchanged. No IC instruction/cycle claim follows.

Full `make clippy` passes before longer tests. Runtime/type unit tests pass on
native and Rust 1.91 (15 each); parser properties pass (2). Standalone Principal
and CDK consumer proofs pass on native/MSRV/Wasm; the final runtime adapter also
compiles in every normalized Rust 1.91 size recipe. Provenance guards, selected
Wasm graphs, feature mutation/selection tests, formatting, inventory,
dependency boundary/pins, docs, ShellCheck and pending changelog admission pass.
Evidence/logs: `/tmp/platform-size-work`. No full CI, native macOS, installed
canister or consumer deployment was run. No staging/commit/push/publication;
real index and Cargo.lock are unchanged, and siblings remain read-only.

Inspected Canic's service-generation cfg and Core crypto feature/development
declarations, plus Canic/IcyDB release profiles. They do not establish a removable
production capability; no speculative sibling edits or crypto/API hard cuts were
made. Actual application artifact/instruction qualification remains consumer-owned.

## Source and authorized boundary

Released main is Platform 0.7.0 at
`60c43108ca7f2a99bfac62ec21075afff5ed7a7e`; npm browser 0.1.1 is separately
published. The maintainer released the preceding #38 cut. Shared Tooling 0.26.26
at `5683060a5d8ce79af2fcfedb53b3eb24037fd064` was reviewed and exported
canonically with 109 payloads. Incoming Shared adoption and #65 documentation
work were preserved. That snapshot adoption changed no runtime or dependency
contract; the later optimization batch is recorded above. The next compatible
changelog draft is 0.7.1; release metadata
remains 0.7.0.

Shared #314 now supplies standalone Rustfmt inputs. Platform's existing source
roster feeds `FORMAT_RUST_ARGS` with explicit edition 2024; the paired command
overrides are retired. Grouped manifest sorting, workspace Rustfmt and the full
standalone pass remain in order, with visible timings and retained failures.
Standalone selections deliberately run uncached; no result-reuse claim is made.
The snapshot includes the revised opaque-submodule hook, shared Rust lexer and
Node prerequisite guidance. Gzip guidance names Tooling's existing bounded
encoder; no local encoder implementation required migration. Earlier policy updates
for CandidType separation and error/subprocess handling remain included.
Platform's owned Principal/Account remain Candid-free; required external protocol
DTOs and dedicated Candid codec/refusal proofs retain their boundary role.
The stale 0.6.2 overlay and completed #38 instruction were reconciled with 0.7.0.
Full `make clippy` passed before focused tests. The maintained
`make check-formatting` fixture exercises actual Platform Make/hook inputs:
standalone check/write, changed configuration, exact failure status/stopping and
selected-file hook updates preserving unrelated working edits and Cargo.lock.
It is included in release CI. Snapshot integrity, owned links, actual
`make fmt-check`, ShellCheck, pending changelog admission and diff checks pass.
Evidence: `/tmp/platform-shared-02626`. No native macOS, installed-canister or full
release CI execution is claimed. Prior runtime evidence below remains scoped.
No real-checkout staging, commit, push, release, publication or deployment was
performed. Git setup/commit effects were confined to disposable hook fixtures;
the real Cargo.lock and index are unchanged. Siblings stay read-only.

## Instrumented provider and formatting assessment (#65 / #66)

#65 remains unimplemented; #66's formatter adoption is now prepared locally.
The migration contract records the instrumented provider's source/ownership
boundary and required native acceptance, and the inventory explicitly excludes
it from the existing registry PocketIC capability. The patched client cannot be
substituted with the same-version registry client. Replica hooks must remain
below the facade to avoid dependency cycles; no direct-dependency exemption,
unused Platform dependency or incompatible source substitution was added.

Platform locked/offline metadata with `pocket-ic` confirms registry 16.1.0;
the selected client lacks `start_system_profile`. Profiler's assembled native
metadata fails offline on uncached pinned `dfinity-lab/build-info`; its latest
local build receipt has exit 101 and no artifacts. Existing passing extraction
receipts do not qualify a newly routed provider. Evidence is retained under
`/tmp/platform-issues-65-66`. No native build/runtime pass is claimed.

For #66, the 0.26.26 adoption above replaces the former Shared #314 supplier
blocker. Platform's actual Linux caller/hook proofs pass. Native macOS and
default-branch integration remain unqualified, so the issue is not closed.
The source roster still has one owner in Makefile; no copied orchestration or
additional source registry was introduced. No runtime implementation changed.

## Compiler retirement (#38): released and closed

Removed native `candid_parser::{compile_javascript, compile_typescript}` and the
unused compiler-only `IDLMergedProg` export. Removed only the exhausted browser
compiler assertions from the standalone parser consumer. No compatibility alias,
replacement generator, parser fork or fallback codec was added. Remaining APIs
retain external parsing/typechecking, import detection, textual argument encoding,
constructor typing, service comparison, dynamic refusal/value serializers and
Rust type equality. The separate external `candid-service` feature stays intact.
The migration contract includes the retained API/consumer inventory and exact
[0.7.0 removal instructions](../migration.md#pending-070-compiler-only-retirement-38).

The audit found no references to any removed name in Canic default `aed32be2`
crates or current Canic/Query/Host/Tooling/Blob/Auth/Backup/IcyDB/Jobs/Metrics/
Persistence/Relay crate sources. Host retains its external service inspector;
Canic retains `fleet_ensure::ops::init_arguments`. Owned frontend artifacts now
use Wire metadata/bindings. Canic's owner records passing installed frontend
handoff (275s) and later dedicated/embedded generated-client proof (637s) at
local `b9482ec526c5dfb9f5dd349d22b1f60190b2dfdf`. The inspected qualification
record is captured under `/tmp/platform-issue38-20261027/canic-qualification.json`;
its source/hash are recorded in the migration contract. These are retained owner
observations, not new sibling execution, full #613 closeout or live deployment.

Full `make clippy` passed before longer tests. Standalone parser (2), dynamic
refusal (2), external service (1) and pinned external schema (5) tests pass on
native and Rust 1.91 with no direct consumer Candid/parser dependencies.
Fixed nat64 argument bytes, malformed grammar, undefined types, import detection,
service drift and constructor/type equality remain covered. The service consumer
also compiles for Wasm. Rust 1.91 parser-feature Wasm library compilation passes;
a real Wasm consumer import of the host parser is correctly refused.
Actual native/Wasm default, executor, runtime, entrypoint, parser and external
service normal/build graphs pass. Snapshot, inventory, formatting, doc links,
diff checks and 0.7.0 pending changelog admission pass.

Evidence: `/tmp/platform-issue38-20261027/{clippy,consumers,graphs,wasm,policy}.log`,
`target/boundary-consumer.jtMvNt` (parser), `.Kkxhr9` (dynamic), `.PjyEZx` (service),
`.36PvZw` (external schemas), `target/issue38-graphs` and
`target/parser-wasm-exclusion.rKTWnF`. No new native macOS, installed-canister or
full release CI run is claimed; those remain separate release/consumer duties.

The dependency remains candid_parser 0.4.1; actual native graphs still include
Handlebars 6.4.4, derive_builder 0.20.2 and Darling 0.20.11. Only supported facade
surface is retired. No transitive dependency removal, cryptographic/runtime/data
change, Wasm-size improvement or performance gain is claimed. Existing external
Candid bytes, checkpoints, aggregate limits and retry/effect policy stay unchanged.
Verified the non-yanked public 0.7.0 archive, SHA-256
`677aad73f8b1339cbe1cd7c851f4140bdcdc91c3838fda425bb82549dcd7c35d`;
its VCS equals released main and its parser facade matches the reviewed source.
#38 is closed now that both default-main and minor-release conditions are met.
Verification evidence is retained under `/tmp/platform-shared-02622`.
Other hard cuts retain their restriction; no broader retirement is authorized.

## Existing published suppliers and consumer work

Platform 0.6.20 supplies optional CMC, ICP Ledger, Cycles Ledger, Registry and
native asset DTOs. Published 0.6.21 adds the exact pinned management lifecycle/
settings/ECDSA/history and ICRC transfer/memo/consent exports. Its verified archive
SHA-256 is `f82b7556c1a6acf55a82b0bdc789fc6d4cb4f9c3e0113e911354817387b866e0`.
Fresh registry-only management/ICRC native/MSRV/Wasm proofs passed before this
cut (`/tmp/platform-0621-live`, `target/boundary-consumer.Zt7IyX`, `.j3pDdx`).
The [external contract guide](../external-contracts.md) and migration contract own
exact imports/provenance and canonical shape differences. Those external contract
surfaces remain unchanged by #38. #61–64 stay open for Canic's actual adoption;
Canic's dirty 0.112 migration remains owner-controlled, not a new published release.
No old 'management export supplier is unpublished' blocker should remain.

#59 is closed: public `@tokostudios/ic-platform` 0.1.1 supplies the four SDK
signing-request/expiry exports with Core exactly 6.1.1. Archive SHA-1 is
`33bf72eaf3039111c462072d5d2bc0aa6dd80f45`; registry metadata, exact integrity,
packed/Chromium proofs and fresh registry-only strict TypeScript/runtime evidence
are retained in `/tmp/platform-npm-011`, `target/npm-registry-consumer.1ClKKR`.
Auth/application adoption and provider acceptance remain separate.

## Advisory status

#56 is closed: released CI has complete index observations for both maintained
locks; the remaining workspace audit failure is #50. Live workflow evidence is
[40472663771](https://github.com/dragginzgame/ic-platform/actions/runs/40472663771),
retained in `/tmp/platform-backlog-20261027/advisory-ci-full.log`. No advisory was
suppressed; earlier prepared/yanked/missing-index regressions remain retained.

#50 remains unresolved. Latest observed compatible crypto-bigint is 0.5.5;
patched >=0.7.6 is outside the selected K256/P256/elliptic-curve 0.13 family.
Latest Agent 0.49.3 and exact ic-secp256k1 0.3.0 retain those coupled paths.
Resident private-key signing precludes a blanket verification-only exemption.
No compatible correction, exemption, override, Agent fork or in-house crypto
change is supplied. Raw records are in `/tmp/platform-0621-live` and the earlier
backlog evidence. This compiler retirement does not qualify crypto safety.
