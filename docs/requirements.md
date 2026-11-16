# Consumer requirements for the rebuild

This initial contract/proof map implements [DESIGN.md](DESIGN.md), based on a
read-only Platform issue review on 2026-11-15. GitHub owns issue status and
resolution history. This is not a complete consumer inventory or a duplicate
issue tracker. No row establishes replacement qualification.

Active follow-up for the two remaining old-repository issues now lives here:
[Registry settings #3](https://github.com/dragginzgame/ic-platform/issues/3)
and [crypto/Agent advisory #4](https://github.com/dragginzgame/ic-platform/issues/4).
Old #169 and #50 were closed as migrated at the maintainer's request; their
links below remain provenance, not active work queues or claims of completion.

## First slice: Cloud Engine projection

The named consumer contract is Query's selected `getEngineViews` topology read,
recorded in [Platform #168](https://github.com/dragginzgame/ic-platform-legacy/issues/168).
The design pins old Platform source to
`c24e3a59583eabcf5b6bb87987269c13a209a51f`. Read-only inspection found Query HEAD
`263a8e92da873543825c25e7dd6530c4db5ab673`, with the actual caller in
`crates/ic-topology/src/cloud_engine/engine/mod.rs`: `collect` encodes one ID and
decodes through `decode_get_engine_views_response_with_skip_limit`; `project`
owns result binding and relationship admission. Its contract is
`docs/design/cloud-engine-topology.md`. Both inspected files are tracked and
unchanged against HEAD; the checkout has unrelated dirty work. This pins the
existing caller, not adoption or qualification of a replacement.

The [#168 delivery record](https://github.com/dragginzgame/ic-platform-legacy/issues/168#issuecomment-6892377968)
reports publication in Platform 0.33.7 at
`663cf913997490f448553119a6caf66b3cdecc81` and registry-consumer checks. The
[#156 implementation record](https://github.com/dragginzgame/ic-platform-legacy/issues/156#issuecomment-6792536498)
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

[Platform #156](https://github.com/dragginzgame/ic-platform-legacy/issues/156) supplies
the declaration ergonomics contract: numeric labels, docs, visibility, attributes
and derives without repeated field types. Preserve record/variant obligations as
the surface grows; EngineView alone cannot prove general variant support.

## Next Runtime release priority: Auth execution requirements

The maintainer selected these gaps as the next Runtime release priority after
Auth's review of Platform source `ea359de` and Runtime 0.23.9. They are retained
consumer requirements, not intentionally retired behavior.
Independent publication of the implemented slices did not qualify Auth migration.

| Required capability | Owner and acceptance evidence |
| --- | --- |
| Caller, canister identity and IC system time | Runtime system APIs using canonical Types. Verify actual caller/canister bytes and IC-provided time in installed executions; preserve execution-context restrictions. Time access does not establish application freshness or authorization. |
| Certification-root publication and certificate retrieval | Runtime system APIs. Verify the required root size and exact bytes, query certificate presence/absence and context behavior, and independently authenticate the resulting installed certificate against the published root. Retrieval alone does not authenticate evidence. |
| Query and lifecycle exports used by Auth | Runtime's single generator and coordinated macro package. Inventory the exact Auth entrypoint kinds before implementation; qualify their exported ABI, permitted contexts and lifecycle behavior in Auth's installed fixture. Preserve stop/drain/upgrade/start without callback persistence or a new Runtime pre-upgrade guard. |
| Guard/decode/encode integration | Explicit Runtime entrypoint composition. Qualify guard and pre-copy ordering, decoder failures, explicit encoding and exactly one reply owner, including manual and automatic replies required by Auth. Keep application authorization and codec selection in the consumer. |

Implement the system observations and certification boundary first, then the
required export/composition modes, and qualify them together through Auth's actual
installed fixture. Use read-only pinned Auth source in an isolated qualification
workspace here; this priority grants no sibling migration or delivery authority.
Do not substitute import renaming, compatibility shims or a synthetic-only fixture
for the missing behavior and real consumer proof.

Acceptance for this work includes native/Wasm boundary and compile-fail checks,
installed positive/refusal cases, one canonical Types source and one executor,
and the packaged candidate cohort. Record source, dependency and fixture identities
with the evidence. These cases extend the existing Runtime obligations rather
than replacing rollback, resource accounting or context-isolation checks.
Auth remains on its working Runtime cohort until this acceptance passes. Release
Runtime, macros and guard together; select the next version when the concrete
change's compatibility impact is known. Runtime 0.23.10 now implements these
surfaces and passes the actual pinned Auth installed fixture; see the
[source and boundary evidence](qualification/runtime-auth.md). Packaged-cohort
release admission and production migration remain separate. GitHub remains the issue tracker.

## Scheduling and Canic Host migration requirements

Scheduling source `170599fbf3e4fac51b09519645d8dc11d9633e4f` uses Timers 0.45.11
and exact Runtime 0.22.5. The latter conflicts with the current executor singleton.
The [timer contract](qualification/runtime-timers.md) records the new Runtime
prerequisites, installed evidence and remaining real-consumer migration. Keep the
singleton; do not resolve the conflict by allowing both executors.

Read-only Canic source `e84702b3bdcf4eb209476aadd053df21f18dd8eb`,
`crates/canic-host`, still requests the old Platform Agent and protocol features.
Platform 0.34.3 does not provide those features. Retained calls are:

| Owner | Actual consumer obligation |
| --- | --- |
| Platform native | Agent transport, selected roots and verification, request IDs, exact signed envelope bytes, status responses and bounded certified-controller decoding. Current `signed` code delegates keys/signing to ICP CLI; that does not waive supported Agent identity dependency obligations. |
| Protocol management | Provisional creation with explicit cycles and controllers; install/upgrade with exact Wasm and empty arguments; controller handover through update_settings; bounded canister-ID result decoding. Preserve optional sender version and install-mode distinctions. |
| Protocol cycles ledger | Creation amount, timestamp, optional subnet selection and settings; all creation success/error results, particularly Duplicate with optional canister ID. A retained error does not authorize retry. |
| Protocol/Types ledger boundary | ICRC account balance arguments, bounded arbitrary-natural balance/fee responses and ICP account identifiers using one canonical Principal/Account cohort. Keep absent subaccounts distinct from explicit zero bytes on the wire. |

Fresh crates.io inspection on 16 November 2026 still selects Agent 0.49.3 as
latest, with nonoptional elliptic-curve 0.13, k256 0.13.4 and p256 0.13.2 paths.
The [current advisory](https://rustsec.org/advisories/RUSTSEC-2026-0387.html) still
requires crypto-bigint >=0.7.6. Disabling Agent default features does not remove
those paths. [Issue #4](https://github.com/dragginzgame/ic-platform/issues/4)
therefore remains an admission blocker; no Agent dependency, incompatible
override, fork or exemption was added. Protocol codecs are independent work,
still required before Host migration, and must not introduce Agent or Runtime.
Actual Canic installed/native consumer qualification remains required.

## Canister-signature consumer

For Auth's broader migration, [Platform #7](https://github.com/dragginzgame/ic-platform/issues/7)
now has qualified local primitives, certified-issuer mappings, certificate
construction and all five installed cases without legacy Platform. Published
cohort delivery remains separate. See [Auth evidence](qualification/auth-crypto.md).
The separate [Wire/Types cohort issue #8](https://github.com/dragginzgame/ic-platform/issues/8)
is resolved by published Wire 0.24.2/Types 0.21.1 with actual unchanged generator
and retained browser-codec checks; see [cohort evidence](qualification/auth-wire.md).

Current read-only inspection of ic-auth at `b78991e9833320692921bf766e51ad361c9c8c4d`
found its unchanged `crates/ic-auth/src/issuer_signature.rs` caller of
`verify_canister_sig_with_time`. It owns protected issuer/seed, fixed-domain claims
message framing, signature limits, root selection and signing-time freshness.
The isolated qualification adapter now passes native/Wasm execution against
observed signatures for that domain; it preserves the caller's lack of delegation
freshness through an explicit separate full-range window. Source hashes,
acceptance differences and remaining migration obligations are recorded in the
[canister-signature contract](qualification/canister-signature.md).
Actual Auth compilation/adoption, complete credential policy and Canic integration
are not established. No sibling changes or consumer migration are included.

## Other retained contracts

| Consumer / canonical source | Owner | Required acceptance |
| --- | --- | --- |
| Query selected-subnet maintenance time, [replacement #1](https://github.com/dragginzgame/ic-platform/issues/1) | Platform certificate projection | Authenticate signing time for the requested subnet, preserve explicit root/freshness inputs, require trusted root-subnet binding without delegation, and retain original response bytes. The local `authenticate_subnet_time` public API and PocketIC evidence are documented in the [certificate record](qualification/platform-certificate.md#selected-subnet-signing-time--issue-1); The borrowed Agent response type also passes isolated consumer qualification without re-encoding; Query sampler integration and delivery remain consumer work. |
| Query Registry settings, [#169](https://github.com/dragginzgame/ic-platform-legacy/issues/169) | Protocol | Absent vs present-empty nested messages, absent vs false/zero scalars, raw replica-default sentinels, shared nested budgets, duplicate-singular and malformed-wire refusals. Configured SEV is not attestation. |
| Query marketplace/availability, [#166](https://github.com/dragginzgame/ic-platform-legacy/issues/166) | Protocol | Six node classes, arbitrary naturals, signed timestamps, daily units, exact arity and application errors; unready Registry copy differs from an empty page. Consumer owns pagination. |
| Backup signed recovery inspection, [#167](https://github.com/dragginzgame/ic-platform-legacy/issues/167) | Platform native packet boundary | Strict read-state field/kind admission, original bytes/paths, upstream request-ID identity, duplicate/unknown/malformed/trailing refusal and honest input-bound semantics. No transport/retry or Backup policy. |
| Canic endpoint composition, [#164](https://github.com/dragginzgame/ic-platform-legacy/issues/164) | Runtime | Guard and pre-copy admission before decoding, explicit codecs, one dispatch/reply owner and installed rollback/callback proof; no Wire-specific execution owner. |
| Query certified classification, [#158](https://github.com/dragginzgame/ic-platform-legacy/issues/158) | Platform native | Actual upstream SubnetType identity, unknown strings and absent values; classification is not capacity or deployment permission. |
| Crypto/Agent boundary, [#50](https://github.com/dragginzgame/ic-platform-legacy/issues/50) | Platform crypto/native | Recheck selected graphs and current advisory evidence before admission. Direct ECDSA correction alone does not resolve Agent/derivation paths. No in-house crypto, forced incompatible override or inherited risk acceptance. |
| Tooling/compiler adoption, [#157](https://github.com/dragginzgame/ic-platform-legacy/issues/157) | Repository tooling | Retain Rust 1.100.0, common lints and shared dependency commands. This checkout selects published Toko 0.44.9 through its authenticated installer; product acceptance remains separate. |

The first projection must not import native/crypto dependencies to satisfy an
unrelated row. Conversely, an isolated codec pass cannot qualify those rows.
Closed supplying work remains contract evidence; read its resolution and pinned
implementation before deciding what to reuse or retire.

The opt-in [Registry settings projection](qualification/registry-settings.md)
now has local wire/refusal tests, shared nested field accounting and independent
public consumer/Serde tests. Standalone Wasm admission and seven native zero-heap
workloads pass locally. Ten focused tests and topology/CLI Clippy now pass against
a source-bound snapshot of Query's implemented settings report using Git-pinned
Protocol 0.1.0. The later
[published-cohort qualification](qualification/registry-settings.md#published-cohort-and-delivered-consumer--16-november-2026)
completes supplier acceptance for #3 with delivered settings source and registry
Protocol 0.2.3/Types 0.21.1. Query's production dependency delivery remains separate.
Native-only composition with published Platform's Registry envelope now also
passes application-error, content-presence, mutation-evidence and refusal tests.
Envelope and payload budgets are independent; the selected payload's nested
messages share its allowance. The [consumer evidence](qualification/registry-settings.md#source-bound-query-settings-acceptance--2026-11-16)
records the exact dirty source/lock and retains the unresolved Agent advisory.

The [fee/status slice](qualification/cloud-engine-observations.md) adds two
bounded responses from #166. Literal and published-oracle checks pass locally;
the separate [marketplace slice](qualification/marketplace-prices.md) now adds
price rows, bounded arbitrary naturals and all six node classes. Availability
[responses and application errors](qualification/available-nodes.md) now preserve
unready/empty distinctions and strict present-page admission. Bounded page
requests now match the published encoder's bytes; actual Query migration remains
pending.

The [standalone marketplace Wasm corpus](qualification/marketplace-wasm.md)
passes for price rows, availability, page requests and fee/status observations.
This includes 32-bit count overflow and large-natural conversion; installed IC
execution and actual consumer adoption remain separate.

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
An [isolated Query integration experiment](qualification/query-engine-view.md)
now passes the actual committed caller's seven EngineView tests before and after
adaptation, topology Clippy and CLI compilation, with one canonical Types
identity and no registry-version updates. That isolated experiment left the
sibling checkout untouched and established consumer compatibility.
The same checks now also pass with Protocol pinned to its public GitHub commit;
a reviewable six-file patch and lock are retained with that evidence. After
separate authorization, it was applied to the real Query working tree and the
same checks passed there. The change remains uncommitted/unpushed in Query;
consumer delivery is not claimed.

The first slice's full gate remains open: Query integration is applied locally,
but consumer delivery, broader
numeric/Serde compatibility and physical allocation, instruction-count and Wasm
performance thresholds have not yet been fully qualified. The private
declaration supports required and strict optional fields, not a public generic
Candid-compatible optional API. No legacy surface is retired on this evidence.

Extend the map with actual consumers of remaining protocol families, identity,
Runtime, verification and browser subpaths before removing their surfaces. Record
source/lock/compiler identities and positive/negative proof owners. Missing tests
do not establish unused APIs; issue closure does not establish replacement proof.

## Testkit management HTTP request encoding

[Issue #5](https://github.com/dragginzgame/ic-platform/issues/5) requires one bounded
Candid management `http_request` argument for Testkit held replies. Protocol's
opt-in `http-request` capability supplies GET, verbatim UTF-8 URL/body, explicit
response limit/replication, empty headers, absent transform and pricing version.
Complete encoded-size admission precedes allocation; exact cap passes and
cap-minus-one refuses without partial output. See the
[independent schema evidence](qualification/http-request.md). Testkit owns cycles,
URL construction, simulator behavior and downstream installation/adoption.

The crypto/Agent retained obligation was rechecked on 2026-11-16 against
[RUSTSEC-2026-0387](https://rustsec.org/advisories/RUSTSEC-2026-0387): the patched
range remains `crypto-bigint >=0.7.6`. Platform 0.34.4's published primitive addition
now selects K256 0.14.0 and crypto-bigint 0.7.11 with an explicit compatible
security floor. See the [Auth primitive evidence](qualification/auth-crypto.md).
Agent/derivation #4 remains unresolved; neither a corrected direct ECDSA graph
nor the absence of Agent from this product graph qualifies those consumer paths.


## Storage synchronous system operations

[Issue #6](https://github.com/dragginzgame/ic-platform/issues/6) requires raw
argument/caller copies with offsets, live controller checks, task-free synchronous
replies, byte traps and instruction counter zero for caller-owned fixture exports.
Runtime owns this opt-in dependency-free `system-raw` surface; Storage retains
buffer, authorization, persistence, export and upgrade policy. The
[contract and qualification](qualification/runtime-storage.md) records installed
supplier evidence and pinned actual consumer compilation. Publication and
Storage's own installed migration cases remain distinct acceptance obligations.

### Scheduling system probes and Entropy management framing

The [stable-memory and cycle boundary](qualification/runtime-memory-cycles.md)
maps Scheduling's checkpoint and submission-refusal requirements to dependency-free
Runtime APIs. Installed supplier proof does not replace actual Scheduling timer,
once/watchdog and upgrade qualification. The [raw_rand contract](qualification/raw-rand.md)
maps Entropy's fixed seed framing and explicit version-one HTTP cost/encoding
requirements without moving source custody, retry or mixing policy into Platform.

Wasm Store's [management chunk-cache contract](qualification/management-chunks.md)
assigns bounded request/hash-record codecs to Protocol while leaving cache
admission, authority, epoch fences, hash comparison and retries with the consumer.

Backup's [original ingress packet boundary](qualification/ingress-packets.md)
preserves strict content, original bytes and upstream request identity. Decoding
never substitutes for direct-key policy, certificate authentication or recovery
admission. Snapshot/status supply and installed recovery remain separate gates.

The pending [snapshot/status protocol boundary](qualification/management-snapshots.md)
supplies Backup's finite management message profile with independent native/Wasm
checks. Actual caller migration, installed recovery and publication are not
established by these codec tests.

### Blob signed and Caffeine supplier slices

[Platform #17](https://github.com/dragginzgame/ic-platform/issues/17) belongs to Platform
native transport; [Platform #18](https://github.com/dragginzgame/ic-platform/issues/18) belongs
to Protocol. Explicit Ed25519 signing/original recovery and selected bounded
Caffeine codecs are implemented with independent and installed qualification.
Full CLI custody/management migration remains separate. See the
[contracts, evidence and remaining gates](qualification/blob-suppliers.md).

The selected [management/ledger supplier](qualification/management-ledger.md) now
implements Platform #20 locally, including provisional/ledger creation, controller
settings, exact install/upgrade arguments, complete duplicate/error outcomes and
canonical balance/fee accounts. Production Canic migration remains separate.
