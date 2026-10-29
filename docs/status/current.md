# Current handoff

## Pending 0.8.1: paired Profiler product source contract

Local source release **0.8.0** is `80b424a`. The current merge brings incoming
`c5740cd`, including #93's Profiler product ownership work. Its former pending
0.7.7 notes now belong to 0.8.1; package versions stay at 0.8.0. Preserve the
released Principal/ICRC packaging and standalone signature Serde correction.
Publication and consumer/provider adoption retain their separate qualification.

Profiler owns the injected CLI, query renderer and private PNG/range helpers.
Platform retains the pinned IC source, native state/trace adapters, HTTP admission,
patched client and Cargo/Bazel inputs. Older consumers keep their previous
provider pin until supplying the files in the
[paired contract](../../native/ic-profiler/README.md#paired-product-source-contract).
The source profile rejects overlapping owners and records provider/consumer digests.

The maintainer chose to retain Platform gzip. Native imports use that facade;
exports preserve core gzip bytes. API input/expanded limits remain 128 MiB and
core expansion remains 256 MiB. Corrupt, truncated, trailing and concatenated
members are rejected. PNG remains zlib with its own image budgets and admission.
Server-library product regressions now run before installed native scenarios.

The native Cargo/Bazel recipe selects published Metrics 0.17.2, Platform 0.7.6
and Host Artifacts 0.69.12. Those source-profile inputs are independent of this
repository's 0.8.0 library version. Replica-internal metrics remain distinct;
physical measurements never acquire charging or allocation authority. Profiler
retains its product policy, and siblings remain read-only.

The incoming handoff records assembler and independent PNG/range checks. They
are historical evidence for their source inputs, not a new server qualification
for this merge. Full owner Clippy, 45 assembler fixtures, inventory/docs/format,
changelog validation and protected-source checks pass; evidence is in
`target/merge-resolution`. Full release CI and native build/runtime acceptance
remain separate.

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
