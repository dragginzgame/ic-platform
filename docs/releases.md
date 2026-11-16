# Initial crates.io deployment

The maintainer authorized crates.io deployment from public
`dragginzgame/ic-platform` and then selected **Types independently after
qualification**. Downstream production migration remains a separate gate.
This sequencing does not waive retained identity contracts, archive/real-consumer
proof, supported hosts, source provenance or registry readback. Siblings remain
read-only. Platform, Runtime, Protocol and qualification packages remain unpublished.

## Selected release

Types 0.21.1 follows registry 0.20.4 and delivered source 0.21.0. Its patch increment
identifies changed package metadata/documentation after source delivery. Principal,
Account/Subaccount, Serde and opt-in self-authentication retain their contracts;
caller-buffer text rendering is additive. Empty defaults remain mandatory.
Protocol 0.2.1, Platform 0.34.2 and coordinated Runtime 0.23.8 advance because
normalized exact Types dependency contents change. They are not upload candidates.

The repository publisher is `https://github.com/dragginzgame/ic-platform`.
Read-only `cargo owner --list` reported `borovan` on 2026-11-16. Credentials stay
with Cargo and are never recorded. The only declared publication package is
`ic-platform-types`, targeting `crates-io` with all features.

## Qualification and execution

[The Types profile catalog](../ci/types-release-profiles.json) names the retained
contract, independent consumer, portable graph, actual archive and Runtime type
crossing checks. The [real-consumer proof](qualification/types-release.md) uses
pinned Auth passive model source in an isolated workspace; it does not migrate
Auth's working tree or claim full Auth application acceptance.

The authenticated Toko 0.44.5 installation supplies publication selection,
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

Platform admission still needs its own authentication acceptance and a qualified
registry Types consumer graph. Types release does not qualify other groups or
the full rebuild. The pinned shared contracts are
[release execution](https://github.com/dragginzgame/toko-tooling/blob/626da7cd9626cc2e152594862a2d82ea82ef51da/docs/releases.md)
and [Cargo publication](https://github.com/dragginzgame/toko-tooling/blob/626da7cd9626cc2e152594862a2d82ea82ef51da/docs/cargo-publication.md).

The Types host workflow preserves repository/secret/format checks, Types and
consumer profiles, native/Wasm graph checks and Runtime identity crossing. The
full CI workflow remains unchanged in scope. Its simulator and other product
checks are separate release obligations, not Types dependencies. The preceding
source run passed Linux but failed macOS PocketIC server setup; no simulator
qualification is inferred from the focused Types workflow.
