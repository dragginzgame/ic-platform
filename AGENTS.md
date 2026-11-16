# IC Platform vNext rebuild instructions

Read [README.md](README.md) and [DESIGN.md](docs/DESIGN.md). The latter is the
current architecture and migration/qualification plan; earlier tentative drafts
are superseded. Consult [consumer requirements](docs/requirements.md) before
implementing or removing an affected surface.

The maintainer has requested a ground-up rebuild and local Rust/Toko setup.
Proceed through real vertical slices under the design's acceptance gates.
Prefer existing crate names when concept and functionality are retained; changes
are allowed when the ownership boundary warrants them. Before creating a local
package, check crates.io: use the next SemVer version for an existing package and
0.1.0 for a new package. Breaking pre-1.0 contracts advance the minor version;
compatible changes advance the patch. After a GitHub source snapshot, changes
to a product crate require a new version for its next delivery, even while
unpublished. Apply this to every product group, including Protocol and Types;
unchanged groups retain their versions. Coordinate Runtime support packages
with Runtime. Do not reuse a delivered version for changed crate contents.
Check the published baseline at selection
time rather than copying an old checkout's version. Private qualification packages
also start at 0.1.0 and remain outside product release groups. Publisher selection
and publication remain separate. A package move alone is not rewrite progress.

This repository adopts the installed [Toko engineering baseline](TOKO-TOOLING.md)
from published Toko 0.44.11 and its revision-bound specialist rules. Cargo.lock and
`.toko-tooling.snapshot` bind the selection and installed bytes. Keep managed
files intact; use the supported installer for updates. [Development commands](docs/development.md)
describe the local Make interface and focused checks.

The adoption applies common authorization, source-integrity, dependency and
validation rules. Product architecture, empty defaults, unpublished package
status and focused acceptance remain local choices. The workspace uses the
common lints and Rust 1.100.0 as required by the design. Unsafe code is denied
workspace-wide and forbidden at the Types/Protocol crate roots and existing
non-export qualification crate roots. The maintainer approved the IC-specific
import/wrapper boundary in Runtime's private `call/ic0` module and, subsequently,
IC entrypoint export attributes in dedicated export fixtures. Both
use item-scoped justified lint expectations; `unsafe_op_in_unsafe_fn` stays denied.
The macro implementation and wrapper bodies remain safe Rust. No crate-wide
allowance or non-IC unsafe code is authorized; see the
[approved boundary](docs/qualification/runtime-ic0-boundary.md).
No common-policy exception is selected. Tool installation alone
does not establish product or native-host qualification.

- Use ICP CLI (`icp`) exclusively for canister operations. Never invoke `dfx`
  or introduce commands, wrappers or fallbacks that use it.
- Keep `../ic-platform` and every other sibling read-only unless separately authorized.
- The maintainer separately authorized applying the six-file EngineView patch
  to `../ic-query` and validating it. That local application is complete; it
  grants no Query commit, push, publication or unrelated sibling-edit authority.
- Preserve unique work here. The maintainer authorized source delivery and,
  subsequently, starting crates.io deployment from public `dragginzgame/ic-platform`.
  Release preparation and publication are authorized subject to the design's
  existing acceptance gates; this does not waive qualification. Keep package
  publication disabled until the selected release is admitted. Repository deletion
  and sibling migration remain excluded. GitHub source delivery is not registry
  publication. See the [initial release plan](docs/releases.md).
- The maintainer approved releasing Types independently after its retained-contract,
  packaged-crate and real-consumer qualification. Downstream production migration
  remains a separate gate; this scoped sequencing choice does not waive Types
  correctness, source integrity, host or publication evidence.
- The maintainer subsequently authorized independent publication of the implemented
  Platform, Protocol and Runtime scopes, including coordinated macros and guard.
  Qualify locally and verify archives/source/registry evidence; do not wait for CI.
  Keep production migrations and broader design qualification separate, with
  experimental limitations explicit.
- Create manifests only for real implementation slices. Do not add empty Cargo
  packages solely to select tooling; this restriction was explicitly reaffirmed.
- Prioritize Auth's missing Runtime capabilities for the next Runtime release:
  query/lifecycle exports, caller/canister identity and IC time, certification
  publication/retrieval, and explicit guard/decode/encode/reply composition.
  Follow the [consumer acceptance contract](docs/requirements.md#next-runtime-release-priority-auth-execution-requirements)
  and qualify Auth's actual installed fixture. These are retained
  requirements; do not replace their behavior with consumer compatibility shims.
  Runtime 0.23.10 now passes the pinned actual Auth installed fixture; retain the
  [scope and legacy crypto boundary](docs/qualification/runtime-auth.md).
  Packaging and production migration remain separate admission gates.
- The initial Protocol package implements request encoding and selected response
  decoding, with initial version 0.1.0 and publication disabled. The separate
  qualification workspace proves a public consumer/type crossing. Full Query
  adoption and release execution remain pending; do not claim the first
  projection gate has passed.
- Maintain implemented product groups in `ci/cargo-release-groups.json` and their
  notes in `releases/<group>/CHANGELOG.md`; root `CHANGELOG.md` is navigation only.
  Types now implements Principal and Account/Subaccount, with its own group;
  opt-in self-authentication and independent consumer qualification are present.
  Actual production cohort migration remains pending.
  Runtime now has call preparation, one-way submission and experimental awaited
  transport using local Types, with an experimental method-attached executor.
  Its group is separate. The [installed qualification](docs/qualification/runtime-installed.md)
  covers rollback/cleanup, task ownership, refunds, resource refusals, bounded
  uncertainty and draining callbacks before upgrade. Keep its remaining limits
  explicit; passing these cases does not qualify the entire design.
  The supported upgrade workflow is stop, wait for IC status Stopped, upgrade,
  then start. Qualify this operational boundary; do not require live-call
  reconstruction or add callback persistence, automatic cancellation/retries,
  or a Runtime pre-upgrade guard. Operator tooling reports stages and refuses
  installation unless stopped. Suspended futures and callback IDs stay heap-local.
  Native-only WAT/Candid tools support independent fixtures and management calls;
  keep them out of product graphs. Retained custody after failed cleanup remains
  a separate obligation, not a reason to invent an upgrade recovery subsystem.
  Keep child/migratory tasks outside this slice.
  Its exact-version Cargo singleton guard is now implemented in the Runtime
  group. The explicit-update macro implementation shares that group. Platform
  0.34.3 has its own group, a strict untrusted certificate decoder and an opt-in
  direct-root/single-delegation authentication path; follow its
  [reuse and acceptance boundary](docs/qualification/platform-certificate.md).
  Decode success must not imply authentication. Direct-root evidence requires
  explicit root, target, time window and traversal limits. Delegated evidence also
  checks subnet authority, complete selected ranges and a separate time window.
  Keep nested-delegation refusal and bytewise range ordering; do not substitute
  Principal's Rust ordering. Selected-subnet time must match the delegated subnet,
  or require an independently trusted root-subnet binding without delegation.
  Never substitute effective-canister authority for this subnet contract.
  Keep fresh PocketIC observations and frozen public-API replay tests in the
  qualification path. Independently observed mainnet direct-root and delegated certificates now pass
  native/Wasm replay, including explicit root-subnet binding. Broader mainnet interoperability and production consumer proof remain
  pending before the full authentication gate.
  Protocol 0.2.2 now selects local Types 0.21.1 alongside Runtime. Preserve that
  canonical source identity; registry Types is only a qualification oracle.
  Query's older 0.1.x integration needs cohort migration before adoption.
  Qualification stays outside product release groups.
  The independently qualified releases are recorded in the
  [publication evidence](docs/qualification/2026-11-16-publication.md); broader
  product and production migration gates remain separate.
- Keep the native heap profiler in its private qualification package. Allocation
  comparisons do not establish execution speed, Wasm size or macOS qualification;
  retain its preselected criteria and source-bound evidence.
- Wasm size acceptance is undecided. The maintainer deferred contribution
  analysis; preserve the measurements without reinstating an unapproved
  zero-growth gate or weakening correctness/resource checks to reduce size.
- Start with the EngineView/EngineSubnet projection and its consumer requirements.
  Published Types may serve early experiments; do not count dependency reuse as
  rewritten implementation.
- Preserve one canonical Principal/Account identity and one executor. Matching
  versions or bytes do not establish matching Cargo source/type identity. Qualify
  local Types/Runtime as one cohort before crossing their public type boundaries.
- Evaluate executor mechanisms against explicit task, callback, rollback and
  reentrancy contracts. One executor is required; the old implementation is not
  automatically exempt from redesign. Use separate artifacts for comparisons.
- Keep required external protocol/storage bytes and every cryptographic,
  certificate, delegation, range, identity-key and authenticated-time check.
  Preserve resource accounting and refusal semantics; API simplification must
  not weaken them. Decode capability must not require Encode.
- Do not implement cryptographic primitives, fork upstream algorithms, restore
  textual DID tooling, or add storage allocation, scheduling, trust-root discovery,
  application authorization or business retry policy.
- Do not bulk-copy old crates, tests, generated code or workflow machinery.
  Admit narrowly bounded reuse only after documenting its destination contract,
  pinned provenance/license, dependencies, safety obligations and independent proof.
- Keep Protocol independent of Runtime, and Platform independent of Protocol and
  Runtime. Keep qualification out of production graphs and defaults empty.
- Existing issues define retained obligations even when closed. Read relevant
  resolution evidence before reuse; closure does not qualify the new code.
  GitHub remains the issue tracker; the local requirement ledger maps contracts
  and proof owners, not a duplicate issue-status queue.
- Use focused checks for each coherent slice. Preserve full release obligations
  until a qualified replacement exists; do not import historical orchestration
  solely to obtain a green gate or claim native/macOS proof from Linux checks.

- Canister-signature envelope decoding is now implemented as an opt-in unverified
  capability. Keep the embedded certificate opaque at that stage; no evidence of
  authentication may escape before key/message/witness, authority and explicit
  time policies are checked. Follow the [consumer contract](docs/qualification/canister-signature.md).
