# Current handoff

## Current work

Current source is the maintainer's `d4f3a5b` commit, based on released Platform
0.6.17 (`8d80630`). Pending notes use 0.6.18; Rust package versions remain 0.6.17.
No commit, push or Rust release was performed by this repair. The maintainer
authorized initial npm publication separately; its supply status is below.
Existing sibling checkouts remain read-only.
Shared Tooling stays at 0.26.18 (`37652c0`); its canonical exporter added the
Cargo graph checker at that same revision. All 104 snapshot payloads verify.

The release attempt stopped in consumer admission because its isolated fixture
omitted `check-blake3-graph.sh` and the shared graph checker. The fixture now
copies those real helpers and supplies representative BLAKE3 package/feature
observations; duplicate identities and extra features reject before compilation.
All-profile admission, runtime graph refusal, final Cargo failure and concurrent
source-edit checks pass after full Clippy. This is fixture setup/coverage only;
the production runner, graph policies and cryptographic code are unchanged.
The full `make check-consumer` target now passes all 23 real standalone profiles,
including development/MSRV 1.91 native tests and applicable Wasm checks/builds.
Logs: `/tmp/platform-consumer-{admission-clippy,admission-after,check}.log`.
Syntax, ShellCheck, formatting, inventory, links and pending changelog admission
pass. Native macOS and the complete release CI were not rerun by this repair.

The maintainer retired [Platform #58](https://github.com/dragginzgame/ic-platform/issues/58)
as an unnecessary central coordination queue. Its acknowledgement/routing
requirements are withdrawn. Existing producer/consumer issues own their work;
agreed compatibility decisions stay in the migration contract and inventory.
No independent work waits for a central acknowledgement or approval.

The optional `blake3` feature exposes upstream `hash`, `Hasher` and `Hash` only,
using compatible BLAKE3 1.8.7 with defaults disabled and `std` selected. There
are no default Platform features, wrapper types, generic hash abstraction or
cryptographic implementation changes. Existing SHA/IC/external/persisted formats
are unchanged. The capability is not available in published Platform 0.6.17.
Use a local path for qualification until the supplying release is published.

The [migration contract](../migration.md#optional-blake3-maintenance-boundary),
[README](../../README.md) and [inventory](../ic-inventory.md) cover ownership,
selection and limits. Consumers own domain/format policy and adoption. Root
boundary policy admits direct BLAKE3 only in the facade; sibling guards remain
their owners' work. The lock adds only BLAKE3, arrayvec 0.7 and constant_time_eq;
the prior arrayvec 0.5 is an unrelated host dependency, not a deployed duplicate.
The independent runtime fixture lock is byte-identical to the starting input.

## Graph enforcement and qualification

`make check-primitives` checks explicit native/Wasm selections before release CI
Clippy. It uses the shared checker for at most one full rendered BLAKE3 identity,
including version/source qualifiers, then inspects Cargo's actual feature union
and requires exactly `std` when selected. It uses locked/offline normal/build
graphs, performs no fetching/compilation and retains its observations. Source
rendering is not source authentication, and this is not Canic artifact admission.

A real separate consumer proves an optional renamed dependency can enable `pure`
on the same BLAKE3 identity; feature admission rejects it. Shared graph contract
fixtures cover version/source conflicts, aliases and failed observations. The
ordinary selection test now checks exhaustive completeness on four synthetic
features instead of allocating millions of production combinations. The real
singles/pairs/critical matrix remains: 262 selections, 241 native and 144 Wasm
Cargo queries. Full production exhaustive mode remains an explicit manual audit.

Full all-target/all-feature Clippy passes on Rust 1.99.1 before longer tests.
Standalone consumers declaring only Platform pass native tests and Wasm checking
at development Rust and MSRV 1.91. All-feature Wasm checking also passes at 1.91.
Primitive graphs, real feature-union rejection, the routine feature matrix,
manifest inventory, dependency pins/boundaries, snapshot, formatting, links and
imported runtime/crypto source guards pass. No full release CI or installed IC
qualification ran; native macOS remains separate. Logs: `/tmp/platform-blake3-*.log`.
Core evidence: `target/blake3-boundary.QmcewQ`; standalone consumer:
`target/boundary-consumer.RpGXDE`; feature matrix: `target/feature-graphs/attempt.FbfYzc`.

## Optimized Wasm probe

Rust 1.91, opt-level=z, fat LTO, one codegen unit, aborting panics and stripped
symbols, followed by `ic-wasm 0.11.1 shrink`, produced these freestanding sizes:

| Streaming ABI probe | Bytes |
| --- | ---: |
| Control | 109 |
| BLAKE3 through Platform | 23,340 |
| Direct BLAKE3 | 23,340 |
| SHA-256 | 10,215 |

Facade/direct BLAKE3 artifacts are byte-identical. Optimized ABI execution passes,
including the empty BLAKE3 vector and SHA-256 reference comparisons. This proves
no facade overhead for that retained code path; BLAKE3 is larger than SHA-256
here. It is not a Canic size report, speed comparison or IC instruction/cycle
benchmark. Sources, lock, metadata, hashes and execution: `wasm-probe` under the
core evidence directory. No product hash was replaced.

## Canic integration

[Canic #602](https://github.com/dragginzgame/canic/issues/602#issuecomment-6412520754)
retains actual per-deployment admission, feature policy, cached-artifact binding
and optimized-size qualification. Read-only review used dirty `cf2d3d1fb`; its
active Wire batch freezes dependencies and defers builds/tests. Preserve that
owner and deferral. The current canonical-role crypto gate drops source fields
and omits feature admission; it does not inventory arbitrary configured Apps.

A focused gate/fixture patch is prepared at `canic-graph.patch` under core
evidence (SHA-256 `c9cce10aa0338829d58eebc014dc1c1aaf550ce0423b1f68cde2562a28d0e13f`).
Bash syntax, ShellCheck and read-only apply-check pass. It has not been applied;
Canic fixtures/builds have not run. It preserves source-qualified identities,
adds optional BLAKE3 feature checks and removes implicit fetch. It alone does
not complete Host admission for actual generated/configured build selections.

## Retained boundaries and supply

The maintainer authorized npm publication and identified the actual npm
organization as `tokostudios`; `borovan` is a verified owner. The earlier
`@dragginzgame/ic-platform` proposal was never published. The package, lock root,
standalone imports and proof now use `@tokostudios/ic-platform` 0.1.0; dependency
versions/integrities are unchanged, including sole production SDK Core 6.1.1.
The source-publish hook passed and the public upload completed on 2026-10-26.
Public registry metadata and downloaded archive match the qualified SHA-512
integrity. A fresh registry-only consumer declares Platform 0.1.0, resolves one
Core 6.1.1 identity, compiles strictly and runs the signing/raw-query/Candid/
certificate-refusal smoke proof without declaring SDK Core directly.

Fresh full Clippy preceded all 15 tests, strict packed-only/provider consumers,
bundle checks and actual Chromium page/module-worker execution under the final
name. Logs: `/tmp/platform-npm-tokostudios-{clippy,runtime,docs}.log`. Final archive:
`target/browser-consumer.iS3dze/tokostudios-ic-platform-0.1.0.tgz`; 17 files,
8,314 bytes, SHA-1 `28a1e46b7c5a81040ff8e3bfab39848f6a9af4c6`. Its `1-npm.log`
retains SHA-512 integrity. `target/browser-consumer.9gC9iy` is the source-publish
hook's matching proof. Final registry readback/standalone log:
`/tmp/platform-npm-registry-final.log`; evidence `target/npm-registry-proof.22TQKL`
includes metadata, downloaded archive, independent lock and `verified.json`.
Earlier archive hashes are historical. npm 0.1.0 is independent of the pending
Rust 0.6.18 release; do not repeat its publication for that Rust push.

The committed runtime at `8d80630` passed Linux and native macOS Intel/Apple
Silicon [browser CI](https://github.com/dragginzgame/ic-platform/actions/runs/40293386118)
under the earlier unpublished npm name. Final-name local qualification is Linux;
no new macOS run is claimed. Runtime implementation/SDK identities are unchanged.
[Platform #57](https://github.com/dragginzgame/ic-platform/issues/57) records completed supply;
[Blob #103](https://github.com/dragginzgame/ic-blob-storage/issues/103) and
[Canic #637](https://github.com/dragginzgame/canic/issues/637) retain actual adoption,
installed IC/provider/login and application acceptance. The earlier Blob copy
logs retain their scope at `/tmp/platform-blob048-*.log`; no sibling was edited.
Final npm-name, fixture and publication-document changes are committed locally
in `d4f3a5b` for the maintainer's 0.6.18 push. Close #57 after they land on the default
branch; consumer adoption does not block this Rust release. The normal release
gate still needs to run; focused checks do not claim full release CI acceptance.

RustSec `21d909e` still reports only crypto-bigint 0.5.5 / RUSTSEC-2026-0387 in
the focused no-fetch/no-yanked scan; no new BLAKE3 dependency finding appears.
[Platform #50](https://github.com/dragginzgame/ic-platform/issues/50) stays unresolved;
this is not an exemption or a yanked-package check. Executor =2.0.0, ic0 =1.2.0,
external DTO bytes and protected crypto/runtime bodies remain unchanged.
