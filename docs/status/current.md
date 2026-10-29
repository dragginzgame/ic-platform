# Current handoff

## Pending 0.8.3: paired native Profiler boundary

[PR #99](https://github.com/dragginzgame/ic-platform/pull/99) is refreshed onto
released Platform 0.8.2 at `b0bf82f`. Package versions remain 0.8.2. Preserve its
published Host 0.70 gzip error identity, Testkit/managed editor adoption, runtime
fixtures and released notes. Release/publication remain maintainer-owned.
Shared Tooling 0.27.7 stays at `a8baa51099ca48cdb12e2171f031d488db89d67d`;
do not replace it with an older supplying snapshot.

Profiler 0.11.0 at `651448415a3a7eaed799c43be6662000fe0659cd` already supplies the
paired query/cache/render and recorder-description interfaces. Profiler #8 is
completed and #9 is closed as incorporated by that release. Older consumers keep
their provider pin until adopting the paired interface. PR #99 is not merged;
Profiler currently selects its qualified code revision `fd845c4`.

This refresh preserves the complete native profile tree from
`fd845c4a633f0c82e80e2edaeec9963a3fe14144`. Its Cargo/Bazel inputs select Wire
0.7.23, gzip-only Platform 0.8.1 at `c8bbe7e` and Tooling `c20dbd1`. The ordinary
Platform 0.8.2 registry Host 0.70 gzip graph is separate. Do not silently switch
the native graph or mix supplying identities during this merge refresh.

Tooling #265 is merged, but published Host process 0.70.0 still lacks the consumed
observation module at the latest registry inspection. Published adoption remains
separate. Profiler owns capture activation, process selection/identity admission,
cadence, aggregation, retention, diagnostics and reports; Host owns raw OS facts
and workload custody. Rebuild private ICPROF03 peers together.

## Native behavior and qualification

Platform owns textual-canister admission, native DTOs, Busy/Started operation
completion and successful-result pruning. Domain errors are not resubmitted;
a timeout can leave an already-started operation running. Raw PocketIC JSON
routes remain unchanged. Python retains export-before-clear and file policy.

Compiled scheduler/defaults, round interpretations, source hashes and component
coverage belong here. Captures retain those facts for offline reporting; arbitrary
backend labels infer nothing. Profiler preserves schema-1 recorded dictionaries
with bounded admission. No runtime, crypto or charging body changes in this PR.
Retain query admission order, diagnostics, 8 MiB/query, 32 MiB total, 128-entry FIFO
and cross-run cache checks. Existing Value routes and saved formats remain.

Qualified native server SHA-256 at `fd845c4`:
`1bff7f3c31ce8dd2a2bd679593c6af64a17f0e06f8497384fdebee0403c57571`.
Its actual build passes 78 recorder, 109 server-product and six installed tests,
including the real Python export/guarded-clear command. Evidence remains in
Profiler's `artifacts/wire-0723-qualification.json` and `.build/receipts`.
The current merge refresh requires its own owner lint/governance and assembly
identity check; unchanged native execution evidence is reused, not a new build.

Wire and derive src trees match 0.7.22 exactly; the prior all-fields frame proof
remains applicable. Earlier Python/browser/assembler/script, offline CLI, MSRV
and substituted release-fixture evidence retains its exact input scope. Profiler
0.11.0's delivered Linux, Intel/Apple Silicon macOS core and browser CI passes.
Do not relabel those core jobs as native-server macOS qualification.

Native IC-server Darwin, full replica Bazel/NNS, fresh-host provisioning and
live-provider/release acceptance remain separate. #98 owns the extra strict
native lint findings; that probe is not claimed as passing. #50 stays deferred
with its advisory check intact. Hosted qualification runs asynchronously.

The maintainer retained Platform gzip. API input/decoded limits remain 128 MiB;
the recorder decoded limit remains 256 MiB. Strict single-member framing rejects
corrupt, truncated, concatenated and trailing input. PNG remains distinct zlib.
The IC pin stays `ee8086a9773725fd1058832a9400297cf0990112`, Rust 1.99.0.

## Released 0.8.0: Principal and selected ICRC packaging

`principal` supplies owned Principal/Account bytes, bounded text and Serde without
registry Principal or hashing. Runtime/CDK use that capability. Select
`principal-external` for registry identities/byte conversions and
`principal-self-authenticating` for unchanged upstream hashing. Required external
protocol/client and signature profiles select their own prerequisites explicitly.

Selected ledger DTOs/callbacks now have Platform Rust identities under the existing
feature/module. Candid/Serde bytes, effective-zero accounts, absent versus explicit
zero fields, legacy Nat64 and unchanged upstream value hashes/vectors remain.
The registry ledger package and unused storage/CBOR, metadata-key and consent
formatting machinery are retired. The [packaging contract](../migration.md#minimal-principal-and-selected-icrc-packaging)
retains the migration and 28 independent frozen external schema obligations.

The released handoff at `80b424a:docs/status/current.md` records 505 native/Wasm
feature selections, eight standalone profiles, schema/type/byte/hash proofs,
source/license guards and MSRV/Rustdoc/format checks. Evidence locations include
`/tmp/platform-dependencies-20261029` and `target/feature-graphs/attempt.FZeIyu`.
Do not infer installed-runtime, registry publication or consumer adoption from
those package checks, or claim a Wasm-size saving from removed dependency edges.

Standalone verification additionally requires `ic_principal/serde` to decode
certified canister ranges. That release correction preserves protected verifier
bodies and is not supplied by unrelated Candid features. Full owner Clippy and
four standalone native/MSRV/Wasm profiles pass; only verification adds external
Serde. Minimal Principal, external conversion and creation graphs remain narrow.
Its retained evidence is `target/signature-serde-fix`.

## Retained library and safety contracts

#73's Candid guard permits exactly 16 constructed wire levels and rejects more
before upstream decoding, retaining its quotas and combined decode/skip budget.
It does not repair upstream Candid or bound arbitrary trusted schemas/visitors.
Installed/frozen cost evidence and its limits remain in
[the migration contract](../migration.md#admission-cost-2026-10-29).
Owned endpoints remain Wire; external identities, bytes and persisted contracts
retain their owners. Do not restore retired CDK/compiler paths.

#86 supplies external ICRC-10 SupportedStandard; Canic #706 owns optional adoption
on its pending base. Source/installed proofs do not establish registry adoption;
mounting, consent, authorization and budgets remain consumer-owned.

#72's narrow short-form DER refusal is the sole protected-source exception.
Crypto bodies, constants, vectors and licenses remain intact; long-form support
and oversized encoder conversions are outside that change. #74 retains native
Host/Origin protection and local nonbrowser access.

#50 remains set aside for integration only, with its advisory check intact.
No ignore, suppression, incompatible override, Agent fork or in-house crypto is
authorized. Original #97 scratch receipts/binary were externally removed after
qualification; its PR retains those historical observations. Older evidence and
recovery locations remain in the
[previous committed handoff](https://github.com/dragginzgame/ic-platform/blob/d662c19501a16047030e9afb40ac635470a6942d/docs/status/current.md).
Do not delete retained build/recovery artifacts when refreshing this summary.
