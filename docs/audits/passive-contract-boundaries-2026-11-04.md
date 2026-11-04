# Passive contract dependency boundaries — 2026-11-04

## Completed in Platform

The existing workspace and three release groups remain unchanged. Types owns
canonical Principal/Account identity, Runtime owns execution, and the facade owns
cryptography, certification, external protocols and native clients.

One feature correction was necessary: Types principal-external exported the
registry ExternalPrincipal but did not select its upstream Serde implementation.
Canic's unconditional Candid feature masked that missing capability. A passive
record containing ExternalPrincipal failed to derive Serialize/Deserialize with
Types alone (E0277, reproduced before the correction).

The existing principal-external feature now selects ic_principal/serde explicitly.
No defaults, Candid, Agent, Runtime, Wire or hashing are added. Owned identity
selection is unchanged; external conversions retain the exact registry type.
Upstream Serde bodies and cryptographic source are unchanged. The facade already
forwards this feature, so it needs no new feature or export.

A regression exercises external-principal record fields at every supported
principal length using exact Serde byte tokens, Postcard length-prefixed bytes,
CBOR, human-readable JSON and owned/external conversions. Manifest admission
also refuses missing Serde, implicit default features, self-authentication
coupling and a Candid dependency in Types.

This correction is **published in Types 0.18.1**, from commit
`2307ec3e88132913b44c2db201b19c98d8fcca04`, tagged `types-v0.18.1`. Published
Types 0.18.0 already supports owned Principal/Account-only consumers; it does not
supply the standalone external-Serde capability. Registry-only Canic adoption
must select the corrected Types supplier. The initial audit changed no package
versions or sibling sources; subsequent delivery advanced only the Types group.
Platform, Runtime, release-group membership and sibling sources remain unchanged.

## Observed paths and proposed consumer paths

These are package-level normal/build paths, not claims that an entire service
application becomes Candid-free.

| Consumer | Observed current selection | Proposed passive path | Necessary optional facade path |
| --- | --- | --- | --- |
| canic-contracts 0.114 working tree | contracts → Platform 0.21, candid → Types 0.18 + Candid | contracts → Types, principal-external (including upstream Serde after the fix) | external-candid → Platform/candid for Nat conversions and Nat-bearing DTO adapters |
| ic-blob-storage-contracts 0.70 working tree | contracts → Platform 0.21, candid → Types 0.18 + Candid | contracts → Types with no features; existing Wire schema opt-in targets canonical Types | caffeine-candid → Platform/candid for provider request/reply codecs, Int/Nat DTOs and Candid derives |
| ic-crates-protocol 0.2.1 working tree | protocol → Platform 0.8.3/candid; protocol → Blob contracts 0.59.1 → Platform 0.8.3/candid | protocol → Types; protocol → migrated Blob contracts with its Candid capability off | http-gateway → Platform/candid for the standard gateway request/response types |

Full external-API selections still genuinely need the facade. Moving only
identity imports while leaving an unconditional facade dependency does not
contain release churn. Disabling only HTTP in IC Crates is also insufficient:
its old Blob contracts dependency still brings the old facade.

IC Crates currently has no Types package in this selected protocol graph: its
owned Principal comes from Platform 0.8.3. Its legacy Blob 0.59.1 permission fields
use that same old identity. Other application packages in the workspace having
Types 0.18 does not change this protocol boundary.

## Required consumer changes — not implemented here

All feature names below are proposals for the existing consumer crates. Their
owners must implement and qualify them; this is not completed consumer adoption.
Implementation tracking belongs to [Canic #731](https://github.com/dragginzgame/canic/issues/731),
[Blob #115](https://github.com/dragginzgame/ic-blob-storage/issues/115) and
[IC Crates #22](https://github.com/dragginzgame/ic-crates/issues/22).

For identity imports, preserve the distinction between the owned and external
types:

~~~rust
use ic_platform_types::Principal;
use ic_platform_types::principal::ExternalPrincipal;
~~~

Do not replace ExternalPrincipal fields with owned Principal, or owned fields
with external Principal. They have different contracts, including Candid support
and some deserializer admission behavior.

Use inherited workspace declarations, with defaults disabled. Canic's catalog
must require at least Types 0.18.1 rather than leave
an older lock eligible. Blob's existing 0.18.0 requirement is sufficient for
owned-only fields; using the corrected patch as a common minimum is also valid.
Do not move Nat, Int, CandidType, protocol codecs or Wire traits into Types.

For an existing workspace adopting the published correction:

~~~toml
[workspace.dependencies]
ic-platform-types = { version = "0.18.1", default-features = false }
~~~

The optional facade declarations below inherit each consumer's existing facade
cohort; they do not require an unrelated upgrade to Platform 0.22.

### Canic

Evidence: canic-contracts/Cargo.toml:23; src/cycles/mod.rs;
src/ops/unsigned_magnitude/mod.rs; src/dto/mod.rs::prelude.

1. Add Types to Canic's root catalog and make it a direct contracts dependency.
   Replace owned identity imports with Types and the DTO prelude's Principal
   alias with Types' ExternalPrincipal. Keep all existing field types and aliases.
2. Make the facade and num-bigint optional in this crate:

~~~toml
[features]
default = []
external-candid = ["dep:ic-platform", "dep:num-bigint"]

[dependencies]
ic-platform-types = { workspace = true, features = ["principal-external"] }
ic-platform = { workspace = true, optional = true, features = ["candid"] }
num-bigint = { workspace = true, optional = true }
~~~

3. Gate the Nat import, CyclesConversionError and TryFrom<Nat> together. Keep the
   exact overflow error type/payload and checked u128 conversion when enabled.
   Keep Cycles' other constructors, parsing and Serde behavior independent.
4. Gate ops::unsigned_magnitude and the Candid-coupled conversion entrypoints in
   ops::inspection and ops::funding_read. Preserve zero as an empty magnitude,
   reject leading zero bytes, retain Bytes<N> bounds, and accept values above
   u128 where the existing bounded-magnitude API does.
5. Gate Nat-bearing declarations and their dependent adapters, not whole mixed
   modules indiscriminately: CanisterStatusResponse, CanisterSettings,
   MemoryMetrics, QueryStats, the dependent CanisterInspectionOutcome, and the
   Nat-bearing refill response family need the opt-in. CanisterInfo,
   CanisterHistoryResponse, EnvironmentVariable and native refill input records
   can remain passive. Keep existing module paths for enabled APIs.
6. Stop exposing the entire facade unconditionally through __reexports. Preserve
   any required macro identity paths through Types; expose the facade only where
   an enabled Candid adapter actually needs it.
7. Host/Core callers that convert management or ledger values must explicitly
   select external-candid. Passive users and schema-only users must not acquire
   it just by depending on contracts. Canic's current Wire/Auth passive suppliers
   already use canonical Types; do not replace those owners.

### Blob contracts

Evidence: ic-blob-storage-contracts/Cargo.toml:16,25,35; src/provider/wire/mod.rs;
src/provider/onboarding/mod.rs; src/provider/gc/mod.rs; src/dto/account/mod.rs;
src/dto/upload/certificate/mod.rs; src/endpoints/mod.rs.

1. Use the existing root Types catalog for owned identity fields:

~~~toml
[features]
default = []
caffeine-candid = ["dep:ic-platform"]

[dependencies]
ic-platform-types.workspace = true
ic-platform = { workspace = true, optional = true, features = ["candid"] }
~~~

2. Keep provider::wire and the onboarding request encoder and GC Candid decoders
   behind the Caffeine capability, together with the Candid decoder configuration
   they use. Retain their method names, exact fields and original work/header/
   skipping limits. Low-level decoder APIs must not silently acquire new policy.
3. Make the CandidType derive and candid_path attribute on
   CaffeineUploadCertificateResponse conditional; its two String fields can
   remain passive. Preserve the original derive body/field names when enabled.
4. Gate complete Int/Nat-coupled account DTOs and their dependent response/endpoint
   family as needed. Keep signed and arbitrary-width provider observations;
   do not substitute u128 or change the canonical decimal Serde adapter.
5. Separate Caffeine callback entries and Candid-coupled account entries from the
   owned endpoint catalog/macro paths. Select dependency features at the defining
   crate, not by accidentally testing the invoking crate's feature namespace.
   Do not renumber method IDs, conditionally remove fields/variants from a shared
   serialized shape, or turn provider Candid methods into Wire endpoints.
6. Keep provider-json and wire-schema independent where their existing code
   permits it. Full-service schema generation must explicitly select the same
   Caffeine capability as the service. Hashing/manifest/identity helpers under
   identity::caffeine are not Candid codecs and must not be gated just because
   they share the provider name.
7. Move test-only self-authentication selection to Types' dev dependency.
   Keep native candid-service dev selection only for actual DID-generation tests.
   Dev/test feature unification is not evidence for a minimal normal graph.

### IC Crates protocol

Evidence: ic-crates-protocol/Cargo.toml:14–23; src/lib.rs:6,12;
src/http/mod.rs; src/publication/mod.rs. Root catalog currently selects
Platform 0.8.3, Wire 0.7.25 and Blob contracts 0.59.1.

1. First adopt a published Blob contracts supplier that implements the Types/
   Caffeine split above. The currently selected 0.59.1 cannot provide that split.
2. Add a direct Types dependency and change publication identity imports to
   Types. Align nested UploadAdmissionRequest and all caller imports with that
   same canonical supplier. Keep existing fields and conversions.
3. Make the facade optional and select it through an http-gateway feature.
   Gate both mod http and its HttpRequest/HttpResponse re-exports. Registry
   configuration, publication and Cargo/index helpers do not need this feature.
   Gateway canisters/native bridges must select it explicitly.
4. Retain the genuine gateway Candid: method/url/headers/body and optional u16
   certificate_version; response u16 status_code/headers/body and optional bool
   upgrade. Preserve Vec<u8>, optionality, names and Candid derive paths.
5. Replace the retired facade wire-schema feature edge with the matching
   Wire-owned Types integration. Do not blindly upgrade the root facade/Wire
   catalog for every application package: this is an old identity cohort.
   If moving Wire 0.7 to a current cohort is required, qualify its frames and
   schema separately. No cross-version Wire byte compatibility is established
   by this audit; retain existing contract IDs/bytes unless an explicit protocol
   migration is required and authorized.

### Legitimate dependencies and limits of the cut

IcyDB's icydb-core uses Platform/blake3 for predicate fingerprints and commit
hashing. Keep that edge; it is not an identity-only dependency. Execution users
keep the independent Runtime owner. Native clients and required external Candid
interfaces keep their explicit facade selections.

Canic also directly hashes network/component/release identities, and Blob directly
hashes content/provider manifests. Removing their facade identity edge does not
make those entire libraries crypto-free. Those operations remain consumer-owned
and unchanged. A strict field-only profile would also need explicit consumer
gates for those hashing operations; it must not be advertised as crypto-free
while they remain selected. Types' owned/external identity selections themselves
are codec/execution/hash-free, as checked here.

Cargo features are additive: selecting Candid in another part of an application
can legitimately unify additional upstream features. Qualify the passive package
selection separately from a full service graph. Keep all live external-interface
callers enabled when making capability defaults empty; removing default APIs is
a consumer release/feature migration, not an automatic compatible rollout.

For Canic and Blob already using Types 0.18, direct imports preserve Rust type
identity because the facade re-exports those exact definitions. IC Crates' old
Platform-owned identity does change Rust identity when it adopts Types. Rebuild
its nested permission and caller cohort. Audit Account-containing retained data
against Types 0.18's existing subaccount byte-format cut; do not apply a
Principal-only conclusion to Account bytes. No stored messages, endpoint IDs or
schema definitions were rewritten here.

## Focused evidence and remaining qualification

Evidence is retained under target/passive-contract-audit. All sibling source
files, package/root manifests and lockfiles observed before and after the checks
were byte-identical. Canic and IC Crates had concurrent dirty work; none was
staged, edited or released. Build outputs for their selected tests were directed
into Platform's target directory.

Completed checks:

- Native and Wasm normal/build closures for Types base, Types principal-external
  and facade principal-external contain no Candid, Agent, execution or crypto.
  Their package sets are identical before/after the Serde feature correction.
- Facade candid and blake3 closures retain only their explicitly selected
  capabilities. The BLAKE3 edge remains intact.
- Types Clippy with warnings denied; 16 identity/Serde/account tests pass,
  including the regression that failed before the feature correction.
- Facade selected Clippy and three principal/external-consumer tests pass.
  Six Candid boundary tests preserve explicit external bytes.
- Feature-contract mutations refuse the five Types boundary violations, and the
  existing Principal cryptographic source/vector/license guard is unchanged.
- Current Canic conversion checks: converts_wire_nat_only_when_it_fits (1 test)
  and ops::unsigned_magnitude::tests (3 tests).
- Current Blob provider check:
  explicit_full_width_terms_match_independent_current_cashier_candid (1 test),
  compared with the existing independent Caffeine request.hex fixture.
- Current IC Crates check:
  gateway_records_retain_candid_binary_bodies_and_upgrade_fields (1 test).

These consumer checks cover their current source and published dependency
selections, not a completed migration to the proposed optional features. Owners
must additionally qualify passive defaults, the opt-in external profiles and
host schema generation, including frozen Serde/Wire/Candid vectors and the
numeric limits above. The initial audit ran no blanket fleet matrix, full CI,
installed canister qualification or release.

The subsequent Types 0.18.1 release passed all twenty local Linux release stages,
including installed Runtime/Timers composition and final candidate graph checks.
Receipt: `target/release-validation/qualification.LfiUP1/receipt.json`.
Publication: `target/publication/publish.selection.2WZvBD`.
The downloaded registry archive, published index checksum and locally verified
archive match; evidence is in `target/types0181-publication-verification/`.
GitHub main and the annotated Types tag resolve to the release commit above.
This supplies the correction; it does not implement or qualify sibling adoption,
or establish native macOS or hosted CI results.

