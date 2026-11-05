# Current handoff — 2026-11-05

## Delivered maintenance cohort

Runtime **0.19.2** and Platform **0.22.2** are pushed, tagged and published on
crates.io. Types stays at **0.18.1**; its source edits only tighten lint scopes.
No browser npm package, sibling release or Profiler consumer pin changed.

| Group | Published packages | Release commit / tag |
| --- | --- | --- |
| Runtime | ic-platform-runtime, ic-platform-macros, ic-platform-runtime-guard 0.19.2 | `5e9437fb5cbd6221b5ec7c9edb4ea33164297fce` / `runtime-v0.19.2` |
| Platform | ic-platform 0.22.2 | `53eae551b9d73ae01b4275059b78e03bbac0317a` / `platform-v0.22.2` |

Both complete twenty-stage Linux release gates passed on Rust 1.99.1, including
installed owner and Timers composition. Candidate graphs passed after metadata
preparation. Receipts: `target/release-validation/qualification.R0xwqQ/receipt.json`
(Runtime), `target/release-validation/qualification.BDiYdI/receipt.json` (Platform).
Publication attempts: `target/publication/publish.selection.OuPX7F` and
`target/publication/publish.selection.NKcUG5`. Registry index checksums match
the downloaded archives; their clean Cargo VCS provenance matches the release
commits above. Remote annotated tags were independently checked. Summary:
`target/platform0222-delivery/deployment-verification.json`.

Shared **0.30.0**, committed supplier
`4d0961821a31766e3044c08fbe5954ed81213ec9`, is adopted through the canonical
exporter. Its payload integrity is intact. The [audit](../audits/shared-030-practices-2026-11-05.md)
records rule coverage, remaining native limits and detailed evidence.
The normal documentation gate now covers every exported Shared Markdown payload
and rejects a missing target in an isolated regression. Source/runtime bodies,
external bytes, numeric ranges and cryptographic/freshness checks are unchanged.
No consumer regeneration or new identity owner is introduced.

## Qualification limits and follow-through

- [#140](https://github.com/dragginzgame/ic-platform/issues/140) owns remaining
  native Profiler path-module wiring and paired native acceptance. Its moved
  tests/lint edits are source-only; no new provider pin is qualified here.
- [#50](https://github.com/dragginzgame/ic-platform/issues/50) remains unresolved.
  Runtime’s [hosted advisory job](https://github.com/dragginzgame/ic-platform/actions/runs/42019538519)
  fails on crypto-bigint 0.5.5 / RUSTSEC-2026-0387. No exemption or waiver was added.
- Runtime’s hosted Browser SDK and Installed runtime runs passed. Its general CI
  was cancelled by the subsequent main push. Platform’s release-commit workflows
  were still running at observation; see [CI](https://github.com/dragginzgame/ic-platform/actions/runs/42022979876).
  Local Linux results do not claim native macOS, Chromium or full IC-server acceptance.

Failed gates remain retained: `qualification.B8Ovtd` (management lint scopes),
`qualification.hLObj9` (fixture companion closure), and `qualification.WG3FSK`
(Timers dependency appended into LOC metadata). All are under
`target/release-validation/`; none is counted as a pass. Corrections use scoped
expectations, Shared’s fixture copier and explicit workspace dependency inheritance.
Maintained supplier versions/locks were not re-resolved to hide those failures.

## Evidence custody and retained contracts

The maintainer-approved historical archive and all bundles remain at
`recovery-evidence/retired-checkouts/20261104-issue139/`, with byte inventories,
checksums and relocation receipts at the old/new paths. The two reproducible
Runtime 0.18.1 executables moved to
`target/recovery-evidence/runtime-0181-delivery.LgU8r6/`; original logs/checksums
and relocation receipts remain in Git-side release state. No refs or evidence
were deleted. Platform’s maintainer owns review by **2026-12-04**.
`make check-retained-evidence` and Shared’s Git-side checker enforce their
budgets without automatic deletion or retention extension. Cargo clean removes
target evidence, while the separate historical archive survives.

The [pre-delivery handoff](https://github.com/dragginzgame/ic-platform/blob/53eae551b9d73ae01b4275059b78e03bbac0317a/docs/status/current.md)
preserves earlier release, failure, supplier and retirement details in Git.
Supplier artifacts/bundles also remain in `.git/issue-139-suppliers/`.
#139’s PocketIC transfer, #138’s Types/Account bytes and existing Wire/schema
regeneration obligations retain their [migration contract](../migration.md).
The [matched Canic report](../audits/platform-022-canic-wasm-2026-11-05.md) remains
the 0.22.1 size evidence; this hygiene batch makes no new Wasm-size claim.
