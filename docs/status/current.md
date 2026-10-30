# Current handoff

## Pending 0.9.0: selected canister management DTOs

The maintainer authorized adopting only the management types used by sibling
canisters. Wasm now exposes 43 existing names backed by 51 complete maintained
0.11.0 declarations, including required field types and all enum variants.
Selection follows Canic Core/control-plane, Blob and Canic's runtime-probe
canister. Native builds retain all 57 registry exports and their type identities;
14 host-only snapshot-transfer exports are unavailable on Wasm. Source/license
guards and provenance record this target-dependent boundary. Siblings are untouched.

The Wasm management graph excludes the registry DTO package and its implicit
Candid printer, removing pretty, arrayvec 0.5, typed-arena and unicode-width.
The final owned status-codec probe saves 55,386 bytes (6.6%) without custom
sections: 842,550 to 787,164 bytes. This differs from the earlier ~63 KB
manifest-only experiment and is not a measured Canic production binary.
Candid wire behavior stays unchanged; diagnostic formatting can differ.

Full owner Clippy passed before tests. All 51 schemas match pinned upstream;
three owned unit tests and five native consumer tests pass. Current/MSRV native
and Wasm consumer checks, feature/runtime graph refusals, source mutation guards,
strict Wasm rustdoc, inventory, boundary, pin and documentation checks pass.
All eight installed runtime configurations pass, with owned management HTTP,
history, status, snapshot and lifecycle byte proofs in the four external-Candid
configurations. Evidence: `target/management-adoption/evidence.json`; installed
fixture: `target/cdk-runtime.Sh5GVo`. Versions remain 0.8.5 with pending 0.9.0
notes; no release, consumer adoption or live IC qualification is claimed.

## Pending 0.9.0: embedded production runtime

The maintainer authorized embedding IC0 1.2.0 and executor 2.0.0 inside Platform.
The old registry pins/no-fork restriction are superseded for this scope. Preserve
Apache-2.0 licenses, unchanged IC0 source/ABI, callback-source fingerprints and
all protected crypto code. SlotMap remains the executor's sole registry runtime
dependency; its version_check dependency is build-only.
Task/method handles are Platform-owned; Cargo's inherited links guard prevents
coexisting executors and makes coordinated consumer adoption mandatory. The
qualification fixture selects published Timers 0.32.3 and one local Platform.

Executor fixes retain wake-routing metadata while polling, coalesce wakeups,
cache wakers, reclaim completed membership/expired queues and preserve drop order.
The sanity review puts wake queues in their owning methods, removes redundant
task context, and clears cancelled wakeups before destructors can spawn new work.
The maintainer selected standard Vec task lists to prioritize production Wasm
size after the benchmark in `target/vec-smallvec-benchmark/results.json`.
The final Rust 1.99.1 Wire fixture saves 2,762 bytes (1.1%) without custom sections;
small fresh methods pay one extra allocation and a few hundred IC instructions.
SmallVec remains only through host/tooling dependencies. Keep generation checks.

After Vec adoption, full owner Clippy, ten native regressions, source/license
mutation guards, executor standalone native/Wasm proofs on current/MSRV, scoped
feature graphs, inventory and documentation checks pass. Earlier IC0, package
contents and strict rustdoc evidence remains unchanged. All eight installed
raw/ic-call × Wire/external-Candid × Rust 1.99.1/1.91 combinations pass on Linux
PocketIC 16.1.0, including query self-wakes, callback traps, Timers and upgrades.
Latest receipts/source/artifact hashes: `target/vec-runtime-20261030/evidence.json`;
installed fixture: `target/cdk-runtime.gTkMdr`. Earlier import/review evidence
remains in `target/owned-runtime-20261030/` and `target/runtime-review-20261030/`.
Vec adoption changes only Platform's direct SmallVec lock edges; retained host
package versions and unrelated dirty work are preserved.
No publication, sibling adoption, macOS or live IC qualification is claimed.
Existing unrelated dirty macro/graph/Profiler work is preserved.

The pending changelog is 0.9.0 because runtime identities and graph admission
change. Manifest versions remain 0.8.5; no version/tag/publication was performed.
The previously prepared compatible 0.8.6 notes are included in that draft.

## Complexity cleanup

The earlier compatible cleanup derives all 32 standalone feature selections
from Cargo tests, shares graph classifications and centralizes entrypoint-kind
facts. Native queries filter before copying, share locked cursor facts and typed
snapshot coverage, and use borrowed pricing keys and shared argument primitives.
HTTP unsigned strings retain their distinct lexical admission. The maintainer
then authorized cutting the viewer's blanket integer projection: producers now
encode counters directly, nested PocketIC DTOs retain their canonical Serde
encoding, and viewer-wide encoding/version labels are retired. No parallel DTO
owner, compatibility reader or saved-format migration is introduced.

Evidence: `/tmp/platform-cleanup-20261030/final-qualification.json`. Full owner
Clippy preceded tests; 14 macro tests, Rust 1.91, standalone native/Wasm and scoped
graph checks passed before independent executor/IC0/manifest edits appeared in
the shared checkout. Consumer/runtime/feature graph fixtures also pass with those
concurrent edits; their Cargo/runtime qualification remains the separate receipt
above. No full local CI or release validation was run for this cleanup.

The viewer encoding cut is qualified separately in
`/tmp/platform-json-cut-20261030/qualification.json`: full owner Clippy before
tests, strict native library Clippy, 113 product tests and all six installed Linux
PocketIC scenarios pass. The HTTP/MCP regression covers counters above JavaScript's
safe range and canonical DTO/charge-dictionary serialization. Canonical assembly
uses isolated provider/Profiler 0.11.1 sources and prepared registry dependencies;
917 dependency metadata hashes match, 19 sysroot artifacts are compiler-validated,
and all 936 artifact stamps stayed unchanged through compilation/execution.
Library/tests/server were freshly linked; the unchanged installed-test executable
was reused with its recorded hash. Final server SHA-256:
`f04177cb991dfdd11ed8e2fc3d0b84019b628677465f97b238c4ba4980057ea6`.
Native macOS, fresh-host, full Bazel/NNS and live IC acceptance remain separate;
no performance or Wasm-size improvement is claimed without measurement.

## Release accounting before the embedded-runtime cut

Latest tagged release is 0.8.5 (`ec51bb0`, 2026-10-30); package versions remain
0.8.5. Its initially empty notes now record the workspace lock's Wire/derive
0.7.24 selection. The 0.8.4 notes match that release's tagged content again.
No manifests, locks, tags or publication state change in this documentation repair.

PR #100 merged at `d0ce012` and entered the local release branch through
`af5b128`, after the 0.8.5 tag. Its compatible native dependency-source notes were prepared for 0.8.6 and are
now included in pending 0.9.0 alongside the breaking runtime cut. That native
source update itself changes no public API, wire or storage contract.
Release/publication remain maintainer-owned; keep existing evidence and siblings intact.

It replaces the native Git library suppliers with published Host
Process/FS/Artifacts 0.70.3 and gzip-only Platform 0.8.3, retaining Wire 0.7.23.
Cargo/Bazel locks are regenerated from the selected registry sources. Profiler
adopts Shared Tooling 0.27.9 and the paired consumer lint fixes from Profiler #10.
Registry code revision `62a165b12f5ffe94808cde95dfde3ab793f6a218` passes full owner
Clippy, native primary-package Clippy, a fresh Cargo server build, 78 native
recorder tests, 110 server-product tests and all six installed scenarios. The
consumer additionally passes all 78 recorder tests on current Rust and 1.91,
governance and its actual patch/minor/major/refusal/recovery release fixtures
(CI/native execution substituted; no real release). Canonical assembly matches
all five generated dependency inputs, and the selected native graph excludes
lifeline and CLI-only NNS initialization. Bazel repin/target loading passes.

Profiler retains exact inputs, hashes and fresh receipts in
`artifacts/registry-20261030-qualification.json` and `.build/receipts`. Server
SHA-256: `288f4c7c0bfcf3a374822e9cd97ae8e01dca2e7afb521e1027f4f182dc28350e`.
Older parent execution records below retain their original source identities;
native macOS, full replica Bazel/NNS, fresh-host and live acceptance remain separate.

## Released 0.8.4: paired Profiler boundary and qualification fixes

Release 0.8.4 (`8302c3d`) includes PR #99's paired boundary and qualification fixes.
Shared Tooling 0.27.7 stays at `a8baa51099ca48cdb12e2171f031d488db89d67d`.
Siblings stay read-only; separate isolated contributions supply consumer fixes.

PR #99 owns textual-canister requests, Busy/Started completion, successful-result
pruning and compiled backend descriptions. Domain errors are not resubmitted;
timeouts retain uncertainty about started operations. Raw PocketIC JSON and
saved schema-1 reports remain. Profiler owns export-before-clear, file publication,
capture/report policy and rendering. The delivered 0.11.0 paired product interface
remains required; older consumers retain their previous provider pin.

Installed-runtime CI now selects Testkit's required `cli` feature. Its first
refreshed run passed server setup, then exposed a stale source qualifier in the
maintained runtime lock after the local version update. The fix removed that
qualifier without changing registry identities, checksums or edges; it retains
strict registry comparison. Runtime proofs retain shared dependency caches,
but a kernel file lock covers each build and
artifact copy. The inherited descriptor survives child execution and releases on
exit. A concurrent-run regression pauses after Cargo and before retention, proves
a second run is blocked and checks every retained Wasm's run identity. Failed
builds propagate without copying stale outputs. Protocol/feature graphs stay fixed.

## Native lint boundary

Platform #98's supported check is strict `pocket-ic-server --lib --no-deps` on
Rust 1.99.0, with the IC pin's existing Clippy configuration. `native-verify`
checks it before native builds/tests. The existing provider command also exposes
`clippy -- --no-deps --lib -- -D warnings`. No warning allowance is added.

Provider-owned fixes keep retry/disposition order, use the final event directly,
name the metrics-registry tuple and add meaningful must-use messages before
unchanged async-trait expansion. The native patch changes no cryptographic body.
Three consumer-owned CLI/render conditionals are supplied separately by
[Profiler #10](https://github.com/dragginzgame/ic-profiler/pull/10), commit
`a947b33`, merged at `b72a0b0`. Adopt that source with this profile. Platform #98
is closed after both supplying changes reached their default branches.

## Historical native evidence

PRs [#99](https://github.com/dragginzgame/ic-platform/pull/99) and
[#100](https://github.com/dragginzgame/ic-platform/pull/100) retain prior input
identities, execution receipts and limitations. Earlier Git-supplier results do
not qualify registry inputs or this cleanup's changed native server.

## Supply and retained contracts

The native recipe retains IC `ee8086a9773725fd1058832a9400297cf0990112`, Rust
1.99.0, registry Host Process/FS/Artifacts 0.70.3, Wire 0.7.23 and gzip-only
registry Platform 0.8.3. Testkit/editor selections remain independent. Preserve
one registry Host source family. Private ICPROF03 peers rebuild together; saved formats remain unchanged.

Profiler's independent recorder owns retention/report policy and private
PNG/range mechanics. Platform owns native state/HTTP admission and pricing.
Keep 8 MiB/query, 32 MiB total, 128-entry FIFO, cross-run checks, diagnostic order,
128 MiB API import and separate 256 MiB recorder decoded limits. Preserve exact
read/write PNG/JSON schemas and absent/unknown coverage semantics.

Owned Principal/Account remain Candid-free under `principal`. External DTOs are
separate Platform-maintained types; registry Principal/Nat/function identities
and required external Candid/Serde bytes stay unchanged. Account Display shares
the owned formatter; external parsing/errors and absent/explicit zero fields stay.
The [packaging contract](../migration.md#minimal-principal-and-selected-icrc-packaging)
retains 28 independently frozen selected-ledger schemas. Value hashes are intact.

Management =0.11.0 and stable structures =0.7.2 remain fixed. Executor/IC0
now use the maintained embedded implementations described above. Memory/Storage own allocation, checked reopening and leases. Owned endpoints
use Wire; required external Candid and legacy checkpoints retain their contracts.
Bounded Candid admits at most 16 constructed wire levels, including unused and
skipped data, within existing quotas; arbitrary trusted visitors are not covered.
The [nesting contract](../migration.md#external-candid-nesting-qualification-73)
retains original trap/16–17-level and cost evidence. #72's narrow DER envelope
exception and #74 Host/Origin admission stay unchanged. Protected crypto bodies,
encodings, vectors and licenses are preserved. #50 remains deferred with its
advisory check intact; no ignore, incompatible crypto override or Agent fork.
