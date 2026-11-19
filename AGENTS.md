# IC Platform vNext rebuild instructions

Read [README.md](README.md) and [DESIGN.md](docs/DESIGN.md). The latter is the
current architecture and migration/qualification plan; earlier tentative drafts
are superseded. Consult [consumer requirements](docs/requirements.md) before
implementing or removing an affected surface.

Derive real implementation slices from the design's acceptance gates; do not
restore predecessor architecture merely to satisfy old commands. Package identities
and current versions belong in Cargo metadata and the group ledgers. Keep
qualification packages private and outside product release groups.

This repository adopts the installed [Toko Tooling engineering baseline](TOKO-TOOLING.md)
from published Toko Tooling 0.45.15 and its revision-bound specialist rules. Cargo.lock and
`.toko-tooling.snapshot` bind the selection and installed bytes. Keep managed
files intact; use the supported installer for updates. [Development commands](docs/development.md)
describe the local Make interface and focused checks.

The adoption applies common authorization, source-integrity, dependency and
validation rules. Product architecture, empty defaults, private qualification
packages and focused acceptance remain local choices. The workspace uses the
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
- This checkout is the current Platform implementation. Sibling source remains
  read-only unless separately authorized; historical source identities belong in
  the design and qualification records, not a self-referencing sibling path.
- Types, Protocol, Platform and Runtime products (including Runtime macros/guard)
  are authorized for independent registry publication under their existing gates.
  Qualify selected products and verify exact archives/source/registry evidence.
  Consumer production migration and repository deletion remain outside that scope.
  The earlier six-file Query application is completed historical work, not ongoing
  sibling-edit authority. Current delivery is owned by [release guidance](docs/releases.md)
  and [development commands](docs/development.md).
- Create manifests only for real implementation slices. Do not add empty Cargo
  packages solely to select tooling; this restriction was explicitly reaffirmed.
- Before changing Runtime execution, read the [Auth consumer acceptance contract](docs/requirements.md#next-runtime-release-priority-auth-execution-requirements)
  and [installed qualification](docs/qualification/runtime-installed.md). Preserve
  entrypoints, caller/canister identity, IC time, certification, guard/frame/reply
  composition, rollback, refunds, bounded uncertainty and callback ownership.
  Current versions and completed witnesses belong in those records.
- Maintain independent product groups and notes through `ci/cargo-release-groups.json`
  and `releases/`; root changelog is navigation. Keep one canonical local Types
  identity and the Runtime singleton/executor guard across the selected graph.
- Upgrades stop the canister, wait for IC status Stopped, upgrade, then start.
  Do not add live-call reconstruction, persisted callbacks, automatic cancellation
  or retries, or a Runtime pre-upgrade guard. Suspended futures remain heap-local.
  Failed cleanup retains its existing custody obligation.
- Native WAT/Candid fixture tools remain outside product graphs. Protocol defaults
  stay empty; decoding does not imply authentication. Certificate admission must
  preserve explicit root, target, time and traversal limits, subnet authority,
  complete ordered ranges and independently trusted root-subnet binding where needed.
  Keep nested-delegation refusal and bytewise range ordering. Read the
  [certificate contract](docs/qualification/platform-certificate.md) before changes.
  Selected-subnet time must match its delegation; effective-canister authority
  cannot substitute for subnet authority. Qualify fresh observations and frozen
  public-API replay without treating them as complete production migration.
- Keep the native heap profiler in its private qualification package. Allocation
  comparisons do not establish execution speed, Wasm size or macOS qualification;
  retain its preselected criteria and source-bound evidence.
- Wasm size acceptance is undecided. The maintainer deferred contribution
  analysis; preserve the measurements without reinstating an unapproved
  zero-growth gate or weakening correctness/resource checks to reduce size.
- Select further work from current consumer requirements. The original
  EngineView/EngineSubnet bootstrap is historical, not a directive to restart it.
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
