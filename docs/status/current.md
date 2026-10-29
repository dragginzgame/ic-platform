# Current handoff

## Active paired Profiler contribution

[PR #99](https://github.com/dragginzgame/ic-platform/pull/99) moves Profiler's
remaining IC client protocol and compiled backend descriptions into this native
provider. #95 and #97 are merged; the contribution now targets main at `c8bbe7e`.
The consumer reuses Profiler #9's pure query/cache/tool/render source interface.
Refresh the consuming provider pin to the final contribution identity. Older consumers must
retain their existing source profile until adopting the paired interface.

Prior qualified native source is `d662c19501a16047030e9afb40ac635470a6942d`.
The new adoption changes the gzip-only Platform library to 0.8.1 at `c8bbe7e`;
its new native qualification is pending. Profiler retains fresh
assembly/build/test receipts under its `.build/receipts`, plus
`artifacts/platform-client-qualification.json` and the offline CLI proof. Its
working-tree source is bound by assembly digests; no clean-HEAD claim is made.
Linux executable SHA-256:
`b6f9e457b6782bcde883040b5a9a2a9eb2a3d1b68cc9dc08ae7fae1b4e0ed57f`.

Platform owns textual-canister admission, native operations, busy/Started result
completion and successful-result pruning. Operation errors are not resubmitted;
a timeout may leave an already-started operation running. Existing PocketIC REST
JSON arguments/replies and raw endpoints retain their external contract. The
Python CLI remains standard-library-only and retains export-before-clear policy.
The independent process transport remains IC Wire, with no Candid fallback.

Compiled scheduler models/defaults, round interpretations, source hashes and
PocketIC component/coverage facts belong here. Local captures retain them for
offline queries/bundles; arbitrary backend labels infer no IC facts. Profiler
preserves recorded schema-1 dictionaries with bounded admission before copying.
No runtime, crypto, charging body or own release version was changed.

## Current validation and remaining scope

Owner all-target/all-feature Clippy and the independent recorder lint pass.
The final paired native build passes 78 recorder tests, 109 server-product tests
and all six installed scenarios, including the real Python export/guarded-clear
command. The consumer's four new description tests pass Rust 1.91; Python,
browser, script, assembler and release-fixture proofs are detailed in #99.
Bazel loads the test's declared Python runfile and tempfile dependency; full
replica execution is not claimed. The actual offline CLI uses saved provider
facts rather than substituting this binary's compiled description.

Native IC-server Darwin, full replica Bazel/NNS genesis, fresh-host provisioning
and live-provider/release acceptance remain separate. Hosted checks run
asynchronously. The additional strict native lint findings remain in #98;
required owner gates pass, and that separate probe is not represented as a pass.
Historical receipts never qualify another binary.

## Supplying contracts

#95 supplies Host process and Wire inputs already adopted in Profiler 0.10.0.
All selected Host packages use Tooling revision
`c20dbd11785c17d369651f51fa0967fbec69c57a`; the gzip-only Platform scaffold is
`c8bbe7e1aee45c905de05ec5083d6bf7ba95819e` (Platform 0.8.1). Supplying publication remains separate.
Profiler owns capture policy, selected identities and accounting; Host owns raw
OS facts and workload custody. Rebuild private process peers together.

#97 consolidates native inputs and returns pure query/cache/render plumbing to
Profiler. Preserve 8 MiB/query, 32 MiB total and 128-entry FIFO/cross-run policy.
It adopts Shared Tooling #322 at immutable `1e89c0d` through the exporter; feature
activation comes from Cargo metadata, while identity admission stays explicit.
Its protected runtime/crypto sources and public library contracts stay unchanged.

The maintainer retained Platform gzip. API input/decoded budgets remain 128 MiB;
the recorder decoded budget remains 256 MiB. Strict framing rejects corrupt,
truncated, concatenated or trailing members. PNG stays a distinct zlib contract.
The IC source pin remains `ee8086a9773725fd1058832a9400297cf0990112`, Rust 1.99.0.

## Released library packaging

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
