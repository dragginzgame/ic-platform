# Platform changelog

Release group: `platform`. Covered package: `ic-platform`.
Return to the [group index](../../CHANGELOG.md).

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
