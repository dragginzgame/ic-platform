# Current handoff

## Integration and release state

The local branch contains the maintainer's 0.7.1 release commit `b301438`;
Rust release metadata and the dated changelog are 0.7.1. This merge incorporates
remote main `9dcef4b`, including merged PRs #69, #71 and #70. Tooling prerequisite
[#69](https://github.com/dragginzgame/ic-platform/pull/69) merged at
`aa973e13bae7a6dd9027c0dea47d0736f37acb31`, gzip #71 at
`8f2b478ef96936754284976cd75dd9cd4b929ef7`, and native provider #70 at
`9dcef4bc65493480801d976095719652a530083d`. npm browser 0.1.1 is published;
this merge does not establish publication of the integrated Rust package.

The optimization evidence below records the pre-release qualification, including
its original working-tree and index state. The provider and gzip qualification
remain separately scoped below. Shared Tooling 0.26.26 has 109 verified payloads;
its adoption changed no runtime or dependency contract. No release, publication,
deployment, advisory ignore or sibling checkout edit is performed by this merge
resolution. Profiler's Git/release operations remain maintainer-owned. Preserve
its incoming Tooling 0.69.2 lock update.

## Optimization qualification (0.7.1)

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


## Gzip feature admission (#71)

The optional native gzip facade delegates to published IC Tooling 0.69.1 and
adds no default/Wasm dependency or IC/runtime cycle. Consumers own levels,
limits and payload admission. Single-member decoding rejects trailing data.
See [the migration contract](../migration.md#host-gzip-boundary).

The cache checker previously assumed every native leaf enabled Principal.
Policy now declares exact prerequisites and dependencies per leaf; gzip has no
Principal prerequisite. Optional/target/count/alias checks remain strict.
Regression coverage includes malformed policy, changed contracts, native/Wasm
mixed selections and real Cargo metadata. Focused closure and selection fixtures
pass at incoming `4ef8c39`. Its retained qualification records full Clippy,
formatting, ShellCheck, gzip consumer, inventory and links; Rust 1.91 gzip-only
Wasm compilation; and the complete routine feature matrix (415 selections per
target, 384 native/234 Wasm queries). Codec/API/library inputs are unchanged by
the cache correction or tooling merge; reuse their passing evidence.

## Native provider (#65 / #70)

The maintainer approved public Platform ownership under `native/ic-profiler`;
private Profiler retains recorder/viewer policy. The provider owns the IC pin,
patches, native adapters, patched client, prerequisites and legacy assembler.
Keep the registry PocketIC feature separate and preserve origin licenses.

At provider `0d325ae`, reproducible assembly, server build, 69 recorder tests and
five native regressions passed. Server SHA-256:
`b3e6fbc7c9a5678c2acbbfe96e9400ae968a1aff9c0c4ad22a0d2ba552c26218`.
The graph excludes lifeline/Motoko; embedded routing is the actual IC snapshot.
Receipts bind source origins, provider/consumer commits and the executable hash.
Bazel locks were generated with pinned Bazel 9.2.0; the real recorder target
resolves without repinning. Full Bazel builds and NNS-genesis artifacts remain
unqualified. Forty-three provider fixtures and seven consumer binding fixtures
passed. Retain old assemblies, inputs and receipts when adopting new pins.

Merged #70's native Cargo/Bazel pins select the merged gzip facade
`8f2b478ef96936754284976cd75dd9cd4b929ef7`; regenerate the Bazel lock checksum
and qualify Profiler's actual merged provider. #65/#66 retain applicable acceptance.

## Shared rules and published contracts

Shared Tooling 0.26.26 at `5683060a5d8ce79af2fcfedb53b3eb24037fd064`
has 109 verified payloads. Standalone Rustfmt has one roster in Makefile;
formatting fixtures preserve opaque gitlinks without copying their source/Git
metadata. Check/write/failure/staged-hook qualification passed, with source at
`/tmp/platform-shared-02626`. #69 CI passed on Linux and both macOS architectures;
its separate advisory failure remains #50. Hosted CI continues asynchronously.

The #38 compiler-only cut shipped in 0.7.0 and is closed. Existing external
Candid parsers/service operations and legacy bytes remain; no broader hard cut
is authorized. Canonical evidence is in [the migration contract](../migration.md)
and `/tmp/platform-issue38-20261027`, `/tmp/platform-shared-02622`.
Published external DTO and browser suppliers remain qualified independently;
[external contracts](../external-contracts.md), #61–64 and #59 own migration
obligations. Canic/Auth/application adoption is not implied by these PRs.

## Security blocker (#50)

[#50](https://github.com/dragginzgame/ic-platform/issues/50) remains the canonical
RUSTSEC-2026-0387 remediation issue. Last qualified graph has crypto-bigint 0.5.5
through elliptic-curve 0.13.8, direct ECDSA, Agent identities and pinned derivation;
patched >=0.7.6 requires a coupled supplier/type migration. No blanket public-data
or verification-only exemption is justified. Preserve upstream crypto bodies,
vectors, source identities and no-Agent-fork constraints; no incompatible override,
renaming to evade detection, in-house crypto or advisory ignore is supplied.
#56 index coverage is resolved; the advisory failure remains independently real.
Evidence: `/tmp/platform-backlog-20261027`, `/tmp/platform-0621-live` and #50.
