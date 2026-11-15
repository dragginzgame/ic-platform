# Consumer requirements for the rebuild

This initial contract/proof map implements [DESIGN.md](DESIGN.md), based on a
read-only Platform issue review on 2026-11-15. GitHub owns issue status and
resolution history. This is not a complete consumer inventory or a duplicate
issue tracker. No row establishes replacement qualification.

## First slice: Cloud Engine projection

The named consumer contract is Query's selected `getEngineViews` topology read,
recorded in [Platform #168](https://github.com/dragginzgame/ic-platform/issues/168).
The design pins old Platform source to
`c24e3a59583eabcf5b6bb87987269c13a209a51f`. Read-only inspection found Query HEAD
`263a8e92da873543825c25e7dd6530c4db5ab673`, with the actual caller in
`crates/ic-topology/src/cloud_engine/engine/mod.rs`: `collect` encodes one ID and
decodes through `decode_get_engine_views_response_with_skip_limit`; `project`
owns result binding and relationship admission. Its contract is
`docs/design/cloud-engine-topology.md`. Both inspected files are tracked and
unchanged against HEAD; the checkout has unrelated dirty work. This pins the
existing caller, not adoption or qualification of a replacement.

The [#168 delivery record](https://github.com/dragginzgame/ic-platform/issues/168#issuecomment-6892377968)
reports publication in Platform 0.33.7 at
`663cf913997490f448553119a6caf66b3cdecc81` and registry-consumer checks. The
[#156 implementation record](https://github.com/dragginzgame/ic-platform/issues/156#issuecomment-6792536498)
describes concrete declaration macros, compile-time label checks and independent
fixtures. These resolution comments were reviewed; neither proves the new
decode-only declaration design or its replacement implementation.

| Obligation | Replacement owner and acceptance |
| --- | --- |
| Exactly one `vec text` request argument and one `vec EngineView` result | Protocol encoder and independent Decode implementation; reject extra arguments, malformed frames and trailing bytes. |
| EngineView: id, subnet_id, node_ids, deleted_at, engine_operator_id, subnet | One declaration with canonical numeric labels, types and required/strict-optional policies. Verify labels against the pinned external contract. |
| EngineSubnet: node_ids, replica_version, halted, maximum_state_size | Preserve nested absence, empty text/vectors, full i64 timestamps and optional u64 zero/max. Malformed present selected values must refuse rather than become None. |
| Root and nested node lists have different meanings | Preserve both and their order. No inferred membership agreement, running-version, provisioning or active-engine claim. Empty results mean not returned, not proven nonexistent. |
| Unknown fields are validated and skipped | Charge cumulative skipped work and total work; reject malformed skipped data. Do not allocate complete discarded DTOs. |
| Caller-selected resource bounds | Test exact/over byte, header, type-table, work, logical-allocation, vector and skip limits; preserve the 16-level constructed-wire ceiling. Failed optional attempts cannot refund work. |
| Canonical Principal and external Serde forms | Select one Types source; qualify native/Wasm consumer identity and independent byte/text/Serde fixtures. Exclude Runtime, Agent, registry Candid and crypto from the production projection closure. |
| Response projections need not encode | Compile a decode-only projection; generate encoding only where needed. No mandatory Serde decoding for Candid. |

Query owns endpoint/root admission, request selection, deadlines, node-list
admission, relationship checks and presentation. Its example budget values in
#168 are consumer selections, not new Platform defaults. Supplier tests must use
frozen literal frames and independent oracle declarations; generated round trips
are insufficient. Native/Wasm compilation and real consumer adoption remain
separate from unit decoder tests.

[Platform #156](https://github.com/dragginzgame/ic-platform/issues/156) supplies
the declaration ergonomics contract: numeric labels, docs, visibility, attributes
and derives without repeated field types. Preserve record/variant obligations as
the surface grows; EngineView alone cannot prove general variant support.

## Other retained contracts

| Consumer / canonical source | Owner | Required acceptance |
| --- | --- | --- |
| Query Registry settings, [#169](https://github.com/dragginzgame/ic-platform/issues/169) | Protocol | Absent vs present-empty nested messages, absent vs false/zero scalars, raw replica-default sentinels, shared nested budgets, duplicate-singular and malformed-wire refusals. Configured SEV is not attestation. |
| Query marketplace/availability, [#166](https://github.com/dragginzgame/ic-platform/issues/166) | Protocol | Six node classes, arbitrary naturals, signed timestamps, daily units, exact arity and application errors; unready Registry copy differs from an empty page. Consumer owns pagination. |
| Backup signed recovery inspection, [#167](https://github.com/dragginzgame/ic-platform/issues/167) | Platform native packet boundary | Strict read-state field/kind admission, original bytes/paths, upstream request-ID identity, duplicate/unknown/malformed/trailing refusal and honest input-bound semantics. No transport/retry or Backup policy. |
| Canic endpoint composition, [#164](https://github.com/dragginzgame/ic-platform/issues/164) | Runtime | Guard and pre-copy admission before decoding, explicit codecs, one dispatch/reply owner and installed rollback/callback proof; no Wire-specific execution owner. |
| Query certified classification, [#158](https://github.com/dragginzgame/ic-platform/issues/158) | Platform native | Actual upstream SubnetType identity, unknown strings and absent values; classification is not capacity or deployment permission. |
| Crypto/Agent boundary, [#50](https://github.com/dragginzgame/ic-platform/issues/50) | Platform crypto/native | Recheck selected graphs and current advisory evidence before admission. Direct ECDSA correction alone does not resolve Agent/derivation paths. No in-house crypto, forced incompatible override or inherited risk acceptance. |
| Tooling/compiler adoption, [#157](https://github.com/dragginzgame/ic-platform/issues/157) | Repository tooling | Retain Rust 1.100.0, common lints and shared dependency commands. This checkout selects published Toko 0.44.3 through its authenticated installer; product acceptance remains separate. |

The first projection must not import native/crypto dependencies to satisfy an
unrelated row. Conversely, an isolated codec pass cannot qualify those rows.
Closed supplying work remains contract evidence; read its resolution and pinned
implementation before deciding what to reuse or retire.

## Evidence required before completing the first slice

The pinned old request/response implementation was inspected without copying
source or tests. The initial package now implements request encoding and selected
response decoding with independent literal frames; see [development](development.md).
The response declarations use canonical numeric labels, verified independently
from field names using Candid's specified label calculation. Published Types
0.20.4 supplies Principal. A separate qualification consumer proves the public
type crossing and JSON shape on native, plus Wasm test compilation. Its native-only
published Platform 0.33.7 oracle exercises independent record declarations,
30 combinations of optional/numeric values, exact request bytes, malformed
literal frames and 189 single-byte mutations. Both implementations share the
same registry Types identity; comparison uses the published dependency, not
copied implementation or test source. This is bounded differential evidence,
not exhaustive protocol qualification.
Two additional literal comparisons establish the published decoder's value-based
admission for empty vectors and None payloads: incompatible declared inner types
are not checked when no element/payload is read. Present incompatible boolean
payloads still refuse with Type. This is observed compatibility, not proof of
full schema-level admission; tightening it requires a deliberate contract choice.

Separate literal JSON tests now cover 12 nested signed/unsigned boundary
combinations, omitted/null optional fields, unknown fields, exact integer output
and 32 malformed-input refusals. Both successful values and admission match
published Platform. See the [Serde evidence](qualification/engine-view-serde.md).
This proves the selected JSON contract, not every serializer or browser number
representation.

Four fixed workloads also pass the [resource-admission criteria](qualification/engine-view.md)
using Query's existing budgets, with exact logical allocation/work/skip minima
and no retained-allocation growth from larger discarded text.
The separate [native heap comparison](qualification/engine-view-heap.md) also
passes five fixed workloads with no total/peak allocation regression against
published Platform and no allocation growth from discarded text. This is bounded
Linux evidence; it does not establish the broader performance or host gates.
The [standalone Wasm comparison](qualification/engine-view-wasm.md) executes both
decoders successfully and reports a 1,693-byte code increase. Size acceptance is
undecided: the initial agent-selected zero-growth criterion was not grounded in
a launch requirement. Contribution analysis is deferred at the maintainer's
request; required validation and resource limits remain intact.
The separate admission binary also executes numeric/Principal fixtures,
truncation, trailing-data, arity and resource refusals in both Wasm artifacts
under Node. This is standalone execution evidence, not installed IC or Query
adoption proof.

The first slice's full gate remains open: real Query adoption, broader
numeric/Serde compatibility and physical allocation, instruction-count and Wasm
performance thresholds have not yet been fully qualified. The private
declaration supports required and strict optional fields, not a public generic
Candid-compatible optional API. No legacy surface is retired on this evidence.

Extend the map with actual consumers of remaining protocol families, identity,
Runtime, verification and browser subpaths before removing their surfaces. Record
source/lock/compiler identities and positive/negative proof owners. Missing tests
do not establish unused APIs; issue closure does not establish replacement proof.
