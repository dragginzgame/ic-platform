# Current handoff — 2026-11-08

## Delivered Runtime 0.20.2 and Platform 0.26.1

The reviewed simplification/audit batch was committed as
b8997bbad037f5934b41791b3667739a18da7ae2 and delivered through frozen batch
batch-AaHi2pgO, supplier order runtime,platform. Both groups are on crates.io and
origin/main with their annotated tags:

- Runtime/macros/guard **0.20.2**: bb76a069da4d442e1e830636bace9760fbebd880,
  runtime-v0.20.2.
- Platform **0.26.1**: 3fa6ccbf934bc1f0b5215ad2f116ffe832f9f988,
  platform-v0.26.1.

Types remains **0.18.2** and browser **0.2.0**; neither was released. The normal
main branch was reconciled by the standard fast-forward command. The batch's
schema-2 journal has completed index 2 under .git/release-state/frozen/batch-AaHi2pgO.
No batch, build output, retained archive or receipt was retired or cleaned.

Both groups passed all **20/20** required local stages on Linux / Rust 1.99.1,
including Clippy, consumer/feature/Wasm checks and installed owner/Timers raw and
prepared Wire/external-Candid profiles. The unchanged policy-fixture leaf was
executed for Runtime and reused with fresh Platform admission. Complete receipts:

- target/release-frozen/batch-AaHi2pgO/target/release-validation/qualification.Y7ujMF/receipt.json
- target/release-frozen/batch-AaHi2pgO/target/release-validation/qualification.hLqe8Q/receipt.json

All four registry archives match their official index checksums, retained local
archive bytes and exact release VCS commits; GitHub main and peeled tags matched.
Evidence: target/deploy-runtime0202-platform0261-20261108, including registry/
archives and runtime-registry-verification.json / platform-registry-verification.json.

## Hosted CI and post-release fixture correction

One observation at the published Platform commit found Browser SDK success;
CI, installed runtime and dependency advisories were still running. Wasm size
workflow [42482741311](https://github.com/dragginzgame/ic-platform/actions/runs/42482741311)
failed before measurement: new Node fixture helpers assumed another test had
already created target/. This was a fresh-checkout test race, not observed Wasm
growth. Both helpers now create their parent independently. Isolated fresh-copy
runs pass 4 capture tests, 3 section/report tests and the concurrent 9-test suite.
The correction and this handoff are Git-only maintenance; public package code,
requirements and release tags are unchanged. No additional crate release is needed.

The follow-up push restarts the hosted workflows with a code-classified test
change. Leave their native/integration outcomes asynchronous; no new macOS, real
browser/worker or native Profiler acceptance is claimed. The completed Wasm
failure log and fresh-copy evidence are retained with the deployment evidence.

## Delivered scope and baseline

Shared **0.31.11** at 6c529e6940a97226cca826726491e03173b08cd2 supplies 149 verified
payloads, including the common-tools action. The batch consolidates owned error
boilerplate, group-plan/catalog reads, graph classifications and macro tests;
retires dated 0.4/0.22 runners with immutable reproduction references; and shares
Rust/browser response fixtures while retaining their distinct CBOR framing.

Captures verify copied bytes, fresh rosters and Git identity. Runtime snapshots
inputs before deriving fixture selections and checks both snapshot and originals
at completion; canonical dependency paths remain, so stop source writers during
qualification. Explicit lock refresh exempts only its selected original lock.
Wasm reporting inspects one byte snapshot. Runtime/benchmarks share artifact
retention, including failed-Clippy propagation. Feature metadata is per-attempt;
remaining fixtures reuse compiler/catalog and completion helpers. Native protocol
fixtures reject oversized payloads before cloning, preserving bytes and errors.
Cryptographic implementations, canonical identities and verification policy are
unchanged. Earlier focused evidence remains in target/complexity-second-20261108
and target/complexity-third-20261108; delivery qualification is the receipts above.

Current ownership remains in the [migration contract](../migration.md). The
[prior handoff](https://github.com/dragginzgame/ic-platform/blob/9880ba816461f777266a0aedd54763f412af1522/docs/status/current.md)
retains earlier release and qualification history. No sibling checkout or release
was changed by this delivery. The deferred [#50](https://github.com/dragginzgame/ic-platform/issues/50)
remains unimplemented/open; no advisory waiver or cryptographic rewrite was made.

## Deferred draft and recovery custody

The incoming Cargo.lock draft is durably retained at
refs/retained/platform0261-runtime0202-lock-draft-20261108, commit
b91e65b11dae893e510e1d459718903594840768. Its exact original SHA-256 is
d47ee5c95cd219e49a36e45a890596fdc810a39ad5f87f204080d2ee28f87e00.
Only its supplier-update patch was reapplied after reconciliation: all 391
external records match that draft, and all five local owner records match the
released lock. The restored working Cargo.lock SHA-256 is
99591563adb27ffadd705cf60456098506788cdcbb8eebb07a92a5db7f5c0ce9.
It remains deliberately uncommitted; never restore the whole older lock over
released owner metadata. Restoration proof: draft-restoration.json in the
deployment evidence directory. Earlier retained draft refs remain untouched.

Preserve all historical failed/completed batches and interrupted release state,
including completed batch-_leIreCI and batch-C5BoFKA5. Use each attempt's supplying
source/journals; never reinterpret old receipts under new tooling. Historical
archives, bundles and receipts remain in
recovery-evidence/retired-checkouts/20261104-issue139/, with maintainer review by
**2026-12-04**. Retention refuses excess/expiry without deletion. No cleanup,
mtime renewal or retirement is authorized. Previously removed target evidence
and Runtime 0.18.1 binaries must not be claimed present.
