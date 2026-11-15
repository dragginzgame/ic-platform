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
from published Toko 0.44.3 and its revision-bound specialist rules. Cargo.lock and
`.toko-tooling.snapshot` bind the selection and installed bytes. Keep managed
files intact; use the supported installer for updates. [Development commands](docs/development.md)
describe the local Make interface and focused checks.

The adoption applies common authorization, source-integrity, dependency and
validation rules. Product architecture, empty defaults, unpublished package
status and focused acceptance remain local choices. The workspace uses the
common lints, strengthened to forbid unsafe code, and Rust 1.100.0 as required
by the design. No common-policy exception is selected. Tool installation alone
does not establish product or native-host qualification.

- Keep `../ic-platform` and every other sibling read-only unless separately authorized.
- Preserve unique work here. The maintainer authorized source delivery to public
  `dragginzgame/ic-platform-new`. Registry publication,
  releases, repository deletion and sibling migration remain excluded. Keep
  every package unpublished; GitHub source delivery is not registry publication.
- Create manifests only for real implementation slices. Do not add empty Cargo
  packages solely to select tooling; this restriction was explicitly reaffirmed.
- The initial Protocol package implements request encoding and selected response
  decoding, with initial version 0.1.0 and publication disabled. The separate
  qualification workspace proves a public consumer/type crossing. Full Query
  adoption and release-group setup remain pending; do not claim the first
  projection gate has passed.
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
