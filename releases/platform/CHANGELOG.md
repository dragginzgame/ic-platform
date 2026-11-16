# Platform changelog

Release group: `platform`. Covered package: `ic-platform`.
Return to the [group index](../../CHANGELOG.md).

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
