# Current handoff — 2026-11-07

## Local audit improvements awaiting delivery

The maintainer requested deployment of the reviewed audit improvements and Shared
0.31.1 repair. Deliver Runtime/macros/guard **0.19.5**, then Platform **0.23.6**,
through one Shared patch/publication batch to `origin/main` and crates.io. Types
**0.18.2** and browser **0.2.0** are unaffected. Current package metadata is still
Platform 0.23.5 / Runtime 0.19.4 until the standard runner prepares each group.
Retain the independent lock draft durably, exclude it from captured source and
restore it over delivered owner versions only after batch reconciliation. Full
local release gates are required; development checks below are not their receipt.

Release lock/source-roster readers now fail closed on failed, partial or empty
observations. Adjacent release fields share one admitted plan per command, with
fresh later admission; preparation fixtures parse unchanged catalogs once.
Publication carries its admitted package selection to Clippy, preserving the full
release gate. Relevant main pushes trigger the existing matched Wasm comparison.
The earlier three-group real-editor rollback/retry/Git-resume fixture passed in
435 seconds with product gates substituted; its logs were subsequently cleaned.

Executor interior removal retains stable Vec order while locating each task once
and bulk-shifting its tail. The unused private null-context entry is removed;
null method IDs, migratory wake routing and generation checks remain. Five System
API readers now share one private all-or-trap spare-capacity copy; length becomes
visible only after full initialization. UTF-8 admission, absence and traps remain.
Candid admission stores child references in one buffer, preserving traversal,
unused/skipped-value inspection, quotas and the fixed 16-level ceiling. Crypto,
public signatures, dependencies and canonical identities are unchanged.

## Current focused evidence

Fresh evidence is under `target/evidence/platform-audit-followup/`. Its baseline
reconstructs the starting source while retaining the earlier executor removal
improvement (verified source SHA-256 `53e3f13240ddad4a250da9f704e80e460b6e04ca18ab347a437384d66fbdc615`).
Both arms use identical probes, compiler 1.99.1, external locks and `3-fat-1`.
The zero-growth comparison passes for total uncompressed and code sections:

| Optimized probe | Total bytes before → after | Code bytes before → after |
| --- | ---: | ---: |
| Runtime | 38,741 → 38,713 | 32,776 → 32,748 |
| System readers | 29,005 → 28,920 | 22,605 → 22,520 |
| External Candid | 609,093 → 608,721 | 556,109 → 555,707 |
| Candid admission | 29,636 → 29,436 | 26,723 → 26,493 |

Installed measurement saves 1,227 instructions per 1,111-byte certificate read
and 249 per 133-byte root-key read. Wide 1,024-type admission falls from 1,241,774
to 695,195 instructions; simple recursive refusals cost 161 extra instructions.
Exercised acceptance/error outcomes match; the driver compares root-key bytes
against Testkit's explicitly selected NNS key. These are probes, not fleet gains.

Runtime and Candid focused Clippy pass; each selected native library passes 21
tests with one benchmark ignored. Source/license/ABI/callback guards and failed,
partial, empty and malformed roster refusal checks pass. Installed owner profiles
pass raw/prepared calls with Wire/external Candid, including awaits, callback traps,
cancellation, upgrades, stable rollback, certificates and nesting admission.
Matching Timers composition profiles pass the same four installed selections.
Full CI, macOS and native Profiler qualification are outside this task.
Documentation/format checks pass.

The independent Cargo.lock draft (Candid/derive 0.10.40, cc 1.11.1 and SmallVec
1.16.5) is unchanged, SHA-256
`8fa03623313c4866557072e88365b73c6ee05488b1a06417898ddd55881af070`.
Keep it separate from delivered qualification; never restore an old whole lock
onto newer owner versions. The pinned metadata editor remains ic-build 0.73.0.

## Cleanup and evidence custody

External cleanup removed the complete `target/` during this task, including old
release receipts, first comparison artifacts and reproducible Runtime 0.18.1
binaries. The maintainer confirmed cleanup stopped; the affected current evidence
above was regenerated afterward. Earlier dated successes remain observations,
not available raw receipts. No evidence was deleted by this session. Cargo cleanup
can reclaim target state; preserve active receipts/logs while recovery needs them.

The completed Platform batch's durable state still exists at
`.git/release-state/frozen/batch-hEX13lnF`; it has not been retired. Earlier retained
refs and the interrupted unpublished 0.23.1 journal remain untouched. Approved
historical archives/bundles and custody receipts remain outside target at
`recovery-evidence/retired-checkouts/20261104-issue139/`; maintainer review remains
due **2026-12-04**. Retention refuses expiry/excess without automatic cleanup.

## Baseline, deliveries and host limits

Shared **0.31.1** at `5efbda2ab81ad84e30f490f3137d8ef083c303d2` is adopted through
the canonical exporter. VERSION/changelog say 0.31.1; the commit title says 0.31.2.
All 141 payloads were verified. Its Bash 3.2 checksum-parser repair and Platform's
portable batch-qualifier digest calls remain local and unpublished. Supplier
macOS ARM/Intel setup/native reuse passed, but its broader portable suite failed
at `test-release-editor-setup.sh`; see [the supplier run](https://github.com/dragginzgame/shared-tooling/actions/runs/42274707843).
No fresh Platform macOS acceptance is claimed. #50 awaits upstream and is excluded.

| Group | Published version / tag | Release commit |
| --- | --- | --- |
| Platform | 0.23.5 / platform-v0.23.5 | `64a52fcf79749f30bcc7715a0f6b0c95bcae49ec` |
| Runtime/macros/guard | 0.19.4 / runtime-v0.19.4 | `8ec8a395a54998bca3e995ea1a78c07e9977eff0` |
| Types | 0.18.2 / types-v0.18.2 | `5eeb016469b15de9bdf9f4d2092a8dbff8ea4d21` |

These were verified on crates.io and GitHub with archive/checksum/VCS/tag proofs.
Each latest Platform/Runtime delivery passed all 20 local release stages before
cleanup. Hosted Linux/runtime/browser passed for Platform 0.23.5; both macOS jobs
failed at the old checksum parser before validation; see [the delivered CI run](https://github.com/dragginzgame/ic-platform/actions/runs/42266799710).
Runtime 0.19.4 explicitly supersedes the earlier stable64_read_uninit exclusion.
Storage and Tooling work belong to their other sessions; no sibling is edited here.

Future authorized delivery uses the [batch lifecycle](../platform-release-validation.md#commands-and-effects)
and affected groups. The native-host reuse allowlist remains empty: real releases
retain full gates. Root evidence does not qualify the independent native Profiler
provider; its [paired Linux evidence](../audits/native-profiler-140-2026-11-05.md)
remains historical, with macOS/Bazel unqualified.
