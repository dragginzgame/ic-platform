# Current handoff

## Pending 0.8.2: graph and native product cleanup

PR #95 is merged at `1e3d24a`; PR #97 is refreshed onto it and released
0.8.1. Preserve the released Principal/selected ICRC packaging and standalone
signature Serde repair. Package versions remain 0.8.1. Release/publication are
separate maintainer actions; the normal checkout and siblings are untouched.

Shared Tooling's single-observation supplier is PR #322 at immutable
`1e89c0d3f8cbd6b68190f34a7c6975fb1063ea52`, adopted through its exporter.
Feature activation comes from Cargo metadata; identity/source admission remains
explicit. Standalone proofs share preparation only. Entrypoint adapters share
syntax facts while retaining their admission order, diagnostics and codecs.
The 0.8 Principal tiers and selected ICRC exclusions remain enforced.

Profiler #9 supplies pure query/cache/tool/render plumbing under its
`server/system_profile_product/` directory. Platform retains HTTP protection,
native start/stop, metrics/pricing adapters and scope cleanup. Query admission
preserves lexical/error order. Immutable image caching retains 8 MiB/query,
32 MiB total, 128-entry FIFO and cross-run checks. Existing Value API paths remain.
One native dependency patch replaces four overlapping patches. The compiled
scheduler constant supplies catalog cadence without changing charging behavior.
Older Profiler consumers must retain their provider pin until paired adoption.

The original qualified pair is provider `f5051bf` and Profiler `e9f68b6`:
74 recorder, 26 assembled product, three catalog and five installed profiling
tests passed. All seven dependency inputs match the prior ordered recipe's final
bytes. Frozen Linux server SHA-256:
`b863005e0272c82e2f94a6a3684bbdeb0652fdb8162b6aa823d1da02b2b290e1`.
Original temporary receipts/executable were externally removed after those runs;
results survive in the session observations and paired PR summaries. The native
profile is unchanged by this 0.8.1 base refresh; this is reused evidence, not a
new native-server build. Profiler #9 must select the final merged provider pin.

Integration evidence is under
`/tmp/ic-platform-merge-95-97-20261029/evidence97`. Full owner Clippy passed before
longer checks. All 505 native/Wasm feature selections, admission/refusal fixtures,
14 macro tests on current Rust and 1.91, five standalone Platform profiles and
all three ic-call profiles passed native/MSRV/Wasm. The real BLAKE3 unified-feature
refusal, 45 assembler tests, snapshot integrity, inventory, source/license guards,
ShellCheck, formatting and links pass. The native profile matches qualified
head `9747fa9` exactly. Previous functional hosted checks pass; refreshed hosted
checks run asynchronously. #50's advisory failure remains unchanged. The extra strict native Clippy probe has existing findings in #98;
it did not pass and changes no owner gate. Native macOS server, full replica
Bazel/NNS, supplier publication and live acceptance remain separate. No size or
speed improvement is measured.

## Integrated Host process and private Wire inputs (#94)

The native recipe supplies IC Wire 0.7.21 and Host process at Tooling
`c20dbd11785c17d369651f51fa0967fbec69c57a`. Its gzip-only Platform library
scaffold is `ba98bd33ef1cbe463d6daf34f0511f17d7770c20`. All selected Host
packages use one Git source family; no downstream Cargo patch is required.
The public gzip facade follows that same source. This is the reviewed supplier
for Profiler's already released 0.10.0 process adoption, not a broad upgrade.

Profiler retains process selection, creation-identity admission, cadence,
aggregation, retention, reports and CLI policy. Host owns raw OS observations,
launch gates and child custody. Rebuild all private peers together for ICPROF03;
external IC bytes, saved report/bundle formats and retry authority stay unchanged.
The IC pin remains `ee8086a9773725fd1058832a9400297cf0990112`, with native
Rust 1.99.0. Platform's library floor remains 1.91. Source/feature graph checks
admit only the declared immutable Host supplier and reject foreign/mixed sources.

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

The 0.7.6 handoff retains the qualified IC pin
`ee8086a9773725fd1058832a9400297cf0990112`, Rust 1.99.0 and frozen server hashes
with their exact source/receipt scope. Prior installed tests do not qualify
changed provider inputs. Native macOS, full replica Bazel/NNS genesis, release
CI and live-provider acceptance remain separate. The detailed handoffs are
preserved at `80b424a` and incoming `c5740cd`; no build artifacts are removed.
