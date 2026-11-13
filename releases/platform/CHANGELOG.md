# Platform changelog

This group covers ic-platform. The retired PocketIC [ledger](../pocket-ic/CHANGELOG.md) preserves its history.
Earlier coordinated history is retained once in [history](../history.md).
See also the [Types ledger](../types/CHANGELOG.md).

## [0.32.1] - Unreleased

### 🔧 Changed

- Adopt Toko Tooling 0.41.0 and its package inventory command. Remove the unused
  facade catalog alias while retaining its release-group membership; keep
  generated-fixture selections in contributor metadata instead of Cargo dependency
  declarations. Preserve archived-catalog reading and refuse duplicate selections.
- Apply common Cargo dependency/inheritance lints and Rustdoc table admission to
  both workspaces and generated consumers. Enforce Make/publication Clippy
  warnings through Cargo without changing compilation flags; qualify refusal on
  fresh and cached artifacts. Keep singleton graph checks separate from empty
  compilation and preserve real cross-generation type/refusal checks.
- Classify native-only Agent/asset consumer tests' Platform edges as
  dev-dependencies. Remove the empty Wasm check for the native asset tests;
  portable consumer libraries retain normal edges and feature-graph checks.
- Adopt Testkit 0.102.0 and required Host 0.77 selections for native qualification;
  its registry Platform 0.32 / Types 0.20 generation stays separate from local
  product identity. Refresh the private BLS baseline to published Platform 0.32.0
  and keep canister dependencies off its native driver graph.
- Review first-party registry freshness before capture. Toko 0.41.0 and Testkit
  0.102.0 are current. Defer optional qualification Wire 0.10.0 → 0.21.5 and
  release-editor IC Build 0.73.0 → 0.84.0 migrations to the next dependency
  maintenance batch; retain this release's qualified schema/editor contracts.

## [0.32.0] - 2026-11-13

### ⚠️ Breaking

- Require Rust 1.100.0 and adopt the common Rust/Clippy/Rustdoc lint policy across
  product, qualification and generated consumers. Mark query results as must-use
  and preserve narrowly reviewed callback/codec ownership and safety boundaries
  ([#157](https://github.com/dragginzgame/ic-platform/issues/157)).

### 🔧 Changed

- Select published Toko Tooling 0.40.1 in the product and qualification
  workspaces, preserving the installed entrypoints. Apply its directory-module
  layout to codec, signature tests and generated Runtime fixture sources without
  changing public module paths or behavior. Preserve standalone formatting coverage
  and qualification input-custody checks after the moves.
- Adopt deployment-latency guidance while retaining the complete required release
  gates and qualified evidence contracts for reuse.
- Adopt 0.40.1's deployment sequencing and phase-specific recovery guidance:
  qualify changed wiring first, retain one full-gate owner and original successful
  evidence, and explain unsupported recovery and rerun scope before retrying.
- Retain compatible native-client lock updates, including rustls-webpki 0.103.17;
  the owned production Wasm dependency selections are unchanged by that refresh.

### 🧪 Testing

- Review release suppliers before freezing: Testkit 0.101.0 and Toko 0.40.1 are
  the selected published releases. Wire 0.21.5 and Build 0.83.0 are available;
  keep the older fixture codec and metadata editor until their separately
  qualified migrations, without inserting optional upgrades into this batch.
- Advance the private qualification benchmark's registry Platform baseline to
  published 0.31.3, matching the local candidate. This private benchmark update
  alone needs no product publication.

## [0.31.3] - 2026-11-12

### ✨ Added

- Add native certificate/witness-pair authentication under Agent and strict CBOR
  capabilities. Preserve Agent's certificate, delegation and freshness checks,
  the caller’s trust root and explicit combined byte bound;
  return only a witness bound to the exact canister’s certified-data commitment.

### 🔧 Changed

- Adopt Toko Tooling 0.39.15 and expose its canonical `make deps-update` command
  for sequential root/qualification Cargo updates, with optional package/version scope.
- Adopt published Testkit 0.101.0 for native qualification and benchmarks; retain
  its upstream PocketIC identity and native-only registry management codec.
  Advance the qualification benchmark's published Platform baseline to 0.31.2,
  matching its prepared-key API to the local candidate, and retain compatible lock updates.
  PocketIC's exact thiserror 2.0.18 requirement remains confined to native
  qualification; the product workspace already selects 2.0.21.

### 🩹 Fixed

- Include the dependency-maintenance Make entrypoint in release-routing fixtures
  so the full gate exercises the current installed command surface.
- Treat the known workspace crypto-bigint advisory as an expected CI finding,
  retaining its full raw evidence and failure status. Keep runtime findings,
  additional advisories and incomplete/tool failures blocking. This disposition
  leaves [#50 — Resolve the crypto-bigint advisory at the Agent boundary](https://github.com/dragginzgame/ic-platform/issues/50)
  unresolved and adds no global audit exemption.

### 🧪 Testing

- Refresh the pre-freeze supplier review: Testkit 0.101.0, Toko 0.39.15 and
  Host helpers 0.76.4 are current published selections. Defer optional fixture
  Wire 0.10/0.12.1 → 0.21.3 to the next Wire fixture migration, where its codec
  and identity contracts can be qualified together; these are not product edges.
  Defer editor 0.73 → 0.83 to the next release-adapter maintenance batch; the
  selected editor retains the qualified metadata transaction. Revisit both at
  the next applicable dependency review rather than treating them as exemptions.

## [0.31.2] - 2026-11-11

### 🔧 Changed

- Select Toko Tooling 0.39.12 for development, with terminal-width-aware LOC
  labels, an explicit wide report option and supplier freshness reviews.
- Adopt Testkit 0.100.2 for installed qualification and benchmarks, using its
  upstream async PocketIC re-export and preserving public identity checks.

### 🩹 Fixed

- Include the independent benchmark lock in release metadata transactions so
  local product identities advance without moving the historical registry baseline.
- Prepare the required Wasm optimizer in hosted Runtime qualification.

### 🧪 Testing

- Review supplier freshness before freezing this batch: Testkit 0.100.2,
  Toko 0.39.12 and native Host helpers 0.76.4 are current. Keep the intentional
  Platform 0.30.0 comparison baseline. Defer optional Wire 0.10/0.12.1 → 0.21.2
  fixture migration to the next qualification-dependency maintenance batch and
  metadata editor 0.73 → 0.83 to the next release-adapter maintenance batch;
  revisit both selections before the next release freeze.

## [0.31.1] - 2026-11-11

### ✨ Added

- Accept caller-owned prepared roots in canister-signature verification, with and
  without authenticated signing time. Recheck certificate/delegation signatures,
  issuer ranges, witnesses and identity-key refusal on every call; consumers retain
  trust-root selection, freshness and preparation custody.

### ⚡ Performance

- Bulk-write Candid byte vectors and arbitrary integer payloads, avoid copying
  positive integer magnitudes, reuse schema-refinement buffers and resolve final
  type references directly. Preserve exact bytes, output limits and custom mappings.
- Use a fixed stack buffer for certificate signing-message framing. Consolidate
  vector/slice encoding and 22 order-compatible management DTO declarations without
  changing field/variant order, Serde bytes or the decoder's admission budgets.

### 🧹 Cleanup

- Remove the unused `leb128` production dependency from ICRC ledger support,
  preserving its public types, codecs and protocol bytes.

## [0.31.0] - 2026-11-11

### ✨ Added

- Let `candid_record!` and `candid_variant!` declare concrete Rust DTOs and owned
  codec mappings together, retaining visibility, docs, derives and attributes.
  Check numeric label order/uniqueness at compile time and preserve existing wire
  bytes, optional admission, unbounded naturals, recursion, callbacks and decoder
  limits. Add native/Wasm standalone consumer qualification without registry
  Candid or new dependencies. Fixes [#156 — Define DTOs and codec mappings together](https://github.com/dragginzgame/ic-platform/issues/156).

### 💥 Breaking

- Adopt the independently qualified BLS wrapper in `ic_platform::bls`. Signature
  and deserialization errors now have Platform identity; their bytes and error
  variants remain unchanged. Registry verifier callers exchange serialized bytes
  explicitly. Add caller-owned `PreparedPublicKey` for repeated verification.
  Use SHA-2 0.11 with external curve/hash-to-curve/pairing primitives and a Digest
  trait adapter; the old verifier remains dev-only or a native Agent transitive.
  Certificate verification uses this same checked boundary, retaining subgroup
  and identity-key refusal at root/delegated keys without a duplicate point parse.

### 🩹 Fixed

- Adopt Toko Tooling 0.39.10's narrowly scoped Digest-adapter admission and
  include the private BLS benchmark workspace in formatting and pin checks.
- Let hosted Wasm regression checks compare deployments with changed dependencies,
  retaining each revision's lock selection and reporting external package/feature
  changes alongside absolute total/code-section bytes. Keep strict comparisons
  available, require identical nested probes and build settings, and preserve the
  growth threshold in both modes. Include self-authenticating Principal in the
  default probes so its supplier changes are measured.

## [0.30.0] - 2026-11-11

### 💥 Breaking

- Retire textual DID completely: remove `candid_codec::service`, its native
  export/check workflow, text fixtures and the `candid_parser` test dependency.
  No declaration-Wasm exporter or native inspector replaces it. The maintainer
  declined [#154](https://github.com/dragginzgame/ic-platform/issues/154) and
  [#155](https://github.com/dragginzgame/ic-platform/issues/155). Consumers must
  remove their DID build/sidecar requirements. Binary Candid codecs, DTOs,
  callback schemas and wire bytes are unchanged; independent binary fixtures
  retain the former schema and value comparisons. Remove stale parser-consumer,
  service-registration and upstream-decoder instructions from current docs and
  retire parser activation scaffolding in the feature checker.

## [0.29.3] - 2026-11-10

### 🩹 Fixed

- Return an error when converting an empty Value array to an Account instead of
  panicking. Int now accepts its existing sign/magnitude Serde representation,
  restoring JSON/CBOR roundtrips without changing serialized or Candid bytes;
  malformed pairs and trailing tuple elements remain rejected.
- Reject incorrect fixed-array lengths as malformed values, including inside
  `Option<[u8; 32]>` subaccounts and nested options. Invalid lengths no longer
  disappear as `None`; valid arrays, absent options and ordinary incompatible-type
  fallback retain their behavior.

### 🔧 Changed

- Replace ICRC hash composition and ICP account framing with independently
  authored protocol adapters over the existing SHA/CRC providers. Preserve
  exact outputs, map ordering and legacy integer limits; hash ICRC3 values
  directly without first rebuilding a Value tree. Independent public comparisons
  replace source-fingerprint checks.
- Replace non-cryptographic Value/ICRC3Value helpers through a specification-only
  context, preserving conversions, errors, formatting and external bytes.
- Replace passive CMC, ICP Ledger, Cycles Ledger and certified-assets declarations
  through a specification-only context, preserving public types and Serde/Candid
  behavior. Replace four full service DID fixtures with independent schemas for
  the 14 supported methods, checked against original contracts and frozen public
  observations.
- Adopt Toko Tooling 0.39.8, explicitly retain grouped-release entrypoints, and
  remove installed shared reference guides through the exporter. Organize native
  docs by topic and update dependency/source ownership statements and checks.
- Replace all selected shared and native management DTO declarations through a
  specification-only context. Preserve public exports, traits, numeric ranges,
  floating-point bits and Serde/Candid bytes. Replace source fingerprints with
  frozen public observations and registry schema/value comparisons.
- Replace the remaining passive ICRC metadata/transfer/allowance/block/consent
  declarations and helpers through a specification-only context. Preserve public
  fields, variants, ranges, traits, diagnostics and Serde/Candid bytes; document
  fields instead of retaining blanket documentation-lint exceptions. The imported
  value/hash implementation and its source guards remain unchanged.
- Replace imported ICRC Account/archive adapters through a specification-only
  context. Delegate account identity behavior to Types, preserve detailed text
  errors and Serde/Candid bytes, and move canonical archive Principal fields
  directly. Retain the other ledger imports and hash-source guards.

- Independently implement canister-signature key framing and protocol verification
  with canonical Principal, while retaining external BLS, SHA-256 and certified-tree
  primitives. Remove Candid and raw IC0 from production signature profiles;
  preserve DER/identity-key admission and extract authenticated time without
  decoding the certificate twice. DFINITY signature packages remain test-only
  comparison and benchmark references.

### 🛠️ Tooling

- Retire the instrumented Profiler Git source provider, IC submodule, Python
  assembler and provider-only CI/fixtures. Existing immutable consumer pins
  retain their source; future provider ownership belongs outside Platform.

### ⚡ Performance

- Move the owned blob buffer when converting Value to Vec instead of copying it;
  format ICRC3Value through borrowed children without cloning the complete tree.
- Remove temporary padding strings and decoded vectors from external ICRC Account
  text parsing. Preserve Unicode error offsets, oversized-input error precedence
  and checksum diagnostics with fixed scratch buffers. No speed or Wasm-size
  claim is made for this change.
- Replace the copied Nat hash-input encoder with a specification-only unsigned
  LEB128 writer using borrowed digits and 64 bytes of stack scratch. Preserve
  arbitrary magnitudes, canonical bytes and IO failures without cloning or
  allocating a full-magnitude temporary; ledger hash bodies remain unchanged.
- Reject mismatched fixed-size array lengths before allocating a temporary
  vector or decoding its elements. Preserve valid bytes; an incorrect length
  now takes precedence over later faults.
- Remove temporary function-mode copies and unnecessary schema-merging work
  for simple encodings. Keep decoder budget charges and external bytes unchanged;
  measured probes shrink further, with the native timing tradeoff recorded in
  the codec audit.

### 🧪 Testing

- Make the existing 28-schema ICRC oracle runnable with ICRC alone and share its
  comparison helper. Add public conversion/range/default tests and frozen
  before/after JSON, CBOR and Candid observations for the passive DTO families.
- Replace copied native Agent PEM fixtures with newly generated test keys and
  retain upstream cryptographic parsing, verification and wrong-root checks.
- Expand the shared codec comparison catalogue to 101 valid and 16 discard or
  admission cases, and the Wasm comparison to seven shapes. Keep known upstream
  admission differences explicit and record slower encoding cases alongside wins.

- Qualify array admission and optional fallback budget retention, and add an
  isolated array Wasm probe. Record current upstream/owned codec footprint and
  throughput comparisons in the [codec audit](../../docs/benchmarks/candid-codec-2026-11-10/README.md).

## [0.29.2] - 2026-11-09

### 🩹 Fixed

- Expose upstream `agent::EffectiveId` for native canister/subnet routing without
  a direct Agent dependency. Preserve upstream type identity and routing behavior.

## [0.29.1] - 2026-11-09

### ➕ Added

- Export native external-service DIDs from owned mappings, with bounded output,
  explicit argument/result sequences and method modes. Qualify frozen external
  Caffeine contracts with pinned didc structural equality; no parser dependency,
  registration or Wasm surface is added. Document the caller migration for
  [#153 — Document the native external-service schema/export owner](https://github.com/dragginzgame/ic-platform/issues/153).

### 📚 Documentation

- Record completion of the supplier/consumer qualification separation already
  delivered in 0.28, with source identities, actual crossing checks and focused
  regression evidence:
  [#152 — Separate supplier qualification from downstream adoption](https://github.com/dragginzgame/ic-platform/issues/152).

## [0.29.0] - 2026-11-09

### ➕ Added

- Add caller-selected vector bounds checked before allocation, strict option
  decoding, and independent skipped-work budgets to the owned Candid reader.
  Existing entrypoints and encoded bytes remain unchanged; consumers can reject
  all skipped values while selecting separate limits for nested fields.

### 🛠️ Tooling

- Adopt Toko Tooling 0.38.2 and audit product/qualification package placement.
  All five maintained crates already use the product layout; disposable fixture
  templates and vendored upstream sources retain their documented locations.
  Production boundaries and the three release groups remain unchanged.

## [0.28.0] - 2026-11-09

### ⚠️ Breaking

- Upgrade the owned codec and ledger `num-bigint` selection to 0.5.1. Public
  `Nat.0`/`Int.0` and bigint conversions now use that generation; downstream
  code accessing those fields must select matching bigint types. Preserve
  arbitrary numeric ranges and Candid/Serde bytes. Optional upstream suppliers
  retain their independent 0.4 selection.

- Remove `principal-external`, `ExternalPrincipal` / `ExternalPrincipalError`
  aliases and registry conversion bridges. Platform-owned DTOs, ledger accounts,
  callbacks and certified-controller results now use canonical Types Principal.
  Native Agent and transport APIs retain supplier identities.
- Remove `candid`, `candid-parser`, `candid-service` and `candid-endpoint`,
  including upstream trait/derive forwarding, dynamic values, reflection,
  text parsing, service registration and printer APIs. No compatibility feature
  remains. Owned DTOs no longer implement upstream CandidType.
- Replace upstream Nat/Int/function-reference carriers in owned contracts with
  Platform codec carriers. Numeric ranges and qualified Candid/Serde bytes are
  preserved, but Rust identities, diagnostic errors and resource accounting change.

### ➕ Added

- Own native install-code, provisional-create and log request/reply DTOs required
  by Testkit. Preserve registry 0.11.0 fields, Candid/Serde bytes and independent
  source guards without widening Wasm or restoring the registry dependency.

- Provide opt-in `candid-codec` typed binary encoding and bounded decoding,
  explicit argument arity, record/variant mappings, recursive containers,
  canonical Principal, arbitrary integers and checked callback signatures.
- Provide `candid-codec-endpoint` query/update adapters through Runtime's existing
  generator, with explicit decode/reply budgets and argument policy.

### 🔧 Changed

- Share SHA-2 0.11.1 between ledger hashing and owned Principal self-authentication.
  Preserve the imported SHA-224/SHA-256 hash bodies, domains and fixed vectors.
  Remove Platform’s direct SHA-2 0.10 selection; native/signature suppliers still
  retain their older dependencies.

- Use the owned codec for registry, CMC, ICP, ICRC, Cycles, asset and Wasm
  management contracts. Preserve qualified external schemas, ledger hash inputs,
  account-identifier derivation and cryptographic verification. Direct Candid/parser dependencies are development oracles only;
  native and signature suppliers can still select Candid transitively.

### 🧭 Migration Notes

- Replace external Principal annotations with canonical `Principal` and construct
  owned Nat/Int/Func carriers directly. Select the required DTO feature and use
  `candid_codec` encoding/decoding with explicit limits and arity. Replace legacy
  endpoint attributes with the owned codec adapters. Text parsing needs a separate
  native parser owner. Requalify consumer budgets; old upstream quotas are not
  equivalent to the owned work/allocation limits. No storage reset or external
  protocol migration is required by the preserved bytes. See the
  [migration contract](../../docs/architecture/migration.md).

### 🧪 Testing

- Identify selected canister, native driver and Testkit infrastructure owners
  separately in qualification evidence (#152). Admit actual Agent/CanisterId
  identity crossings without forcing Testkit's older facade onto local sources.

- Retain frozen schemas, byte fixtures, independent upstream oracles, malformed
  input/resource-limit tests and installed endpoint qualification. Add repeatable
  Wasm comparison against the published implementation; the management-status
  probe saves 501,988 optimized uncompressed bytes (84.9%) before the final
  Principal cut. This is workload evidence, not a whole-canister size guarantee.
- Adopt published Toko Tooling 0.37.1 and its corrected LOC classification;
  distinguish portable production, native production and development dependencies
  in the root catalog without changing product feature activation.

### 📚 Release preparation

- Native management DTO ownership is being migrated concurrently. Finalize its
  Rust identity changes, consumer migration instructions and qualification before
  releasing this pending batch; the earlier native registry-identity guarantee
  must not be carried into the release without reviewing that completed work.

### 🛠️ Tooling

- Adopt published Toko Tooling 0.37.4 with corrected conditional/development
  dependency reporting, its standalone baseline, and current direct dependency
  selection independent of older DFINITY transitive versions.

- Remove downstream Timers composition from Platform release admission and
  qualification-lock custody. Retain standalone Runtime and singleton-refusal
  coverage; Scheduling owns scheduler compatibility tests.

- Adopt published Toko Tooling 0.37.1, migrating to `TOKO-TOOLING.md`,
  `.toko-tooling.snapshot`, `toko-tooling-*` commands and Toko LOC metadata.
  CI and publication adapters admit the renamed snapshot before effects;
  existing product release requirements remain unchanged.

## [0.27.1] - 2026-11-08

### 🔧 Changed

- Preallocate complete small Candid argument buffers for unit lists and text,
  preserving bytes, arity and UTF-8/LEB128 boundaries without new dependencies.

### 🛠️ Tooling

- Select published `toko-tooling` 0.36.0 once in root workspace dependencies,
  inherited as a development dependency by all five real members. Remove the
  superseded metadata/separate-lock selection, retain Cargo.lock and preserve
  snapshot entrypoints and unrelated application selections.
- Admit the tooling declaration, lock and installed snapshot before custom
  publication, release metadata and frozen setup effects. Production dependency
  graphs, package versions and product release requirements are unchanged.
- Share immutable source capture between runtime and raw_rand qualification,
  retaining separate matched Wasms and installed assertions. Parse runtime
  options once, accepting either order and refusing conflicting selections.
- Reuse Shared's regular-file and streaming digest helpers for release evidence,
  with one owned subprocess wrapper and unchanged receipt/recovery contracts.
  Keep their full helper closure in compiler-floor qualification fixtures.
- Use Shared's validated journal projection for resume policy checks, retaining
  policy-drift refusal and completed-plan behavior without positional parsing.

## [0.27.0] - 2026-11-08

### ⚠️ Breaking

- Remove `candid::protocols::management::raw_rand` without an alias. Select
  `raw-rand` and use the sole typed `raw_rand::{METHOD, encode, decode}` boundary;
  encoding is infallible and decoding returns exactly `[u8; 32]`. General raw
  transport remains method-agnostic; other external Candid protocols are unchanged.

### ➕ Added

- Add dependency-free `candid_args::{empty, units, text}` for the deliberately
  small subset of empty, unit/null and single-text external arguments. Share
  empty framing with entropy and existing protocol helpers, preserving Candid
  bytes without selecting the general encoder or parser.
- Add the dependency-free `raw-rand` feature and `raw_rand::{METHOD, encode,
  decode}` for management entropy. The bounded decoder accepts the documented
  single-blob producer profile, including padded LEB128 framing and all-zero
  results; callers retain transport, wait/retry and seed policy
  ([#151 — Candid-free raw_rand codec](https://github.com/dragginzgame/ic-platform/issues/151)).

### 🧪 Testing

- Qualify the narrow entropy codec against reference Candid, malformed input,
  native/Wasm feature graphs and installed management calls. A matched entropy
  consumer shrinks from 376,466 to 44,796 total uncompressed Wasm bytes and
  343,496 to 38,114 code-section bytes with Rust 1.99.1, Candid 0.10.40 and the
  existing opt-level 3 / fat-LTO / `ic-wasm shrink` recipe. These are focused
  consumer results, not a saving claim for canisters that retain other Candid use.

## [0.26.1] - 2026-11-08

### 🧹 Changed

- Derive owned Candid admission and signing-time errors through the existing
  `thiserror` supplier, preserving variants, diagnostics and original sources.
  Cryptographic verification and wire admission behavior are unchanged.
- Refuse oversized native protocol-fixture payloads before cloning them,
  preserving encoded bytes, caller-selected output budgets and error variants.

### 🛠️ Tooling

- Adopt Shared 0.31.11: format only Cargo workspace members and use
  `make cloc CLOC_DEPTH=0` for crate rows or `CLOC_DEPTH=1` for module rows.
  Crate/module selectors narrow the report. Private tooling maintenance does
  not require a crate release.
- Cache authenticated pinned downloads before hosted tool setup and save after
  setup. Run one complete-history, redacted secret scan in repository CI,
  including documentation changes, independently of host and release matrices.
- Retire the unadopted Candid prototypes while preserving immutable reproduction
  sources, licenses and historical results. Separate current migration contracts
  from their earlier investigations.
- Share benchmark preparation, admission-test tools and fixture copying; batch
  release-lock observations and retain dependency-graph refusals in one pass.
- Organize installed qualification by scenario and declare fixture method names
  and IDs once, preserving their bytes, assertion order and lifecycle checks.
- Reuse one admitted release plan per adapter operation and one root dependency
  catalog per generated fixture; select the policy-fixture compiler from the
  admitted workspace floor. Keep later phases and graph observations fresh.
- Share Wasm framing measurements, graph classifications, generator-test
  assertions and Rust/browser response fixtures. Retire dated 0.4/0.22 runners
  with immutable reproduction references, retaining read-only Canic capture and
  reporting. Use Shared's cached common-tools action for full Platform CI only.
- Bind captured fixture bytes and file rosters to qualification evidence, inspect
  one Wasm snapshot per report and retain benchmark executables under the shared
  artifact lock. Isolate feature metadata per invocation; reuse completion-aware
  cleanup and admitted compiler/catalog selections in the remaining fixtures.
- Require every Rust/browser response-corpus case and share the remaining Agent
  and dynamic fixture hex decoding without changing retained protocol vectors.

## [0.26.0] - 2026-11-08

### 💥 Breaking

- Use owned management DTOs and the owned codec on native as well as Wasm. Remove
  the production registry management dependency; retain all existing native
  snapshot transfer exports and external bytes. Native consumers migrate their
  Principal/Nat identities and Candid encoding/decoding calls with explicit budgets.

- Retire the pure BLAKE3 feature and forwarding module after IcyDB and Wire
  removed their source callers. Existing hashing consumers select the same
  upstream implementation directly; their algorithm, features and bytes stay
  unchanged ([#150](https://github.com/dragginzgame/ic-platform/issues/150)).

### 🧹 Cleanup

- Share exact Candid reply-arity admission and protocol decoder generation while
  preserving diagnostics, budgets, optional-result behavior and external bytes.
- Consolidate test data decoding, native fixture setup and source-fingerprint
  normalization. Keep separate source guards, refusal tests and explicit cleanup.
- Render runtime qualification manifests in one pass and make repeated copy
  measurements opt-in. Ordinary qualification retains both copy paths at every
  payload size, with ten queries instead of sixty.

## [0.25.1] - 2026-11-07

### 🧪 Testing

- Adopt the published async Testkit client in native fixtures, with explicit
  finite request budgets, one caller-owned runtime and awaited teardown. Preserve
  installed protocol, executor and cryptographic integration coverage
  ([#148](https://github.com/dragginzgame/ic-platform/issues/148)).
- Reuse unchanged Linux release-policy fixtures through Shared's exact-input
  proofs and fresh per-group admissions. Keep every required release stage,
  version-dependent check and recovery obligation
  ([#149](https://github.com/dragginzgame/ic-platform/issues/149)).

### 🔧 Changed

- Adopt Shared Tooling 0.31.6 and its approved direct pure-hashing policy.
  BLAKE3 facade retirement remains gated on caller migration
  ([#150](https://github.com/dragginzgame/ic-platform/issues/150)).

## [0.25.0] - 2026-11-07

### 💥 Breaking

- Remove ordinary facade entrypoint macros and its `cdk` feature. Select
  `ic-platform-runtime/cdk` and import entrypoints from its canonical owner;
  explicit external-Candid adapters remain in Platform.

### 🔧 Maintenance

- Require the complete release gate for every group; retire the inactive
  selected-stage pilot and its native qualification workflow. Receipt, source,
  metadata and recovery admission remain mandatory.
- Preserve real preparation/rollback/custody checks for all three groups while
  running the complete metadata-fixture delivery/recovery cycle once, for Runtime.
- Pin the immutable Timers qualification source that also accepts Runtime 0.20,
  retaining the existing implementation and all installed composition checks.
- Resolve qualification locks offline inside metadata preparation, preserving
  foreign selections and necessary inactive registry owners across minor bumps.
  Exercise actual post-version graphs in the preparation fixture, and retain
  consumer failure/concurrent-edit checks with the current admission profiles.

### 🔧 Tooling

- Adopt Shared Tooling 0.31.5, including release-checker isolation and grouped
  dispatch, clearer refusal after active LOC scratch is removed, and the rule
  that output owners create their required directories on every fresh run.

## [0.24.0] - 2026-11-07

### 💥 Breaking

- Remove native offline IC public-key derivation, its `secp256k1` feature and
  the `ic-secp256k1` dependency. Consumers must remove the feature/imports;
  [Canic’s fetch-and-cache migration](https://github.com/dragginzgame/canic/issues/739)
  remains consumer-owned. The separate `ecdsa` capability and Agent are unchanged.
- Remove unconfigured `candid::decoder::{decode_one, decode_args}` shortcuts.
  Select the bounded helpers or an explicit decoder configuration. Caller
  migrations remain in [Canic #740](https://github.com/dragginzgame/canic/issues/740),
  [Blob #119](https://github.com/dragginzgame/ic-blob-storage/issues/119) and
  [IC Crates #25](https://github.com/dragginzgame/ic-crates/issues/25).

### 🔧 Tooling

- Stop replaying Shared's generic changelog-parser regression matrix in
  `check-release-commands`; retain Platform's group preparation, adapter and
  recovery coverage and normal release changelog admission.
- Retire the remaining unadopted crypto packaging patches; registry suppliers
  and their cryptographic implementations remain unchanged.

## [0.23.6] - 2026-11-07

### 🩹 Fixed

- Adopt Shared 0.31.1's Bash 3.2 checksum-parser correction for macOS tool setup.
- Use Shared's portable checksum helper throughout the optional frozen-batch
  qualifier, preserving archive checksum records without requiring GNU tools.
- Refuse runtime-lock checks and preparation when the release-group roster
  reader fails, including partial output, before changing either fixture lock.
- Refuse embedded IC0/executor source verification when its checksum-roster
  reader fails or emits no entries, including failure after partial output.

### ⚡ Performance

- Store Candid admission's type-child references in one shared buffer, preserving
  traversal order, wire admission, work quotas and the 16-level nesting ceiling.
  The matched external-Candid probe shrinks 372 total uncompressed bytes and
  402 code-section bytes. A 1,024-type header uses 44% fewer IC instructions;
  simple recursive refusals cost 161 additional instructions in the same probe.

### 🔧 Tooling

- Project adjacent release fields from one admitted group plan per command;
  retain fresh admission at subsequent phases and all rollback/recovery cases.
- Carry publication's admitted package selection into Clippy without changing
  compiler or lint flags; ordinary release qualification retains its full gate.
- Compare Wasm sizes on relevant main-branch pushes as well as pull requests
  and manual dispatches.
- Add opt-in matched System API and Candid admission instruction probes alongside
  the existing Wasm size profiles; ordinary installed regressions stay separate.

## [0.23.5] - 2026-11-06

### 🩹 Fixed

- Adopt Shared 0.31.0, including stricter snapshot/checksum verification,
  schema-2 release journals and guarded batch reconciliation and retirement.

### 🔧 Tooling

- Capture fresh grouped releases in a frozen candidate, prepare its own tools
  before version lookup and admit recovery offline. Explicit supplier-ordered
  cohorts use the shared batch lifecycle commands and renamed `RELEASE_GROUPS`,
  setup and publication settings; retired settings refuse. Publication remains
  opt-in.

## [0.23.4] - 2026-11-06

### 🔧 Tooling

- Adopt Shared 0.30.7 and enforce reviewed Rust CBOR source allowances and
  standalone fixture Cargo boundaries. Use Platform's typed codecs in runtime
  and fuzz callers instead of their direct generic CBOR dependencies.
- Supply the opt-in frozen release runner without activating it before
  Platform's actual release and publication hooks are qualified.

## [0.23.3] - 2026-11-06

### ✨ Features

- Expose standard ICP account-identifier derivation under `icp-ledger-types`,
  preserving upstream SHA-224/CRC32 bytes so consumers can remove local helpers
  ([#147](https://github.com/dragginzgame/ic-platform/issues/147)).

### 🔧 Maintenance

- Remove the facade's unused `serde_test` development dependency; Types and
  Runtime retain their own test dependencies.
- Select allocation-only encoding and byte-Serde utility features, preserving
  external bytes without forcing their standard-library features on consumers.

### 🧪 Testing

- Extend malformed-input fuzzing to certificate, hash-tree and typed IC packet
  CBOR, and share ingress compatibility fixtures between Rust and the browser.
- Compare representative Wasm capabilities against a committed baseline with
  identical build inputs, reporting total and code-section growth separately.

## [0.23.2] - 2026-11-05

### ➕ Added

- Expose the exact management `CreateCanisterArgs` on native and Wasm and add the
  narrow `candid::protocols::management::create_canister` codec, reusing existing
  settings/reply identities and preserving caller transport, cycles and recovery
  policy ([#146](https://github.com/dragginzgame/ic-platform/issues/146)).

### 🐛 Fixed

- Format rendered release manifests before staging so the normal commit hook
  preserves the exact release tree; qualify real hooks in grouped recovery tests.

### 🔧 Tooling

- Adopt Shared 0.30.5 with the qualified generic inherent-header correction and
  enforce named external Candid declarations in the existing boundary gate.
- Keep compact experimental conclusions in Git, retire obsolete trial tables
  and probes, and default new benchmark output to `target/evidence/`.
- Assemble native Profiler adapters through ordinary Rust module discovery,
  preserving the independent recorder, viewer assets and public aliases; include
  extracted patch tests and compiled-owner lint checks in native qualification
  ([#140](https://github.com/dragginzgame/ic-platform/issues/140)).

## [0.23.1] - 2026-11-05

The local release stopped before tag/push/publication when the commit hook
formatted a rendered feature array after staging. Its commit and journal are
retained; 0.23.2 delivers the reviewed changes through forward recovery.

## [0.23.0] - 2026-11-05

### ⚠️ Breaking

- Preserve Testkit's optional subnet-metrics leaf contract: `decode_subnet_metrics`
  now returns `Option<SubnetMetrics>`, accepting CBOR null as absence while
  retaining complete-input limits, malformed/trailing-data refusal and full u128
  cycle counts ([#145](https://github.com/dragginzgame/ic-platform/issues/145)).
  Callers must handle the optional result; certificate path lookup stays consumer-owned.

### 🔧 Tooling

- Use Shared's complete validation proofs instead of locally interpreting runner
  events and timings. Keep source/host/release custody and digest-bound recovery
  after metadata preparation; a stage PASS does not imply fresh execution
  ([#141](https://github.com/dragginzgame/ic-platform/issues/141)).

## [0.22.3] - 2026-11-05

### ➕ Added

- Add explicit `certification-cbor` APIs for byte-bounded standard certificate
  and canister-signature encoding/decoding. Keep caller-selected budgets,
  witness construction, store policy and verification with consumers; preserve
  upstream certificate/tree identities and standard CBOR bytes.
- Supply native typed IC packet codecs and explicit development fixtures so
  consumers can remove direct CBOR dependencies without dropping protocol or
  malformed-input checks ([#142](https://github.com/dragginzgame/ic-platform/issues/142),
  [#143](https://github.com/dragginzgame/ic-platform/issues/143)). Include the existing
  strict certificate/tree admission contract and preserve separate tagged/untagged
  packet bytes, upstream identities and caller-owned verification/effect policy.
- The same native fixture selection supplies Canic's query, controller-list and
  canister-range packet tests without generic CBOR exports
  ([#144](https://github.com/dragginzgame/ic-platform/issues/144)).

## [0.22.2] - 2026-11-05

### 🔧 Tooling

- Adopt Shared 0.30’s source-role LOC report, label generated proof/benchmark
  packages and use ordinary source files for the Canic benchmark driver.
- Keep CI-selection and release-recovery fixtures complete through Shared’s
  declared companion closure.
- Repair exported Shared documentation through committed adoption and check every
  exported Markdown payload in the normal documentation gate.
- Enforce bounded, dated recovery-evidence admission before release/publication;
  preserve the approved historical archive outside Git administrative state.

### 🧪 Testing

- Scope management schema comparisons to their native registry supplier and lint
  expectations to the actual native/Wasm visibility, preserving external DTO bytes.
- Move native Profiler inline tests into ordinary test modules and tighten owned
  lint scopes. Paired native-server qualification remains required under
  [#140](https://github.com/dragginzgame/ic-platform/issues/140).

## [0.22.1] - 2026-11-05

### 🐛 Fixes

- Restore upstream packed-map and sequence canister-signature envelope admission
  while preserving verification, delegated-key admission and authenticated time.

### ⚡ Performance

- Remove a redundant BLS key serialization/reparse from signature admission using
  the unchanged upstream checked point parser. Matched Canic release artifacts
  save about 3.3 KB of code each; retain all cryptographic and freshness checks.
  See the [matched Wasm report](../../docs/audits/platform-022-canic-wasm-2026-11-05.md).

## [0.22.0] - 2026-11-04

### ⚠️ Breaking

- Return BLS cryptography to upstream dependencies and delete the embedded curve,
  verifier, algorithm tests and source guards. Preserve key/signature bytes and
  identity-key rejection through small delegating key adapters. Signature and
  error types now have upstream identity; align consumers that exchange them.
  Defaults and RNG remain disabled, with allocated verification selected.

- Return certification and canister-signature formats/hashing/verification to
  registry dependencies. Certificate/tree types now share upstream identities
  with native Agent; remove recursive conversion adapters. Preserve owned issuer
  fields, DER framing, root/delegated identity-key refusal and authenticated time.
  Key admission/time extraction repeat CBOR decoding. Upstream packaging restores transitive
  Candid/printer, raw IC0, CBOR and a lazy root cache; no store API is exported.
  Remove embedded algorithm suites/source guards and the unused direct 4.x
  representation-independent-hash test dependency. No Wasm/performance win claimed.

### 🧪 Testing

- Refresh owner/Timers qualification locks for the registry certificate boundary
  and compatible Testkit 0.84.1; retain its published native Platform separately.

- Use ordinary module discovery for the shared malformed-Candid test/fuzz fixture.
  Adopt Shared 0.29.3's test-layout, changelog and snapshot-export rules; preserve
  imported source-layout exceptions and keep the root changelog as navigation.
- Format the shared malformed-Candid corpus directly while keeping the fuzz
  harness's copied module from breaking standalone formatting checks.
- Separate stable-identity and Candid packaging experiment tests from fixture
  implementations, preserving feature selection and generated-package coverage.

## [0.21.0] - 2026-11-03

### ⚠️ Breaking

- Transfer the maintained PocketIC client to `ic_testkit::pocket_ic`. Remove the
  client package, facade `pocket-ic` feature and module, and its release group.
  Testkit now owns one version for client and lifecycle helpers. Keep historical
  releases; migrate native test fixtures to Testkit 0.84.0. No Runtime,
  Types, Agent or crypto implementation changes
  ([#139 — Transfer PocketIC client ownership to Testkit](https://github.com/dragginzgame/ic-platform/issues/139),
  [Tooling #324 — Merge the PocketIC client into ic-testkit](https://github.com/dragginzgame/ic-tooling/issues/324)).
- Adopt Shared 0.29.1 ownership and contribution rules. Release selection now admits Platform,
  Runtime and Types only; retain full release validation pending fresh native
  qualification of this three-group mapping.

### 🧪 Testing

- Integrate #137's validation receipts with release preparation and recovery;
  reject changed source, execution settings or retained evidence. Add full and
  selected qualification on Linux and both macOS architectures. Enable qualified
  owner-source plans on Linux x86_64 and macOS 15 Intel/ARM64 after all three
  native qualifications pass. Facade, shared/unclassified changes and unknown
  hosts retain the full roster; fresh releases do not reuse qualification receipts.

## [0.20.0] - 2026-11-03

### ⚠️ Breaking

- Forward PocketIC 0.20's bounded snapshot transfer API and the new exhaustive
  `ClientError::ResponseTooLarge` variant. Consumers matching every error variant
  must handle the additional case. Types and Runtime identities are unchanged.

### 🧪 Testing

- [#137](https://github.com/dragginzgame/ic-platform/issues/137): add a guarded
  four-group validation pilot with complete stage rosters, source-bound receipts
  and failure/recovery tests. Linux full/selected mappings pass; keep the full
  release gate pending recovery integration and remaining native qualification.

## [0.19.0] - 2026-11-02

### ⚠️ Breaking

- Adopt the new canonical Types family and its Account subaccount byte encoding.
  Affected RPCs need matching schemas and fresh contract IDs; persisted data
  requires consumer-owned migration. Runtime retains the same canonical Principal.
- Forward PocketIC's fallible client API. Handle ClientError explicitly for
  construction and operations; uncertain submissions must not be replayed.

### 🧹 Maintenance

- Retain upstream ic-agent behind the explicit native facade; withdraw the
  unreleased owned Agent and its cryptographic-provider migration. Keep upstream
  type identities and consumer-owned trust, retry and admission policy.
- Remove unused direct ic_principal and thiserror dependencies from the facade;
  Types retains Principal implementation ownership and feature forwarding.

### 🩹 Fixed

- Prepare native-provider CI's manifest tools before reading the compiler floor
  ([#133](https://github.com/dragginzgame/ic-platform/issues/133)). Fail immediately
  if that lookup fails, before rendering an empty Cargo `rust-version`.

## [0.18.0] - 2026-11-02

### ⚠️ Breaking

- Remove the unused `stable-structures` feature, `stable_structures` module and
  `ic-stable-structures` dependency. Runtime's raw stable-memory system calls
  remain available; storage and allocation stay with their product owners.

- Require Rust 1.99.1 for the facade (#133). Keep
  isolated native/Wasm capability and archive proofs; qualify each compiler
  selection once instead of repeating the old 1.91/development matrix.
- The optional PocketIC facade now uses management DTO identity 0.11. Native
  clients can depend directly on the independently released `ic-platform-pocket-ic`;
  migration details are in the [client contract](../../docs/history/pocket-ic-client.md).

### 🧹 Maintenance

- Adopt Shared Tooling 0.27.27 at committed revision 651f5f7. Keep the full
  release gate for this combined compiler/ownership batch; the optional
  group-validation plan requires separate consumer mapping qualification.

- Keep the README's checked feature catalog, setup and release-group choices
  concise; move contributor and runtime qualification detail into a linked guide.

- Remove the direct test-only `assert_matches` dependency ([#123](https://github.com/dragginzgame/ic-platform/issues/123)).
  Standard assertions preserve match guards and failure values;
  source reconstruction retains the original upstream test fingerprints.
- Prepare historical registry owners used by singleton/composition rejection
  fixtures on fresh CI runners, and print fixture logs on failure. Principal
  mutation checks verify that the source changed and use the shared checksum
  verifier for vector diagnostics.
- Select documentation-only native CI only for an exact reviewed prose allowlist;
  retain all native hosts and full qualification for code or uncertain changes (#125).
- Bound new native profiler compiler scratch to 256 GiB/seven idle days under
  assembly custody. Retain executable copies and immutable receipts separately;
  preserve historical and operator-selected external caches (#132).

## [0.17.0] - 2026-11-01

### ⚠️ Breaking

- Delegate runtime capabilities to the independent Runtime 0.17 owner. Align
  Timers and Wire Endpoint once; later crypto/certification facade releases
  leave executor and generator identity unchanged. See the
  [Runtime ledger](../runtime/CHANGELOG.md) for ownership and qualification.

- `principal-self-authenticating` selects only owned Principal and upstream
  SHA-224. Select `principal-external` separately for registry conversions.
  The exact upstream derivation and resulting bytes remain unchanged;
  self-authentication-only production graphs drop `ic_principal`.

- Maintain the PocketIC 16.1 Rust client in Platform's native implementation
  package. Keep existing `ic_platform::pocket_ic` imports, method signatures,
  HTTP/JSON protocol and operation behavior; client, builder and client-owned DTOs
  now have Platform implementation identity. Consumers exchanging concrete
  registry `pocket-ic` types must align on the facade after adoption.
  The simulator server, execution engine and Testkit server pin are unchanged.

### 🧹 Cleanup

- Separate installed Platform runtime qualification from Timers composition.
  Keep both release gates and independent CI/advisory results; preserve the
  singleton guard and reuse the fixture with separately maintained locks.

- Keep client dependencies behind the existing native-only feature, with no
  Candid/crypto expansion in default, executor-only or Wasm PocketIC selections.
  Preserve Apache source provenance and qualify the existing Testkit contract.


### 🩹 Fixed

- Use physical workspace paths during grouped release preparation, including
  aliased macOS temporary directories; retain rollback and resume checks.

### ⬆️ Dependencies

- Adopt Host Process/FS/Artifacts 0.73.2 and core Wire 0.13.1 in the native
  Profiler Cargo/Bazel inputs. Keep Metrics 0.24.0 independently locked and
  preserve process observations, gzip bytes, frame admission and saved reports.

## [0.16.0] - 2026-11-01

### ⬆️ Dependencies

- **Breaking:** update the explicit Ed25519 surface to ed25519-dalek 3 and
  curve25519-dalek 5/signature 3, preserving fixed signing and strict-verification
  byte fixtures. Native Agent retains its separate older generation.
- Update the owned entrypoint generator to Syn 3 and the development-only
  representation-independent hash reference to 4.0.1.

- Maintain the existing BLS12-381 arithmetic inside Platform's private BLS
  module, preserving algorithms, original tests/vectors and MIT/Apache notices.
  Remove the registry curve from standalone BLS and canister-signature
  verification; these profiles now have zero DFINITY registry dependencies.
  Native Agent retains its independently selected wrapper/curve.

- **Breaking:** use the canonical owned Principal in canister-signature keys and
  certified subnet-range decoding. Remove registry Principal from standalone
  creation/verification profiles while preserving DER/CBOR bytes, length-first
  ordering and the original cryptographic bodies.

- **Breaking:** own certificate, hash-tree and RbTree identities and maintenance
  inside Platform. Standalone certification and canister-signature profiles no
  longer select registry certification. Preserve upstream algorithms, Serde
  bytes and tests; retain explicit native Agent certificate conversions while
  Agent/PocketIC remain externally maintained.

- **Breaking:** maintain the upstream BLS signature wrapper inside Platform,
  preserving public paths, crypto bodies and key/signature bytes while changing
  Rust type ownership. Remove its direct registry dependency and use std's
  lazy cache; retain upstream curve arithmetic. Native Agent still brings its
  registry wrapper transitively.
- **Breaking:** move the optional ECDSA surface to upstream K256 0.14/signature 3
  identities and enforce corrected bigint selection in downstream locks. Callers
  must adapt public-key serialization and explicit normalization APIs. This
  security upgrade retains a modest measured Wasm footprint increase with the
  qualified backend; Agent and native public derivation retain their older
  dependency generation.
  [Migration and compatibility](../../docs/architecture/migration.md#standard-secp256k1-ecdsa-maintenance-boundary).

### 🩹 Fixed

- Scope native entrypoint symbols to the consuming Cargo crate so multiple
  canister libraries can link in one host process. Retain portable method-name
  encoding and exact Wasm exports, codecs, guards and executor contexts.
  [#129](https://github.com/dragginzgame/ic-platform/issues/129).

- Apply the exact upstream BLS identity-public-key rejection and regression
  vector for [#127](https://github.com/dragginzgame/ic-platform/issues/127).
  Raw BLS and canister-signature verification reject forged identity-key
  signatures; source guards preserve the original fingerprints and admit only
  this reviewed correction. Native Agent's registry verifier is unchanged.

- Select RustCrypto's existing 32-bit backend for repository Wasm builds,
  recovering most of the ECDSA upgrade's size growth and reducing measured IC
  verification instructions. Canister consumers must carry the same flag in
  their build configuration; a published dependency cannot impose it.

### 🧪 Testing

- Qualify published Timers 0.35.1 with the current embedded runtime, restoring
  combined installed callback, cancellation and upgrade coverage. Preserve
  independent Types release custody when registry consumers reference local
  package identities.
- Retain BLS's allocation feature after measuring the upstream alternative:
  modest Wasm savings cost roughly 51% more verification instructions. Add
  reproducible size/instruction probes without maintaining cryptographic code.
  [Measurements and tradeoffs](../../docs/benchmarks/upstream-2026-10-31/README.md).

## [0.15.3] - 2026-10-31

### ⬆️ Dependencies

- Adopt published Host Process/FS/Artifacts 0.73.1, arithmetic-only Metrics 0.24
  and core Wire 0.13 in the native Profiler Cargo/Bazel inputs. Keep gzip bytes,
  counter quality, frame admission and saved report formats unchanged.

- Keep native Profiler gzip on Host Artifacts directly and remove the
  forwarding-only registry Platform dependency. Preserve codec bytes, levels,
  strict admission and existing API/recorder byte limits.
  [#122 — Native Profiler Host gzip](https://github.com/dragginzgame/ic-platform/pull/122).

### 🥾 Governance

- Adopt Shared Tooling 0.27.22 with a complete root member catalog, group alias
  rosters and cross-group checks for paths and full current versions. Label local
  groups and external dependency roles without adding a graph edge.
- Present release groups once in the catalog with short titles and separate
  changelog comments; remove the duplicate header map and repetitive commentary.
- Preserve normal whole-target Cargo cleanup and explicit evidence-retention
  boundaries introduced by Shared 0.27.20.

## [0.15.2] - 2026-10-31

### 🩹 Fixed

- Adopt symmetric crate groups with explicit package versions and group selection
  for release, resume and publication. Preserve numeric versions, dependency
  selections, transactional runtime metadata and recorded release evidence.

### 🥾 Governance

- Adopt Shared Tooling 0.27.19, including symmetric group ownership and delegated
  release/publication workflows, and move canonical group ledgers under releases/.
  Root Cargo has no shared version; root changelog is an index.

## [0.15.1] - 2026-10-31

### 🩹 Fixed

- Restore independent Platform/Types release groups in one workspace, starting
  from existing 0.15.0 identities. Preserve Types during runtime-only releases,
  qualify transactional runtime metadata custody and select publication by group.
  [#118 — Independent identity releases](https://github.com/dragginzgame/ic-platform/issues/118).

### ⬆️ Dependencies

- Adopt Shared Tooling release commit 0.27.17 (`39ba983`), including
  common metadata custody, Cargo group comments and canonical-ledger policy,
  through the snapshot exporter.
