# Platform changelog

Release group: `platform`. Covered package: `ic-platform`.
Return to the [group index](../../CHANGELOG.md).

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

- Add immutable selected-subnet signing-time projection for [Query issue #1](https://github.com/dragginzgame/ic-platform-new/issues/1),
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
