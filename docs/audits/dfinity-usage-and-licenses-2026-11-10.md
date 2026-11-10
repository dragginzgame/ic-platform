# DFINITY source and license inventory — 2026-11-10

Current source status: the final ICRC hashing and ICP account framing now have
specification-only replacements with independent original/replacement comparisons.
Their old source/test/guard records below are historical. See
[ledger qualification](../design/ledger-composition-qualification.md) and the
[current inventory](../architecture/dependency-inventory.md#source-maintenance).

Subsequent ICRC helper replacement: Value/ICRC3Value non-cryptographic
declarations/conversions/formatting now have an independent implementation.
The two original hash methods, integer helpers and four tests/vectors remain
unchanged in `value/hash/mod.rs`, retaining ICRC's license, notice and reviewed
source guard. See [value qualification](../design/icrc-value-qualification.md).

Subsequent external DTO replacement: CMC/ICP/Cycles/assets declarations and four
small protocol fixtures were authored in separate specification-only contexts.
Their original declaration adaptations and full DID copies are retired after
exact 14-method schema comparison and frozen public observations. Their four
license/notice pairs are removed with that material; ICRC Value/hash and ICP
account-identifier records remain. See
[external qualification](../design/external-dtos-qualification.md).

Subsequent shared management replacement: all 49 shared declarations and two
aliases now come from a specification-only context. The existing owned
TransformFunc and codec remain unchanged. Public observations and registry
oracles qualify all 52 shared schemas; the shared license/provenance and source
guards are retired. See [shared qualification](../design/management-shared-qualification.md)
and the [current inventory](../architecture/dependency-inventory.md) for the six
remaining imported source areas. Earlier statements below are historical.

Subsequent native management replacement: the 18 snapshot/install/create/log
declarations and alias now have specification-only implementations. Their two
native provenance files and native source checker are retired after public and
registry-oracle comparison. Shared maintained management declarations still
retain Apache licensing and source guards. See
[native qualification](../design/management-native-qualification.md).

Subsequent passive ICRC replacement: metadata, transfer/memo, allowance,
block/certificate and ICRC-21 modules now have specification-only implementations.
The current ICRC import list contains only `icrc/generic_value`; its complete
value/hash source, vectors, license and guards remain. See
[DTO qualification](../design/icrc-dtos-qualification.md). Management and other
external-schema provenance elsewhere in the facade is unaffected.

Subsequent ICRC adapter replacement: Account and archive callback/list adapters
now come from an authorized specification-only context. Account identity behavior
delegates to Types; detailed external text/Serde contracts and callback signatures
are preserved. Their former file digests are removed from the current import
list; the remaining ICRC declarations/value/hash source and its common license
remain. See [adapter qualification](../design/icrc-adapters-qualification.md).

Subsequent generator replacement: proc-macro dispatch, entrypoint emission and
shared syntax helpers now come from the authorized specification-only context,
with fresh private tests and separate public consumers. The earlier export
directory/tests and macro license file were retired. The existing owned codec
adapter changed only its private generator-module references. See
[generator qualification](../design/entrypoint-generator-qualification.md).
No local Runtime or macro LICENSE/NOTICE/provenance files remain; registry IC0
keeps its supplier licensing, and management/ledger imports remain attributed.
First-party license text and manifest metadata still await replacement.

Subsequent call transport replacement: the remaining `cdk::upstream` call source
was independently implemented from the reviewed public/lifecycle contract.
The old directory, license/provenance and fingerprint scripts are removed;
native/public tests and all four installed profiles pass, including multi-call
trap recovery. See [call qualification](../design/call-transport-qualification.md).
The earlier observations below remain historical. Generator and management/
ledger imports retain their notices; this is not blanket licensing clearance.

Subsequent System API/stable/task adapter replacement: separate specification-only
contexts authored these modules and fresh tests. Native/Wasm checks and all four
installed lifecycle profiles pass. Their old source directories are removed;
the remaining CDK call state machine still has its Apache notice/license and
unchanged source fingerprint. The generator remains separately attributed.
See [adapter qualification](../design/system-adapters-qualification.md).

Subsequent executor replacement: its implementation and tests now come from
separate specification-only contexts. Native and installed raw/prepared lifecycle
qualification passes; the old executor source notice/provenance and its
fingerprint checker were retired. See
[executor qualification](../design/executor-qualification.md). The original
inventory below remains historical; CDK adapters/generator and retained ledger/
management imports still have their own source obligations.

Subsequent specification-only replacements: RejectCode, unsigned Nat payload
encoding, base Principal and Account now have separate implementation-context
records under `docs/design`. Their prior implementation/tests were excluded from
that context; the reviewer supplied protocol/API requirements and black-box
observations, then qualified compatibility. Optional Principal derivation now
delegates to registry `ic_principal`; its copied body/vectors are gone. Agent PEM
fixtures were regenerated with OpenSSL rather than retaining copied test keys.
First-party MIT files were subsequently removed at the maintainer's request;
replacement licensing is pending. These changes do not remove attribution
from retained CDK, executor, macros, management/ledger DTOs or ICP derivation.
The retained CDK/ICRC/ICP notices were restored where prematurely deleted.
This update is an implementation inventory, not legal clearance.

Subsequent IC0 migration: Runtime now selects registry `ic0` 1.2.0 instead of
the embedded binding directory. Its local sources, license and provenance were
removed together after matching their original fingerprints to the registry
source. The table below records the original inventory, not a claim that the
IC0 copy remains. Other pending rewrites/deleted notices are not certified as
independently authored or approved for relicensing by this audit.

Subsequent maintainer-authorized change: the entire `native/` source provider,
its IC submodule registration and provider-specific CI/fixtures were retired
from the working tree. The native-provider section and linked source-count
reports below retain the pre-retirement observation, not a current maintained
dependency. Historical Git pins and licenses at those revisions are preserved.

Scope: this repository's product source, root Cargo graph, browser npm graph,
maintained fixtures, optional native provider and declared external tool pins.
This is an inventory of source provenance and recorded licenses, not a legal
opinion or a statement that every listed dependency enters every Wasm.

Committed baseline: `a37b15165a4f87a99f2845cb8263ad565f4a4864`, including published
Platform 0.29.2. The working tree is dirty and changing concurrently. In
particular, another session rewrote the account-identifier/Nat hash helpers and
deleted several related licenses/notices. Those edits were not modified or
qualified by this inventory. The committed provenance is listed explicitly
rather than treating disappearance of a notice as proof of changed authorship.

## Embedded/adapted product source

All entries below have Apache-2.0 provenance in the committed baseline. Versions
refer to source origins, not additional registry dependencies.

| Source origin | Local source | Scope / provenance records |
|---|---|---|
| `ic-cdk` 0.20.3 | `crates/ic-platform-runtime/src/cdk/upstream/` | `api`, `call`, `futures`, `futures/internals`, `stable`, module root; adjacent `NOTICE.md`, `LICENSE-APACHE`, `provenance.json` |
| `ic-cdk-executor` 2.0.0 | `crates/ic-platform-runtime/src/executor/runtime/mod.rs` | Maintained executor; adjacent `LICENSE-APACHE`, `provenance.json` |
| `ic0` 1.2.0 | `crates/ic-platform-runtime/src/ic0/upstream/{mod.rs,sys/mod.rs}` | Imported System API bindings; adjacent `LICENSE-APACHE`, `provenance.json` |
| `ic-error-types` 0.2.0 | `crates/ic-platform-runtime/src/cdk/reject_code/mod.rs` | Selected RejectCode catalogue; adjacent `LICENSE`, `NOTICE.md` |
| `ic-cdk-macros` 0.20.3 | `crates/ic-platform-macros/src/{lib.rs,export/mod.rs,options/mod.rs,signature/mod.rs}` | Adapted generator; crate `LICENSE`, `README.md`; owned external-codec adapters are separate |
| `ic_principal` 0.1.5 | `crates/ic-platform-types/src/principal/` | Owned adapted Principal plus retained optional self-authentication body; `NOTICE.md`, `LICENSE-APACHE` |
| `icrc-ledger-types` 0.2.0 Account | `crates/ic-platform-types/src/account/mod.rs` | Adapted Account representation/text/traits; `LICENSE`, `LICENSE-APACHE`, `NOTICE.md` |
| `icrc-ledger-types` 0.2.0 selected DTOs/helpers | `crates/ic-platform/src/icrc_ledger_types/` | See full module list below; committed `LICENSE-APACHE`, `NOTICE.md`, `provenance.json` |
| `ic-management-canister-types` 0.11.0 | `crates/ic-platform/src/management_canister_types/` | `maintained/mod.rs`, `maintained/transform_func/mod.rs`, `native/mod.rs`, `native/client.rs` and root export scaffolding; maintained Apache license and three provenance records |
| `ic-ledger-types` 0.16.0 account identifier | `crates/ic-platform/src/icp_ledger_types/account_identifier/mod.rs` | Imported derivation in committed source; committed adjacent Apache license, notice and provenance are deleted in the live draft |
| Candid 0.10.40 Nat hash encoding | `crates/ic-platform/src/icrc_ledger_types/hash_encoding/mod.rs` | Committed imported `Nat::encode`, separately retained from the owned wire codec; adjacent committed license is deleted in the live draft |

The ICRC module inventory is:

- `account/mod.rs`
- `allowance/mod.rs`
- `archive/mod.rs`
- `blocks/mod.rs`
- `icrc21/mod.rs`, `icrc21/errors/mod.rs`, `icrc21/requests/mod.rs`, `icrc21/responses/mod.rs`
- `metadata/mod.rs`
- `transfer/mod.rs`
- `value/mod.rs`, including representation-independent hash bodies and its inline
  imported tests/vectors
- Root `mod.rs` export/adapter scaffolding and the separate hash helper above

The optional Principal derivation and ICP account identifier use upstream
RustCrypto SHA-224 implementations; this inventory identifies the DFINITY-derived
integration/derivation source, not an embedded SHA implementation. Base Principal
and Account likewise remain adapted source despite having Platform type identity.

## External schemas adapted into owned DTOs

These are external-schema-derived declarations, separated from copied Rust source.
Their locations and protocol provenance remain relevant even without upstream
Candid traits or an upstream crate edge.

| Local module | Source / license record |
|---|---|
| `crates/ic-platform/src/cmc_types/` | `dfinity/ic` CMC DID; adjacent `LICENSE`, `NOTICE.md` (Apache-2.0) |
| `crates/ic-platform/src/icp_ledger_types/` DTOs | `dfinity/ic` ICP ledger DID; committed adjacent `LICENSE`, `NOTICE.md` deleted in live draft |
| `crates/ic-platform/src/cycles_ledger_types/` | `dfinity/cycles-ledger` DID; adjacent `LICENSE`, `NOTICE.md` (Apache-2.0) |
| `crates/ic-platform/src/certified_assets_types/` | `dfinity/sdk` certified-assets DID; adjacent `LICENSE`, `NOTICE.md` (Apache-2.0) |
| `crates/ic-platform/src/nns_registry_types/` | `dfinity/ic` registry DID; source pinned in `docs/protocols/external-contracts.md`; no separate adjacent license/notice found |

The management and ICRC declarations also describe external contracts, but are
already included in the imported Rust table; do not double-count them as unique
source. The owned general `candid_codec` is not classified as copied upstream
Candid merely because it implements the protocol. Test-only upstream oracles and
the separately retained ledger hash helper are distinct uses.

## Root Cargo registry graph

Exact packages from `cargo metadata --locked --offline --all-features`. There
are 15 DFINITY/IC-stack supplier packages, including transitive and development
edges. The repository field identifies DFINITY for 14; the BLS verifier manifest
has no repository field and is retained as an explicitly selected IC crypto
supplier, rather than assigning every line an inferred author.

| Package | Selected version | Recorded license | Role |
|---|---|---|---|
| `ic-agent` | 0.49.3 | Apache-2.0 | Optional native Agent |
| `ic-transport-types` | 0.49.3 | Apache-2.0 | Optional native protocol/Agent types |
| `ic-ed25519` | 0.6.0 | Apache-2.0 | Transitive native Agent identity implementation |
| `ic-certification` | 3.2.0 | Apache-2.0 | Optional certificate/tree identity |
| `ic-canister-sig-creation` | 1.3.1 | Apache-2.0 | Optional canister signatures |
| `ic-signature-verification` | 0.3.0 | Apache-2.0 | Optional upstream signature verification |
| `ic-verify-bls-signature` | 0.6.0 | Apache-2.0 | Optional BLS supplier; no repository field in selected manifest |
| `ic_bls12_381` | 0.10.1 | `MIT/Apache-2.0` | DFINITY-maintained curve fork; license string copied verbatim from Cargo metadata |
| `ic-representation-independent-hash` | 3.2.0 | Apache-2.0 | Transitive signature/protocol hashing |
| `ic0` | 1.2.0 | Apache-2.0 | Transitive upstream signature capability; distinct from the embedded Runtime bindings |
| `ic_principal` | 0.1.5 | Apache-2.0 | Transitive upstream identities and Types' dev oracle |
| `candid` | 0.10.40 | Apache-2.0 | Transitive signature/native suppliers; direct development oracle |
| `candid_derive` | 0.10.40 | Apache-2.0 | Upstream derive supplier; proc-macro runs on the host, not as a Wasm runtime crate |
| `candid_parser` | 0.4.1 | Apache-2.0 | Development-only schema/parser oracle in this root graph |
| `ic-management-canister-types` | 0.11.0 | Apache-2.0 | Development-only independent DTO oracle in this root graph |

These are available target/feature graphs, not a minimal production closure or a
claim that every crate is embedded in a canister. There are no default features.
Raw IC0/executor selections do not select all these suppliers. Generated fixture
and native-provider workspaces have their own graphs; this root list is not a
replacement for their metadata.

Registry source/license material lives in the Cargo cache under the exact
package names; it is not embedded source owned by Platform. RustCrypto, leb128,
num-bigint, Serde and generic Apache-licensed tooling are not classified as
DFINITY code just because they are upstream dependencies or share a license.

## Browser dependencies

Exact entries from `packages/browser/package-lock.json`, all Apache-2.0:

| Package | Version | Selection |
|---|---|---|
| `@icp-sdk/core` | 6.1.1 | Production dependency; dfinity/icp-js-core |
| `@dfinity/cbor` | 0.2.3 | Production transitive dependency of Core; dfinity/cbor-js |
| `@icp-sdk/auth` | 10.0.2 | Development compatibility dependency; dfinity/icp-js-auth |
| `@icp-sdk/signer` | 6.1.0 | Development transitive dependency; dfinity/icp-js-signer |

The browser's own package declares MIT. Its `src/` adapters import SDK Core
instead of embedding those SDK algorithms. The external licenses remain with
their npm packages. The compiler-floor fixture lock contains only its compiler
selection, not another SDK cohort.

Related but not automatically classified as DFINITY-authored:
`@caffeineai/object-storage` 1.1.2, development compatibility dependency,
Apache-2.0, repository `caffeinelabs/skills/packages/object-storage/frontend`.
Its namespace/license alone is insufficient to assign corporate authorship.

## Native provider and fixtures

- `native/ic-profiler/upstream/ic` is the full DFINITY IC Git submodule at
  `ee8086a9773725fd1058832a9400297cf0990112`. This is an optional source provider,
  outside the five product packages. Its root `LICENSE` names DFINITY, describes
  directory/file license inheritance and Apache-2.0, and references the IC
  Community Source/Shared Community Source variants. This inventory does not
  normalize the entire checkout to a single license.
- Its source-count inventory enumerates 566 packages plus 16 unassigned Rust
  sources. This is the full pinned checkout, **not** the compiled Profiler
  closure; it includes replica, crypto, execution, ledger, governance, networking,
  PocketIC, libraries and tests. The [JSON inventory](dfinity-source-inventory-2026-11-10.json)
  lists the package directories/counts and imported product source files; the
  [text version](dfinity-source-inventory-2026-11-10.txt) lists every package.
  These pre-existing count reports describe their captured
  source snapshot and were left unchanged by this audit.
- `native/ic-profiler/overlay/` has 26 manifest-selected overlay files, and
  `native/ic-profiler/patches/ic/` has ten selected patches. Their exact source
  destinations/digests are in `integration/manifest.json`; they integrate with
  the DFINITY source above. The surrounding Profiler layer retains its own
  Neutrinomic MIT license; that is not a replacement for the upstream licenses.
- `scripts/fixtures/agent-installed/{prime256v1.pem,secp256k1.pem}` contains
  public test keys copied from Agent 0.49.3 fixtures at
  `0857d7dc6a47edbcc754ee41cfc66994ae38b254`. Adjacent `README.md` and
  `LICENSE-APACHE` record provenance. They are test fixtures, not operational keys.
- Imported adapter tests/vectors remain under their respective source/license
  owners above. Owned tests also call registry Candid/parser/Principal/management
  as independent oracles; that is a development dependency use, not another
  embedded algorithm implementation.
- `scripts/fixtures/cdk-runtime/Cargo.lock` records the installed-runtime Testkit
  qualification cohort; its upstream dependencies are fixture/native evidence,
  separate from the product crate's production closure. Disposable archived
  comparison baselines under `target/evidence` likewise intentionally retain the
  old upstream implementation and licenses; they are not a second product owner.

## External tools and retained material

The selected common IC tool catalogue (`ci/ic-tools.tsv`) contains DFINITY/IC
ecosystem tool pins for didc 0.6.2, ic-wasm 0.11.1, Quill 0.5.4 and ICP CLI 1.7.0.
PocketIC is prepared via Testkit for installed qualification. These are external
tool selections, not code embedded in the Rust crates; a catalogue pin alone does
not prove a tool is installed or invoked in every workflow. Binaryen/wasm-opt is
not classified as DFINITY-authored here.

Ignored historical recovery archives, frozen release checkouts and Cargo/npm
caches can retain earlier source/licenses. They are custody/build material,
not additional maintained source owners. This audit neither enumerates each
historical copy nor deletes or alters retained evidence.

## Concurrent license/provenance deletions observed

These tracked files were missing in the working tree at inspection and remain
present in the committed baseline:

- `crates/ic-platform/LICENSE`
- `crates/ic-platform/src/icp_ledger_types/LICENSE`
- `crates/ic-platform/src/icp_ledger_types/NOTICE.md`
- `crates/ic-platform/src/icp_ledger_types/account_identifier/LICENSE-APACHE`
- `crates/ic-platform/src/icp_ledger_types/account_identifier/NOTICE.md`
- `crates/ic-platform/src/icp_ledger_types/account_identifier/provenance.json`
- `crates/ic-platform/src/icrc_ledger_types/LICENSE-APACHE`
- `crates/ic-platform/src/icrc_ledger_types/NOTICE.md`
- `crates/ic-platform/src/icrc_ledger_types/hash_encoding/LICENSE`

The remaining ICRC DTO/value files and their provenance JSON still exist. The
facade manifest still declares `MIT AND Apache-2.0` and includes `LICENSE` in its
package selection. This read-only source audit does not approve the deletions,
attribute independence to the rewrites, or claim the draft is release-ready.
No source, dependency selection, license, notice or sibling checkout was changed.
