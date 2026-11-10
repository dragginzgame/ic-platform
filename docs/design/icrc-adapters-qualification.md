# ICRC Account/archive adapter qualification

Subsequent tooling adoption resolves the earlier IC0 admission and root-file
metadata blockers. This report retains its original qualification scope; see
[the current handoff](../status/current.md) for remaining work.


Local Linux qualification on 2026-11-10 covers only the Account and archive
adapters. The [contract](icrc-adapters-contract.md),
[implementation record](icrc-adapters-implementation.md) and separate
[public-test record](icrc-adapters-public-tests-implementation.md) describe source
separation and its limits. Neither isolated author read the retired ICRC
implementations/tests. The coordinator reviewed and integrated their work.

## Result and compatibility

The external ICRC Account remains a distinct public DTO with its existing fields
and traits. Equality, ordering, standard-library Hash and Display now delegate to
canonical Types behavior. Plain sequence-shaped subaccount Serde remains distinct
from Types' byte adapter, preserving Some(zero) versus None. The external parser
keeps its detailed errors and precedence rather than adopting Types' text ceiling
or smaller error catalogue.

Parsing uses fixed scratch instead of a padded String and decoded Vec. It still
checks the conceptual Unicode-scalar padding, UTF-8 byte parity, first invalid
byte/offset, decoded length and checksum in the same order. This removes those
temporary allocations; no instruction speed, total heap or Wasm-size measurement
was performed.

Archive Func conversions move already canonical Principal and method fields
directly. They retain the public TryFrom/Error=String contract, generic bounds,
query signature and Serde asymmetry. No byte reparsing, unreachable parse refusal
or impossible Principal-conversion panic is needed. Decoding does not authorize
calling a callback. The existing owned codec mappings and external bytes remain.

No production/development dependency or feature selection changed; the manifest
only registers the new feature-gated test. The existing dirty Cargo.lock is
byte-identical to its pre-task snapshot. Remaining ledger declarations and the
value/hash implementation retain their common Apache license and source guards.
Only the two retired adapter digests are removed from the current import list.
No cryptographic algorithm, domain or hash-input encoding changed.

## Focused evidence

Compiler: Rust 1.99.1, `58cf8f9c35b355199727627d07ae34a4e2846d67`, LLVM 23.1.1,
native host `x86_64-unknown-linux-gnu`. Commands used `--locked --offline` and
`--no-default-features`:

- Native library Clippy passed before replacement. The six independently
  authored public tests and their Clippy passed against that original owner.
- After replacement, Clippy passed for the facade library and the
  `icrc_adapters_consumer` / `owned_icrc_contracts` tests with
  `icrc-ledger-types,cycles-ledger-types`. Wasm library Clippy passed with
  `icrc-ledger-types`. All Clippy selections denied warnings.
- Nineteen ICRC-filtered library tests passed with the two-feature selection.
  They include four fresh adapter tests, existing public Account parity and
  Serde/Candid oracle comparisons, archive bytes and unchanged hash vectors.
- Six new public adapter tests plus two existing frozen wire/archive integration
  tests passed without a test-name filter: 27 passing tests after replacement.
- The same 48 deterministic text observations matched byte-for-byte before and
  after, including Display/Debug errors, Unicode offsets, error ordering and
  suffixes up to 65,537 bytes. The original observations now form the permanent
  `crates/ic-platform/tests/fixtures/icrc-adapter-observations.json` assertion.
  SHA-256: `78ac539ae0493260b4d161e96caf0c9a0cc5bdc42b312b0230274cb584c63985`.
- Native and Wasm normal/build Cargo trees for `icrc-ledger-types` are identical
  before/after. Neither includes registry Candid, Agent or Runtime. The ICRC
  value capability still legitimately selects its existing SHA-2 provider.
- The retained ICRC value/hash source reconstruction and license checks pass;
  mutation checks refuse changed hash bodies, vectors, license, provider aliases,
  carriers and lint normalization.
- Manifest inventory, ordinary and design-document link checks, scoped Rustfmt
  and diff-whitespace checks pass. The new public test is registered with its
  required ICRC feature and is included by the existing all-feature test caller.

Evidence is retained under `target/icrc-adapters.yeW4NY`: original modules/lock,
compiler description, public source digest, before/after observations and Cargo
graphs. The old modules matched the working source before replacement; those
two modules had no pre-existing diff from commit
`a37b15165a4f87a99f2845cb8263ad565f4a4864`. Logs are also under
`/tmp/platform-icrc-*.log`; the maintained tests and this record retain the durable
contract/evidence summary. The permanent observation assertion was added by the
coordinator after capture, without changing inputs or observation construction.

No installed IC, native macOS or sibling qualification was run: this step changes
no execution or transport mechanism. No release, commit or publication was
performed. First-party replacement licensing and the previously recorded
supplier/ICP-derivation blockers remain separate unfinished work.
