# Independent external DTO and schema qualification

## Scope and authorship

The [reviewed contract](external-dtos-contract.md) records public declaration and
protocol facts for CMC, ICP Ledger, Cycles Ledger and native certified assets.
The [implementation author](external-dtos-implementation.md) received those facts
without original bodies, tests, DID fixtures, codec mappings or supplier source.
A [separate test author](external-dtos-public-tests-implementation.md) wrote public
consumers and four minimal schema fixtures from the contract. The parent
integrator read the original source, reviewed the contract and captured original
behavior before installing the new declarations. These records describe context
separation and its limits; they do not establish legal clearance.

The replacement preserves 33 structs/enums, six aliases and all public traits,
field/variant order, Serde attributes, numeric ranges and feature/target gates.
The three provider-block records still require ICRC and retain Value::Nat64.
The parent appended each module's existing owned codec mapping tail unchanged.
ICP's callback and account-identifier children, ICRC Value/hash and root module
selection are byte-identical to the pre-change files. There is no cryptographic,
transport, authentication, freshness, storage or application-policy change.

## Baseline and commands

Baseline: the existing dirty checkout over
`a37b15165a4f87a99f2845cb8263ad565f4a4864`, not two published archives. Original
source, source records, fixtures, compiler identity, lock, graphs and boundary
hashes are retained at `target/external-dtos.before`; after graphs are at
`target/external-dtos.after`. Unrelated work is preserved. Compiler: Rust 1.99.1,
local Linux. All Cargo commands used `--offline --locked -p ic-platform` and
`--no-default-features`.

The main selected feature union was:

```text
cmc-types,icp-ledger-types,cycles-ledger-types,certified-assets-types,icrc-ledger-types,nns-registry-types
```

Clippy `-D warnings` covered the selected library and public targets before
replacement tests. Native unit validation used `--lib tests::external_contract`
and `--lib tests::owned_external_contracts`. Public validation selected
`--test external_dtos_consumer --test asset_contract_consumer` with the five
DTO/ICRC features (Registry is unnecessary for those targets). Wasm Clippy used
`--lib --target wasm32-unknown-unknown` with the main union and separately with
`cycles-ledger-types` alone to check the provider gate's absent selection.

| Check | Result |
| --- | --- |
| Original external contract/schema unit tests | 12 passed |
| Original owned external codec unit tests | 4 passed |
| Original native asset public test | 1 passed |
| New public tests against original DTOs | 4 passed; repeated with frozen fixture assertion |
| Complete selected method equality, original versus minimal DIDs | 14 methods passed |
| Replacement external contract/schema unit tests | 12 passed |
| Replacement owned external codec unit tests | 4 passed |
| Replacement public tests | 4 new + 1 asset passed |
| Native library/public Clippy, Wasm library Clippy, Cycles-only Wasm Clippy | Passed |
| Native/Wasm normal/build graphs and Cargo.lock | Byte-identical |
| Owned codec mapping tails, callback, crypto helpers, root module selection | Original hashes/bytes match |
| Scoped Rustfmt | Passed |
| Manifest inventory, docs links and Candid/CBOR/dependency boundaries | Passed |
| Retained ICRC source guard and mutation refusals | Passed |
| Scoped diff whitespace check | Passed |

Baseline Clippy found a complex test-only roster type annotation. The independent
author replaced it with two local aliases without suppressions or behavior
changes; Clippy then passed. The parent added only the exact frozen-fixture
assertion after capturing original observations.

## Observations and schema preservation

The permanent `external-dtos-observations.json` fixture records 27 grouped
explicit JSON/CBOR-input observations and two constructed callback/provider rows.
These DTOs deliberately lack Serialize: observations freeze admitted input,
decoded public Debug/fields and Candid output rather than inventing a serialization
implementation. Native tests compare decoded JSON and CBOR values, clone/codec
roundtrips and original/replacement observations. They cover every selected enum
variant, absent/present options, u64 extrema, Nat above u128, arbitrary text/bytes,
callback signatures and provider Nat64 without narrowing it to ICRC3Value.

The fixture is 45,497 bytes; SHA-256:
`be5f25f6a5638b09cc51958dc433484c154503d062ae258e7f4107cb6cc18be6`.

Before retiring the original four DIDs, a temporary parent comparison parsed
both versions and used Candid's structural equality on each complete selected
function type after merging environments. All 14 methods matched, including
arity, query/update modes and recursive callback types. The comparison source
and original fixtures remain in the task's baseline evidence; the temporary test
was removed after its run. Permanent tests check the exact minimal method roster,
arity/modes and complete selected Rust schemas against the independent DIDs.

| Independent fixture | Methods | SHA-256 |
| --- | --- | --- |
| `owned-cmc.did` | 3 | `d6a6f9e4613ea3441fef7fca2cb05de725c8f79321721fc7ba8a1181c33cea98` |
| `owned-icp.did` | 2 | `c68e6cf15b0dad083f226de35c2e156fce5be98f45485fdbb1fc6749956b80b8` |
| `owned-cycles.did` | 7 | `0f7886277b9784722fa3bf51dae06b83e6bf176878ad55a9224cc4cac047c6ae` |
| `owned-assets.did` | 2 | `c903fd6dcc4307a3cafe01ca0fad4cec183357a4aacc95a605353df7e0aa9c56` |

Fixtures now contain 171 lines instead of 1,559; this removes unrelated service
schema maintenance, not production Wasm or runtime work. Original schema sources
and hashes remain in the [historical record](../history/external-contracts-pre-replacement-2026-11-10.md).

## Retired source and limits

The four original declaration adaptations, their license/notice pairs and the
four full upstream DID fixtures are removed. Existing schema consumers now select
`owned-*.did`. No declaration, method or callback contract was hard-cut, and no
new dependency was introduced. The existing release test entrypoint includes the
new target; no fleet qualification or provider installation was added.

ICRC Value/hash and ICP account-identifier derivation still retain separate
source records. The pre-existing ICP draft still needs resolution and was not
qualified here. No new installed IC, macOS, downstream, performance, Wasm-size or
full-release claim is made. Logs: `/tmp/platform-external-dtos-*.log`.
No commit, push, release or publication was performed.
