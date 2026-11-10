# Passive ICRC DTO qualification

Value's non-cryptographic helpers have subsequently also been replaced. Only
its hash composition, integer helpers and four upstream tests remain imported;
see [value qualification](icrc-value-qualification.md). The source-boundary
statements below describe this earlier passive DTO batch.

Subsequent tooling adoption resolves the earlier IC0 admission and root-file
metadata blockers. This report retains its original qualification scope; see
[the current handoff](../status/current.md) for remaining work.


Local Linux qualification on 2026-11-10 covers the seven metadata, transfer,
allowance, block and ICRC-21 modules. The [contract](icrc-dtos-contract.md),
[implementation record](icrc-dtos-implementation.md) and separate
[public-test record](icrc-dtos-public-tests-implementation.md) describe admitted
inputs and source separation. Neither isolated author read the retired target
implementations/tests. The coordinator reviewed the old interface and integrated
the replacements. These records are not legal clearance or a claim about model
training exposure.

## Result and compatibility

Public structs/enums, fields and variant order, trait sets, numeric ranges,
explicit defaults, Serde shapes and Candid mappings remain. Metadata conversions
preserve signed/unsigned extrema and owned buffers. Memo still encodes a u64 as
exactly eight big-endian bytes. TransferError keeps every Display string without
adding Error. Block-range narrowing checks start before length, borrows the
magnitudes and does not add the narrowed values. Consent remains passive data,
with no authorization or presentation policy.

Field/variant docs replace seven blanket documentation-lint exceptions. The
existing 28-schema test moved from the all-protocol gate into the ICRC-only
consumer tests. Its selected schemas and independent DID remain unchanged. The
named-schema comparison helper moved into the existing reference helper module
and is reused by the other protocol tests instead of duplicated.

Within ICRC, only `value/mod.rs` remains an imported implementation. Its complete
declarations/conversions, hash bodies, signed encoding and vectors are untouched.
The common notice/import inventory now states that scope; its original full
Apache license, fingerprints and guards remain. This is source-ownership work,
not a measured performance or Wasm-size improvement.

## Focused evidence

Compiler: Rust 1.99.1 (`58cf8f9c35b355199727627d07ae34a4e2846d67`), LLVM 23.1.1,
native `x86_64-unknown-linux-gnu`. Cargo commands used `--locked --offline` and
`--no-default-features`.

- Before replacement, library/public-test Clippy, seven new public tests and the
  relocated 28-schema check passed with `icrc-ledger-types` alone. One test-only
  unused assignment was fixed by checking the changed input; no lint exception.
- After replacement, native Clippy passed for the facade library and the
  `icrc_dtos_consumer`, `icrc_adapters_consumer`, `owned_icrc_contracts` targets
  with `icrc-ledger-types,cycles-ledger-types`. Wasm library Clippy passed with
  `icrc-ledger-types`. Both selections denied warnings.
- Twenty ICRC-filtered library tests passed, including all 28 frozen schema
  comparisons, existing wire/Serde oracle cases, adapter regressions and retained
  hash vectors. Seven new DTO tests, six Account/archive tests and two frozen
  wire/archive integration tests passed: **35 tests after replacement**.
- All 23 grouped DTO observations matched exactly before/after. They cover every
  selected enum variant and record family, numeric extrema and nested archive/
  consent values. JSON is retained as text to observe field order; CBOR and
  Candid are retained as bytes. The permanent assertion fixture is
  `crates/ic-platform/tests/fixtures/icrc-dto-observations.json` (20,249 bytes),
  SHA-256 `dbe96031ef0b3a91583abbc4fe0fe2816239a2dcd8951c3a0cb3c2114333b455`.
- The earlier 48 Account text observations and frozen Account/block bytes also
  still pass. Serialization tests do not invent JSON roundtripping for callback
  types whose Serde input/output intentionally differ.
- Native and Wasm normal/build ICRC dependency trees and Cargo.lock are identical
  before/after. No production or development dependency or feature changed; only
  the new feature-gated test is registered in the manifest. Registry Candid,
  Agent and Runtime remain absent from this production feature's graphs.
- ICRC source/license reconstruction and mutation refusal tests pass, covering
  hash bodies, vectors, license, provider aliases, carriers and lint scaffolding.
- Manifest inventory, ordinary/design-document links, scoped Rustfmt and
  diff-whitespace checks pass. The existing all-feature test caller discovers
  the new test through its feature-gated manifest entry.

Evidence: `target/icrc-dtos.O22GVg`, including old modules/lock, compiler record,
public-test source digest, before/after transcript and native/Wasm graphs. The
seven target modules had no pre-task diff from commit
`a37b15165a4f87a99f2845cb8263ad565f4a4864`. Logs are under
`/tmp/platform-icrc-dtos-*.log` and retained with the evidence. The maintained
tests, frozen fixture and this summary are durable; generated build evidence may
be cleaned normally.

No installed IC, native macOS, sibling or full-workspace qualification was run.
This batch changes no transport/execution mechanism. No commit, release or
publication occurred. First-party replacement licensing and the previously
recorded supplier/ICP-derivation blockers remain separate unfinished work.
