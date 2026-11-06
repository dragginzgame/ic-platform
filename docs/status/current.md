# Current handoff — 2026-11-06

## Pending audit improvements

The maintainer selected audit items 2–10, explicitly excluding #50. The working
tree adds CBOR fuzz coverage, a shared Rust/browser ingress corpus, Wasm regression
comparison/CI, and a stable-order executor tail-removal fast path. Pending notes
are Platform 0.23.3 and Runtime 0.19.3; package versions are unchanged.
The unrelated Candid lock draft is preserved. No sibling edits or releases occurred.

Focused Clippy/tests and installed Runtime/Timers qualification pass. Final-source
fuzzing passed 420,463 executions. Reverse cancellation of 8,192 tasks improved
from 7.55 ms to 0.14 ms in a native benchmark; the optimised Runtime probe grows
58 bytes in both module and code section. These are not application IC timings.
Two decoder allocation prototypes were reverted after adding roughly 9 KB each;
Candid traversal was measured but remains unchanged. See the
[audit evidence and limitations](../audits/platform-improvements-2026-11-06.md).
The maintainer authorized this batch's scoped commit/push and hosted qualification,
excluding the lock draft and package publication. Release-group activation awaits
fresh committed-input Linux/macOS qualification;
native Profiler macOS/Bazel qualification remains incomplete. No host allowance
was enabled. Hosted qualification is authorized and tracked separately from the
passing local evidence; starting a run does not establish acceptance.

## Published Platform 0.23.2

**ic-platform 0.23.2** is published on crates.io and GitHub at commit
`81b3f05f53ed113d02a671d47d13f1a36222e845`, annotated tag `platform-v0.23.2`.
Registry SHA-256 matches the downloaded archive:
`8e18706c6bac24e9d0bc0afa3568cafa14c1cf20901780c8d27d40cd4b38a0af`.
The archive is non-yanked and records that exact clean VCS source.
Runtime remains **0.19.2**, Types **0.18.1**, browser SDK **0.2.0**.
Shared **0.30.5**, supplier `be3771031ebfb340583b238dee5446be65929c62`, is adopted.

#146 is closed: native/Wasm `management_canister_types::CreateCanisterArgs`
and `candid::protocols::management::create_canister` preserve the pinned 0.11.0
schema, bytes, settings identity and complete numeric ranges. The installed Wasm
proof also exercises request encoding. Select `management-canister-types` for
the DTO and add `candid` for the codec. Canic keeps transport/cycles/recovery;
its adoption remains separate. See the [migration](../migration.md#management-create_canister-request-0232).

All **20 Linux release stages passed** on Rust 1.99.1 against committed source
`9972e5f`, including all-feature Clippy/tests/docs, consumer and feature isolation,
and installed Runtime/Timers profiles. Receipt:
`target/release-validation/qualification.mgXj17/receipt.json`.
The standard metadata, exact-tree, annotated-tag and atomic-push checks passed.
Publication status is zero under `target/publication/publish.selection.520zh8/`.
Registry/archive verification: `target/platform0232-delivery/registry-verification.json`.
Hosted CI remains asynchronous; this is Linux evidence, not native macOS evidence.
The [advisory workflow](https://github.com/dragginzgame/ic-platform/actions/runs/42087238475)
failed on existing crypto-bigint 0.5.5 / RUSTSEC-2026-0387 (#50); no waiver or
correction is claimed. This is separate from the twenty passing local stages.

## Forward recovery and unrelated draft

The local 0.23.1 attempt passed its gate but stopped before tag/push/upload when
its normal hook expanded a feature array after the staged tree was recorded.
Commit `12cab9c` and its original journal/receipt remain; there is no 0.23.1 tag
or crates.io upload. It is retained in the delivered main ancestry.
The renderer now sorts candidates before staging; real-hook regressions cover
all three groups, rollback, exact custody, delivery and resume. A fresh 0.23.2 gate
qualified the fix. No history, saved journal or failed evidence was rewritten.

The unrelated Candid/derive 0.10.40 lock draft is separately retained at
`refs/retained/platform-0231-candid-draft` and restored onto the released lock
without changing its owner versions. It is not included in the release archive.
Custody/patch evidence remains under `target/platform0231-delivery/`.

## Native Profiler #140

#140 is closed. Its ordinary module layout and preserved public/private paths
are delivered in Git; recorder/viewer ownership stays with Profiler. The IC pin
is unchanged. Committed Profiler `50d9548` supplied read-only fixture input;
no sibling source, release or consumer pin was changed.

The paired Linux proof passed 83 recorder, 114 product, 52 other selected and six
installed tests against server hash `e2240c66dc9f`. All 71 provider inputs plus
orchestration still match that proof. See the
[qualification](../audits/native-profiler-140-2026-11-05.md) and
`target/platform0231-delivery/native-evidence-reuse.json`.
Native macOS and Bazel execution remain unqualified; root CI is not that evidence.

## Shared cleanup and other packages

The Candid boundary admits 153 exact declarations across 226 Rust files and
refuses new/stale allowances. Twenty-five obsolete experiment files are retired;
compact conclusions remain in the [audit](../audits/shared-030-practices-2026-11-05.md).
New benchmark output uses `target/evidence/`. No new size/performance claim is made.
#50 remains open without a waiver. Prior 0.23.0 metrics/CBOR contracts remain;
see the [previous handoff](https://github.com/dragginzgame/ic-platform/blob/07b442e/docs/status/current.md).

Browser SDK 0.2.0 remains published from `d30e01c`, tag `browser-v0.2.0`, over
SDK Core 6.1.1. Its earlier packed-consumer and Chromium page/worker evidence is
under `target/browser020-delivery/`; no npm publication was repeated here.

## Evidence custody

Approved archives/bundles remain in
`recovery-evidence/retired-checkouts/20261104-issue139/`, with checksum and
relocation receipts. Runtime 0.18.1 binaries remain under
`target/recovery-evidence/runtime-0181-delivery.LgU8r6/`. Review remains due
**2026-12-04**, owned by the maintainer. Retention refuses excess/expiry without
cleanup or extension. Cargo clean can reclaim target evidence; retain active
release receipts and their stage logs while recovery may be needed.
Recovery refs `refs/retained/platform-023-unrelated` and
`refs/retained/platform-023-late-candid` remain. No evidence cleanup was performed.
