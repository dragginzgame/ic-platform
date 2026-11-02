# Platform changelog

This group covers ic-platform. PocketIC now has its own [ledger](../pocket-ic/CHANGELOG.md).
Earlier coordinated history is retained once in [history](../history.md).
See also the [Types ledger](../types/CHANGELOG.md).

## [0.19.0] - Unreleased

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
  migration details are in the [client contract](../../docs/pocket-ic-client.md).

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
  [Migration and compatibility](../../docs/migration.md#standard-secp256k1-ecdsa-maintenance-boundary).

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
