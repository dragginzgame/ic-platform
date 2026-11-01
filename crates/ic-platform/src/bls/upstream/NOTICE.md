# BLS source packaging

Imported from DFINITY's `verify-bls-signatures`, package
`ic-verify-bls-signature` 0.6.0, exact revision
`31fbeda277d35e28e2c7ba27919a2475e2fb5091`:
https://github.com/dfinity/verify-bls-signatures/tree/31fbeda277d35e28e2c7ba27919a2475e2fb5091

The registry source matches this revision's `src/lib.rs` exactly. `tests/mod.rs`
retains the complete original `tests/tests.rs`, including the threshold-signature
vectors and random roundtrips. Apart from the exact reviewed identity-key
rejection below, no cryptographic body, constant, domain, encoding or original
vector is changed. The same ic_bls12_381 0.10.1
implementation is now maintained in the private sibling [curve module](../curve/NOTICE.md);
pairing 0.23 and SHA-256 remain external dependencies.

Admitted packaging changes:

- One import aliases the private maintained curve module as `bls12_381`.
- The containing facade supplies std; the imported module retains forbid lints.
  The upstream unit-error lint allowance becomes a documented expectation.
- The previously selected alloc path is unconditional. Its Debug implementations
  remain; the no-alloc branch stays verbatim behind an always-false cfg.
- `std::sync::LazyLock` replaces the lazy_static macro, with the identical
  prepared-generator initializer and the unchanged optimized verification body.
- The original random-key helper and random tests compile only in unit tests.
  Rand is development-only; no production RNG capability is exposed.
- Tests import the containing module. One `rng.gen` spelling becomes `rng.r#gen`
  for Rust 2024; test bodies and vectors otherwise remain exact.

`check-bls-source.sh` reverses only these admissions and checks the original
source/test fingerprints. Never refresh them to permit cryptographic edits.
Both alloc and no-alloc function bodies remain recoverable; no new algorithm
or cryptographic implementation is authored here.

The maintainer authorized the exact upstream fix for
[Platform #127](https://github.com/dragginzgame/ic-platform/issues/127) in
[commit 1f423f7](https://github.com/dfinity/verify-bls-signatures/commit/1f423f7b9d2e9c2a4c41e7a252ff999beb5489a5).
`PublicKey::verify` now rejects an identity key before hashing/pairing. The
original upstream regression test is retained verbatim. Deserialization and
valid-key behavior are unchanged. The guard requires that exact check at entry,
checks the added vector separately, then removes only those two additions before
checking the unchanged original source/test fingerprints. Missing, changed or
moved checks and changed vectors are refused. Native Agent retains registry
0.6.0 independently; this does not correct its verifier.

The original Cargo manifest declares Apache-2.0 and is preserved as
`Cargo.toml.original`. That exact upstream release tree contains no LICENSE or
NOTICE file. `LICENSE-APACHE` supplies the complete standard Apache-2.0 text from
https://www.apache.org/licenses/LICENSE-2.0.txt; it is not represented as an
upstream repository file. The containing facade remains MIT AND Apache-2.0.

The exposed BLS key/signature/error types now have Platform identity. External
Agent keeps its registry verifier privately. Consumers exchange the unchanged
serialized bytes rather than treating the two Rust owners as interchangeable.
