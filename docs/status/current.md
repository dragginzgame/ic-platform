# Current handoff — 2026-11-06

## Authorized three-group deployment

The maintainer authorized deploying the pending Platform changes. Release order:
Types **0.18.2**, Runtime **0.19.3** (with macros/guard), Platform **0.23.3**.
Use this checkout/main and origin/main through the standard explicit-group
workflow. Browser **0.2.0** is unchanged. No sibling release/adoption is included.
These are pending candidates, not publication claims. Shared **0.30.5** at
`be3771031ebfb340583b238dee5446be65929c62` remains the adopted baseline.

The unrelated Candid/derive 0.10.40 and cc 1.11.1 lock draft is retained exactly
at `refs/retained/platform-20261106-deploy-lock-draft` (blob), with original lock
and separate draft patch under `target/deployment-20261106/`. Release source
retains only the owned facade serde_test-edge removal. Restore the draft patch
after all publications, preserving new owner versions; never copy the original
whole lock over released metadata. No unrelated dirty selection is included.

## Included behavior and focused evidence

#147 adds `icp_ledger_types::account_identifier` for owned Principal plus exactly
32 subaccount bytes. The selected Apache-2.0 ic-ledger-types 0.16.0 constructor
body is unchanged and source/license guarded. Native vectors/public consumer and
native/Wasm Clippy pass; an altered upstream body refuses. No Agent, Runtime,
CDK, parser, ledger-client package or Candid printer enters that selection.
The pending migration is in `docs/migration.md`; Canic adoption follows publication.

Utility defaults narrow data-encoding/serde_bytes to alloc. Standalone Types'
graph is narrower; probe savings are only 0–6 bytes, not a material Wasm win.
Types' fourteen tests, forty selected Platform tests and five ICRC consumer tests
pass. See `docs/benchmarks/utility-features-2026-11-06.md`. The unused facade
serde_test development edge is removed; Types/Runtime retain real uses.

Audit commits `43f48b0` and `244e085` supply CBOR fuzzing, shared Rust/browser
ingress vectors, Wasm regression tooling and stable-order executor tail removal.
The latter improved native reverse cancellation of 8,192 tasks from 7.55 ms to
0.14 ms, adding 58 bytes each to the optimized Runtime module/code section.
This is not an application IC timing. Two decoder allocation prototypes were
reverted after adding about 9 KB each; Candid traversal was measured but unchanged.
Focused Clippy/tests and installed Runtime/Timers qualification passed. Fuzzing
passed 420,463 executions. See `docs/audits/platform-improvements-2026-11-06.md`.

Hosted source `244e085` passed normal CI, browser, installed-runtime and Wasm-size
workflows. Three-host release-mapping qualification run **42184881886** passed
Linux and macOS ARM64 at the latest observation; Intel remains asynchronous.
The host allowlist remains empty: all actual releases still use the full gate.
#50 remains open awaiting upstream, without a fix, risk acceptance or waiver.
Its known advisory workflow failure is separate from release-stage evidence.

## Last verified publications and recovery

Platform **0.23.2**, commit `81b3f05f53ed113d02a671d47d13f1a36222e845`,
tag `platform-v0.23.2`, was verified on crates.io/GitHub. Types is **0.18.1**,
Runtime/macros/guard **0.19.2**. Previous registry receipt:
`target/platform0232-delivery/registry-verification.json`; all twenty Linux
release stages passed at `target/release-validation/qualification.mgXj17/`.

The interrupted 0.23.1 commit `12cab9c` remains in main ancestry, with its original
journal/receipt. There was no 0.23.1 tag/upload. Metadata rendering now sorts
before staging; exact-tree and real-hook regressions cover all three groups.
No history or failed evidence was rewritten. Earlier unrelated draft refs
`refs/retained/platform-0231-candid-draft`, `refs/retained/platform-023-unrelated`
and `refs/retained/platform-023-late-candid` remain preserved.

Native Profiler #140 is delivered in Git. Its IC pin is unchanged; 71 provider
inputs plus orchestration match the paired Linux proof against server
`e2240c66dc9f`. See `docs/audits/native-profiler-140-2026-11-05.md`.
Native Profiler macOS/Bazel remain unqualified; root CI is not that evidence.
Browser SDK 0.2.0 was published from `d30e01c`, tag `browser-v0.2.0`, over SDK Core
6.1.1. No npm release is part of this deployment.

## Evidence custody

Approved archives/bundles remain under
`recovery-evidence/retired-checkouts/20261104-issue139/`, with receipts/checksums.
Maintainer review is due **2026-12-04**. Retention refuses excess/expiry without
cleanup or extension. Reproducible Runtime 0.18.1 binaries remain under
`target/recovery-evidence/runtime-0181-delivery.LgU8r6/`. Cargo clean can reclaim
target evidence; retain active release receipts/stage logs while recovery may be
needed. No evidence cleanup was performed.
