# Current handoff

## Pending 0.8.2: IC Tooling adoption

The local base is released Platform 0.8.1, `10c2610`. Package versions remain
0.8.1. The maintainer explicitly approved 0.8.2 as a one-time exception for the
unused gzip surface, including Host Artifacts 0.70's `GzipError` identity.
This does not relax future compatibility policy. Platform-only imports stay
unchanged; direct Host APIs exchanging that type must align to 0.70.
Release/publication remain maintainer-owned; siblings are read-only.

Adopt published IC Tooling 0.70.0 (source `ecc8d61`) for Host Artifacts,
Testkit qualification and the managed IC Build release editor. Preserve codec
levels/budgets, external bytes, Testkit transport/retry authority and exact
executor/ic0/management pins. The main lock changes only Host Artifacts relative
to this batch's captured input; retain its pre-existing Wire 0.7.23 selection.
The runtime lock updates Testkit and its required Host family together, retaining
Wire 0.7.22, Timers and the simulator selection. Its native dependency versions
are independent of the deployment graph. Testkit retains registry Platform 0.8
without a global local-source patch, including during release lock preparation.
The native Profiler Cargo/Bazel recipe
retains its separate qualified pins and is outside this update.

Shared Tooling 0.27.7 remains exported at
`a8baa51099ca48cdb12e2171f031d488db89d67d`, including source main `11d519c`
and PR #322's reviewed graph-observation interface required by merged #97.
All 109 snapshot files match the independently selected source commit.
Both Wire workspace minima remain 0.7.22. Native-only metadata and Wasm/default/
executor exclusions remain; the optional sccache catalog entry does not enable it.

Earlier Tooling/Wire proof is in `target/tooling-wire-adoption-20261029`.
Current adoption evidence is in `target/ic-tooling-adoption-20261029`.
Full owner Clippy passed before longer checks. Standalone gzip identity, prior
byte parity and refusal proofs pass on Rust 1.91/1.99.1. The Testkit driver passes
strict Clippy and six native tests on each compiler. Raw/owned Wire and external
Candid canisters build at MSRV; graph isolation, current Wire schema/TypeScript,
source guards and formatting pass. The prepared CLI-feature editor passes real
lock drift/refusal/idempotence checks, preserving same-version registry records.
Full CI, installed canisters and native macOS remain separate.

PRs #95/#97 are merged on remote main at `c8bbe7e`, closing #94/#96. This
working checkout has not advanced its Git base; retain those integrated Host/
process and Profiler product changes when combining work. Profiler #9 needs the
merged provider pin. #50 remains deferred; #98 owns extra native Clippy findings
and #99 remains draft. These integration records do not waive their checks.

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
