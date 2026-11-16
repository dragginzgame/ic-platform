# Initial crates.io deployment

The maintainer authorized crates.io deployment from public
`dragginzgame/ic-platform` and then selected **Types independently after
qualification**. Downstream production migration remains a separate gate.
This sequencing does not waive retained identity contracts, archive/real-consumer
proof, supported hosts, source provenance or registry readback. Siblings remain
read-only. All six product crates are now published; qualification packages remain private.
See the [complete publication record](qualification/2026-11-16-publication.md).

## Initial Types release selection

Types 0.21.1 follows registry 0.20.4 and delivered source 0.21.0. Its patch increment
identifies changed package metadata/documentation after source delivery. Principal,
Account/Subaccount, Serde and opt-in self-authentication retain their contracts;
caller-buffer text rendering is additive. Empty defaults remain mandatory.
Protocol 0.2.1, Platform 0.34.2 and coordinated Runtime 0.23.8 advance because
normalized exact Types dependency contents change. Those were prepublication source increments; the subsequent independent releases
are recorded below.

The repository publisher is `https://github.com/dragginzgame/ic-platform`.
Read-only `cargo owner --list` reported `borovan` on 2026-11-16. Credentials stay
with Cargo and are never recorded. The first publication selected only `ic-platform-types`, targeting `crates-io`
with all features. The later roster covers the remaining five packages.

## Qualification and execution

[The Types profile catalog](../ci/types-release-profiles.json) names the retained
contract, independent consumer, portable graph, actual archive and Runtime type
crossing checks. The [real-consumer proof](qualification/types-release.md) uses
pinned Auth passive model source in an isolated workspace; it does not migrate
Auth's working tree or claim full Auth application acceptance.

The authenticated Toko 0.44.7 installation supplies publication selection,
retained attempts, source checks, release metadata checks and archive provenance.
The generic release runner is not enabled. The first Types release uses an
explicit reviewed commit and annotated `types-v0.21.1` tag, followed by the
supported `make publish-check PACKAGE=ic-platform-types` and
`make publish PACKAGE=ic-platform-types` adapter. Its Types admission requires:

- Clean source, only Types publishable, empty defaults and the intended repository.
- An annotated tag at HEAD, independently observed at the remote.
- Record exact-commit hosted CI observations separately. The maintainer selected
  passing local qualification as the upload gate; do not wait for CI completion
  or claim uncompleted host checks passed.
- Successful registry observation showing the exact version absent; lookup failure
  and an existing version both refuse instead of triggering a blind retry.
- Security audit, the complete profile catalog and archive provenance fetched
  from the declared repository. Evidence is retained below `target/`.

The supported runner rechecks source and retains intent, dispatch, Cargo output
and completion statuses. An interrupted upload requires registry/provenance
reconciliation before any new attempt. Success still needs independent sparse
index and archive readback. Types 0.21.1 is now published; its [publication record](qualification/types-0.21.1-publication.md)
binds the upload and independent registry readback.

## Supplier review

The 2026-11-16 direct-supplier review retained crc32fast 1.5.5, data-encoding
2.11.1, serde 1.0.229, serde_bytes 0.11.19, sha2 0.11.1, serde_json 1.0.154
and serde_test 1.0.177. The existing thiserror 2.0.18 selection is retained for
PocketIC cohort alignment; optional refresh to 2.0.21 is deferred to the next
dependency-maintenance batch. No dependency upgrade is needed for this release.
RustSec database commit `02e2517f457a359fc6d3440ea4560c8ea188fee8` matched its
remote HEAD at review. Root-lock audit found no vulnerabilities. The existing
serde_cbor unmaintained warning belongs outside the Types product closure.

The initial Types release did not qualify other groups. Their subsequent local,
archive and registry evidence is recorded separately below; none of these releases
claims completion of the full rebuild. The pinned shared contracts are
[release execution](https://github.com/dragginzgame/toko-tooling/blob/626da7cd9626cc2e152594862a2d82ea82ef51da/docs/releases.md)
and [Cargo publication](https://github.com/dragginzgame/toko-tooling/blob/626da7cd9626cc2e152594862a2d82ea82ef51da/docs/cargo-publication.md).

The Types host workflow preserves repository/secret/format checks, Types and
consumer profiles, native/Wasm graph checks and Runtime identity crossing. The
full CI workflow remains unchanged in scope. Its simulator and other product
checks are separate release obligations, not Types dependencies. The preceding
source run passed Linux but failed macOS PocketIC server setup; no simulator
qualification is inferred from the focused Types workflow.

## Remaining groups: independent publication

The maintainer subsequently directed publication of every remaining implemented
product scope and confirmed updated local release rules. This extends independent
sequencing to Platform, Protocol and Runtime/support packages. It does not claim
production migration or completion of the broader design. Hosted CI is not awaited.

Select Platform 0.34.3, Protocol 0.2.2 and coordinated Runtime/macros/guard 0.23.9.
Types 0.21.1 is already published and unchanged. The new patch increments account
for publication metadata and packaged documentation after source delivery.
Registry baseline checks returned Platform 0.33.7 and Runtime/support 0.22.5;
Protocol returned an observed 404, distinguishing a new package from lookup failure.

The [local catalog](../ci/release-profiles.json) preserves all implemented group
checks, including installed Runtime and fresh observed certificates/signatures.
`make qualify-release` binds complete logs, audit and catalog to clean source.
Pass its explicit `RELEASE_RECEIPT` to `make publish-check PACKAGE=...` and
`make publish PACKAGE=...`. Admission verifies receipt/log hashes, source, annotated
remote group tag, registry absence, all-feature Cargo packaging, extracted-archive
native/Wasm Clippy and native tests, and independently fetched archive provenance.
No existing receipt is silently selected and no upload is automatically retried.

Publish guard and macros first, then Runtime, Protocol and Platform. Independently
read back each registry archive and checksum before proceeding to dependents.
The same registry Types version must be used across public identities. Keep the
experimental Runtime limits, strict Platform refusals, and selected Protocol
surface explicit; no removed legacy facade is restored or silently promised.

## Native-read and consumer publication

Platform 0.34.5 and Protocol 0.2.4 are published from clean source
`1e92703bc803b81dbe1e3d815fd1077d440d62b4`, with annotated
`platform-v0.34.5` and `protocol-v0.2.4` tags pushed atomically with main.
Both add compatible opt-in capabilities; existing defaults stay empty. Types
0.21.1 and Runtime/macros/guard 0.23.12 have unchanged product contents and are
not republished. The complete local release catalog now includes actual Query,
Registry envelope/placement and native read qualification alongside existing
installed and certificate checks. Hosted CI remains asynchronous.

Registry review on 2026-11-16 confirmed the selected versions absent and current
baselines Platform 0.34.4 / Protocol 0.2.3. Types 0.21.1 and Testkit 0.107.1 remain
current. Toko 0.44.8 is newer than the installed, reviewed 0.44.7; its optional
adoption is deferred outside this active release under the baseline's release
latency rule. The existing thiserror hold and frozen Auth supplier selections
remain unchanged. Newly added transport suppliers were reviewed at implementation
selection; their provenance and qualification are recorded in
[Query's hard-cut boundary](qualification/query-hard-cut.md).

The complete local catalog passed at that commit; its receipt is
`target/release-qualification/1794853493857-3415366/receipt.json`. Packaging caught
and corrected an accidental thiserror lock refresh before delivery: the root now
retains the documented exact 2.0.18 constraint required by published Types. Both
extracted archives passed all-feature native/Wasm Clippy, native tests and
independently fetched source provenance checks. Hosted CI was not awaited.

| Package | Registry archive SHA-256 | Dry run / upload evidence under `target/publication/` |
| --- | --- | --- |
| [ic-platform-protocol 0.2.4](https://crates.io/crates/ic-platform-protocol/0.2.4) | `0a3177e65f7005b317a0eac7f3fafc257d7aee16f72bf2be9419dd43b1fd8409` | `check.selection.aKLPRZ` / `publish.selection.Ln2b2D` |
| [ic-platform 0.34.5](https://crates.io/crates/ic-platform/0.34.5) | `acc6568e65c44808ebdf058087f24398ca165c79a17ce8ea84bbeac85608946d` | `check.selection.SAbgGR` / `publish.selection.16j0OP` |

Each upload ran once. Independent downloads matched sparse-index checksums,
locally admitted archives and exact clean-source VCS identities; reproduction and
readback are retained at `target/registry-readback/1794853805046/verified.json`.

The published product cohort passed actual consumer qualification:
184 Query topology/CLI tests plus Clippy, and all five installed Auth tests.
Product dependencies resolve from crates.io, with one Platform 0.34.5 and Types
0.21.1 identity and no Agent or legacy Platform. Protocol is 0.2.4 and Runtime
remains 0.23.12. Reproduction, full metadata and results are retained at
`target/registry-cohort/1794853809864/verified.json`. The independent schema
fixture remains private; sibling production sources are unchanged. macOS proof
and the separate Agent signing/derivation obligation remain open.

## Next supplier cohort: 16 November 2026

The selected compatible releases are Types 0.21.2, Runtime/macros/guard 0.23.13,
Protocol 0.2.5 and Platform 0.34.6. Registry review confirmed their preceding
versions as current and each candidate absent before preparation. Toko Tooling
0.44.9 is adopted through its supported installer in both workspaces and in
materialized candidate consumer manifests. Original archived source and explicitly
frozen historical comparison evidence remain unchanged. Testkit 0.107.1 and Wire
0.24.2 remain current. Registry Types 0.21.1 and Platform 0.34.4 are deliberately
retained as independent qualification oracles, not candidate product suppliers.

Types removes the exact thiserror hold; the product lock selects 2.0.21 while
PocketIC qualification can retain 2.0.18. Runtime adds synchronous stable-memory,
liquid-cycle/burn and version-one HTTP cost operations. Protocol adds bounded
raw_rand, explicit version-one HTTP encoding and chunk-cache codecs. Platform
adds strict native ingress packet admission using upstream request identities.
The corresponding qualification documents retain each consumer's remaining gates.
These releases do not close the outstanding Query, Backup, Scheduling, Entropy
or Wasm Store production migrations or the separate Agent crypto obligation.

The complete local release runner now includes the existing Types profile catalog
alongside the other product groups. Both catalog hashes are receipt-bound, and
Types uses the same package/source/remote-tag/registry/archive admission as every
other selected package. Publish Types first, then the Runtime guard and macros,
Runtime, Protocol and Platform. Every upload requires independent readback before
its dependents; hosted CI remains asynchronous and is not awaited.

This cohort is now published and independently read back. See the
[complete supplier publication evidence](qualification/2026-11-16-supplier-publication.md)
for source/tag identity, checksums, retained attempts and registry-backed Query/Auth results.
