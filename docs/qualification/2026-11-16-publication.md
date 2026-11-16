# Independent product publication — 16 November 2026

The maintainer authorized publication of all implemented product scopes with
local qualification and no CI-completion prerequisite. Production migration and
the broader design remain separate. Runtime retains its documented experimental
limits; this release does not claim whole-design completion.

Source: `3f972ec7d43e2fcd087f69bae90767f6bc405c85` in public
`dragginzgame/ic-platform`. Annotated source tags: `runtime-v0.23.9`,
`protocol-v0.2.2`, `platform-v0.34.3`.

| Package | Published version | Independently observed archive SHA-256 |
| --- | --- | --- |
| ic-platform-runtime-guard | [0.23.9](https://crates.io/crates/ic-platform-runtime-guard/0.23.9) | `3a75c3507b9c81128d48d858cc9e56dbd3f25b115f0a67083f02c94f41907e0b` |
| ic-platform-macros | [0.23.9](https://crates.io/crates/ic-platform-macros/0.23.9) | `f87298c86f56a66ca62bbd046efeac8be22d07d9a9c9584265aaeb26a2c54366` |
| ic-platform-runtime | [0.23.9](https://crates.io/crates/ic-platform-runtime/0.23.9) | `3b4f8c548f008420e4527abffc72663a29dc6712b00014d373b341ced0d74381` |
| ic-platform-protocol | [0.2.2](https://crates.io/crates/ic-platform-protocol/0.2.2) | `73abb281bd2689e9e14724637e555a5e88d1e1154dbf432e4ecf37d325591f08` |
| ic-platform | [0.34.3](https://crates.io/crates/ic-platform/0.34.3) | `677657d25b5ac9b25fd9b7a632cd8f9b99b6be35f4c93b6f0a172132533dde68` |

Types [0.21.1](types-0.21.1-publication.md) was already published and unchanged.
No upload was retried. Each registry archive matched the sparse-index checksum,
the admitted local archive and the clean release source/path in its VCS record.

## Qualification

The complete [local profile catalog](../../ci/release-profiles.json) passed:
repository integrity, graphs, Protocol wire/resource/allocation/consumer tests,
Runtime native/Wasm/guard/export/installed rollback and upgrade checks, and
Platform certificate/signature tests, fresh observations and frozen replay.
Receipt: `target/release-qualification/1794843333666-1872416/receipt.json`.
Its source, catalog, compiler and complete log hashes were checked before every
package. Negative checks refused dirty source, missing or mismatched receipts,
changed source/catalog/compiler, incomplete profiles and an unknown selector.

Every package passed its supported publication dry run and extracted-archive
all-feature native/Wasm Clippy and native tests. The extraction added only a
workspace-discovery marker; archives and registry dependency declarations were
unchanged. Provenance fetched the exact source from the declared repository.

| Package | Dry-run evidence | Upload evidence |
| --- | --- | --- |
| ic-platform-runtime-guard | `target/publication/check.selection.0ttUuJ` | `target/publication/publish.selection.xI7GRD` |
| ic-platform-macros | `target/publication/check.selection.4kR9kL` | `target/publication/publish.selection.4a0eUx` |
| ic-platform-runtime | `target/publication/check.selection.WJY3Cg` | `target/publication/publish.selection.lc54WJ` |
| ic-platform-protocol | `target/publication/check.selection.j5ZPSI` | `target/publication/publish.selection.Gufrgz` |
| ic-platform | `target/publication/check.selection.uErM0X` | `target/publication/publish.selection.hagOOS` |

Readbacks and their JSON records are under `target/registry-readback/`.

## Supplier disposition

The product and qualification lock audits found no known vulnerabilities using
RustSec database `02e2517f457a359fc6d3440ea4560c8ea188fee8`. Existing unmaintained
notices remain: serde_cbor in the selected certificate grammar, and paste in
qualification dependencies. They are not silently represented as vulnerability
findings or as removed obligations. The narrowly admitted serde_cbor boundary
and its resource/refusal checks remain unchanged.

Current stable supplier versions were reviewed. The exact ic-certification 3.2.0
selection remains for its admitted public tree/Serde contract; newer 4.0.1 is a
breaking supplier line and is deferred to the next dependency-maintenance review.
The existing thiserror 2.0.18 selection remains coordinated with PocketIC, with
optional patch refresh deferred to that same boundary. No supplier was upgraded
as part of publication.

## Registry-only consumer proof

`target/registry-cohort/1794843683642/verified.json` records successful checks with
all six selected product versions resolved from crates.io. A disposable copy of
this release's qualification sources replaced product path declarations with
exact registry versions, without product code copies or Cargo patches. The
retained metadata establishes registry source identity for every selected crate.

Native Clippy/tests passed for Types/Runtime and Protocol consumers. The four
certificate, mainnet, signature and Auth-policy admission binaries executed on
native and standalone Wasm. Runtime's actual exported Wasm methods passed the
existing boundary harness. The Protocol consumer compiled for Wasm, and the
Runtime consumer graph admitted one canonical Types and one Runtime identity.

The first disposable certificate replay attempt omitted data files stored under
`crates/ic-platform/tests/fixtures`; it failed at compile-time `include_bytes!`.
Adding those original frozen fixture bytes completed the proof. No published
crate, product dependency or assertion was changed. Both attempt logs and the
corrected reproducible harness are retained with the evidence. This is registry
consumer proof, not an application deployment or production migration.
