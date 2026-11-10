# Current handoff — 2026-11-10

## Accepted scope and current state

Continue the maintainer-authorized specification-only replacement of adopted
DFINITY source, preserving public behavior, bytes and canonical identities.
Independent authors receive reviewed contracts and black-box requirements, not
old implementations/tests. Parent integration has read existing source; records
under `docs/design` describe this separation without claiming legal clearance.
Cryptographic primitives stay with registry suppliers. Keep notices/licenses for
all retained imports. Siblings remain untouched. The maintainer now authorizes
deployment of the three Rust groups only; browser packaging changes receive Git
delivery but no npm publication. Freeze reviewed source before release execution.

Installed Toko is 0.39.8; compiler is 1.99.1. Latest published Platform is 0.29.2
(`platform-v0.29.2`, `f687d96c1d423e87e3083c840aa6d6472f41c165`). Runtime/macros/
guard remain 0.21.0 and Types 0.19.0. Pending notes select Platform 0.29.3,
Runtime 0.21.1 and Types' existing ledger; no package version bump was performed.
Browser remains 0.2.0. Historical release and earlier session details are preserved
in the [prior handoff](../history/source-retirement-handoff-2026-11-10.md).

## Tooling and documentation adoption

The authenticated 0.39.8 snapshot selects the baseline and required executables;
shared reference guides now use pinned upstream links. Product guides are grouped
under architecture, protocols and development, with a [docs index](../README.md).
The release Make opt-in is explicit. Runtime → IC0 admission now passes.
Canonical root files and inherited Cargo license-file metadata are installed;
browser packaging generates its distribution copy. See
[adoption evidence](../development/toko-0396-adoption.md) for checks and limits.
The 0.39.8 refresh changed only Toko in Cargo.lock and retained all 96 snapshot
paths. Tooling/pins/inventory/links/boundaries and browser package qualification
pass (23 tests, two compiler consumers, packed-root comparison and provider SDK
identity). No macOS/Chromium run or delivery was performed.

## Latest completed work

The final ICRC value-hash and ICP account-framing replacements are complete.
Separate source-unexposed authors implemented protocol composition over the
existing external SHA/CRC suppliers; a third author wrote independent public
tests. All 782 frozen original observations agree. Twenty focused tests and
native/Wasm Clippy pass. The original bodies/tests, adjacent source records and
obsolete source-guard scripts are retired; ordinary Cargo tests retain coverage.
Both production dependency trees and Cargo.lock are unchanged. The previous ICP
draft is preserved in target evidence. See
[ledger qualification](../design/ledger-composition-qualification.md).

The maintainer-authorized empty-account and Int Serde bugs are fixed. Empty Value
arrays return an Account conversion error; Int accepts its emitted sign/magnitude
pair and rejects malformed/extra elements. Serialized JSON/CBOR and Candid bytes
are unchanged. Both regressions failed before repair; 15 focused tests and
native/Wasm Clippy pass afterward. See
[follow-up evidence](../design/icrc-value-qualification.md#follow-up-bug-fixes).

The preceding independent Value helpers transfer blob buffers and borrow Display
trees. Ledger composition now hashes ICRC3 directly without a second tree.
Frozen observations remain intact. After detecting stale
Cargo reuse, management was also recompiled: all 23 tests pass. Staged source
installation must refresh destination mtimes before same-checkout Cargo checks.

## Earlier completed components and preserved work

- Principal/Account/RejectCode and Nat payload encoding: see
  [identity qualification](../design/identity-qualification.md). Base identity
  remains crypto/Candid/Runtime/Agent-free; self-authentication uses registry
  ic_principal. IC0 bindings are registry-owned, not embedded.
- Executor, System API/task/stable adapters, raw call transport and macros:
  [executor](../design/executor-qualification.md),
  [adapters](../design/system-adapters-qualification.md),
  [calls](../design/call-transport-qualification.md) and
  [generator](../design/entrypoint-generator-qualification.md) retain exact
  public contracts and installed evidence. Latest generator comparison passed
  four profiles per arm and exact imports/exports. No new macOS evidence.
- Passive ICRC adapters and DTOs: [adapters](../design/icrc-adapters-qualification.md)
  and [DTOs](../design/icrc-dtos-qualification.md) preserve text/Serde/Candid
  behavior. Ledger hashing is covered by the latest replacement above.
- [External DTOs](../design/external-dtos-qualification.md) preserve 33 declarations,
  six aliases and 14 methods with 21 passing tests; full upstream DIDs are retired.
- All selected management DTOs: [shared](../design/management-shared-qualification.md)
  and [native](../design/management-native-qualification.md) preserve 52 shared
  and 19 native schemas, frozen observations and unchanged codec mappings.
- The other session's signature composition and measurements remain separate;
  [contract](../design/canister-signatures/contract.md) and codec
  [comparison matrix](../benchmarks/candid-codec-2026-11-10/matrix.md) own detailed
  evidence. Preserve signature, codec, native-provider retirement and lock edits.
  Fixed-array decoding now refuses incorrect lengths even inside optional fields.

## Delivery readiness

The two identified ledger source blockers are resolved. Tooling, pins, inventory,
documentation and dependency-boundary checks pass. Workspace Rust formatting and
diff-whitespace checks pass; the only unrelated formatting repair wrapped two
existing codec test assertions without changing them. No current imported
DFINITY implementation area is identified in maintained Rust source; external
packages retain their separate ownership. This is not full release qualification.

The authorized patch cohort is Types 0.19.1, Runtime/macros/guard 0.21.1 and
Platform 0.29.3, in that supplier order. Full gates must complete in the managed
frozen batch before publication; focused checks alone do not qualify it. Platform no
longer qualifies downstream Timers: Scheduling #91 owns composition. Native
Agent and installed Runtime evidence identify tested owners separately from
Testkit infrastructure. Published Wire 0.18 / Types 0.19 identity unification was
not established by Platform releases; sibling adoption remains consumer-owned.
#50 remains deferred; no broader release readiness is claimed.

## Recovery custody

Frozen batch `batch-JKeeLzH0` stopped before publication: standalone consumer
admission still rejected the explicit self-authentication provider. Its Clippy,
workspace tests, release-adapter checks and documentation stages passed. The
corrected source retains base Principal isolation and tests optional-provider
admission plus unrelated-client refusal; deployment requires a fresh full batch.
Preserve the failed candidate and evidence.

Preserve failed batches batch-G3rEbkX7 and batch-QJcFkCdU and all earlier
failed/completed batches, including batch-_leIreCI, batch-C5BoFKA5,
batch-AaHi2pgO, batch-O3V4GL4T, batch-PIIqwepD and batch-mG_yG4uo.
Do not resume old batches over published versions. Historical lock custody:
`refs/retained/platform036-lock-draft-20261108` at
`0263ab6e679ae3b8e9f89bddd796ca951b3f1df4` and
`refs/retained/platform0270-lock-draft-20261108` at
`588b11a38c60b15ab5af19f30aef1c310a8b0975`. Never restore an old complete lockfile.

Historical bundles/receipts remain at
`recovery-evidence/retired-checkouts/20261104-issue139/`; maintainer review is due
2026-12-04. No cleanup, mtime renewal or custody-limit change is approved.
Previously removed target evidence and Runtime 0.18.1 binaries must not be claimed
present. Qualification artifacts are reclaimable; durable reports record limits.

The unreferenced root signature-comparison draft is preserved verbatim at
`recovery-evidence/deferred-signature-comparison-20261110.rs` (SHA-256
`9fdee2e8b41c9b13e0d2a3127ddf95559a1094fa48d14dd17b7577d4ec86ed4a`).
It is outside the release; maintained comparisons live in the facade tests.
