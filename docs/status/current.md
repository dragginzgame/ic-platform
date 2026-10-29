# Current handoff

## Pending 0.8.2: adoption and integrated cleanup

Resolve local `1971797` with incoming `c8bbe7e` (#95/#97), retaining both
batches. Package versions remain 0.8.1. The maintainer approved 0.8.2 once for
the unused gzip surface, including Host 0.70's error identity; future compatibility
policy is unchanged. Release/publication remain maintainer-owned. Siblings stay
read-only; the incoming native recipe and cryptographic bodies are preserved.

Keep published Tooling 0.70.0 for the ordinary gzip facade, Testkit and managed
release editor with `cli`. Testkit's raw PocketIC imports route through Platform;
managed instances use `instance()`. Its registry Platform dependency stays
independent of local release versions. The runtime lock retains Wire 0.7.22;
workspace Wire minima remain 0.7.22 with the main lock's existing 0.7.23 selection.
Shared Tooling 0.27.7 remains at `a8baa51099ca48cdb12e2171f031d488db89d67d`,
including #322's graph observations. Do not replace it with the older revision.

Incoming #97 derives feature activation from metadata, shares consumer setup and
private macro syntax facts, and reuses one locked BLAKE3 graph observation.
Preserve Principal tiers, ICRC exclusions, admission order, diagnostics and codecs.
Its native dependency patches consolidate without changing seven assembled inputs.
Profiler #9 owns query/cache/tool/render plumbing; Platform retains HTTP admission,
native start/stop, cleanup, metrics and pricing. Retain 8 MiB/query, 32 MiB total,
128-entry FIFO and cross-run checks. Older consumers keep their previous pin until
paired product adoption. Catalog cadence reads the unchanged compiled constant.

The native profile stays exactly at incoming `c8bbe7e`: Host supplier
`c20dbd11785c17d369651f51fa0967fbec69c57a`, Wire 0.7.21 and gzip-only Platform
scaffold `ba98bd33ef1cbe463d6daf34f0511f17d7770c20`. All its Host packages retain
one Git family; ordinary Platform gzip 0.70 is a separate graph. Rebuild private
ICPROF03 peers together; saved reports, external bytes and retry authority remain.

Prior native qualification is provider `f5051bf` + Profiler `e9f68b6`: 74 recorder,
26 product, three catalog and five installed tests; Linux server SHA-256
`b863005e0272c82e2f94a6a3684bbdeb0652fdb8162b6aa823d1da02b2b290e1`.
Original scratch receipts/executable were externally removed; PR #97 retains the
observations. These unchanged-source results are reused, not a new server build.

Merge evidence is under `target/merge-resolution-20261029`. Full owner Clippy
passed before longer checks. All 505 native/Wasm selections, 14 macro tests on
both compilers, standalone CDK/external-Candid/BLAKE3 native/MSRV/Wasm proofs,
admission/refusal fixtures, 45 assembler tests, source guards, snapshot provenance,
inventory, formatting, links and 0.8.2 changelog admission pass. Prior supplying
proofs remain in PRs #95/#97 and `target/ic-tooling-adoption-20261029`.
Full CI, installed-canister/native-server execution, native macOS, full Bazel/NNS
and live acceptance remain separate. #98 owns extra native Clippy debt; #99 stays
draft. #50's advisory failure remains deferred and unchanged. Profiler #9 needs
its paired product module and final qualified provider pin before integration.

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

## Retained protocol and safety boundaries

Owned endpoints use Wire. Required external Candid and existing checkpoint bytes
retain their contracts. Bounded Candid admission rejects more than 16 constructed
wire levels, including unused definitions/skipped values, using existing work
quotas. This does not repair arbitrary trusted Rust schemas/custom visitors.
The original 16/17-level and 8,192-opt trap, management, callback, timer and upgrade
proofs are recorded in the released handoff and `/tmp/platform-fixed-depth-20261029`.
The [cost report](../migration.md#admission-cost-2026-10-29) retains measured decoder
costs; further tuning awaits consumer metrics.

Management =0.11.0, executor =2.0.0 and ic0 =1.2.0 identities remain fixed.
Memory/Storage retain allocation, checked reopening, layout and exclusive leases.
Crypto bodies, encodings, constants, vectors and licenses remain protected.
#72 admits only its reviewed short-form DER envelope check; long-form support
and oversized encoder conversions are outside that exception. #74 retains native
Origin/Host admission and explicitly supported local nonbrowser clients.

[#50](https://github.com/dragginzgame/ic-platform/issues/50) remains the retained
RUSTSEC-2026-0387 blocker, set aside for integration only. Keep its advisory check;
no ignore, incompatible crypto override, Agent fork or in-house primitive is
admitted. Follow-up remains in GitHub, not a separate local issue queue.

## Prior native qualification

The native profile retains IC pin `ee8086a9773725fd1058832a9400297cf0990112`
and Rust 1.99.0. Frozen server hashes and exact receipt scope remain in the
handoffs at `80b424a` and `c5740cd`; no artifacts are removed. This adoption adds
no installed/native-server, macOS, full Bazel/NNS or live-provider qualification.
