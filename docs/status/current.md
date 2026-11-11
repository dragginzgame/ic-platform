# Current handoff — 2026-11-11

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

Installed Toko is 0.39.9; compiler is 1.99.1. Platform 0.29.3, Runtime/macros/
guard 0.21.1 and Types 0.19.1 are published on crates.io and tagged on GitHub.
Completed frozen batch `batch-nUsSSjzc` delivered commit
`f4c904645e00b435b256d3acb87837493240fdbb`; the normal checkout is reconciled.
Browser remains 0.2.0 and was not published. Historical release and earlier session details are preserved
in the [prior handoff](../history/source-retirement-handoff-2026-11-10.md).

## Tooling and documentation adoption

The 0.39.9 registry refresh updates the baseline's dependency-policy references
and the direct IC dependency checker. Shared now admits native-only PocketIC
dev-dependencies; Platform retains its stricter local prohibition and adds no
simulator dependency. All 96 snapshot paths and selected entrypoints remain.
Only Toko's registry version/checksum changed in Cargo.lock; the pending Types
SHA-2 edge is preserved. No sibling or product release is part of this adoption.

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
identity). These local checks do not establish macOS/Chromium acceptance.

## DID hard cut

The maintainer superseded #154/#155 with complete DID retirement. The native
service exporter, its consumer/check target, text fixtures and candid_parser
dependency are removed. Binary Candid remains unchanged. Independent schemas
were converted to frozen binary headers and checked against the original text
fixtures before retirement; protocol value snapshots preserve byte comparisons.
Canic adoption remains consumer-owned; no sibling source or release is included.
Deployment is now authorized: release Types 0.19.2 first, then Platform 0.30.0.
The different increments require separate sequential frozen batches. Runtime
and browser versions remain unchanged. Source is captured only after the reviewed
combined changes are committed; each group retains the full local release gate.

All 110 captured schemas passed structural comparison before text retirement.
Affected native Clippy and 34 focused schema/value/public-contract tests pass
without candid_parser. Snapshot/pins/inventory/docs, Candid/CBOR boundaries and
feature-selection/refusal checks pass. Cargo pruned 71 packages without other
version changes; binrw is an already-selected test-only oracle dependency. The
normal Wasm codec graph remains free of upstream Candid/parser and runtime.
Generic didc entries remain only in the immutable Toko installer/catalog; no
Platform caller selects them. Prior Principal/tooling changes are preserved.

The follow-up residue audit removed obsolete parser-consumer, service-registration
and upstream-decoder instructions from active qualification docs, and removed
the feature checker's retired parser activation variable. All 143 graph-refusal
cases pass. The all-feature Wasm normal/build workspace graph has no upstream
Candid, candid_derive, candid_parser or pretty. Upstream binary Candid remains
in independent development oracles and native Agent/IC transport; browser
`/candid` remains SDK Core's binary IDL adapter for required external protocols.
These are active capabilities, not text-DID leftovers. Historical evidence and
immutable supplier tool catalogs are preserved.

## Latest completed work

Pending Types 0.19.2 replaces the optional registry Principal adapter with the
maintainer-authorized SHA-224-plus-`0x02` composition. Hashing uses the shared
RustCrypto SHA-2 0.11 supplier. Base identity remains hash-free; the registry
Principal remains only a development oracle and native-client transitive.
No packages from this follow-up have been released. Qualification is recorded
in the [identity report](../design/identity-qualification.md).

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
  remains crypto/Candid/Runtime/Agent-free; pending self-authentication uses
  upstream SHA-224 with owned framing. IC0 bindings are registry-owned, not embedded.
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
packages retain their separate ownership.

The authorized patch cohort was published in supplier order: Types 0.19.1
(`59135d5`, `types-v0.19.1`), Runtime/macros/guard 0.21.1 (`97e96c7`,
`runtime-v0.21.1`), Platform 0.29.3 (`f4c9046`, `platform-v0.29.3`). Each passed
all 19 local gate stages, including Clippy, workspace tests, consumer/feature
graphs and installed execution. Retained receipts under the completed candidate
are `qualification.Ivi4nw`, `qualification.sDGekt` and `qualification.R2gECn`.
Publication also verified each selected package archive; the registry index
confirms all five versions are available and not yanked. Platform no
longer qualifies downstream Timers: Scheduling #91 owns composition. Native
Agent and installed Runtime evidence identify tested owners separately from
Testkit infrastructure. Published Wire 0.18 / Types 0.19 identity unification was
not established by Platform releases; sibling adoption remains consumer-owned.
#50 remains deferred. Hosted Linux/macOS and browser jobs are asynchronous;
local Linux qualification is not macOS acceptance.

The hosted Wasm-size job failed before measuring because catalog projection
looked for installed tools inside a Git archive. The post-release harness fix
uses the invoking helper's tools while preserving the selected catalog bytes.
Archive-without-tools admission and all nine benchmark harness tests pass.
The exact failed comparison against `97e96c7` now passes all four profiles
(`principal-binary`, `runtime`, `verification`, `certification-cbor`) with zero
raw/optimized total/code-section deltas. Evidence is retained at
`target/evidence/wasm-regression.upepSR`; no Rust package changes are required.

The hosted history scanner identified seven non-secret findings: the IC root
public key, current/historical deliberately published test identities, and
migration prose. `.gitleaksignore` records only those exact commit/path/rule/line
fingerprints; future keys and edits remain scanned. Both post-release CI repairs
are tooling-only and do not require another crates.io version.

## Recovery custody

Frozen batch `batch-JKeeLzH0` stopped before publication: standalone consumer
admission still rejected the explicit self-authentication provider. Its Clippy,
workspace tests, release-adapter checks and documentation stages passed. The
corrected source retains base Principal isolation and tests optional-provider
admission plus unrelated-client refusal; `batch-nUsSSjzc` subsequently passed.
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
