# Maintained certification source

Imported from `ic-certification` 3.2.0, published by DFINITY under Apache-2.0:
[response-verification at 37c1565e04ac0fdd449619e5e25f51b985d98c05](https://github.com/dfinity/response-verification/tree/37c1565e04ac0fdd449619e5e25f51b985d98c05/packages/ic-certification).
The original license and package manifest are retained alongside this notice.

Platform owns the maintained certificate, delegation, hash-tree and RbTree types.
The public feature/module stays `certification`; canister-signature creation and
verification select that same owner. Consumers retain decoding budgets, trust
roots, authentication, freshness, storage, pruning and root-publication policy.
No signature verification is implied by deserialization, conversion or lookup.

The complete `certificate`, `hash_tree` and `rb_tree` source and original tests are
retained. The never-exposed `nested_rb_tree` module is omitted; it is not needed
by `RbTree<Hash, RbTree<Hash, Vec<u8>>>` or the public consumer surface. Its rstest
test dependency is therefore not introduced. Production dependencies are the
existing hex, Serde, Serde bytes and SHA-256 packages.

Admitted source transformations, exactly reversed by
`scripts/ci/check-certification-source.sh`:

- Map crate-root imports into `crate::certification::upstream`.
- Rename `lib.rs` to `mod.rs`; omit the two unused nested-module declarations.
- Select original Serde implementations with the owning `certification` feature.
- Drop crate-specific nightly docs.rs attributes; the facade owns documentation.
  Render three links to unexposed node types as code text.
- Move dead-code admission to the private import boundary. Retain narrow lint
  expectations for an upstream traversal and test assertion.
- Explicitly spell one byte-slice AsRef and two elided lifetimes; use Rust 2024
  implicit borrowing in two original test patterns. These do not change behavior.

Hashing bodies, domain strings, ordering, tree balancing, witnesses, Serde bodies
and test vectors are unchanged. Original source hashes remain in the guard;
never change them to admit an algorithm change. The signature source guard also
reconstructs its old certification imports without changing any existing hash.

Upstream maintenance review on 2026-10-31 compared the import revision to
[4.0.1 / 2de9e04b37ae14b0adcef4616077dc300ccf441c](https://github.com/dfinity/response-verification/compare/37c1565e04ac0fdd449619e5e25f51b985d98c05...2de9e04b37ae14b0adcef4616077dc300ccf441c).
The only change under this package is removal of the obsolete `doc_auto_cfg`
attribute, which the facade's documentation handling already removes. The
monorepo's 4.0 release adds no certificate/tree implementation change to adopt.

Native Agent still selects registry certificate types. Explicit From conversions
live outside this import in `../external.rs` and preserve bytes, including all
five tree node variants and optional delegation. They rebuild boxes and copy
labels; they do not validate input or confer authentication. No registry
certificate dependency is selected by certification-only or signature-only Wasm.
