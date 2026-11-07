# Current handoff — 2026-11-07

## Runtime 0.20 / Platform 0.25 deployment in progress

Requested publication selects Runtime **0.20.0** (Runtime/macros/exact guard), then
Platform **0.25.0**, to crates.io and `origin/main` with annotated group tags.
Types **0.18.2** and browser **0.2.0** remain unchanged. No new package upload,
release commit or tag has happened yet. Source preparation is pushed through
`a13f2974e1944127ce327fe32cf76ccb771c832f`; metadata-workflow corrections follow.

First frozen batch `batch-d_fzxuMA` stopped during `check-consumer`, before version
preparation: its admission fixture still selected the removed facade `cdk` profile.
The committed fix uses current runtime/external-Candid profiles and retains final
Cargo failure/concurrent-edit tests. Focused admission and shell lint pass.

Second batch `batch-TC9AK4RR` passed all **20** stages, but post-version graph
admission refused the mechanically rewritten fixture lock. Native Testkit's old
facade still needs an inactive registry Runtime 0.19.5 record after the local
owner moves to 0.20. No second executor is enabled. The saved plan remains at
`prepare`; prepared metadata, source and receipt are retained without reinterpretation.
Both attempts remain under `target/release-frozen/` and durable frozen Git stores.

The correction resolves locks offline from their maintained selections inside
the metadata transaction. Real Runtime 0.20 preparation, rollback, post-version
graphs and delivery/resume pass at `target/platform-group-preparation.zzxUOi/`.
Foreign versions/checksums stay pinned. The fixture now uses real graph admission
and the shared changelog validator to select a pending increment or default patch,
avoiding an unqualified future Runtime minor after publication. Evidence:
`target/deployment-platform0250-runtime0200-20261107/`. Start a fresh batch from
the corrected committed source; all 20 stages remain mandatory. Hosted/native CI
remains asynchronous; no fresh macOS/Profiler acceptance is claimed.

## Current source contract and qualification supplier

Runtime owns the single conventional `cdk::{query, update, init, ...}` macro family.
The facade `cdk` feature/ordinary macro exports and implementation `runtime_*`
family are removed. Facade raw `runtime` adapters and external-Candid endpoints
remain. Consumers and scaffold generators must select Runtime `cdk` directly;
explicit crate overrides, codecs, ABI, guard ordering and executor identity stay.
See [the migration](../migration.md#canonical-ordinary-entrypoints-runtime-020--platform-025).

The inactive selected-stage pilot, host allowlist and qualification workflow are
removed. Receipts retain full source/metadata/evidence and recovery admission.
Real preparation/rollback/custody covers every group; Runtime supplies one full
fixture delivery/resume cycle instead of three.

Scheduling qualification commit `e50ca4f91e7cc213e089d7ef79f9c224d36d61ad`, branch
`adopt/runtime-020-platform-qualification`, widens only the previous supplier's
Runtime upper bound to <0.21. It preserves Timers 0.37 implementation and accepts
Runtime 0.20. No Scheduling working files or release were changed. Platform's
pin and fixture lock select it without unrelated dependency upgrades. Native/Wasm
composition graphs pass; installed/release qualification remains batch-owned.

Earlier focused owner checks pass: Runtime/macros/CDK tests, external-Candid
standalone contracts, feature closures, singleton identity/refusal, planner and
recovery fixtures, plus installed owner/Timers Wire/external-Candid raw/prepared
profiles with awaits, traps, cancellation, upgrades, rollback and certificates.
Evidence: `target/complexity-cleanup/`, `target/cdk-runtime.y9J8xH/` and
`target/cdk-runtime.6ReMRn/`. Those checks used the local 0.19.5 package identities;
they are not publication proof for 0.20. No size/performance improvement is claimed.

## Shared baseline and deferred dependency draft

Shared **0.31.5**, `b2026a118e38de4af1555c527af327cb604eacdf`, is the latest
observed remote main and the adopted baseline. The canonical exporter refreshed
and independently verified all 141 payloads. Evidence: `target/shared0315-adoption/`
and the current deployment directory. The metadata editor remains ic-build 0.73.0.

The independent Candid/derive **0.10.40**, cc **1.11.1**, SmallVec **1.16.5** draft
is excluded from release inputs and restored in normal Cargo.lock during capture.
Its exact original SHA-256 is
`4ce766eee04d278fe4885ece92342bf4fa27f21f572259e5021b0422f435fe3f`.
Durable custody: `refs/retained/platform0250-runtime0200-20261107-lock-draft`;
current deployment evidence retains the original file and external-only patch.
Select the committed lock before capture; after completed batch reconciliation,
reapply that external patch over delivered owner versions. Never restore an old
whole lock over new versions. Earlier retained refs remain untouched.

## Last completed delivery and deferred callers

Platform **0.24.0** is published on crates.io/GitHub: release commit
`fc6d419474a94a606e3c49729747e61978b4c9c7`, annotated `platform-v0.24.0`, source
`467279b2134e5812048786c6c2b0616eff606ac8`. Batch `batch-KU2DvnyL` completed and
reconciled. Registry/archive/VCS/checksum and remote tag identities agree;
all 20 Linux/Rust 1.99.1 local stages passed. Runtime/macros/guard remain **0.19.5**.
Evidence: `target/deployment-platform0240-20261107/`; receipt in that candidate's
`target/release-validation/qualification.mXhnnO/`, logs `validation.Oc5mXN`.
No fresh macOS or native-provider acceptance was claimed.

That release removes offline secp derivation and unconfigured Candid decoders.
Caller work stays with [Canic #739](https://github.com/dragginzgame/canic/issues/739),
[Canic #740](https://github.com/dragginzgame/canic/issues/740),
[Blob #119](https://github.com/dragginzgame/ic-blob-storage/issues/119) and
[IC Crates #25](https://github.com/dragginzgame/ic-crates/issues/25).
The native Agent still brings crypto-bigint 0.5.5 / RUSTSEC-2026-0387; #50 remains
open/deferred by the maintainer, with no waiver. Advisory run 42365731394 records it.

## Retained recovery custody

Preserve completed `batch-KU2DvnyL`, `batch-U1NhGrpl`, `batch-hEX13lnF`, the failed
current attempt, earlier retained refs and interrupted unpublished 0.23.1 state.
Durable stores remain under `.git/release-state/frozen/`; no cleanup is authorized.
Historical archives/bundles/receipts remain at
`recovery-evidence/retired-checkouts/20261104-issue139/`; maintainer review is due
**2026-12-04**. Retention refuses expiry/excess without deleting evidence.
External cleanup previously removed older target evidence; do not claim those
receipts or Runtime 0.18.1 binaries still exist. Earlier complete history is in
[the prior handoff](https://github.com/dragginzgame/ic-platform/blob/c2742d9/docs/status/current.md).
Follow [the batch lifecycle](../platform-release-validation.md#commands-and-effects).
Root receipts do not qualify native Profiler; its paired Linux evidence remains
historical, with macOS/Bazel unqualified.
