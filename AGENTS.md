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
compatible changes advance the patch. Check the published baseline at selection
time rather than copying an old checkout's version. Private qualification packages
also start at 0.1.0 and remain outside product release groups. Publisher selection
and publication remain separate. A package move alone is not rewrite progress.

This repository adopts the installed [Toko engineering baseline](TOKO-TOOLING.md)
from published Toko 0.44.4 and its revision-bound specialist rules. Cargo.lock and
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
generated IC entrypoint export attributes in dedicated export fixtures. Both
use item-scoped justified lint expectations; `unsafe_op_in_unsafe_fn` stays denied.
The macro implementation and wrapper bodies remain safe Rust. No crate-wide
allowance or non-IC unsafe code is authorized; see the
[approved boundary](docs/qualification/runtime-ic0-boundary.md).
No common-policy exception is selected. Tool installation alone
does not establish product or native-host qualification.

- Keep `../ic-platform` and every other sibling read-only unless separately authorized.
- The maintainer separately authorized applying the six-file EngineView patch
  to `../ic-query` and validating it. That local application is complete; it
  grants no Query commit, push, publication or unrelated sibling-edit authority.
- Preserve unique work here. The maintainer authorized source delivery to public
  `dragginzgame/ic-platform-new`. Registry publication,
  releases, repository deletion and sibling migration remain excluded. Keep
  every package unpublished; GitHub source delivery is not registry publication.
- Create manifests only for real implementation slices. Do not add empty Cargo
  packages solely to select tooling; this restriction was explicitly reaffirmed.
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
  Its group is separate. The installed probe covers immediate and awaited updates,
  initial/resumed/post-reply trap rollback and successful callback cleanup.
  Cleanup failure and the remaining installed gates are still pending.
  Keep child/migratory tasks outside this slice.
  Its exact-version Cargo singleton guard is now implemented in the Runtime
  group. The explicit-update macro implementation joins that group; Platform
  still joins alongside a real implementation package.
  Qualification stays outside product release groups.
  This metadata setup does not qualify or enable a release/publication workflow.
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
