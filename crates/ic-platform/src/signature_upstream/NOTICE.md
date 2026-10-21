# Signature source packaging

The maintainer authorized packaging upstream cryptographic source inside Platform
without changing its algorithms. This is not an independently implemented crypto
library. Both imported modules retain all original function bodies and unit tests.

- `creation/mod.rs`: ic-canister-sig-creation 1.3.1, upstream revision
  `3e5786bb06f94e898d3f8371ec088be23abec987`, `src/lib.rs`. Changes are
  `candid::Principal` to the identical registry `ic_principal::Principal`, and
  omission of the unexposed `signature_map` module declaration. Store implementation
  and lifecycle ownership are not imported.
  In 0.5.1, the maintainer additionally authorized deleting the unused,
  unexported `IC_ROOT_PUBLIC_KEY` lazy cache and its `lazy_static` import.
  The embedded DER constants, extraction function, all cryptographic function
  bodies and upstream tests remain byte-for-byte unchanged.
- `verification/mod.rs`: ic-signature-verification 0.3.0, dfinity/ic revision
  `b8613a99eb324c313c6080f84964cb670ced517b`,
  `packages/ic-signature-verification/src/canister_sig.rs`. Its creation import
  points at the sibling imported module; nothing else changes.

For 0.6, seven `cfg(test)` attributes keep the unused delegation-signing helper
and creation CBOR parser (including their imports/type/constant) in reference
tests only. One unused-import allowance retains the original HashTree import.
All bodies, constants and vectors remain exact; the guard also strips only those
eight attributes to recover and verify the original 0.5.1 source hash. Production
verification retains its own unmodified CBOR parser and dependencies.

Each directory retains its original Apache-2.0 LICENSE. Exact source/test/license
fingerprints are enforced by `scripts/ci/check-signature-source.sh`. Rustfmt skips
these imported modules. The original SHA-256, domain separation, DER/CBOR parsing,
BLS and certificate verification behavior (including admission limitations) stays
unchanged. Callers must retain length bounds, canonicality, issuer/seed and trust
policy. No short-DER panic repair or security-policy change is implied.

Platform now owns the packaging identity of CanisterSigPublicKey. It is no longer
interchangeable with the registry creation package's type. BLS, certification and
external Principal identities remain registry types. This requires the 0.4 line.
