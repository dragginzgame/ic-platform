# Current handoff — 2026-11-13

## Published state

Frozen batch `batch-RnWOLldS` completed GitHub delivery/crates.io publication in
supplier order and was reconciled into main. All five downloaded registry packages
have clean VCS metadata matching these annotated tags:

| Group | Version | Commit | Tag | Qualification / runner |
| --- | --- | --- | --- | --- |
| Types | 0.20.1 | `603392624623011845e9a7879141628ada947ecb` | `types-v0.20.1` | `1R7mPo` / `efXUgH` |
| Runtime/macros/guard | 0.22.1 | `714e3324b717e8afac83b5376992dde6618674ce` | `runtime-v0.22.1` | `hOpsif` / `a0u799` |
| Platform | 0.32.1 | `8c888cdf6a1d260a5eb309f078f9b6062e2fccd5` | `platform-v0.32.1` | `aNv8x9` / `soKDDb` |

Source `db93db7` passed all 19 local stages per group (57 parent dispatches), plus
fresh archive/publication admission. The qualified policy-fixture leaf executed
for Types and reused its sealed execution for Runtime/Platform; real metadata,
Cargo, consumer and installed checks ran freshly. The release-policy stages took
414/282/274 seconds respectively. No collector retry or broader test reuse occurred.

Evidence: `target/release-frozen/batch-RnWOLldS/target/{release-validation,publication}/`;
log `target/deploy-041-rust1100-corrected.log`. Durable intent/Git/journals remain in
`.git/release-state/frozen/batch-RnWOLldS/`. Independent registry evidence is
`target/evidence/published-041-{registry.log,verified.tsv}`. Browser stays 0.2.0;
no sibling or npm publication occurred.

## Current selections and boundaries

Deployment is authorized for Types **0.20.2**, Runtime/macros/guard **0.22.2** and
Platform **0.32.2**, in supplier order through the frozen workflow. Browser and
siblings remain outside scope. The first batch, `batch-2eRuqW03`, source
`9cd572d2948e41c1e32847abad77e3663002f5da`, stopped before any delivery/upload:
Types Clippy and earlier stages passed, but `check-browser` retained an old
relative helper import. Preserve the complete failed candidate and
`target/deploy-toko0424.log`; partial-stage resume is unsupported. The corrected
browser caller passed all 23 tests, packed Node/TypeScript consumers, export/
bundle isolation and Caffeine compatibility at `target/browser-consumer.jQXReX/`.
All six static relative script imports resolve. Start a new batch from corrected
committed source, with all 19 local stages per group; do not reinterpret old proof.

Both workspaces select Toko **0.42.4**, checksum
`697efc1801f8c8c3f563364984efff1b941e84eef2cfaade40aece5e09bced2b`, producer
`0ac9269cfb3084826df233bfbb187aa57093b5c2`; the receipt admits 99 files.
Helpers, Make fragments, catalogs/actions and hooks now live under
`vendor/toko-tooling/`. Root baseline/license/contribution paths remain canonical.
Explicit release/publication entrypoints, consumer-root runner binding, the three
groups and full stage roster are preserved. Tool setup reuses admitted binaries.
Snapshot, pins/inventory, formatting/docs and release-routing checks pass; evidence
is under `/tmp/platform-toko0424-*.log` and
`target/platform-release-routing.kHdOMf/`. Earlier policy receipt/reuse and group
metadata fixtures remain at `target/release-plan-test.lJL9rn/` and
`target/platform-group-preparation.sGVQcE/`; they use substituted product stages.

Qualification adopts exact registry BLS baseline **Platform 0.32.1**; Testkit
0.102.0 resolves compatible registry Platform 0.32.1 / Types 0.20.1 there. Local
candidate and Wasm-only BLS/IC0 backends stay distinct from its native driver.
Baseline/native Clippy and both Wasm closures passed; no new instruction/size
measurement is claimed. The maintained Runtime fixture still uses registry
Platform 0.32.0 / Types 0.20.0. Optional Wire 0.21.5 and IC Build editor 0.84.0
refreshes remain deferred, not release blockers. No forced supplier cohort is
required. Compiler minimum/development/release is **1.100.0**; Cargo warning denial,
scoped ownership/FFI expectations and all cryptographic/protocol checks remain.

## Runtime cleanup and qualification

Private context/environment guards use Rust 1.100 `DropGuard`. Context clearing
precedes task destruction; the callback reference transfers immediately after
successful perform. Independent Weak lifetime observations remain in native tests.
New regressions cover normal/native-unwind destructor re-entry and callback
custody after post-submission unwinding. Macro `trim_prefix` preserves export names.
No cryptographic/certificate/delegation/identity-key/witness/freshness check changed.

Matched before/after sources at `target/evidence/rust1100-guards.odTaT8/` use the
same compiler, dependencies, probes and optimization: Runtime raw total/code
45,384/39,082 bytes; optimized 41,338/35,036, all deltas zero. The four installed
cleanup profiles passed at `target/cdk-runtime.wjk3Ao/`; release gates repeated
installed qualification at each selected source/metadata boundary. Logs:
`/tmp/platform-rust1100-guards-*.log`. `Vec::from_fn` and custom allocators remain
unadopted; the former's isolated audit probe produced identical Wasm.

## Hosted CI

Runtime commit `714e332` passed hosted Wasm-size, browser and advisory jobs.
Its installed-runtime job failed before simulator startup because the workflow
still read Testkit from the retired dependency-catalog location. Failure evidence:
`target/evidence/hosted-runtime-43436115086.log`. The post-release Git-only repair
uses the shared fixture reader, exact CLI version and Cargo's packaged lock default.
Actionlint and the actual workflow body with substituted install/server commands
passed version/compiler selection, server handoff and install-failure refusal:
`target/runtime-workflow.hn80rW/`. This is wiring evidence, not hosted IC acceptance.
Hosted jobs for the repair run asynchronously; no fresh macOS acceptance is claimed.
#157 remains open for native-host acceptance. #50 retains only its approved
workspace advisory disposition; no global ignore or security claim was added.

## Recovery custody

Pre-upload `batch-oC0UDdbQ` stopped at `check-consumer` on the test-only Agent
fixture's unused normal dependency. Source `c0d856e`, all successful stage logs
and original paths remain retained; nothing was delivered by that failed attempt.
The correction was qualified before a new batch/full gate. Do not resume obsolete
source, edit old receipts, replace tags or replay dispatched uploads.

The unrelated root/qualification `cc` 1.11.2 draft was excluded from deployment,
preserved and restored after reconciliation. Its patch is
`recovery-evidence/deferred-cc-1.11.2-20261113.patch`, SHA-256
`485b2bdafb36a21f9335b2a3e91fb1d32a839c622ff629abfe92a5831f5f21e2`.
Deployment excludes the unrelated cc 1.11.2 and qualification Host 0.77.1 drafts.
Their exact restoration patch is
`recovery-evidence/deferred-lock-drafts-20261113-toko0424.patch`, SHA-256
`4128ef00d32e9b059d0cfaed3098b3b60013c5df352dc082501c8ccd4803df63`.
Apply it only after successful batch reconciliation. Release locks retain cc
1.11.1 / Host 0.77.0; registry Types 0.20.1 is required by the selected Platform
0.32.1 qualification adoption and stays included.

Keep prior completed batches `batch-q_67SewF`, `batch-CFcV6khZ`, failed pre-upload
`batch-VCzP0I1E` and older custody from the
[preceding handoff](https://github.com/dragginzgame/ic-platform/blob/8c888cdf6a1d260a5eb309f078f9b6062e2fccd5/docs/status/current.md).
The historical verification-size finding against `fc55ae1` (+135 total/+133 code)
remains unresolved; newer zero-delta comparisons use other baselines. See
[the audit](../benchmarks/platform-031-audit.md#post-release-cleanup).
Retired-checkout recovery review is due 2026-12-04. Preserve the deferred signature
comparison and its custody recorded in that preceding handoff.
