# External canister DTO contracts

The pending compatible Platform 0.6.20 surface supplies the contract families
below. Own package versions remain unchanged until maintainer release. No
published supplying version is claimed yet. These are external Candid protocols,
not owned Wire endpoints. Their structs/enums are Platform-maintained Rust
representations of pinned schemas, not identities from an upstream Rust crate.
The Principal/Nat/function machinery retains registry Candid identities.

| Feature | Import module | Selected external methods |
| --- | --- | --- |
| `cmc-types` | `ic_platform::cmc_types` | `notify_top_up`, `notify_mint_cycles`, `get_icp_xdr_conversion_rate` |
| `icp-ledger-types` | `ic_platform::icp_ledger_types` | Legacy ICP `transfer`, `query_encoded_blocks` and encoded-block archive callbacks |
| `cycles-ledger-types` | `ic_platform::cycles_ledger_types` | `create_canister`, `withdraw` |
| `nns-registry-types` | `ic_platform::nns_registry_types` | `get_subnet_for_canister` |
| `certified-assets-types` | `ic_platform::certified_assets_types` | Native-host asset `get`, `get_chunk` |

Each feature explicitly selects `candid`; none is default or implied by runtime,
CDK, executor or another DTO family. Asset exports are absent on Wasm. Selecting
that feature on Wasm still selects its explicit Candid prerequisite; it does not
supply an asset API or acquire host transport. No feature selects the host parser,
Agent, PocketIC, executor, storage engine, payment workflow or retry policy.
`Vec<u8>` represents upstream `blob`/`vec nat8`, and `Nat` remains unbounded.
Account and subaccount allocation, digest length checks and decode limits are
consumer-owned. Decoding never authenticates a rate, asset, archive or payment.

## Source and licensing

The CMC and ICP declarations are schema-derived adaptations of the Apache-2.0
interfaces at IC revision `f83a7646c20ce18100eb87b7865c12852bef8169`:
[CMC DID](https://github.com/dfinity/ic/blob/f83a7646c20ce18100eb87b7865c12852bef8169/rs/nns/cmc/cmc.did)
and [ICP Ledger DID](https://github.com/dfinity/ic/blob/f83a7646c20ce18100eb87b7865c12852bef8169/rs/ledger_suite/icp/ledger.did).
No closer directory license was present along those paths. The IC root license,
including the DFINITY copyright notice, is retained beside both modules.

Cycles Ledger declarations derive from its
[DID at 2faa00a86115d92cce599345b1f5ae04f0f7f2bb](https://github.com/dfinity/cycles-ledger/blob/2faa00a86115d92cce599345b1f5ae04f0f7f2bb/cycles-ledger/cycles-ledger.did).
Asset declarations derive from the
[SDK DID at 271392a3b7746df539925a6333b4cd7ddc9f1972](https://github.com/dfinity/sdk/blob/271392a3b7746df539925a6333b4cd7ddc9f1972/src/canisters/frontend/ic-certified-assets/assets.did).
Their Apache-2.0 licenses are retained beside each module; SDK's asset-directory
license refers back to the root license. Changes are limited to selected DTOs,
Rust names/documentation, standard derives and a typed archive function supplied
by Candid's existing `define_function!` macro. No cryptographic code is copied.

Registry declarations independently express the small protocol facts observed in
[Registry's DID at f83a764](https://github.com/dfinity/ic/blob/f83a7646c20ce18100eb87b7865c12852bef8169/rs/registry/canister/canister/registry.did).
No Registry implementation or DID text is imported or distributed here. Registry's
ancestor `rs/registry/LICENSE` points to IC-1.0, not Apache-2.0; it must not be
mislabelled as an Apache source. The independently written Rust declarations and
test schema are Platform code, using the protocol's published field/type facts.

The four Apache DID fixtures are retained verbatim under
`crates/ic-platform/tests/fixtures/external-*.did`. Their SHA-256 observations:

| Fixture | SHA-256 |
| --- | --- |
| `external-cmc.did` | `abbcd458765c674ae0683cc90f1fa070fbdefa53cd4a46f487a9c1e50a642f37` |
| `external-icp.did` | `18c2fe7ea500c88df92eddd83a69a3e44c07a289465e06941b07c0dffc7099df` |
| `external-cycles.did` | `af8f846f268d1ca34f51d1c9b00ed77f90c11e8c3aa288fd915eae5aea28457f` |
| `external-assets.did` | `af4efb9d921acb40734025cedd6a3424c8a925c5bcd5585d711cb2dbf263227b` |

These pinned source contracts do not establish which revision a deployed service
runs. Contract tests compare complete selected Candid types and callback signatures
with the pinned DID, rather than comparing only partial Canic projections.

The inspected `ic-ledger-types` 0.16.0 package includes `ic-cdk` 0.19 and account
identifier/hash implementations beyond these requests. It is not added solely
for these DTOs. No full Cycles Ledger or asset service/storage package is added.
This is a deliberate maintenance surface, not a claim that no other DTO supplier
exists or that dependency graphs/optimized Wasm are smaller.

## Canic migration

After publication, require the actual supplying Platform version and select only
the features each runtime/host crate uses. These features also supply the existing
`candid::contract` and bounded decoding APIs; no direct Candid dependency is needed.
Do not replace accepted original bytes in already-retained uncertain operations.

| Existing Canic mirror | Platform replacement |
| --- | --- |
| `NotifyTopUpArg`, `NotifyTopUpError` | `cmc_types::{NotifyTopUpArg, NotifyError}` |
| Conversion rate and partial `RateReply` | `cmc_types::{IcpXdrConversionRate, IcpXdrConversionRateResponse}` |
| `NotifyMintArgs`, `NotifyMintOk`, `NotifyMintError` | `cmc_types::{NotifyMintCyclesArg, NotifyMintCyclesSuccess, NotifyError}` |
| ICP `Tokens`, `Timestamp`, `TransferArgs`, `TransferError` | `icp_ledger_types::{Tokens, TimeStamp, TransferArgs, TransferError}` |
| ICP `ReadArgs`, `EncodedBlocksReply`, `ArchivedRange`, `ArchiveReadError` | `icp_ledger_types::{GetBlocksArgs, QueryEncodedBlocksResponse, ArchivedEncodedBlocksRange, QueryArchiveError}` |
| `CyclesLedgerCreateCanisterArgs` and its CMC/settings/subnet/result/error mirrors | Corresponding `cycles_ledger_types` declarations |
| `WithdrawArgs`, `WithdrawError`, `RejectionCode` | Same names under `cycles_ledger_types` |
| Registry request/payload | `nns_registry_types::{GetSubnetForCanisterRequest, GetSubnetForCanisterResponseOk}`; outer reply is `GetSubnetForCanisterResponse` |
| Asset `GetAsset`, `AssetReply`, `GetChunk`, `ChunkReply` | `certified_assets_types::{GetArgs, GetResult, GetChunkArgs, GetChunkResult}` |

Use `TimeStamp`, matching the canonical ICP spelling. ICP accounts/subaccounts
are raw blobs here, not the ICRC Account identity; convert existing arrays with
`to_vec()` without deriving new account identifiers. These DTOs do not validate
fixed-size account identifiers: retain existing
32-byte account/subaccount admission wherever the consumer previously required
arrays. Convert unbounded `Nat` into bounded consumer state only with checked
conversion and the existing refusal policy. The typed ICP archive callback
has its own exact query signature; do not substitute an ICRC block callback.
Complete rate replies include `hash_tree` and `certificate`; encoded-block replies
include optional `certificate`. Neither is automatically authenticated.

Cycles Ledger's CMC arguments omit the direct CMC method's deprecated
`subnet_type`; its settings also differ from management-canister and current
CMC settings. Preserve these types rather than aliasing lookalikes. All failed
creation fee/refund blocks, duplicate canister evidence and withdrawal rejection
variants remain available. Canic retains bounds, target/controller/subnet binding,
cycle accounting, fee policy, journals and outcome reconciliation.

For Registry, construct `principal: Some(target)` in both cost quotation and
submission. `None` requests caller-derived behavior and is never an implicit
replacement. Canonical optional-principal bytes differ from Canic's old required
field, although upstream Candid subtyping can decode the old shape. This patch
provides the exact new DTO; it does not rewrite Canic's retained request bytes.

Asset responses now include canonical `content_type`. Canic retains its admitted
network/target, `identity` encoding selection, original key, expected digest,
maximum total/chunk lengths and index/hash binding. Optional digest admission
and certificate/authenticity verification stay there. No Platform default limit
or automatic retry is added.

Canic must replace its mirrors, exhaust direct dependency/import use, and rerun
refill/operator-mint original-byte/retry, refund/duplicate, receipt authentication,
creation/withdrawal recovery and frontend acceptance tests. Those are consumer
qualifications; Platform's schema/codec proofs do not replace them. No sibling
checkout, deployment, paid operation or publication is part of this work.

## Focused qualification

Run full `make clippy` before longer tests, then
`bash scripts/ci/check-platform-consumer.sh external-contracts asset-contracts external-contract-schema`.
The portable consumer covers native/MSRV/Wasm; the asset exports are native only;
the parser-backed schema consumer is native only. Their manifests declare only
Platform. Parser participation is solely a test selection and never implied by
any production DTO feature. Retain actual default/executor/runtime and selected
DTO normal/build graphs separately. Native macOS and installed services require
their own qualification; local compilation does not claim either.
