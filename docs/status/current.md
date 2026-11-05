# Current handoff — 2026-11-05

## Local Shared 0.30 adoption and hygiene audit

Shared **0.30.0**, exact committed supplier
`4d0961821a31766e3044c08fbe5954ed81213ec9`, is installed through the canonical
exporter. The maintainer authorized deployment of this reviewed maintenance
batch: Runtime 0.19.2 followed by Platform 0.22.2. Types remains 0.18.1; its
changes only tighten lint scopes. Release qualification and publication are
pending, with evidence under `target/platform0222-delivery/`. No sibling source
or provider consumer pin changes are included. See the [audit](../audits/shared-030-practices-2026-11-05.md)
for rule coverage, focused evidence and the remaining native-provider limitation
tracked in [#140](https://github.com/dragginzgame/ic-platform/issues/140).

The first Runtime gate at `95baef5` stopped at Clippy before any version
preparation or upload. Management DTO lint scopes now distinguish private native
schema tests from public Wasm exports; the registry-only schema comparison is
explicitly native. Focused native/Wasm Clippy, three native unit cases and exact
source/mutation guards pass. Failed evidence remains at
`target/release-validation/qualification.B8Ovtd/`.

The second gate at `e9a6419` passed full Clippy and browser checks, then found
consumer fixtures copying Shared helpers without their new companions. Both CI
selection and release recovery now use Shared’s companion-aware fixture copier.
All three groups’ transaction, rollback, custody and delivery/resume cases pass,
as do editor bootstrap and changelog checks. The failed receipt is retained at
`target/release-validation/qualification.hLObj9/`; release qualification will
restart from the committed correction.

The third gate at `0111b94` passed 19/20 stages, including the installed owner
profile. Timers composition exposed an appended dependency landing in the new
LOC metadata table. The renderer now declares Timers in the fixture workspace
catalog and inherits it in the canister dependency table. The unchanged frozen
lock and all installed composition proofs pass at `target/cdk-runtime.wGVVgn/`.
The failed full-gate evidence remains at `qualification.WG3FSK`; a clean-source
full gate is still required before any version preparation or upload.

Platform’s documentation gate now checks every exported Shared Markdown payload
alongside its local document roster. The committed Shared LOC permalink fix and
explicit exporter selection of the missing governance/audit documents close all
three observed broken links; the unnecessary local fleet guide is removed through
the exporter. The actual Make gate passes and rejects a broken exported target
in a disposable regression. Evidence: `target/shared-doc-links-20261105/`.

The approved custody moves preserve the complete historical archive/bundles at
`recovery-evidence/retired-checkouts/20261104-issue139/`, with byte inventories
and relocation receipts at both old/new locations. The old Runtime 0.18.1 driver
executables now live under `target/recovery-evidence/runtime-0181-delivery.LgU8r6/`;
logs/checksums remain at their original locations, with relocation receipts.
Shared’s Git-side admission passes; release/publication also admit the separate
archive budget. Review belongs to the Platform maintainer by 2026-12-04; the
checker never deletes or automatically extends retention.

## Delivered Platform 0.22.1

Platform **0.22.1** is pushed and published from
`fcf540b116e9bf9d511f634b7611a3c0da01dd1c`, tagged `platform-v0.22.1`.
Source/benchmark commit: `55f5115`. Only the facade advanced; Runtime remains
0.19.1 and Types remains 0.18.1.

All twenty local Linux release stages passed on Rust 1.99.1, including both
installed Runtime/Timers profiles and final candidate graph admission. Receipt:
`target/release-validation/qualification.0lxZkr/receipt.json`.
Publication: `target/publication/publish.selection.45kQ6d`.
The registry index, downloaded archive and both locally prepared archives share
SHA-256 `229f2ea285da3b2cfd9b780e50f32c3313be5972c3a5fda70646111dbe794a07`.
Clean archive provenance and the remote annotated tag name the release commit
above. Evidence: `target/platform0221-publication-verification/`,
`target/platform0221-release-with-pocketic.log`, `target/platform0221-publish.log`.
Hosted CI and native macOS results remain separate; none is claimed here.

The first gate stopped at installed Runtime because `POCKET_IC_BIN` was omitted;
its failed evidence remains under `qualification.4DOW53`. No release intent had
yet been saved. A fresh complete gate passed with the documented prepared server
below. The earlier release/resume logs are retained; their failures are not passes.

The [matched Canic Wasm report](../audits/platform-022-canic-wasm-2026-11-05.md)
compares one frozen issuer/receiver source with registry Platform 0.21.0/0.22.0,
Rust 1.99.1 and the identical Canic release/optimizer recipe. Published 0.22 adds
33,175/32,032 total bytes and 32,199/31,544 code bytes respectively, about 1%.
CBOR projection/decoder instantiations dominate; the earlier large Auth reference
increase does not reproduce in these Canic artifacts.

The retained private-adapter correction saves 3,694/3,664 total bytes and 3,337
code bytes each against an unchanged local-source control. It removes the extra
BLS key serialization/reparse and restores packed-map/sequence envelope forms,
with no cryptographic, certificate, delegation, identity or time check removed.
Residual overhead remains. A larger full-envelope decoder-sharing trial was
worse and is reverted. No feature/dependency or Runtime/Types change is required.

Affected Platform Clippy and 15 integration/unit tests pass. A read-only Auth
source capture passes engine Clippy, 197 native tests and 11 live PocketIC cases;
Chromium remains explicitly excluded. Its captured Wire 0.17.1 generated client
was stale and was regenerated only in the diagnostic copy; the original Auth
checkout still needs its own generator refresh. Source inventories, exact sizes,
compatibility limits and evidence paths are in the report. These are pre-release
measurements of the local 0.22.0 candidate; the matching corrected bodies are now
published in 0.22.1. No sibling source or release was changed.

## Delivered Types 0.18.1

Types **0.18.1** is pushed and published from
`2307ec3e88132913b44c2db201b19c98d8fcca04`, tagged `types-v0.18.1`.
Source fix/audit: `dba83f8`. The existing `principal-external` feature explicitly
selects upstream Principal Serde, without Candid, Agent, Runtime, hashing or new
production packages. Canonical identity, conversions and bytes are unchanged.
Types 0.18.0 lacked this standalone external-Serde selection. Owned-only users
already had their required boundary. No other release group advanced.

All twenty local Linux release stages passed on Rust 1.99.1, including Clippy,
Rust tests/docs, browser, recovery, native/Wasm feature and consumer checks, and
installed Runtime/Timers profiles. Final candidate graphs passed. Receipt:
`target/release-validation/qualification.LfiUP1/receipt.json`.
Publication: `target/publication/publish.selection.2WZvBD`.
The registry index, downloaded crate and locally verified archive share SHA-256
`aa881498e10a94f06602424148b166e685170cd9afe0b0f377fb849b47c7bc6f`.
Archive provenance and the annotated tag name the release commit above.
GitHub main contains that release and its subsequent delivery documentation.
Evidence: `target/types0181-publication-verification/`,
`target/types0181-release.log` and `target/types0181-publish.log`.
Hosted CI and native macOS results are separate; none is claimed by this gate.

## Consumer-owned adoption

The [passive contract audit](../audits/passive-contract-boundaries-2026-11-04.md)
records current/proposed paths, compatibility limits and exact migration steps.
Implementation belongs to [Canic #731](https://github.com/dragginzgame/canic/issues/731),
[Blob #115](https://github.com/dragginzgame/ic-blob-storage/issues/115) and
[IC Crates #22](https://github.com/dragginzgame/ic-crates/issues/22).
The Types correction is available; the consumer feature/import migrations remain
unimplemented here. No sibling checkout was edited or released. Earlier read-only
consumer tests retain their actual source/lock identities, not today's independent
supplier updates. Keep genuine Nat/Caffeine/HTTP Candid and IcyDB's BLAKE3 edge.

## Prior Platform ownership cut and current Runtime delivery

Platform **0.22.0** was published at
`b95c9a2e6f4ecfbd25fc2f8466b3ca5826df550f`, tag `platform-v0.22.0`.
Runtime, macros and exact guard **0.19.1** remain published at
`74a5071a7b0fdb6470ceb356d1891510ca8d16b1`, tag `runtime-v0.19.1`.
Their twenty-stage full Linux gates, candidate graphs and archive verification
passed. Receipts: `target/release-validation/qualification.rWavht/receipt.json`
(Platform) and `target/release-validation/qualification.vfRxyF/receipt.json`
(Runtime). Publication attempts: `target/publication/publish.selection.H2GEEB`
and `target/publication/publish.selection.1jdY3S`. Earlier failed preparation and
atomic-push attempts remain retained; no published history was rewritten.

The crypto reversal delegates BLS, certification and canister-signature algorithms
to registry suppliers. Platform retains issuer conversion, DER/key admission and
authenticated signing time. Preserve root/delegated identity-key refusal and
consumer trust/freshness policy. Upstream signature packaging restores transitive
Candid/printer, raw IC0, CBOR, hashing and a lazy root cache; repeated CBOR decoding
remains. No Wasm-size or performance improvement is claimed. Native Agent remains
upstream ic-agent 0.49.3; advisory #50 is unchanged.

That delivery used Shared **0.29.3** at
`afccb9947d26bbae13d4c4a143d2fd5f0c2c07a4`; the local adoption above supersedes it.
Rust remains pinned/minimum 1.99.1. The three-group release host allowlist remains
empty: all releases retain the full gate until fresh mapping qualification on
Linux/macOS. Historical four-group #137 receipts do not activate this mapping.
See [release validation](../platform-release-validation.md).

Installed qualification uses
`POCKET_IC_BIN=$PWD/.tools/ic-testkit-server/16.1.0-x86_64-linux/pocket-ic`.
Testkit admits this PocketIC 16.1 bundle; executable SHA-256 is
`b44e1eccd66e02328146b3e209e3403e3a3428a25d25555aca2f67500e5ae9db`.
Scheduling qualification stays pinned to authorized Runtime 0.19 adoption
`a61f3517`; no Scheduling merge/publication is implied. Fixture Testkit 0.84.1
keeps its registry Platform 0.21 DTO adapter separate from the local facade while
canonical Runtime patches preserve singleton identity.

## Retained contracts and evidence custody

#139 transferred PocketIC's client to Tooling Testkit 0.84.0 and retired Platform's
client/group in 0.21.0. Keep [the migration](../pocket-ic-client.md) and donor source
`41688ce` provenance/licenses/reconstruction guards. Supplier artifacts and Git
bundles remain in `.git/issue-139-suppliers/`. Tooling native CI/publication evidence
is run 41681210184; the prior handoff records its exact scope and limitations.

The retired contribution container, all Git refs/bundles and release evidence are
preserved in `recovery-evidence/retired-checkouts/20261104-issue139/ic-platform-contributions.tar.zst`
following the approved relocation recorded above.
Integrity/full filesystem comparison passed before removal; missing worktree
metadata was preserved before pruning. Earlier delivery logs remain in
`.git/release-state/issue-139-delivery/`, with rejected candidate patch
`.git/release-state/platform021-prepared-before-fixture-fix.patch`.
The [previous handoff](https://github.com/dragginzgame/ic-platform/blob/41213f589fd60850d692dbd6cbd73220467fbbdd/docs/status/current.md)
retains detailed older release, failed-attempt, native-provider and withdrawn-Agent
custody. Use this normal checkout for subsequent work; never discard that evidence.

#138's Types 0.18 Account bytes require matching Wire/browser schemas, fresh RPC
contract IDs and consumer-owned persisted-data handling. Types 0.18.1 changes none
of those obligations. #134/#136's transferred client limits/IDs/bytes remain intact.
