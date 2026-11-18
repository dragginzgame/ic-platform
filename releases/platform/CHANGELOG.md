# Platform changelog

Release group: `platform`. Covered package: `ic-platform`.
Return to the [group index](../../CHANGELOG.md).

## [0.34.17] - 2026-11-18

### 🔧 Maintenance

- Align certificate, ingress and native transport modules with the updated Rust layout policy.

## [0.34.16] - 2026-11-18

### ➕ Added

- Restore the optional `stable-structures` feature with a direct upstream 0.7.2
  re-export for [Platform #30](https://github.com/dragginzgame/ic-platform/issues/30).
  Collection, memory, iterator and error types retain upstream identity and bytes;
  memory allocation and migration policy remain with consumers.

## [0.34.15] - 2026-11-17

### 🔧 Maintenance

- Consolidate the complete qualification catalog into one serial Make graph for
  [Platform #26](https://github.com/dragginzgame/ic-platform/issues/26), retaining
  source-bound receipt admission and extracted-archive checks. Share native unit
  prerequisites, enforce publication group ownership, and make dry-run preflight
  optional. No product release is required by this tooling change alone.

### ➕ Added

- Add independent direct-signature status-envelope admission through
  `PreparedRequestStatus::from_signed_bytes` for
  [Platform #29](https://github.com/dragginzgame/ic-platform/issues/29).
  The signer parent can authenticate the original request ID, sender and exact
  single path without a private key or retained update. Callers compare the
  authenticated expiry with their approved exact expiry and supply routing policy.

## [0.34.14] - 2026-11-17

### ➕ Added

- Add exact offline update and single-path original-request status signing for
  Ed25519 and secp256k1 for [#29 — Expose exact offline update and original-request status signing for external signer composition](https://github.com/dragginzgame/ic-platform/issues/29).
  Preserve nanosecond expiry, original request IDs, sender identity and bounded
  retained envelopes; key discovery and signer process policy stay with Tooling.

- Supply bounded SEC1/PKCS#8 secp256k1 identities and authenticated signed queries
  for [#28 — Supply bounded secp256k1 signed queries for IC DB migration](https://github.com/dragginzgame/ic-platform/issues/28).
  Reuse upstream K256 signing and the existing certified node-key, response-signature,
  time and resource checks. Consumer adoption remains separate.

### 🧪 Testing

- Adopt Testkit 0.109.0 throughout qualification, including materialized Auth and
  Blob consumers, preserving the PocketIC 16.1.0 server selection.

## [0.34.13] - 2026-11-17

### ✨ Added

- Add explicit anonymous prepared update and original-request status admission for
  [Platform #25](https://github.com/dragginzgame/ic-platform/issues/25). Require the
  anonymous sender and absence of every authentication field; signed admission
  never falls back. Preserve original bytes, IDs, expiry, routing, bounded dispatch
  and certificate verification. Qualify ICP CLI 1.7.0 create/install/stop/upgrade/
  start and recovery alongside existing signed regressions.

## [0.34.12] - 2026-11-17

### 🔧 Changed

- Allow compatible Types patches through `0.21.2` instead of `=0.21.2` in the
  published dependency requirement. Preserve the locked selection, empty defaults
  and single canonical Types identity checks.

## [0.34.11] - 2026-11-17

### 🔧 Changed

- Move certificate and native query modules with child directories to `mod.rs`,
  preserving public module paths and implementation bytes under Toko's layout rule.

## [0.34.10] - 2026-11-16

### ➕ Added

- Add certified controller and hosting-subnet observations under native-read for
  [#22](https://github.com/dragginzgame/ic-platform/issues/22), retaining explicit
  root/time/limits, range authority and missing/pruned refusals. See the
  [supplier evidence](../../docs/qualification/canic-management.md).

## [0.34.9] - 2026-11-16

### ➕ Added

- Supply offline bounded ingress envelope construction for
  [Platform #12](https://github.com/dragginzgame/ic-platform/issues/12), preserving
  externally supplied signatures, optional fields and exact request-status paths.
  Count complete output before allocation and share the serializer with native
  signing. Backup migration and installed recovery acceptance remain separate.

### 🧪 Testing

- Update qualification’s registry Platform comparison to 0.34.8 while testing
  the local 0.34.9 candidate; retain the canonical Types 0.21.2 identities.

- Reconcile Blob's published 0.103.0 signed CLI/custody and Caffeine acceptance for
  [Platform #17](https://github.com/dragginzgame/ic-platform/issues/17) and
  [Platform #18](https://github.com/dragginzgame/ic-platform/issues/18).
  See [consumer evidence](../../docs/qualification/blob-suppliers.md#published-blob-consumer-acceptance).

## [0.34.8] - 2026-11-16

### ➕ Added

- Add exact externally signed request-status admission and one-attempt observation
  without private keys, replacement reads or update renewal for
  [Platform #17](https://github.com/dragginzgame/ic-platform/issues/17).
  The opt-in native-signed-secp256k1 profile verifies direct ICP envelopes with
  upstream K256; Ed25519 remains available through native-signed.
  Actual ICP CLI 1.7.0 fixtures and fresh-process recovery are qualified locally;
  full consumer lifecycle migration remains separate.

### 🧪 Testing

- Adopt Testkit 0.107.3 in qualification and generated Auth/Blob fixtures, with
  registry Types 0.21.2 and Platform 0.34.7 comparison identities.

## [0.34.7] - 2026-11-16

### ➕ Added

- Add explicit Ed25519 PEM signing, immutable original update preparation,
  single-attempt dispatch, certified original-request recovery and signed queries.
  Preserve exact proof bytes and independent historical time policy. See the
  [supplier evidence and remaining consumer gates](../../docs/qualification/blob-suppliers.md).

### 🧪 Testing

- Replace Query's private fixture dependency with a versioned independent replay
  corpus for [Platform #10](https://github.com/dragginzgame/ic-platform/issues/10); qualify actual topology/CLI tests, strict Clippy and portable Cargo
  archives against published products. See [the evidence](../../docs/qualification/query-hard-cut.md).

## [0.34.6] - 2026-11-16

### ➕ Added

- Add native strict ingress packet admission with original bytes, canonical Types
  identities and upstream request IDs/signable bytes. Preserve unverified
  delegation/sender-info presence and opaque response certificates. Backup's
  actual recovery qualification remains separate; see the
  [contract](../../docs/qualification/ingress-packets.md).
  Supplier work addresses [#12 — Backup ingress packets](https://github.com/dragginzgame/ic-platform/issues/12);
  consumer and delivery acceptance remain open.

### 🔧 Tooling

- Adopt published Toko Tooling 0.44.9 through its supported installer. The
  fleet-wide thiserror hold is removed. Product behavior is unchanged by
  this contributor-tooling update.

## [0.34.5] - 2026-11-16

### ➕ Added

- Supply bounded certificate/signature construction, canister-key DER framing,
  raw-root admission and separately selected native fixture signing for
  [#7 — Auth crypto migration](https://github.com/dragginzgame/ic-platform/issues/7).
- Map Auth certified verification to explicit replacement APIs in pinned
  qualification; authenticate signatures before reporting malformed timestamps.

- Add opt-in anonymous native reads with authenticated query node signatures,
  explicit root/subnet/time policies, selected-subnet metadata and bounded HTTP.
  Qualify actual Query and all five installed Auth cases without legacy Platform.
  [#4 — Query and crypto boundary](https://github.com/dragginzgame/ic-platform/issues/4)
  retains separate Agent signing/derivation and macOS obligations.

### 🔧 Tooling

- Adopt Toko Tooling 0.44.7 in both workspaces and use its flat qualification
  package layout. Retain the approved thiserror 2.0.18 compatibility selection.

## [0.34.4] - 2026-11-16

### ➕ Added

- Restore opt-in upstream Ed25519 and secp256k1 primitives for
  [#7 — Auth crypto migration](https://github.com/dragginzgame/ic-platform/issues/7).
  Qualify Auth's strict verification and root-delegation paths, including
  feature-unified scalar refusal and independent native/Wasm vectors.
  Certificate construction and complete installed migration remain pending.
- Constrain the direct ECDSA graph to corrected crypto-bigint releases.
  [#4 — Crypto/Agent advisory](https://github.com/dragginzgame/ic-platform/issues/4)
  remains open for the separate Agent and derivation paths.

### 🧪 Qualification

- Adopt Testkit 0.107.1 and published Platform 0.34.3 in the installed
  certificate checks. Name the frozen 0.33.7 codec oracle explicitly; retain its
  independent comparison coverage without treating it as the current supplier.

## [0.34.3] - 2026-11-16

### 📦 Publication

- Publish the implemented scope independently, with explicit repository and
  package documentation. Keep production migration and the broader design gates
  separate; preserve existing experimental limits and all correctness checks.
- Advance package metadata after source delivery; Runtime support packages remain
  coordinated. Types stays at registry 0.21.1.

## [0.34.2] - Unreleased

### 🔧 Dependencies

- Align exact local Types dependency with the 0.21.1 release candidate. Advance
  the group after its delivered source snapshot; registry publication remains
  disabled. Runtime support packages advance together.

## [0.34.1] - Unreleased

### ✨ Added

- Add opt-in bounded, tagged canister-signature envelope decoding, retaining
  original bytes and exposing an immutable, explicitly unverified witness and
  opaque embedded certificate. Reuse the strict CBOR tree visitors.
- Refuse oversized input, missing tags, malformed trees, duplicate/unknown fields
  and trailing data.
- Add opt-in canister-signature authentication with immutable message/key/root/time
  evidence, shared witness/certificate traversal bounds and explicit input limits.
  Reuse certificate signature/delegation/range checks; require a certified known
  non-cloud-engine subnet type for delegated signatures.
- Use RustCrypto `spki` for bounded canonical DER key admission and the existing
  `sha2` selection for seed/message hashing. Keep all cryptographic primitives
  upstream; no Agent, Runtime, signature store or authorization policy is added.

### 🧪 Testing

- Add independently observed direct/delegated PocketIC canister signatures and
  public-API native/Wasm replay, including binding, signature, authority, time
  and resource refusals. Keep capture provenance and frozen offline fixtures;
  production Auth adoption and mainnet signer interoperability remain pending.
- Qualify an isolated Auth issuer-policy adapter against observed signatures for
  its exact claims-message domain on native and Wasm. Keep protected signer/seed,
  raw-root adaptation and signing freshness in the consumer; no Auth or Canic
  product migration is included.

## [0.34.0] - Unreleased

### ⚠️ Breaking changes

- Begin the replacement Platform package with empty defaults and the opt-in
  `certificate-decode` capability. Old facade and feature APIs are not imported.

### ✨ Added

- Decode strict certificate grammar into private fields while borrowing exact
  original bytes. Expose only an immutable, explicitly unverified view.
- Preserve input-byte admission, strict payload/node grammar, duplicate/unknown
  field refusal, recursion admission and trailing-data refusal. Delegate parsing
  and tree representation to reviewed upstream dependencies; add no crypto code.

- Add opt-in `certificate-authentication` using upstream BLS verification, explicit
  root/target/time context and traversal bounds. Return immutable evidence for
  direct roots or one root-signed delegation; refuse nested delegations.
- Check separate signing/delegation windows, combined tree bounds, embedded byte
  limits and complete selected legacy/sharded ranges. Compare principal bytes
  lexicographically, validate pairs after a match and bind shard start labels.

- Add immutable selected-subnet signing-time projection for [Query issue #1](https://github.com/dragginzgame/ic-platform/issues/1),
  sharing signature/time checks while enforcing exact delegated subnet identity
  or an explicitly trusted root-subnet binding. Add no transport or default clock.

- Accept borrowed upstream Agent certificates for subnet time without re-encoding;
  distinguish parsed-object custody from retained original encoded bytes.

### 🧪 Tests

- Independently authored literal direct/delegated envelopes and malformed-input
  tests cover byte boundaries, payload kinds, tree tags/arity and opaque delegation.
- Direct-root fixtures cover wrong roots, altered signatures, identity refusal,
  malformed DER, time boundaries and ambiguous/bounded trees. Compile-fail tests
  prevent construction, mutation and deserialization of authenticated evidence.
- Delegation cases cover wrong targets/signatures, nested/malformed certificates,
  time/resource boundaries, malformed pairs after a match, unequal-length
  principal ordering, and selected-shard refusal without fallback.
- Add independently captured PocketIC direct/delegated certificates, public-API
  observation checks and offline replay with signature-corruption refusals.
- Test selected-subnet root/delegated observations, source-binding refusals, distinct
  signing/delegation time, full u64 values and malformed/future/stale admission.
- Add independently selected non-subgroup G1/G2 inputs, upstream point classification
  and refusal checks for root keys, delegated keys and both authentication paths.
- Add standalone native/Wasm replay of PocketIC and independently captured
  mainnet direct-root/delegated certificates, including root-subnet binding and
  distinct signing/delegation time refusals. CI runs the offline corpus.
- Broader mainnet interoperability and production consumer qualification remain
  pending. Decode success is not security evidence.
