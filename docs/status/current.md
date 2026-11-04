# Current handoff — 2026-11-04

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
Archive provenance and remote main/tag resolve to the release commit above.
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

## Current Platform and Runtime deliveries

Platform **0.22.0** remains published at
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

Shared **0.29.3** at `afccb9947d26bbae13d4c4a143d2fd5f0c2c07a4` is installed.
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
preserved in `.git/retired-checkouts/20261104-issue139/ic-platform-contributions.tar.zst`.
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
