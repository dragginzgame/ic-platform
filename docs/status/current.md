# Current handoff

## Delegated Platform 0.15.3 delivery

The maintainer requested deployment of the pending Platform patch. Scope is
ic-platform, ic-platform-macros and ic-platform-runtime-guard from 0.15.2 to
0.15.3. Types stays at published 0.15.1; the browser package is unchanged.
Use the existing direct release workflow and crates.io publication separately.
The complete release gate, source/metadata admission and registry verification
remain required. This scope includes integrating already-merged main, not a new
native product deployment, package boundary or cleanup operation.

Shared release commit 0.27.22 is adopted at
143a9abb9ae81feae9400ff3844b03b1ddfdd75a through the canonical exporter.
Supplier VERSION still reads 0.27.21; preserve supplied metadata as previously
authorized. No snapshot payload is edited manually. Root Cargo has literal member
versions in complete, grouped dependency blocks, short titles and separate ledger
comments. Every catalog alias, including the unused facade, belongs to its group
policy. No graph edge, Rust API or production implementation changed in this
adoption. Grouped formatting preserves the catalog byte-for-byte.

The current source integrates main 4ae49ad, which merged native Profiler #124.
Earlier #122 is merged at af5fa32. Pending 0.15.3 notes include the native profile
and Shared 0.27.20–0.27.22 tooling work. Published histories, Types metadata and
package versions remain unchanged until the release runner prepares its increment.

Canonical ledgers remain releases/platform/CHANGELOG.md and
releases/types/CHANGELOG.md. Root CHANGELOG.md is an index; coordinated history
through 0.15.0 remains once in releases/history.md. Initial Types 0.1 history stays
in its own ledger; the crate-local pointer is excluded from archives. There is
one workspace/lockfile and no root workspace.package.version.

## Validation, custody and recovery

Focused snapshot, full Clippy, boundary/inventory/pin, formatting/hook, singleton
and current/MSRV native/Wasm identity/schema proofs passed for the adoption.
Full Cargo metadata was unchanged. Six malformed unused-catalog cases are refused
through both release and dependency callers. Disposable Platform preparation with
the actual source-selected editor updates all three catalog aliases, retains
operators and preserves Types/history. Evidence remains under
/tmp/platform-shared-021-consumer.ntNnmG and /tmp/platform-shared-02*-*.log.
These checks do not replace the complete gate required by the current delivery.

Release/publication commands always pass RELEASE_GROUP=platform explicitly.
Types requires its own selection. Missing/empty/unknown groups refuse before
setup or effects; PACKAGE only narrows the selected group. Common runtime-lock
and Platform-only exact guard-pin custody remain transactional. Unfinished saved
attempts cannot be interpreted under changed policy. Older legacy push-phase
journals remain retained and require their original source/policy context.

Current delivery logs are /tmp/platform-deploy-platform-0153.log and
/tmp/platform-deploy-platform-0153-publication.log. The runner owns
.git/release-state/platform-0.15.3.plan; publication owns target/publication/.
Observe those receipts, GitHub and registry state for completion, not these notes.
Retain failure evidence and reconcile uncertain effects before recovery.
Normal builds/releases never clean artifacts. Explicit cargo clean retains its
whole-workspace target scope; durable evidence belongs outside target. The
separate .tools store is unaffected by Cargo cleanup.

## Qualified native Profiler profile

The integrated native inputs are byte-identical to immutable provider
b9f4f0fa8f57f1448c5cd024924339ab546909bb, merged in #124. They select direct
Host Process/FS/Artifacts 0.73.1, arithmetic-only Metrics 0.24 and core Wire 0.13.
The dependency patch SHA-256 is
20ae478ee37d3f2382f1192b6c52976f4fdca721f0e3e1d585e52e2a55e2ec83.
There is no registry Platform/types/endpoint dependency in that native graph.
Gzip levels/framing, 128 MiB API and 256 MiB recorder ceilings, 60 KiB process
frames, counter quality, ICPROF03 and saved report formats are preserved.

Retained #124 qualification includes strict Platform/standalone/native Clippy,
83 recorder tests on current Rust and 1.91, 114 server tests, six installed
scenarios, scoped Bazel repin/alias loading and reconstruction of all seven
build/dependency files. Executed server SHA-256:
50939646443d38c76c116b7e832c4b45568e3c728593ad8e4426c107628cb1e0.
Paired Profiler artifacts/fleet-adoption-20261031/ owns immutable receipts and
source comparisons. Earlier #122 receipts remain historical and bind 6c054c8.
No new installed or macOS proof is inferred from integration/release metadata;
consumers selecting a new provider pin retain their paired source qualification.

## Retained constraints

The installed canister runtime lock still selects Timers 0.32.3 and its older
Platform/executor graph. Fresh installed-runtime/advisory preparation previously
stopped at the separate Timers/executor conflict (#116). Keep every guard/gate;
native Profiler acceptance does not establish that canister composition.
#50 remains unresolved: target/issue50-20261030/evidence.json recorded
crypto-bigint 0.5.5 without a complete compatible correction, excluding yanked
checks. Preserve crypto bodies, DER exception, fingerprints and upstream vectors.
External Candid retains fixed 16-level admission and legacy checkpoint bytes.
Registry Candid stays selected after the rejected 7,631-byte/3k-maintained-line
reader tradeoff. Historical Vec/management evidence remains at
target/vec-runtime-20261030 and target/management-adoption, without fresh acceptance.
The migration contract remains the detailed provenance/ownership authority;
earlier installed context is in the
[0.14 release handoff](https://github.com/dragginzgame/ic-platform/blob/7b4ed265804a60b796c4680880cb7a4e66e53d1e/docs/status/current.md).
