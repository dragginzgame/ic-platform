# Current handoff — 2026-11-05

## Delivered Platform 0.23.0

**ic-platform 0.23.0** is pushed, tagged and published on crates.io. Release
commit: `7d2230c585b1a91c11ff12d0fcfce24bda424550`; tag: `platform-v0.23.0`.
Runtime remains **0.19.2**, Types **0.18.1**. Browser/npm and siblings were not
released or edited. [#141](https://github.com/dragginzgame/ic-platform/issues/141)
and [#145](https://github.com/dragginzgame/ic-platform/issues/145) are closed.

Native `ic-protocol::decode_subnet_metrics` now returns
`Result<Option<SubnetMetrics>, CborError>`, preserving Testkit's CBOR-null absence
and full u128 metrics values, byte limits and malformed/trailing refusal.
Consumers must advance their Platform requirement from `0.22` to `0.23`;
`cargo update` alone cannot cross this minor boundary. See the
[return-type migration](../migration.md#optional-subnet-metrics-0230).

Select defaults off with `certification-cbor` for standard/strict certificate and
tree codecs, native `ic-protocol` for envelope/read-state/metrics operations,
and `ic-protocol-fixtures` only for native development fixtures. The existing
[migration table](../migration.md#typed-ic-packet-and-fixture-boundary-0223)
records separately owned sibling adoption, including Tools 0.83.0 and Testkit
0.87.0. #142–#144 are closed for the supplying APIs; consumer qualification and
publication remain separate. All cryptography, trust, delegation/freshness,
request binding, budgets, transport and replay/retry policies retain their owners.

## Validation and publication evidence

The complete **20-stage Linux gate passed** on Rust **1.99.1**, source
`4170f4da8120412dbb49766d842c76ab5b3011e4`. Receipt:
`target/release-validation/qualification.AvTtIC/receipt.json`.
It contains Shared's complete `validation_evidence`, not a locally interpreted
stage projection. Native/Wasm graphs, Clippy, tests, consumers and both installed
Runtime profiles passed. Candidate graph evidence remains under
`target/cdk-runtime.xGGgrf/`; no Runtime release or new native-provider acceptance
is implied. Release/source recovery retains the
[custody contract](../platform-release-validation.md#release-and-recovery-admission).

After validation, a Candid lock draft reappeared and post-validation admission
stopped before preparation. The identical draft was retained; exact
`release-resume VERSION=0.23.0 RELEASE_GROUP=platform` re-admitted the original
proof/source and completed metadata, commit/tag and atomic main/tag delivery.
Original and recovery logs: `target/platform023-delivery/{release,resume}.log`.
No gate was bypassed or historical proof relabelled as a fresh run.

The downloaded crates.io archive's checksum matches the index and its clean VCS
provenance matches the release commit. SHA256:
`be12bedb3f3a6eaeb38eaa4f834159e181b560e530df59291fa3a486219cfc57`.
Registry-only certificate and protocol consumers pass, including actual
`Some(u128::MAX)`, null absence and refusal cases. Certificate-only excludes
Candid/Agent/Runtime; protocol excludes Agent/Runtime. Publication attempt:
`target/publication/publish.selection.x6Tvkh`. Complete verification:
`target/platform023-delivery/deployment-verification.json`.

Hosted [CI](https://github.com/dragginzgame/ic-platform/actions/runs/42050137062)
was pending and [installed runtime](https://github.com/dragginzgame/ic-platform/actions/runs/42050137222)
was running at observation. Native macOS results remain asynchronous; there is
no new local macOS qualification. Linux results do not claim macOS, Chromium
or complete native-provider acceptance.

## Preserved work and remaining boundaries

Unrelated browser drafts and the Candid 0.10.40/derive lock update are restored,
with browser checksums and the original lock-patch hunks verified. Browser dist
was rebuilt. They remain uncommitted; no npm version or publication changed.
Recovery refs: `refs/retained/platform-023-unrelated` and
`refs/retained/platform-023-late-candid`. No retained work was dropped.

Shared **0.30.0**, supplier `4d0961821a31766e3044c08fbe5954ed81213ec9`, remains
installed without payload edits. The [audit](../audits/shared-030-practices-2026-11-05.md)
retains rule coverage. [#140](https://github.com/dragginzgame/ic-platform/issues/140)
still owns paired Profiler module layout and native acceptance;
[#50](https://github.com/dragginzgame/ic-platform/issues/50) remains the
crypto-bigint advisory, without a waiver. Earlier delivery and supplier details
remain in the [preceding handoff](https://github.com/dragginzgame/ic-platform/blob/7d2230c585b1a91c11ff12d0fcfce24bda424550/docs/status/current.md).
The [Canic report](../audits/platform-022-canic-wasm-2026-11-05.md) remains the
0.22.1 size evidence; this release makes no new Wasm-size claim.

## Published browser SDK 0.2.0

@tokostudios/ic-platform 0.2.0 is published on npm. Source commit d30e01c and
annotated tag browser-v0.2.0 are pushed to origin/main. The release replaces
generic Cbor exports with typed ingress inspection and explicit development
response/corruption fixtures. SDK Core remains exactly 6.1.1; decoding, hashing
and cryptography stay upstream. Consumer limits, request binding, verification
and effect policy retain their owners.

All 21 browser tests, packed consumers on TypeScript 6.0.3/7.0.2, export/bundle
isolation and Caffeine/Auth singleton checks pass. The final archive also passes
actual Chromium page and module-worker qualification. Release Clippy and fresh
prepublishOnly checks pass. Registry integrity/checksum and the sole SDK Core
dependency match the qualified archive. Exact custody and logs are under
target/browser020-delivery/, including publication-complete.json. Earlier
authentication failures are reconciled by that matching registry observation.

Auth is adopting the published package in its own checkout; supplier proofs do
not establish consumer adoption. Unrelated Rust Cargo.lock edits remain preserved
and uncommitted. Other Rust release groups were not advanced by this npm release.

## Evidence custody

The approved archive/bundles remain in
`recovery-evidence/retired-checkouts/20261104-issue139/`, with checksum and
relocation receipts. Reproducible Runtime 0.18.1 binaries remain under
`target/recovery-evidence/runtime-0181-delivery.LgU8r6/`. The maintainer owns
review by **2026-12-04**. Retention checks enforce budgets without deletion or
extension. Cargo clean can reclaim target evidence; the historical archive
survives. Supplier bundles also remain in `.git/issue-139-suppliers/`.
