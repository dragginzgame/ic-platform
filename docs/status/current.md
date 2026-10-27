# Current handoff

## Integration and release state

The maintainer authorized fixing CI and integrating Platform PRs #69, #71 and
#70, then updating Profiler to integrated revisions. Tooling prerequisite
[#69](https://github.com/dragginzgame/ic-platform/pull/69) is merged at
`aa973e13bae7a6dd9027c0dea47d0736f37acb31`. Gzip #71 is merged at
`8f2b478ef96936754284976cd75dd9cd4b929ef7`; native provider #70 is being
integrated. Neither capability is a new published package release. Rust release metadata
remains 0.7.0 with compatible notes under 0.7.1; npm browser 0.1.1 is published.
No release, publication, deployment, advisory ignore or sibling checkout edit
is authorized by this integration. Profiler's Git/release operations remain
maintainer-owned. Preserve its incoming Tooling 0.69.2 lock update.

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

#70 now targets main. Native Cargo/Bazel pins select merged gzip facade
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
