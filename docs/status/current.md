# Current handoff — 2026-11-14

## Published state

Frozen batch `batch-YOdNMnM9` completed GitHub delivery/crates.io publication and
was reconciled into main. Remote/local main is `f04f105e28c201ae4be9d8e35c90b7ba23fe48de`.
All five independently downloaded registry archives have clean VCS metadata matching
their annotated tags; see `target/evidence/published-0424/verified.tsv` and
`remote-refs.tsv` in that directory.

| Group | Version | Commit | Tag | Qualification / runner |
| --- | --- | --- | --- | --- |
| Types | 0.20.2 | `4550a86c52e40207fd0ca3fa8f00f6ac596c1ae5` | `types-v0.20.2` | `sPkEsH` / `d0xa6D` |
| Runtime/macros/guard | 0.22.2 | `6602573408052413eae68b3ae12b63ca58841abe` | `runtime-v0.22.2` | `qINNJW` / `hEiop6` |
| Platform | 0.32.2 | `f04f105e28c201ae4be9d8e35c90b7ba23fe48de` | `platform-v0.32.2` | `oXEuJG` / `kODK78` |

All 57 required parent stages passed (19/group), including fresh real Cargo,
consumer, feature, Wasm and installed Runtime checks. Only the qualified policy
leaf reused its original Types execution for Runtime/Platform. Release-policy
stages took 404/276/275 seconds. Final archive/publication admission passed.
Evidence: `target/release-frozen/batch-YOdNMnM9/target/{release-validation,publication}/`;
log `target/deploy-toko0424-corrected.log`; durable state remains under
`.git/release-state/frozen/batch-YOdNMnM9/`. Reconciliation is recorded in
`target/deploy-toko0424-reconcile.log`. Browser stays 0.2.0; no npm or sibling
publication occurred. This operational handoff remains local after delivery so
it does not supersede/cancel the final release commit's running full hosted CI.

## Current selections and boundaries

Both workspaces now select Toko **0.43.0**, checksum
`060d3507b66eb290ce12bc2572098f7e7cb1e12e9d6697a9b1a5d1552fcb2588`, producer
`bd24a0b7bc95b20a849b16f63d77f6551ff48ab4`; the receipt admits 100 files.
The published batch above used 0.42.4. Deployment of this reviewed adoption is
authorized for Types 0.20.3, Runtime/macros/guard 0.22.3 and Platform 0.32.3,
in supplier order through the standard full-gate frozen workflow; npm is excluded.
The 0.43 update adds the checkout-selected `toko-cloc` PATH launcher and retires
`make cloc` / `CLOC_*` settings. Platform's LOC guide now uses the launcher and
correctly describes the default product-plus-qualification report. The existing
PATH launcher exactly matches the released payload; no global rewrite was needed.
Recorded entrypoints, compiler/tool pins and product release groups are unchanged.
Both locks changed only Toko records, preserving qualification Testkit 0.104.0,
Platform 0.32.2 and cc/Host drafts. Authenticated preview/update, tool/snapshot
admission, pins/inventory, package listing, formatting and docs passed. Real LOC
calls passed for the default six-package report, qualification-only report and
Runtime executor drill-down (schema 2, no unclassified source). Evidence:
`/tmp/platform-toko0430-*`. No product compilation or full gate was repeated.
The retained 0.42.8 hook changes resolve paths independently of ambient `CDPATH`.
Relative and repeated hook installation with
`CDPATH=.` passed; the disposable Platform fixture also passed staged-only
formatting, unrelated-edit preservation and formatter failure admission under
that environment. Logs: `/tmp/platform-toko0428-*.log`. No commit or deployment
was performed for these adoptions; release/publication flow is unchanged.
The exporter retired the former helper paths; Make/scripts/catalogs/actions/hooks
use `vendor/toko-tooling/`. Root baseline/license/contribution paths remain
canonical, release/publication entrypoints explicit, and validation runners bind
the consumer root. No product Rust implementation or cryptographic/protocol check
changed. The three groups and complete local gate remain unchanged.

Qualification now selects exact registry BLS baseline **Platform 0.32.2**, with
Testkit **0.104.0** resolving compatible registry Platform 0.32.2 / Types 0.20.2.
Local candidate and Wasm-only BLS/IC0 stay separate from the native driver.
The Testkit update changed only its qualification lock record; Host 0.77.2 was
already selected. Root Cargo.lock and its Testkit 0.102.0 selection are unchanged.
Native driver and both Wasm Clippy/feature closures passed; Testkit/PocketIC/Host
remain outside both Wasm graphs. Installed PocketIC 16.1.0 passed all 56 driver
observations, including refusal/recovery cases. Both arms measured 246,075 raw
bytes (238,906 code) and 237,381 optimized bytes (230,205 code). Evidence:
`target/evidence/bls-wrapper.W24FwY/`; the original sandbox bind failure is
retained separately from successful `driver.retry.log` / `instructions.retry.tsv`.
Source checksums, snapshot admission, pins/inventory, formatting and docs passed.
Logs: `/tmp/platform-qual-testkit0104-*.log`. No macOS, full gate or deployment
was performed for this qualification-only update.
The Runtime fixture retains registry Platform 0.32.0 /
Types 0.20.0; no forced supplier cohort is required. Optional Wire/editor refreshes
remain deferred. Rust minimum/development/release is **1.100.0**; scoped FFI/custody
expectations and Cargo warning denial remain in force.

## Hosted CI

For final release commit `f04f105`, Browser SDK and Wasm size regression passed.
[CI](https://github.com/dragginzgame/ic-platform/actions/runs/43470097056),
[Installed runtime](https://github.com/dragginzgame/ic-platform/actions/runs/43470097298)
and dependency-advisory runs were still in progress at the last asynchronous
observation (`target/evidence/published-0424/hosted.json`). No fresh native macOS
acceptance is claimed; #157 remains open. #50 retains its approved workspace-only
advisory disposition, with no global ignore or remediation claim.

## Deferred work and failed attempts

The unrelated normal cc 1.11.2 / qualification Host 0.77.1 drafts reappeared after
capture. They exactly matched the saved drafts, never entered the frozen source,
and were restored after clean fast-forward reconciliation. The cc draft remains;
qualification Host subsequently advanced to 0.77.2 and satisfies Testkit 0.104.0.
Before the new capture, unrelated cc 1.11.2, mio 1.2.5 and bumpalo 3.20.4 drafts
were excluded from both locks and retained for restoration after reconciliation:
`recovery-evidence/deferred-lock-drafts-20261114-toko0430.patch`, SHA-256
`2c2fc8e7b9f174b238034b7a4ba1936751e06e1af6e7a49a5250d44e797ad4c4`.
Cargo-generated selected records preserve all other supplier edges; both locked
metadata graphs admit the release selection. Do not lose or silently publish drafts.
Restoration patch: `recovery-evidence/deferred-lock-drafts-20261113-toko0424.patch`,
SHA-256 `4128ef00d32e9b059d0cfaed3098b3b60013c5df352dc082501c8ccd4803df63`.
Published locks retain cc 1.11.1 / Host 0.77.0. Qualification registry Types had
already advanced to 0.20.2 during the completed release metadata transaction;
the current adoption preserves it.

First attempt `batch-2eRuqW03`, source `9cd572d`, stopped at Types `check-browser`
before any delivery/upload: the packed-consumer validator retained a relative
import of the relocated SDK checker. Fix `f62d2e9` passed the focused browser
proof and the subsequent complete release gate. Preserve the failed candidate,
original stage evidence and `target/deploy-toko0424.log`; it must not be resumed
or relabeled as the successful batch. No history/tags or proof origins were rewritten.

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

Keep prior completed batches `batch-q_67SewF`, `batch-CFcV6khZ`, failed pre-upload
`batch-VCzP0I1E` and older custody from the
[preceding handoff](https://github.com/dragginzgame/ic-platform/blob/8c888cdf6a1d260a5eb309f078f9b6062e2fccd5/docs/status/current.md).
The historical verification-size finding against `fc55ae1` (+135 total/+133 code)
remains unresolved; newer zero-delta comparisons use other baselines. See
[the audit](../benchmarks/platform-031-audit.md#post-release-cleanup).
Retired-checkout recovery review is due 2026-12-04. Preserve the deferred signature
comparison and its custody recorded in that preceding handoff.
