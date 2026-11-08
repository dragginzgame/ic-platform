# Current handoff — 2026-11-08

## Maintenance batch and Git delivery

The maintainer authorized delivery to `origin/main` in this checkout. The reviewed
batch contains the ten simplifications, Shared 0.31.10 adoption and cargo-machete
annotations. Public Rust sources and dependency/feature/build declarations are
unchanged. Only private cargo-machete metadata differs in two package manifests;
package inventories confirm the tooling paths are outside their published code.
Under Shared's publication-impact rule this is Git-only maintenance: no version
bump, tag or registry upload. Platform stays 0.26.0, Runtime/macros/guard 0.20.1,
Types 0.18.2 and browser 0.2.0. The 0.26.1 tooling note remains pending.

The deferred Cargo.lock draft is excluded from the Git payload and retains its
exact bytes, as do both qualification locks. Browser and native Profiler source
are unchanged; siblings remain read-only. Comparison inputs and the reviewed
86-path delivery roster are under `target/deployment-maintenance-20261108/`.

- Retired 27 unadopted Candid experiment files, including two private locks.
  [Historical reproduction](../../scripts/benchmarks/0.9/README.md) retains the
  immutable source/license/evidence reference; stale policy/format inputs are gone.
- Shared benchmark-driver preparation, admission-tool stubs and working-tree
  fixture copying. Formatting retains opaque gitlink and staged-index isolation.
- Declared 38 fixture method identities once and moved 31 unchanged handler
  bodies into scenario modules. Driver assertion order/lifecycle remains intact.
- Selected explicit runtime profiles once for graph/build/execution; runtime
  graph admission is one pass. Release-lock primaries are observed once per run,
  with duplicate/missing-owner and rollback/retry refusals retained.
- Separated the [current migration contract](../migration.md) from
  [historical investigations](../history/migrations-through-0.26.md), preserving
  previous heading anchors and correcting stale Runtime ownership/version claims.

Focused Linux / Rust 1.99.1 evidence is in `target/complexity-20261108/`:
strict Clippy/build for both benchmark drivers and all eight owner/Timers Wasm
profiles; six native driver tests per topology; all eight installed profiles
pass cancellation, await, callback-trap, upgrade, cycles, stable rollback and
certificate checks. Also passed: both admission runners (including concurrent
artifact custody), graph refusals, all three groups' preparation/rollback and
Runtime resume in disposable Git repositories, lock recovery, formatting/hook
isolation, documentation mutation/links, boundary/inventory and ShellCheck.
No full CI, release gate, macOS or native Profiler acceptance is claimed.

Owner fixture: `target/cdk-runtime.w1LFSz/`; Timers fixture:
`target/cdk-runtime.cNTf9d/`. Their retained artifacts/graphs/logs bind the actual
observed source; they are focused fixture evidence, not release receipts.

## Existing local adoption and dependency audit

Shared **0.31.10**, `c5285791991c731d266fb5c85dc3ee876d29f6b9`, is adopted
through the canonical exporter (147 verified payloads), including the pinned
download-cache action and CI installer. Six hosted setup owners restore before
installation and save afterward; browser's npm-only setup is unchanged. One
independent Linux CI job installs pinned Gitleaks and scans full history with
redaction, including documentation changes. Release/publication paths add no scan.

Actionlint, actual CI selection/cache-order checks, formatting/hook isolation,
documentation links, crate/module CLOC counts and unknown-selector refusal pass.
The dirty root/qualification locks are unchanged. Evidence is in
`target/adoption-shared03110-20261108/`; earlier adoption evidence remains in
`target/adoption-shared0319-20261108/`. Hosted cache/scan execution is asynchronous
after delivery; no live Gitleaks, hosted/native CI or new release is claimed.

Cargo-machete passes all five workspace crates with documented manifest-only
exclusions for the ECDSA bigint security floor and executor links guard.
Their TOML metadata records only those developer-tool exclusions. Evidence:
`target/dependency-audit-20261108/`.

## Published baseline and evidence

Platform **0.26.0** and Runtime/macros/guard **0.20.1** completed standard frozen
release and crates.io publication. The last published Platform source is
`41a128751528f8ccd3c926cff550dd8b2ea62926`, tag `platform-v0.26.0`.
Runtime release commit is `206cc5990c330cafbe15f18fc7fa5b1b6a31093b`, tag
`runtime-v0.20.1`. Types stays **0.18.2**, browser **0.2.0**.
Official index/archive checks established non-yanked entries, exact checksums
and clean VCS source/path identity for all four published crates.

Implementation commit `6bfa1ebad43fc5859b389224fcf1b3264b0e5667` removed the BLAKE3
facade after caller migration and consolidated codecs, macro parsing and test
plumbing. Cryptographic algorithms/checks and external bytes were preserved.
See the [Platform ledger](../../releases/platform/CHANGELOG.md) and
[Runtime ledger](../../releases/runtime/CHANGELOG.md).

Both Linux x86-64 / Rust 1.99.1 release gates passed **20/20** stages with no
omissions, including installed owner/Timers fixtures and final archive checks.
Runtime batch `batch-_leIreCI` qualified implementation commit `6bfa1eb`;
Platform batch `batch-C5BoFKA5` qualified Runtime release commit `206cc59`.
Receipts remain respectively at:

- `target/release-frozen/batch-_leIreCI/target/release-validation/qualification.xynaOV/receipt.json`
- `target/release-frozen/batch-C5BoFKA5/target/release-validation/qualification.aaSiWQ/receipt.json`

Both schema-2 intents are complete at index 1. Journals/Git objects remain in
`.git/release-state/frozen/BATCH/`; neither batch was retired. Deployment logs,
registry/archive/VCS verification, reconciliation and the complete preceding
handoff are in `target/deployment-platform026-runtime0201-20261108/`.

Hosted evidence is separate: the last observation of final commit `41a1287`
found CI 42468591030, Browser 42468591250 and Wasm size 42468591029 in progress.
Installed runtime 42468591067 and advisories 42468591066 failed host-tool setup
with HTTP 504 before tests/auditing; each was retried once, with outcomes left
asynchronous. Earlier run 42467034589 likewise failed setup before product tests.
No new advisory result/waiver, macOS pass or Profiler/Bazel acceptance is claimed.
Do not repeatedly rerun or wait solely to hold deployment open.

## Deferred draft and recovery custody

Cargo.lock remains deliberately dirty. Its independent dependency draft was
excluded from deployment, then restored as external-only changes over released
owner versions. All **391 external supplier records** match the retained draft.
Current lock SHA-256:
`d47ee5c95cd219e49a36e45a890596fdc810a39ad5f87f204080d2ee28f87e00`.
Preservation ref `retained/platform026-runtime0201-lock-draft` points at
`24c865191d14c628b86e2cc7307988827c3f1e58`. Never restore that whole older lock
over the published owner metadata. Earlier draft refs remain preserved.

Preserve historical failed/completed batches, including `batch-ZNfZ3p9z`,
`batch-2ylQaRca` and interrupted 0.23.1 state. Their details remain in the
[historical handoff](https://github.com/dragginzgame/ic-platform/blob/924828e32c7a5d5e71d656ea373a6825c90314ff/docs/status/current.md)
and retained journals; never reinterpret old receipts under new tooling.

Historical archives/bundles/receipts stay at
`recovery-evidence/retired-checkouts/20261104-issue139/`, with maintainer review
by **2026-12-04**. Retention refuses excess/expiry without deletion. No cleanup,
mtime renewal or retirement is authorized. Previously removed target evidence
and Runtime 0.18.1 binaries must not be claimed present.

[Issue #50](https://github.com/dragginzgame/ic-platform/issues/50) remains explicitly
deferred for native Agent's upstream crypto-bigint advisory; no waiver or crypto
rewrite was added. Other ownership/provenance constraints remain in
[AGENTS.md](../../AGENTS.md) and the [migration contract](../migration.md).
