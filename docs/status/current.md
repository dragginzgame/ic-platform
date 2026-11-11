# Current handoff — 2026-11-11

## Delivered audit improvements

Platform 0.31.1 publishes the post-0.31 audit (`5a6a155`, corrected by `eb98320`). Codec
bulk-byte/bigint encoding, schema-buffer reuse, prepared-root signature APIs,
fixed certificate framing and 22 order-compatible management declarations are
qualified by focused Clippy, 109 tests, matched Wasm sizes and installed instruction
comparisons. The direct-map experiment was rejected for its throughput regression;
the original decoder/budgets remain. The separate unused `leb128` cleanup is
preserved. See the [audit report](../benchmarks/platform-031-audit.md) for gains,
small regressions, memory tradeoffs, exact recipes and retained evidence.
No Runtime, Types, browser or sibling release is part of this work.

## Published state

Toko 0.39.12 is installed; compiler is 1.99.1. The listed Rust releases
are available on crates.io, not yanked, and delivered to GitHub:

| Group | Version | Commit | Tag | Frozen batch / local receipt |
|---|---|---|---|---|
| Types | 0.19.3 | `78548132b1a160894d4053510bd3b3c756283f39` | `types-v0.19.3` | `batch-1kF9qOrj` / `qualification.uqEkUL` |
| Platform | 0.31.1 | `5b6b761df8a029da7411e10aec3e29c97d490bad` | `platform-v0.31.1` | `batch-CqtVCFiE` / `qualification.haDuvA` |

Both groups passed all 19 required local stages, including Clippy, product tests,
consumer/feature graphs, Wasm builds and installed Runtime qualification. Each
publication verified its package archive. Platform's published Types requirement
is `^0.19.3`. Different increments used separate sequential frozen batches; both
were reconciled into the ordinary checkout. The private benchmark lock then
received only the local Platform 0.31.0 → 0.31.1 identity update; its registry baseline
remains exactly 0.30.0. Runtime/macros/guard remain 0.21.1; browser remains 0.2.0.

Hosted release CI has completed. [Native CI](https://github.com/dragginzgame/ic-platform/actions/runs/43024528840)
failed at `check-release-commands` on Linux and both macOS hosts; the underlying
exit 101 remains to be diagnosed. [Installed runtime](https://github.com/dragginzgame/ic-platform/actions/runs/43024528557)
failed because `check-platform-raw-rand.sh` could not find `ic-wasm`.
[Browser](https://github.com/dragginzgame/ic-platform/actions/runs/43024528522)
passed. Local Linux passes do not establish fresh native macOS acceptance.
[Advisories](https://github.com/dragginzgame/ic-platform/actions/runs/43024528517)
failed on crypto-bigint 0.5.5 / RUSTSEC-2026-0387; #50 remains explicitly deferred.

The [Wasm comparison](https://github.com/dragginzgame/ic-platform/actions/runs/43024528679)
failed against `fc55ae12184f9c21b1b0c23a4004cde150edf87d`: ordinary verification's
optimized total grew 477,858 → 477,993 bytes (+135), code 442,347 → 442,480 (+133),
exceeding the unchanged 128-byte allowance. Raw total/code grew +123/+121 bytes.
The other four profiles were unchanged; all 12 harness tests passed. This is a
measured regression, not a setup failure, and remains unresolved after publication.

## Pending development cleanup

Both workspaces select published Toko 0.39.12. Authenticated preview/install
retained all 96 managed files and existing entrypoints. The prior refresh
updated the LOC reporter; 0.39.12 updates the baseline and receipt. Compact, wide and JSON reports pass against
this repository; snapshot verification passes. No unrelated registry selections
changed; the private lock's existing local Platform 0.31.1 update is preserved.
The baseline adds explicit supplier freshness reviews; detailed policy links
retain their reviewed revision. This adoption is local,
uncommitted and unpublished; no product release was requested.

Testkit 0.100.2 is now selected in both catalogs and the maintained Runtime lock.
Drivers use its upstream async client; old ClientError conversions are removed.
Platform 0.24 is gone from qualification, while the intentional BLS 0.30 baseline
remains. Native upstream dependencies increase; this is not a Wasm-size saving.
Clippy/build checks pass for the private BLS example, runtime/Agent callers and
all four benchmark drivers. Installed raw/prepared × Wire/external-Candid passes
at `target/cdk-runtime.pMn8Ou`; raw_rand at `target/evidence/raw-rand.vBgXfS` and
Agent identity/wrong-root proofs at `target/agent-installed.Q0rdFS` also pass.
All three focused release-preparation fixtures pass with the independent benchmark
lock in the metadata transaction; exact baseline and foreign pins are retained.
Hosted Runtime now prepares ic-wasm; fresh hosted/macOS qualification is pending.
Verification-size experiments were rejected and source restored unchanged; see
the [audit follow-up](../benchmarks/platform-031-audit.md#post-release-cleanup).

## Delivered contracts

Platform 0.31 ships #156: declaration forms of `candid_record!`/`candid_variant!`
produce concrete Rust DTOs and codec mappings together, preserving explicit numeric
labels, bytes and budgets. Native/Wasm portable consumers exclude upstream Candid;
independent comparison oracles remain development-only. Query migration is external.

The qualified BLS wrapper now lives solely in the facade. Its signatures/errors
have Platform identity; callers using registry verifier types must select Platform
imports or exchange bytes. External curve/hash primitives remain unchanged;
certificate, delegation, subgroup and identity-key checks are retained. Prepared
public keys are caller-owned, carrying no trust or freshness policy. See the
[BLS qualification](../design/bls-wrapper/qualification.md) for source separation,
39 focused tests, matched Wasm results and installed signature evidence.

The adopted wrapper measured 13.88% fewer warm one-shot verification instructions,
7.65% fewer cached instructions and 2,643 fewer optimized Wasm bytes than Platform
0.30.0. These are fixture results, not whole-canister savings. Toko 0.39.10's
published narrow Digest-adapter policy now admits the actual production boundary;
no snapshot file was hand-edited and no gate was waived.

Types 0.19.3 directly constructs self-authenticating identity frames, saving 197
bytes in the matched probe while retaining upstream SHA-224 and identical bytes.
The prior SHA-2 upgrade still leaves +757 total / +501 code bytes against the older
deployment; no threshold waiver was made. The Wasm harness now retains each
revision's dependencies in explicit deployment mode, reports changed selections,
and preserves strict comparison mode and growth limits. See the
[deployment measurement](../benchmarks/platform-030-deployment.md).

Textual DID remains retired. Owned binary Candid remains the external-protocol
adapter, with independent schema/byte oracles; all-feature production Wasm excludes
upstream Candid/parser/printer. Native Agent/transport and browser SDK binary IDL
remain explicit external capabilities. Empty defaults, canonical Types and
consumer-owned budgets, freshness and authentication policy remain unchanged.

The earlier independent replacements, frozen comparisons and accepted boundaries
remain under `docs/design` and the [source-retirement handoff](../history/source-retirement-handoff-2026-11-10.md).
Source separation is engineering evidence, not legal clearance or a cryptographic
audit. Runtime/ledger/management qualification contracts remain authoritative.
Scheduling owns Timers composition; this repository adds no scheduler dependency.

## Recovery custody

Frozen `batch-uCPG6Jvb` stopped before preparation/publication at `check-features`:
a stale exact assertion required removed `leb128`. `eb98320` corrected admission
and added reintroduction tests; fresh `batch-CqtVCFiE` passed all 19 stages.
Preserve failed receipt `qualification.5kqUXx`; do not resume that frozen source.

Keep completed batches above and earlier `batch-JYb8Sz5n`, `batch-Y0aGfmfa`, `batch-VMBGwxm7` and
`batch-nUsSSjzc`, with their journals, candidate archives and qualification receipts.
Durable records are under `.git/release-state/frozen/`; candidates/evidence remain
under `target/release-frozen/`. Do not resume old batches over published versions.
Retirement/deletion is not part of deployment authority.

Preserve `batch-htvXWsUR` (unnumbered pending heading, preflight refusal) and
`batch-u1XYolFP` (stale Toko inventory, validation refusal); neither uploaded a
package. Preserve failed `batch-JKeeLzH0` and historical `batch-G3rEbkX7`,
`batch-QJcFkCdU`, `batch-_leIreCI`, `batch-C5BoFKA5`, `batch-AaHi2pgO`,
`batch-O3V4GL4T`, `batch-PIIqwepD` and `batch-mG_yG4uo`. Successful batches
supersede failures without rewriting their evidence.

Historical lock refs remain `refs/retained/platform036-lock-draft-20261108`
(`0263ab6e679ae3b8e9f89bddd796ca951b3f1df4`) and
`refs/retained/platform0270-lock-draft-20261108`
(`588b11a38c60b15ab5af19f30aef1c310a8b0975`). Never restore an old complete lock.
Bundles/receipts at `recovery-evidence/retired-checkouts/20261104-issue139/` require
maintainer review by 2026-12-04; no deletion, mtime renewal or budget increase is
authorized. Previously removed artifacts must not be claimed present.

The unreferenced signature draft remains verbatim at
`recovery-evidence/deferred-signature-comparison-20261110.rs`, SHA-256
`9fdee2e8b41c9b13e0d2a3127ddf95559a1094fa48d14dd17b7577d4ec86ed4a`.
It is outside the release; maintained comparisons live in the facade tests.
