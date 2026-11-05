# Current handoff — 2026-11-05

## Local issue fixes, not released

[#145](https://github.com/dragginzgame/ic-platform/issues/145) restores Testkit's
optional subnet-metrics leaf: CBOR null is `None`, while every field/u128 value,
byte bounds and malformed/trailing refusal remain intact. The breaking return
type is noted for **0.23.0**; package versions remain unchanged. Protocol and
fixture Clippy pass; all 20 selected unit tests pass. Logs:
`target/platform-codec-release/issue145-{clippy,standalone-clippy,tests}.log`.

[#141](https://github.com/dragginzgame/ic-platform/issues/141) delegates runner
evidence interpretation to Shared, retains its complete proof and digest-binds
the release journal. Original-location checks and prepared-source custody remain
distinct. Final three-group plan/recovery fixtures pass (substitute product stages):
`target/release-plan-test.kYWKyr/` and `target/release-plan-test.hb2EKu/`.
No real release, new native/macOS qualification or narrowed gate is claimed.
#142–#144 are closed for the published supplying APIs; sibling adoption remains
separate. #140's paired Profiler/native work and #50's advisory remain open.

## Platform codec delivery

**ic-platform 0.22.3** is pushed, tagged and published on crates.io. Release
commit: `f8fab103c5dfa41c6e32c4c6401bfd216173822d`; tag: `platform-v0.22.3`.
Runtime stays **0.19.2**, Types **0.18.1**; browser/npm and Tooling were not released.
No sibling checkout was edited during this delivery.

Select `default-features = false` with `features = ["certification-cbor"]` for
`certification::cbor` and its `strict` certificate/tree decoder. Native Testkit
packet consumers select `features = ["ic-protocol"]`, which includes the former.
Use `ic-protocol-fixtures` only for native development helpers. The
[migration table](../migration.md#typed-ic-packet-and-fixture-boundary-0223)
records separately owned consumer changes, including Tools 0.83.0 and Testkit
0.87.0. No generic CBOR encoder/decoder or Value is public.

Host's strict field/tree admission and complete-consumption checks are retained;
SDK envelope framing, upstream certificate/transport identities and full u128
metrics values are preserved. Consumers retain all budgets, authentication,
trust roots, delegation/freshness checks, request binding, transport, retries
and PocketIC orchestration. No cryptographic implementation changed.

## Delivery evidence

The complete twenty-stage Linux release gate passed on Rust **1.99.1** for source
`766ba97bfa37222b8cf96fe7ba1a430cf417d099`. Receipt:
`target/release-validation/qualification.XEPzDn/receipt.json`. This includes
Clippy, Rust tests, feature closures, Wasm, consumer admission and installed
Runtime/Timers composition (`target/cdk-runtime.v68quN`). Candidate metadata
and clean source were admitted by the standard release/publication workflow.

The registry-only consumer passes both certificate-only and native protocol
profiles without source overrides. Certificate-only excludes Candid, Agent and
Runtime; protocol selects transport 0.49.3/Candid but excludes Agent and Runtime.
The published archive's clean VCS provenance matches the release commit; its
SHA256 matches the crates.io index:
`d73c010faf8ba71b380b35fed865c5a30e2e9ed56d688fb7817d7df978e396fc`.
Independent remote main/tag verification also passed. Summary and logs:
`target/platform-codec-release/deployment-verification.json`; publication:
`target/publication/publish.selection.wnDxDu`.

Hosted [CI](https://github.com/dragginzgame/ic-platform/actions/runs/42038858650)
and [installed runtime](https://github.com/dragginzgame/ic-platform/actions/runs/42038858617)
were running at observation. Native macOS Intel and ARM jobs are asynchronous;
there is no new local macOS evidence. Linux execution does not claim macOS,
Chromium or full native-provider acceptance. Failed local attempts are retained
under `qualification.upw92f` and `qualification.3Fb3um`; neither is a pass.

## Retained boundaries and follow-through

Shared **0.30.0**, supplier `4d0961821a31766e3044c08fbe5954ed81213ec9`, remains
adopted with intact executable snapshots. The [audit](../audits/shared-030-practices-2026-11-05.md)
records rule coverage and native limits. [#140](https://github.com/dragginzgame/ic-platform/issues/140)
owns remaining native Profiler path-module wiring and paired acceptance; no new
provider pin is qualified here. [#50](https://github.com/dragginzgame/ic-platform/issues/50)
remains the crypto-bigint 0.5.5 / RUSTSEC-2026-0387 advisory; no waiver was added.
Prior releases, supplier details and failures remain in the
[preceding handoff](https://github.com/dragginzgame/ic-platform/blob/f8fab103c5dfa41c6e32c4c6401bfd216173822d/docs/status/current.md).
The [Canic report](../audits/platform-022-canic-wasm-2026-11-05.md) remains the
0.22.1 size evidence; this codec release makes no new Wasm-size claim.

Unrelated browser drafts were preserved under
`refs/retained/platform-codec-0223-unrelated`, restored without staged Rust
replay and checked against their original SHA256 inventory. Browser source,
package version and pending docs remain uncommitted; ignored dist was rebuilt.
No npm release or publication was performed.

## Evidence custody

The approved archive and bundles remain in
`recovery-evidence/retired-checkouts/20261104-issue139/`; checksum inventories
and relocation receipts preserve custody. Runtime 0.18.1 reproducible binaries
remain at `target/recovery-evidence/runtime-0181-delivery.LgU8r6/`. The maintainer
owns review by **2026-12-04**. Retention checks enforce budgets without deletion
or extension. Cargo clean can remove target evidence; the separate historical
archive survives. Supplier bundles also remain in `.git/issue-139-suppliers/`.
#139's transfer and #138's Types/Account bytes retain their [contract](../migration.md).
