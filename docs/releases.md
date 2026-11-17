# Initial crates.io deployment

## Module layout release

Select Platform 0.34.11, Protocol 0.3.2 and coordinated Runtime/macros/guard
0.23.15 for the installed Toko module-layout rule. Thirteen source modules move
to `mod.rs` with byte-identical contents and unchanged public paths. Candidate
qualification pins and locks follow the new versions; registry comparison
suppliers stay pinned. Types 0.21.2 is unchanged and is not republished.
Registry review on 2026-11-17 confirmed all selected versions absent and the
preceding published versions current.

Run the complete local catalog at clean source, then push annotated
`platform-v0.34.11`, `protocol-v0.3.2` and `runtime-v0.23.15` tags with main.
Publish guard and macros before Runtime, followed by Protocol and Platform,
using the supported publication adapter and independent archive/source readback
for each package. Hosted CI remains asynchronous; siblings stay read-only.

All five packages are now published from clean source
`80a5b488666629dd28152f6bcb6d489091d55994`. Main and all three annotated
tags were pushed atomically. The complete local catalog passed once at this
source; receipt: `target/release-qualification/1794907293508-4193179/receipt.json`.
The root-lock audit found no known vulnerabilities against current RustSec
`02e2517f457a359fc6d3440ea4560c8ea188fee8`.

| Package | Independent registry archive SHA-256 | Dry run / single upload under target/publication |
| --- | --- | --- |
| ic-platform-runtime-guard 0.23.15 | `4fa857e5bba51d6b57286fa6b09f2bbe430da574bb187376c7c77adb896cfb99` | `check.selection.MDQ6VT` / `publish.selection.A0kNpB` |
| ic-platform-macros 0.23.15 | `b2d98162c6fbea8d9ea5893db8efad17fa4860c221ef4b3a3e66c5e94e6c53cd` | `check.selection.HKkjVt` / `publish.selection.9fDxZw` |
| ic-platform-runtime 0.23.15 | `654fbae9abca1d9f1ccf1aa7835ba1303b08bc828706f12c0e8a45446a206410` | `check.selection.SZZfzo` / `publish.selection.R0XHlY` |
| ic-platform-protocol 0.3.2 | `b2b1db444c839e816c1f2b6edec5ef8966a3f022a73a12819cc0db8477e5b560` | `check.selection.qzr3ST` / `publish.selection.g4bC2B` |
| ic-platform 0.34.11 | `6269645258c22c1cccbaaf62ec710b7a30fbbcfe879aff6a53ba53b8526c9c84` | `check.selection.eFMZSt` / `publish.selection.uKCB9W` |

Every archive matches the sparse index, admitted local archive and exact tagged
VCS source. Extracted-archive all-feature native/Wasm Clippy, native tests and
independently fetched provenance passed before each single upload. Combined
readback: `target/deploy-module-layout/published.json`; individual records are
under `target/registry-readback/<package>-<version>/verified.json`.

Post-push review confirmed byte-identical module moves, unchanged public paths,
coordinated exact Runtime dependencies and updated candidate qualification pins.
Hosted [CI](https://github.com/dragginzgame/ic-platform/actions/runs/44055562803)
and [Types qualification](https://github.com/dragginzgame/ic-platform/actions/runs/44055562983)
were observed in progress once; neither was awaited or claimed passing. Native
execution remains Linux evidence. Siblings and unchanged Types were not delivered.

## Backup fixture and projection release

Protocol 0.3.1 supplies the bounded response fixture constructors and separate
minimal capture-status projection requested in
[Platform #11](https://github.com/dragginzgame/ic-platform/issues/11).
Required-version upgrade status admission is unchanged. Registry review on
2026-11-17 confirmed Protocol 0.3.0 current and 0.3.1 absent. Other product crates
are unchanged and are not selected for publication. Qualification also adopts
published Platform 0.34.10 and Host Process 0.80.3.

Run the complete local catalog at clean source, retaining the explicit receipt;
deliver main and annotated `protocol-v0.3.1`, then run the supported publication
adapter and independently verify registry archive/source identity. Hosted CI
remains asynchronous. Backup migration and installed acceptance remain separate;
see the [supplier contract](qualification/management-snapshots.md).

Protocol 0.3.1 is published from clean source
`68862a50775ec692c0feb176b6b76fb845355a33`; main and annotated
`protocol-v0.3.1` were pushed atomically. Complete local catalog receipt:
`target/release-qualification/1794906541699-4021355/receipt.json`.
The root-lock audit found no known vulnerabilities using current RustSec
`02e2517f457a359fc6d3440ea4560c8ea188fee8`.

The initial attempt at `target/release-qualification/1794906427266-3965735`
refused a stale candidate Protocol pin in the private Query fixture generator.
Candidate versions now derive from product manifests; the frozen registry
comparison is unchanged. The complete catalog passed after this correction.
Pinned Query/Blob qualification copies were used; no sibling source changed.

Independent archive SHA-256:
`20790281ca3d8f457262dc008357c4a95a9fe1176c8f68a1ca674a2e916e02c8`.
It matches the sparse index, admitted local archive and exact clean tagged VCS
source. Extracted-archive all-feature native/Wasm Clippy, native tests and fetched
source provenance passed before the single upload. Evidence:
`target/publication/check.selection.T4NRWz`,
`target/publication/publish.selection.E6fSB9`, and
`target/registry-readback/ic-platform-protocol-0.3.1/verified.json`.
Native execution is Linux evidence. Hosted
[CI](https://github.com/dragginzgame/ic-platform/actions/runs/44053631968) and
[Types qualification](https://github.com/dragginzgame/ic-platform/actions/runs/44053632187)
were observed in progress once; neither was awaited or claimed passing.

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

## Signed transport and selected protocol release

The 2026-11-16 deployment selects Platform 0.34.7, Protocol 0.2.6 and coordinated
Runtime/macros/guard 0.23.14, after checking current registry baselines. Types
0.21.2 is unchanged. Both workspaces and materialized candidate qualification
consumers adopt Toko 0.44.11 through its supported installer/selection. The
reviewed tooling archive checksum is
`7d16b815ae322ae3f88580ee6af1611bdafde08b77592cd886bfa0eece0f1be8`.

Platform adds explicit Ed25519 signing, exact-envelope dispatch, signed queries
and authenticated original-request observation. Protocol adds management snapshot
formats and bounded Caffeine codecs. Runtime adds the synchronous replicated
execution observation. Query's portable schema corpus removes its private fixture
dependency. Full consumer migration and the CLI custody/management acceptance
journeys remain separate from independently qualified supplier publication; see
[the Blob contract](qualification/blob-suppliers.md) and
[management snapshot scope](qualification/management-snapshots.md).

Run the complete local release catalog at the clean release commit, then publish
guard, macros, Runtime, Protocol and Platform in that order with independent
registry readback between dependency layers. Hosted CI is observed asynchronously,
not awaited. No sibling delivery or unchanged Types release is selected.

This cohort is now published and independently verified. See the
[signed supplier publication evidence](qualification/2026-11-16-signed-supplier-publication.md)
for registry checksums, source tags, complete local qualification and retained attempts.

## External signing and management/ledger release

The selected compatible releases are Platform 0.34.8 and Protocol 0.2.7.
Registry checks on 16 November confirmed baselines 0.34.7 and 0.2.6 and absence
of both candidates. Types 0.21.2 and Runtime/macros/guard 0.23.14 product contents
are unchanged. Qualification and generated Auth/Blob consumers select Testkit
0.107.3 (registry SHA-256 `f1abdecdc13e767e8f5f7f83f2c19e48a414d6a7eea2e2ffdbfe95222c56d6c2`).
Its optional managed-server capability is not selected; the existing admitted
qualification server owner remains in use.

Run the complete local catalog once at clean committed source, then retain its
explicit receipt for the existing publication adapter. Deliver main with annotated
`protocol-v0.2.7` and `platform-v0.34.8` tags, publish Protocol then Platform, and
independently verify each registry archive before proceeding. Hosted CI remains
asynchronous. Actual consumer migration and remaining advisory obligations remain
separate; see [management/ledger](qualification/management-ledger.md) and
[external signing](qualification/blob-suppliers.md).

Both packages are now published from clean source
`26834168a74cf76a4b8f93d4450c3da4f9e3f5db`. Main and both annotated tags
were pushed atomically. The complete local catalog passed; receipt:
`target/release-qualification/1794863276300-1287140/receipt.json`.
The current RustSec database was `02e2517f457a359fc6d3440ea4560c8ea188fee8`;
the release lock audit found no known vulnerabilities. Linux execution does not
establish macOS qualification or clear legacy consumer advisory obligations.

| Package | Independently downloaded archive SHA-256 |
| --- | --- |
| [Protocol 0.2.7](https://crates.io/crates/ic-platform-protocol/0.2.7) | `db4ea27c2705897b24b1c7c239db916abbb4c15eb97c1adcda896d0ca3891ff4` |
| [Platform 0.34.8](https://crates.io/crates/ic-platform/0.34.8) | `33bf34291d82ae3a1735c12aa20f1457c87e36bda3215a0a3e48c4c20efedbb1` |

Each archive passed all-feature native/Wasm Clippy, native tests and independently
fetched source provenance before its single upload. Registry readbacks matched
the sparse-index checksum, admitted local archive and clean tagged VCS identity.
Attempts under `target/publication/`: Protocol dry run `check.selection.ZYo8bV`,
upload `publish.selection.pR7uji`; Platform dry run `check.selection.SndGAJ`,
upload `publish.selection.bYoxP9`. Combined readbacks:
`target/deploy-0348/published.json`.

The first catalog attempt retained under
`target/release-qualification/1794863145331-1240725` refused the advanced sibling
Query HEAD. The successful attempt used `QUERY_SOURCE=target/deploy-query-source`
at the existing pinned revision with every recorded input hash verified, and
`BLOB_SOURCE=target/blob-types-source` at its clean pinned revision. Both are
disposable qualification-only copies; no sibling was modified. The existing
adapter lacks partial-stage resume, so a new complete receipt was executed.

Post-push review of the delivered APIs, feature/dependency wiring and directly
affected qualification callers found no new actionable issue. Hosted
[CI](https://github.com/dragginzgame/ic-platform/actions/runs/43957664273) and
[Types qualification](https://github.com/dragginzgame/ic-platform/actions/runs/43957664311)
were observed once as in progress; neither was awaited or reported as passing.

## Offline ingress construction release

Platform 0.34.9 adds bounded offline envelope construction for externally supplied
signatures. Qualification compares against published Platform 0.34.8 and tests the
local 0.34.9 candidate. Types 0.21.2, Protocol 0.2.7 and Runtime/macros/guard
0.23.14 product contents remain unchanged and are not republished. Registry review
confirmed Platform 0.34.8 as current and 0.34.9 absent before selection.

The existing complete local catalog and explicit receipt remain the admission
route. The pinned Query/Blob fixture copies are selected through QUERY_SOURCE and
BLOB_SOURCE, preserving recorded hashes and leaving siblings read-only. Deliver
main and annotated tag `platform-v0.34.9`, then use the supported publication
adapter and independent registry archive/source readback. Hosted CI is asynchronous.
Backup's actual migration/recovery acceptance remains separate from supplier
publication; see [the ingress contract](qualification/ingress-packets.md).

Platform 0.34.9 is now published from clean source
`def09efbb7a19aa8ded742af9eb851c379ef062d`. Main and annotated
`platform-v0.34.9` were pushed atomically. The complete local catalog passed
once at this source; receipt:
`target/release-qualification/1794866137708-2025997/receipt.json`.
The release lock audit found no known vulnerabilities using current RustSec
`02e2517f457a359fc6d3440ea4560c8ea188fee8`. Native execution is Linux evidence.

The independent crates.io archive SHA-256 is
`87c8d12d9b6294a3319e54cbe0ade3ac937a240ba85e3e762b4b56999055f35b`,
matching the sparse index, admitted local archive and exact tagged VCS source.
All-feature extracted-archive native/Wasm Clippy and native tests passed. Retained
dry run: `target/publication/check.selection.S09orl`; single upload:
`target/publication/publish.selection.opPmmH`; independent readback:
`target/registry-readback/ic-platform-0.34.9/verified.json`.

Post-push review of the shared serializer, public construction boundary and
qualification wiring found no new actionable issue. Hosted
[CI](https://github.com/dragginzgame/ic-platform/actions/runs/43965577962) and
[Types qualification](https://github.com/dragginzgame/ic-platform/actions/runs/43965577864)
were observed once in progress; neither was awaited or reported as passing.

## SNS/ICRC supplier and qualification refresh

Protocol 0.2.8 adds the opt-in bounded SNS neuron and independent ICRC display
metadata codecs for #21. It supports any explicitly selected deployment of the
supported contracts; Dragginz and Toko are consumer qualification examples.
Registry review found Protocol 0.2.7 current and 0.2.8 absent. Platform 0.34.9,
Types 0.21.2 and Runtime/macros/guard 0.23.14 product contents are unchanged.

Qualification now selects registry Platform 0.34.9 and Testkit 0.107.4, also in
materialized Auth/Blob fixtures. Testkit's archive checksum is
`8629db43fa7a3e3655184c6173a5ccdecf67a87d29f0c929228bec7f8a1c392b`;
its new optional CLI is outside this dependency refresh. Existing explicit
qualification-server ownership remains selected. Run the complete local catalog,
then publish Protocol alone using its clean source, annotated remote tag and
explicit receipt. Verify archive checksum and source independently after upload.
Hosted CI remains asynchronous; downstream Query/Backup adoption is separate.

Protocol 0.2.8 is published from clean source
`4e14822ca504b43e47942aeda4074ca2766da944`, with main and annotated
`protocol-v0.2.8` pushed atomically. The complete local catalog passed at that
source, including the refreshed qualification suppliers; receipt:
`target/release-qualification/1794867686137-2542022/receipt.json`.
The root-lock audit found no known vulnerabilities against current RustSec
`02e2517f457a359fc6d3440ea4560c8ea188fee8`.

Independent registry archive SHA-256:
`e6c994e205363960ac400252160967bfdec2e9574e5086232d0ccef88ffa5d77`.
It matches the sparse index, admitted archive and exact clean tagged source.
All-feature extracted-archive native/Wasm Clippy and native tests passed.
Retained dry run: `target/publication/check.selection.ZS4Z1A`; single upload:
`target/publication/publish.selection.2i7MzP`; independent readback:
`target/registry-readback/ic-platform-protocol-0.2.8/verified.json`.

Post-push review of the selected codec features, bounds, canonical Types and
qualification dependency wiring found no new actionable issue. Hosted
[CI](https://github.com/dragginzgame/ic-platform/actions/runs/43969961422) and
[Types qualification](https://github.com/dragginzgame/ic-platform/actions/runs/43969961475)
were observed in progress once; neither was awaited or claimed passing. Native
execution is Linux evidence, not new macOS qualification.

## Canic management supplier release

Select Platform 0.34.10 and Protocol 0.3.0 for #22/#23. Registry review confirmed
Platform 0.34.9 and Protocol 0.2.8 current and both candidates absent. Platform
adds certified controller/subnet observations. Protocol separates complete
management settings from unchanged CMC settings, requires status version and
adds ordinary creation/chunked installation. Its breaking Rust/decode changes
require a minor increment. See [migration and evidence](qualification/canic-management.md).
Types 0.21.2 and Runtime/macros/guard 0.23.14 are unchanged.

Use the complete local catalog, clean release source and annotated remote tags
`platform-v0.34.10` and `protocol-v0.3.0`. Publish Protocol then Platform through
the supported adapter, independently verifying each registry archive/source.
Keep CI asynchronous and downstream Canic adoption/recovery acceptance separate.

Platform 0.34.10 and Protocol 0.3.0 are now published from clean source
`efdc02936dde0319ef32825a499fa28db20cb1ff`. Main and both annotated tags were
pushed atomically. The complete local catalog passed at that source; receipt:
`target/release-qualification/1794869942834-3185288/receipt.json`.
The root-lock audit found no known vulnerabilities against current RustSec
`02e2517f457a359fc6d3440ea4560c8ea188fee8`.

| Package | Independent registry archive SHA-256 | Retained dry run / single upload under target/publication |
| --- | --- | --- |
| ic-platform-protocol 0.3.0 | `86ff8ba66bf3212d37cc5d5120f4cae5f1e968cb5c98c9d92bdfa980d53cc628` | `check.selection.cBgnRZ` / `publish.selection.kLr7Hl` |
| ic-platform 0.34.10 | `7798e7684a2e857355408f2c4252d1187589380609b5f2ef259482e6e1a34571` | `check.selection.6B0mi5` / `publish.selection.b6XrYe` |

Both archives match the sparse index, admitted local archives and exact tagged
VCS source. Extracted-archive all-feature native/Wasm Clippy and native tests
passed. Readback records are under `target/registry-readback/`, in each named
package/version directory; combined delivery evidence is
`target/deploy-canic-supplier/published.json`. Uploads completed on 16 November
UTC (17 November in Europe/Monaco).

Post-push review of the certified observation boundary, controller byte parsing,
shared management schema/writer and retained fixtures found no new actionable
issue. Hosted [CI](https://github.com/dragginzgame/ic-platform/actions/runs/43975911701)
was observed in progress and [Types qualification](https://github.com/dragginzgame/ic-platform/actions/runs/43975911516)
queued once; neither was awaited or claimed passing. Native execution remains
Linux evidence; Canic adoption, its independent root-subnet admission and its
installed recovery proof remain separate.
