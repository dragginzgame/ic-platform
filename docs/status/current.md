# Current handoff

## Pending 0.8.2: Host process and private Wire inputs

PR #95 is refreshed onto source release **0.8.1**, `10c2610`. Keep released
0.8.0 Principal/ICRC packaging, the signature Serde repair and 0.8.1's Profiler
product ownership intact. Package versions remain 0.8.1; release/publication
are separate maintainer actions. The normal checkout and siblings are untouched.

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

The original supplier proof is recorded in PR #95 and Profiler's
`artifacts/process-wire-qualification.json` at `55e08a9`: 74 recorder,
104 native product and five installed profiling tests, focused recorder Bazel and
real CLI lifecycle cases. Its frozen server hash is
`2150a037ceb62a2ad46adb9a6ea9705d40973ce9a8845743113270e53771cf4a`.
The native profile inputs are unchanged by this base refresh. Full owner Clippy,
505 native/Wasm feature selections, admission fixtures, 45 assembler tests,
source/license guards, formatting, inventory and links pass. Standalone signature
verification and owned Principal pass native/MSRV/Wasm; focused gzip tests pass
on current Rust and 1.91. Evidence is under
`/tmp/ic-platform-merge-95-97-20261029/evidence95`. Old hosted functional checks
pass and the retained advisory failure is #50. New hosted checks run asynchronously. Native macOS server execution, full replica Bazel/NNS,
supplier publication and live acceptance remain separate; no size saving is claimed.

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
