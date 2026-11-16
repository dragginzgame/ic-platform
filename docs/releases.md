# Initial crates.io deployment

The maintainer requested starting crates.io deployment after source snapshot
`1ed002ce9a59a0272af1a875f5d3b8914d6a09a6`. This supersedes the earlier
source-only authorization. It does not waive the acceptance gates in
[the design](DESIGN.md#9-cutover-and-final-release-gate), authorize sibling changes,
or establish registry publication. No upload or release tag has been attempted.

## First proposed cohort

Start with **Types, then Platform**, supporting Auth's issuer-signature integration.
Runtime and Protocol have independent release obligations; they are not required
by this verifier graph. Qualification packages remain private and outside product
release groups. This is the proposed initial scope, not approval to omit an
obligation that applies to either selected product.

Use this repository, `https://github.com/dragginzgame/ic-platform`, as the intended
source publisher. On 2026-11-16, read-only `cargo owner --list` reported `borovan`
for both existing registry packages. This identifies registered ownership, not
the credentials of a future upload. The publication workflow must admit the
actual credential and exact registry destination without recording secrets.
Do not publish the same replacement packages from the legacy source repository.

| Group | Registry baseline observed | Delivered source candidate | Next action |
| --- | --- | --- | --- |
| Types | `ic-platform-types` 0.20.4 | 0.21.0 | Complete retained-contract/cohort acceptance and archive qualification first. |
| Platform | `ic-platform` 0.33.7 | 0.34.1 | Complete authentication consumer acceptance, then qualify with the selected Types archive/registry identity. |

Registry baselines were checked against the crates.io sparse index; both are
stable and non-yanked. The hard-cut minor lines were selected before this task.
Final publication versions are not frozen. Changing package metadata after the
source snapshot consumes a new version under the local delivery rule: enabling
publication must not silently reuse changed contents under a delivered version.
Recheck registry availability at selection time. Updating Types also requires
reviewing exact dependency pins and normalized package contents in Protocol and
Runtime, even when those groups are not selected for this first upload.

## Readiness evidence

- `cargo package --list --locked --offline` succeeds for both selected products.
  The inventories include source, tests, normalized manifests and the root licence;
  they contain no qualification workspace or native simulator implementation.
- `cargo package --locked --offline -p ic-platform-types` succeeds, including
  Cargo's default-feature compilation from the generated 0.21.0 archive. This
  is packaging evidence, not all-feature consumer or release admission.
- The equivalent Platform command fails before creating an admitted package:
  its normalized manifest requires registry Types `=0.21.0`, which is unpublished.
  This is the expected supplier-first constraint. A local path build does not
  resolve it; do not weaken the exact cohort pin to package against older Types.
- Both manifests currently omit repository/documentation metadata, and all
  products retain `publish = false`. Those metadata changes belong in a reviewed
  release preparation batch with the required version increments.
- [CI run 43889896773](https://github.com/dragginzgame/ic-platform/actions/runs/43889896773)
  failed on all three native hosts at qualification manifest ordering, before
  product validation. The formatter's first pass had interleaved package metadata
  with target tables; the follow-up groups target tables before metadata and
  passes both `make fmt-check` and a subsequent `make fmt`/`make fmt-check` cycle.
  A new hosted run is required; the earlier failure is not a product-test pass.

## Work required before upload

1. **Consumer and retained-contract acceptance.** Types still records incomplete
   production cohort/compatibility proof. Platform's observed signature replay and
   isolated Auth adapter are passing evidence, but actual Auth compilation/adoption
   and broader authentication acceptance remain open. Map those obligations to
   the selected release; do not declare the whole Auth application migrated from
   the isolated issuer-policy test. Existing ECDSA/root-delegation needs and the
   old feature surface need explicit consumer disposition. Siblings remain
   read-only without separate authorization.
2. **Supported release execution.** The installed Toko 0.44.5 selection does not
   contain the release or Cargo publication entrypoints, and this repository has
   no qualified release adapters or complete source-bound release gate. Adopt
   needed tools through the authenticated installer with a complete retained
   entrypoint list; do not edit the vendor tree. Use the supported metadata,
   source/provenance, publication custody and uncertain-upload recovery contracts.
   The design forbids dropping the old obligation floor or activating group-based
   omissions merely because the new group metadata exists.
3. **Candidate preparation.** Review current direct suppliers and security evidence
   for the selected product and qualification graphs, resolve publisher/credential
   admission, select new versions, update exact cohort pins, supply repository
   metadata and enable only admitted product packages. Keep private fixtures
   unpublished and all defaults empty. Qualify all affected independent locks.
4. **Archive and host proof.** Qualify the actual Cargo archives and their public
   consumer crossings, including selected features and supported hosts. Before
   Types is in the registry, use the supported coordinated-archive qualification
   route; a path-patched experiment must not masquerade as registry admission.
   Preserve original-byte, immutable-evidence, authority, time and resource checks.
5. **Delivery and readback.** Run the qualified complete release workflow, bind
   version/tag/archive evidence to the exact source and publish Types first.
   Independently observe registry availability and provenance before admitting
   Platform's registry consumer build and upload. Reconcile uncertain uploads;
   never retry them automatically. Report registry readback separately from Git
   delivery and hosted qualification.

These are release blockers, not another permission request. Do not run a direct
`cargo publish` around missing admission, change acceptance thresholds, or label
an experimental release qualified to bypass them.

The pinned Toko contracts are [release execution](https://github.com/dragginzgame/toko-tooling/blob/626da7cd9626cc2e152594862a2d82ea82ef51da/docs/releases.md)
and [Cargo publication](https://github.com/dragginzgame/toko-tooling/blob/626da7cd9626cc2e152594862a2d82ea82ef51da/docs/cargo-publication.md).
