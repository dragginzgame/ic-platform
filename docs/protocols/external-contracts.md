# External canister DTO contracts

Platform supplies passive, explicitly selected external protocol records. These
are adapters for existing IC Candid interfaces, not owned Wire endpoints or
client workflows. The canonical Principal belongs to Types; Nat and the bounded
binary codec belong to Platform. No DTO feature implicitly selects Agent, the
host parser, executor, timers or a service implementation.

## Selected surfaces

| Feature / module | Selected interface |
| --- | --- |
| `cmc-types` / `cmc_types` | `notify_top_up`, `notify_mint_cycles`, certified conversion-rate response |
| `icp-ledger-types` / `icp_ledger_types` | Legacy ICP transfer, encoded blocks/archive callbacks; separate account-identifier helper |
| `cycles-ledger-types` / `cycles_ledger_types` | Canister creation and withdrawal |
| Both `cycles-ledger-types` and `icrc-ledger-types` | Provider block/archive records, retaining `Value::Nat64` |
| `nns-registry-types` / `nns_registry_types` | Optional-target `get_subnet_for_canister`; bounded protobuf `get_value` and subnet-list adapters |
| `certified-assets-types` / `certified_assets_types` | Native-only `get` and `get_chunk` |

Assets remain absent on Wasm; selecting their feature there supplies only the
owned-codec prerequisite. Features have no defaults. The production DTO graphs
exclude upstream Candid; that package is a development binary schema/byte
oracle and native-client transitive. The workspace no longer selects candid_parser.

## Compatibility and consumer responsibilities

The four CMC/ICP/Cycles/assets families retain 33 declarations and six aliases,
with unchanged fields, variant order, exact traits and Serde/Candid behavior.
They implement Deserialize, not Serialize; do not broaden their trait contract
as a convenience. The three provider-block records additionally require ICRC and
retain their smaller trait set. See the [API contract](../design/external-dtos-contract.md)
for the exact declaration facts and [qualification](../design/external-dtos-qualification.md)
for implementation inputs and original/replacement comparisons.

- Nat is unbounded. Use checked narrowing into consumer storage or counters.
- Byte fields remain ordinary Serde sequences, including optional and nested
  vectors. Consumers retain account/digest length checks; a raw blob DTO does
  not establish a valid account identifier, hash or certificate.
- Cycles creation settings differ from management and direct-CMC settings.
  Preserve the distinct types, optional subnet selection, duplicate evidence,
  fee/refund records and every withdrawal rejection variant.
- Cycles provider blocks retain `Value::Nat64`; standard ICRC3Value is a narrower
  schema and cannot replace it. No value conversion or hash change is introduced.
- ICP archive callbacks and Cycles callbacks retain their separate exact query
  signatures. Decoding a callback grants no authority to follow it.
- Rate and block certificates, asset digest/content metadata and reported total
  lengths remain untrusted inputs. Authentication, request/target binding,
  freshness, limits, recovery journals, fees and retry policy stay with callers.
- Registry requests retain an optional Principal. `None` is not an implicit
  substitute for an explicit target; the canonical bytes differ from a formerly
  required-field projection even when Candid subtyping accepts that projection.

Existing callers use the same module paths and canonical identities. This source
replacement requires no consumer migration. It does not change budgets, accepted
wire formats, uncertain-operation bytes or recovery behavior. The earlier
[release/adoption history](../history/external-contracts-pre-replacement-2026-11-10.md)
records superseded API names and original schema custody; it is not current usage
instruction.

## Source and schema custody

The passive declarations and original schema fixtures were authored in separate
specification-only contexts. Existing owned codec mappings and ICP callback
implementation remain unchanged. The original four declaration adaptations and
full service DID copies are retired after exact comparison of all 14 selected
method types, including modes, arity and recursive callback signatures.

The independently captured `owned-{cmc,icp,cycles,assets}-schema.json` binary
fixtures express only that supported subset. They preserve all named types and
method signatures from the retired text fixtures, checked for structural equality
before conversion. They do not describe the entire remote service or prove
which version a deployed canister runs. Their public protocol facts were checked
against IC `f83a7646c20ce18100eb87b7865c12852bef8169` (CMC/ICP), Cycles Ledger
`2faa00a86115d92cce599345b1f5ae04f0f7f2bb` and SDK assets
`271392a3b7746df539925a6333b4cd7ddc9f1972`. Original source references and fixture
hashes remain in the historical record and the qualification report.

Registry's small declarations/schema were already independently written from
published interface facts. No Registry implementation or full DID is imported.
ICRC hash composition and ICP account-identifier derivation remain separate
retained source areas with notices and guards; non-cryptographic Value helpers
now have an [independent implementation](../design/icrc-value-qualification.md).
The existing ICP derivation draft
fails its source guard; it is not qualified by this passive DTO work. Cryptographic
primitives remain external. This batch makes no blanket source-clearance claim.

## Focused qualification

The qualification report records exact commands and results. The relevant native
feature union is `cmc-types,icp-ledger-types,cycles-ledger-types,certified-assets-types,
icrc-ledger-types,nns-registry-types`. Run affected library/public-target Clippy
before the selected tests. Unit filters `tests::external_contract` and
`tests::owned_external_contracts`, plus public `external_dtos_consumer` and
`asset_contract_consumer` targets, exercise supported DTO and schema contracts.
Use the equivalent Wasm library selection for portable declarations; assets are
intentionally native-only. No full fleet matrix is part of this ordinary change.

The small schemas preserve independent protocol expectations; public observations
freeze original Serde admission and Candid bytes. Consumer-specific policy tests,
installed IC acceptance, macOS and downstream adoption remain separate evidence.

## Registry protobuf subset

`nns_registry_types::protobuf` supplies bounded `get_value` request encoding,
borrowed inline response decoding, and a bounded vector of borrowed subnet-list
entry bytes. Query owns the selected key, endpoint, authentication, deadline,
principal validation and report claims. The response's version is the selected
key's last mutation version, not the Registry's global latest version. Preserve
its raw timestamp and application error; decoding does not certify either.

The implementation and byte fixtures were independently authored from protocol
facts in IC revision `48b6f629d74fdc4dca2c1f8e14810a9342f72fbb`:
[transport schema](https://github.com/dfinity/ic/blob/48b6f629d74fdc4dca2c1f8e14810a9342f72fbb/rs/registry/transport/proto/ic_registry_transport/pb/v1/transport.proto)
and [subnet schema](https://github.com/dfinity/ic/blob/48b6f629d74fdc4dca2c1f8e14810a9342f72fbb/rs/protobuf/def/registry/subnet/v1/subnet.proto).
No upstream decoder implementation or tests were used. Request encoding includes
optional UInt64Value version presence; response decoding targets the inline arm
of HighCapacityRegistryGetValueResponse and refuses chunk references explicitly.

Callers provide byte, field, allocation and item budgets. Response blobs and
error text remain borrowed. Subnet decoding counts and admits entries before
allocating their reference vector, then fills it in a second bounded pass.
Unknown scalar/fixed/blob fields are skipped under the field and byte budgets;
wire groups, malformed lengths/varints, invalid error UTF-8 and duplicate singular
fields are rejected. Missing content remains distinct from an empty inline value.
No recursive generic protobuf engine, chunk retrieval or external dependency is
introduced. The feature additionally selects the existing `thiserror` dependency.

Focused fixtures under `nns_registry_types::protobuf::tests` cover independent
request/response bytes, raw versions/time/error codes, unknown fields, malformed
wire data, duplicate singular fields, chunk refusals and all caller limits.
