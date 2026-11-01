# BLS12-381 arithmetic ownership

Imported complete `src/` from `ic_bls12_381` 0.10.1, originally Sean Bowe,
Jack Grigg and the bls12_381 contributors, as maintained by DFINITY:
https://github.com/dfinity/bls12_381/tree/280560da845158bcc96045c2d2e153a6c88e74b7

`LICENSE-MIT`, `LICENSE-APACHE`, `COPYRIGHT` and the original Cargo manifest are
retained verbatim. The dual license permits either license; Platform retains
both notices and its existing MIT AND Apache-2.0 package declaration.
This is maintenance ownership of the existing implementation, not new arithmetic
or an independent cryptographic audit. The upstream audit caveat remains.

The 2026-10-31 source review compared that revision with upstream main
`bf31d18397b2e1d2f48784975f310c7f310fcb62`. Only workflows, .gitignore and a
Cargo.lock changed; there are no later curve implementation changes to apply.
https://github.com/dfinity/bls12_381/compare/280560da845158bcc96045c2d2e153a6c88e74b7...bf31d18397b2e1d2f48784975f310c7f310fcb62

The private `bls::curve` module retains every original Rust source file, inline
test and all four binary vector files. `lib.rs` becomes `mod.rs`. No curve type
is newly exported. The wrapper imports this owner instead of the registry crate.
The `alloc`, `groups`, `pairings` (including its implicit `pairing` dependency
feature) and `experimental` selections are fixed on with `bls`; `bits`, `zeroize`
and `std` remain off. Their original source is preserved behind inactive cfgs.
Unrelated Cargo features cannot silently change these private selections.

Admitted scaffolding only:

- Internal crate/alloc paths point at the containing private module.
- The facade supplies std and test macros; private docs.rs attributes are inert.
  Original `core` imports are retained to reconstruct the upstream source.
- Rust 2024 reserves `gen`; original test identifiers use `r#gen`.
- Ineffective `must_use` attributes on trait implementations become comments.
- Arithmetic/test bodies keep original spelling under scoped lint expectations.
  Existing allowances become expectations where fulfilled, otherwise inert
  comments. A scoped deprecation allowance covers digest-compatible generic-array
  versions both before and after deprecation; an expectation is unfulfilled with
  older consumer locks. No broad Clippy suppression is added.

`check-bls-curve-source.sh` checks exact admitted source, reconstructs the original
source and checks its independent fingerprints. Binary vectors, licenses and the
original manifest are checked unchanged. Do not refresh original hashes to permit
algorithm changes. Mutation checks cover arithmetic, vectors, feature selection
and licenses.

The existing non-DFINITY dependencies digest 0.10, ff/group 0.13, pairing 0.23,
rand_core 0.6 and subtle 2 remain external. rand_core supplies traits, not entropy.
Original tests additionally use development-only rand_xorshift 0.3 and sha3 0.10;
upstream Criterion/CSV benchmark dependencies are unnecessary and omitted.
Native Agent retains a separate registry curve through its own verifier.
Standalone BLS and canister-signature verification have no DFINITY registry
dependencies. This does not claim whole-workspace or offline build independence.
