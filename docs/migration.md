# Candid and runtime ownership

## Selected canister management DTOs

The maintainer authorized the canister-used subset of management 0.11.0 for the
pending breaking 0.9.0 line. The existing module/feature name stays. On Wasm,
43 explicit public names now refer to Platform-owned types, backed by 51 complete
upstream declarations. The selection follows non-test Canic Core/control-plane
and Blob source references, plus the maintained Canic runtime-probe canister's
HTTP outcall contract. All supporting records, enum variants, optional fields,
Serde attributes and Candid function signatures are retained. This adopts no
transport, storage, signing implementation or application policy.

Native builds retain all 57 existing registry 0.11.0 exports and their identities.
Fourteen snapshot-transfer exports used only by native Backup/Testkit/tooling are
not available on Wasm. Do not assume registry/owned Rust identity across targets;
external Candid bytes and schemas remain compatible. The original exact supplier
constraint below continues to apply to native DTOs and is superseded only on Wasm.
Sibling source checkouts and release versions remain unchanged.

The owned definitions live in `management_canister_types/maintained` inside the
sole consumer library. Provenance records the exact source revision, selected
names and hashes of the audited sibling files. Apache-2.0 terms are retained.
Native tests compare all 51 selected schemas to the pinned original; independent
management wire fixtures and installed Wasm checks cover HTTP, history, status,
snapshot list/delete and lifecycle/signing envelopes. Native transfer tests retain
their original registry identities. No consumer adoption or live IC acceptance is
implied by this qualification.

Wasm selects Candid bignum/serde_bytes explicitly, without the registry DTO package
or its default-enabled printer. The final owned status-codec probe saves 55,386
bytes (6.6%) of stripped Wasm; the earlier manifest-only investigation saved
about 63 KB. Neither measures a Canic production binary.
The same decoder and admission rules remain; some diagnostic type descriptions
use Debug formatting. Native tools retain the printer for diagnosis. Another
dependency explicitly selecting Candid's printer can re-enable it; graph checks
reject its implicit selection through the management-only Wasm profile.

## Candid codec packaging investigation

The 2026-10-30 investigation tests shipping the lean reader without requiring a
consumer Cargo patch. Registry Candid 0.10.39 still owns CandidType, Nat/Int,
Principal, Type/TypeEnv and Error. An isolated codec package supplies the framing
reader, serializer, deserializer and required private helper copies. It accepts
existing registry-derived management DTOs without conversions. The same modules
could live inside Platform; the experimental crate is not a new public offering.

This preserves data-type/trait identities but does **not** transparently replace
the current codec contract. Registry ArgumentDecoder/ArgumentEncoder accept
concrete upstream IDLDeserialize/IDLBuilder values. DecoderConfig also has private
state. Three compile-failure probes demonstrate those incompatible identities.
The candidate retains registry Error identity but maps parser failures to Custom
instead of Binread; diagnostics and public binrw parser interfaces differ.
A production migration would have to resolve these obligations explicitly.

| Status-codec probe, custom sections removed | Bytes |
| --- | ---: |
| One endpoint, registry codec | 793,631 |
| One endpoint, owned codec + registry Candid types | 786,000 |
| Two endpoints, registry codec only | 803,444 |
| Two endpoints, one owned and one registry codec | 1,131,626 |

The reproducible single-codec saving is 7,631 bytes (0.96%); mixed use adds
328,182 bytes (40.85%). The earlier 22,867-byte result replaced the whole Candid
package and does not establish savings for this identity-preserving arrangement.
Cargo still includes binrw, array-init, bytemuck and byteorder, plus build-only
binrw_derive, because registry Candid declares them unconditionally. Maintaining
this candidate copies about 3,040 source lines including comments and helpers.

Recommendation: do not adopt this as an incremental optimization. It would need
a coordinated codec/configuration/argument-API migration and a consumer graph
that excludes calls into the registry codec. Existing external data identities,
Wire ownership and the 16-level admission rule remain required. No production
codec, dependency selection or sibling source changes in this investigation.

Reproduce with `bash scripts/benchmarks/0.9/candid-packaging/run.sh` using prepared
Rust/Node caches. The pinned upstream inputs, generation/edition patch and Apache
license remain in that directory. Its `evidence.json` records source/artifact and
lock hashes; detailed receipts are in `target/candid-packaging.n6WZul`.
Owner/candidate/consumer strict Clippy, two native identity/DTO tests per variant,
191,205 differential cases, the API refusals and Rust 1.91 Wasm compilation pass.
These are probe results, not installed-canister, fuzz or production qualification.

## Runtime-only Cargo singleton guard (#103)

Pending 0.9.1 moves only Cargo's inherited executor `links` declaration and empty
build script into `ic-platform-runtime-guard`. The facade's `executor` feature
selects this optional, dependency-free internal package. Its exact version must
match the facade; a compatible range could let different executor owners share
one guard and evade Cargo's refusal. Release preparation coordinates that exact
requirement, the documented pin exception and the qualification lock. Publication
includes the helper before the facade. It exports no API, implementation or
runtime symbols; the executor and IC0 sources, licenses and callback semantics
stay unchanged in Platform.

Native default/gzip-only and IC0-only consumers can resolve alongside registry
`ic-cdk-executor =2.0.0`. Enabling `executor`, `runtime` or `cdk` still rejects
that mixed graph, with and without gzip. `scripts/ci/test-runtime-singleton.sh`
checks real registry resolution and compiles the gzip consumer. It also checks
two facade patch versions: native-only and single-runtime graphs resolve, while
two enabled runtime owners fail. Cargo resolves all
features declared by workspace roots when making a lockfile: a native root that
itself offers a Platform-runtime feature still requires compatible executor
suppliers, even when that feature is inactive. Merely using a non-runtime
capability through a dependency does not activate Platform's optional guard.

This fixes the packaging prerequisite for native Profiler adoption tracked in
[#103](https://github.com/dragginzgame/ic-platform/issues/103). Publication and
adoption of the supplying release remain separate; the native source profile
continues to select its existing published Platform version.

## Embedded IC0 and executor

The maintainer authorized this breaking production-dependency cut on 2026-10-30.
Platform embeds IC0 1.2.0 and ic-cdk-executor 2.0.0 instead of selecting their
registry packages. This section supersedes the older exact-supplier constraints
and no-executor-fork statements below. Package versions remain 0.8.5 during local
implementation; the pending release must be 0.9.0, not 0.8.6. Publication and
consumer adoption remain separate.

Existing `ic_platform::ic0` and `ic_platform::executor` paths remain. MethodHandle
and TaskHandle now have Platform identity. The executor uses the same owned IC0
implementation as the raw calls and CDK adapters. IC0 selects no dependencies;
executor selects IC0 and SlotMap, without Principal, Candid or a host
parser. Defaults remain empty. The implementation is packaged inside Platform;
no additional consumer crate, patch-based distribution or second executor exists.

The maintainer selected standard Vec task lists after measuring the SmallVec
alternative, prioritizing production Wasm size. This removes SmallVec from the
executor/runtime dependency graph; host tooling can still select it transitively.
The Rust 1.99.1 Wire runtime fixture shrank by 2,762 bytes (1.1%) after removing
custom Wasm sections. Fresh small-method benchmark batches used one extra
allocation and a few hundred additional IC instructions; the tradeoff is not a
general speed improvement or a whole-request latency measurement. Raw samples,
harness, source variants and artifact hashes are retained under
`target/vec-smallvec-benchmark/`.

Cargo's inherited executor `links` value rejects combining this owner with the
registry executor, and rejects two new Platform package identities. Release 0.9.0 applies
this guard package-wide, including native/default-only selections; the pending
[#103 fix](#runtime-only-cargo-singleton-guard-103) narrows it to executor selections. Rebuild the
application and all runtime consumers against the same new Platform line. Timers
must adopt that requirement before production migration; a transitive old
Platform requirement cannot be bridged with handle conversions. The installed
fixture selects published Timers 0.32.3 and patches its compatible 0.8 requirement
to the local pre-release source solely for qualification. After the minor release,
its dependency requirement must be coordinated; this patch is not consumer supply.

IC0's retained ABI wrappers and raw declarations are byte-for-byte source imports,
with complete Apache-2.0 terms and source fingerprints. They retain existing
safe/unsafe signatures, initialization guarantees and IC traps. Native stubs still
panic. The executor is maintained from the pinned upstream source with documented
changes: polling takes only the future, retaining protected-method routing;
wakeups are coalesced without losing a wake during polling; one waker allocation
is reused per task; completed/cancelled task IDs and expired method queues are
reclaimed. Each method owns its wake queue directly, without a secondary map.
Bulk cancellation clears old wakeups before dropping futures and preserves any
tasks spawned by their destructors. Task context is derived from its binding.
Future destructors run outside executor RefCell borrows. Native
bookkeeping no longer installs an IC syscall panic hook; Wasm retains it.
No cryptographic, wire, storage, retry or timer-policy implementation changes.

Qualification covers protected-query self-wakes, wake storms, cross-method wakes,
stale generational handles, cancellation/drop behavior and long-lived method
bookkeeping. The installed lifecycle fixture adds the self-waking query and waker
identity regression alongside raw/Wire calls, external Candid, Timers, callback
traps, cleanup and upgrades, on the development compiler and MSRV. Source guards
retain the imported callback and crypto fingerprints. Native tests and graph
admission alone do not establish installed correctness; Linux PocketIC evidence
is separate from macOS and live-provider qualification. No size or speed saving
is inferred from dependency removal.

## Minimal Principal and selected ICRC packaging

The maintainer authorized this breaking packaging cut after source release 0.7.6.
It shipped in source release 0.8.0 and remains the current 0.8.2 contract; registry
publication and consumer adoption are separate. This contract supersedes the
earlier combined Principal capability and
registry-ledger Rust identity promises in the historical supplier sections below.
Registry executor =2.0.0, ic0 =1.2.0 and management DTO =0.11.0 remain unchanged.

`principal` supplies owned Principal/Account bytes, bounded text and Serde without
the registry `ic_principal`, SHA-2, Candid or runtime. `runtime` and `cdk` select
only this owned capability. `principal-external` adds the registry Principal/error
identity and byte conversions, without self-authentication or a codec. Required
Candid/client/DTO and signature features select the conversion capability.

For a consumer selecting `principal-external`, conversion is still one step:

```rust,ignore
use ic_platform::{Principal, principal::ExternalPrincipal};
let owned = Principal::anonymous();
let external: ExternalPrincipal = owned.into();
assert_eq!(external.as_slice(), owned.as_slice());
```

Without that capability, callers can pass `owned.as_slice()` to an external
constructor at their integration boundary. Select `principal-self-authenticating`
explicitly to use `Principal::self_authenticating`; its hashing delegates to the
unchanged registry implementation. The registry conversion dependency selects
only `convert`; other integration dependencies may independently unify more
upstream features. No new Candid implementation is added to the owned type.

`signature-verification` explicitly enables the registry Principal's Serde support
for certified canister-range CBOR. Its standalone selection remains Candid-free;
conversion-only and owned Principal selections retain their narrower graphs.

Keep selecting `icrc-ledger-types` and using the existing flat module and nested
`icrc21` imports. Platform now maintains only the selected DTOs and helpers from
upstream 0.2.0, retaining its Apache-2.0 license/provenance. Account, transfer,
consent, generic value and archive callback Rust identities are Platform-owned.
Candid Principal/Nat/Int/function and Serde byte identities remain upstream.
There is no normal or development dependency on the registry ledger package.

Preserve account effective-zero equality/order/hash and absent versus explicit
zero serialized fields. Legacy Value's Nat64 remains distinct from ICRC3Value.
The complete imported value-hash bodies, integer encoders and fixed vectors are
unchanged and guarded by original-source reconstruction. No hash/domain policy,
storage format or cryptographic implementation is replaced. The selected types
omit CBOR derives, storage integration, metadata-key helpers and consent display,
number and time formatting; those methods/traits are no longer supplied.

Frozen schemas from the independent registry encoder cover all 28 selected types,
including recursive values and typed query callback signatures. Existing byte,
account and hash vectors remain separate checks. Consumer final graphs, source
and persisted Serde/callback uses need qualification before adoption. Do not
normalize or reconstruct retained uncertain requests. No Wasm-size saving or
live provider acceptance follows from removing package edges alone.

## Shared ICRC-10 discovery record (since 0.7.5)

Select `icrc-ledger-types` and import
`ic_platform::icrc_ledger_types::icrc10::SupportedStandard`. This passive external
DTO is the Candid record `{ name : text; url : text }`, not a tuple. Values are
preserved without name/URL policy. Select `candid` separately when the consumer
needs Platform's encode/decode APIs; no direct Candid dependency is necessary.
The types feature alone supplies the DTO, without selecting those codec APIs.

The independent source contract is [ICRC-10's DID at 1df0c6a](https://github.com/dfinity/ICRC/blob/1df0c6af6ecca4f4eb1956c67b29c0d13279f1cd/ICRCs/ICRC-10/ICRC-10.did).
Platform independently represents its protocol facts; no upstream Rust type or
cryptographic implementation is replaced. ICRC-21 now uses the selected
Platform-maintained DTO identities described above; external Candid types and
wire schemas remain upstream-compatible. The ledger-types feature explicitly
selects the derive dependency without selecting Platform codec APIs.

Canic's prepared optional adapter replaces its local `SupportedStandard`
declaration/derives with this import, retaining one public type identity. Its
required Platform floor becomes **0.7.5**, pending maintainer release/publication.
Keep its mounting macros, framework guards, explicit argument budgets, advertised
standards and application-owned consent handler there. There is no Platform
endpoint, global registry or configuration flag for ICRC discovery/consent.
[Canic #706](https://github.com/dragginzgame/canic/issues/706) tracks the pending
0.113 integration. No sibling checkout or production version is changed here.

The Platform-only `icrc10-types` consumer proves ledger-types-only availability;
`icrc` encodes/decodes a response; `external-contract-schema` independently checks
named field types and identical schema-generated reply bytes. They qualify native
Rust 1.91/current and applicable Wasm profiles. Native macOS and consumer/provider
installation remain separate. The addition does not fix #73 nesting admission.

## External endpoint and protocol codecs (#78, #79)

The compatible additions below are present in **source release 0.7.4**.
Registry publication has not been verified in this pass.
Existing external DTOs remain available from their documented earlier suppliers.
Owned application endpoints, including public ones, remain Wire. Required IC,
ledger and pre-existing provider interfaces explicitly select Candid.

### Canic external entrypoints

Select `candid-endpoint` in the canister manifest. Select `candid-service`
**separately** in native DID-generation/testing profiles; the endpoint feature
registers native metadata but does not select the printer or parser. Neither
feature is implied by raw/runtime/executor or Wire profiles. No direct `candid`
or `ic-cdk` dependency is necessary.

```rust,ignore
use ic_platform::candid::endpoint::{self, ArgumentLimits};
const LIMITS: ArgumentLimits = ArgumentLimits {
    max_bytes: 4096,
    decoding_quota: 100_000,
    skipping_quota: 1_000,
    max_type_len: 64,
    max_header_len: 1024,
};
#[endpoint::query(
    name = "icrc10_supported_standards",
    limits = "LIMITS",
    empty_args = "reject",
    guard = "authorize_external"
)]
fn supported_standards() -> Vec<ExternalStandard> { external_standards() }
```

These allowances are illustrative, not Platform defaults. Canic's wrapper can
emit `platform = "::canic::__internal::ic_platform"` for its existing re-export.
Platform's adapter emits codecs and delegates to the same CDK generator and
registry executor. Application guards execute in their supplied order before
byte admission; the scalar size check precedes argument copying. Decode refusal
traps before the handler. Reply encoding refusal traps, preserving synchronous
mutation rollback. Async boundaries retain the existing executor's commit/trap
semantics; a trap after an await cannot undo earlier committed effects.

Arguments use normal Candid tuples: zero arguments are `()`, one is `(T,)`, and
multiple are `(T, U, ...)`. A scalar return encodes one value, a tuple return
encodes its elements, and `()` encodes **zero results**. For an external single
null result, write a distinct `type Null = ();` return or `((),)`; do not confuse
it with an empty reply. `composite` explicitly selects composite query mode.
`empty_args = "reject"` requires an actual envelope, even for zero arguments.
`"unit"` alone admits missing bytes as a legacy zero-argument envelope; it does
not supply missing required values. Additional arguments retain quota-bounded
Candid skipping. Byte, decode-work, skip-work, header and type allowances are all
caller-selected. Argument decoding also applies the fixed
[16-level wire ceiling](#external-candid-nesting-qualification-73) below.

Canic can delete Candid declaration/decode/reply emission in
`canic-macros/src/adapter` and `endpoint/expand`, and replace its local argument
configuration/decoder in `canic/src/endpoint` with
`ic_platform::candid::arguments::{ArgumentLimits, EmptyArgs, ArgumentDecodeError}`.
Raw codec-owner adapters may use `candid::endpoint::read` after their own guards.
Keep Canic's endpoint names, contract selection, guards, startup order, lifecycle,
measurements and semantic validation. `canic/src/macros/start` may reuse native
registration and existing `candid::contract::export_service!` with an explicit
facade path. Do not add registration to owned Wire endpoints.

### Canic and Host external calls

Select `candid` and only the required DTO feature(s). The pure
`ic_platform::candid::protocols` operations provide `METHOD`, `encode` and
`decode` without selecting runtime, Agent, executor or host parsing. `decode`
requires explicit `DecodeLimits` and `ReplyArity::{Exact, Compatible}`. Exact
requires the declared single result (including optional results) and refuses
extra top-level values before skipping; Compatible retains upstream optional
missing-value and quota-bounded skip behavior. Protocol `Result` errors remain
inside the decoded value, separate from codec and transport failures.

```rust,ignore
use ic_platform::candid::protocols::{cmc::notify_top_up, ReplyArity};
let request_bytes = notify_top_up::encode(&request)?;
// Persist the exact bytes and consumer intent before any paid effect.
let reply_bytes = caller_owned_transport(destination, notify_top_up::METHOD, &request_bytes).await?;
let provider_result = notify_top_up::decode(&reply_bytes, reply_limits, ReplyArity::Exact)?;
```

| Consumer responsibility / existing implementation | Shared operation(s) |
| --- | --- |
| `canic-core` infra/API/workflow generic Candid call glue | Typed `protocols` request/reply codecs; `ic_platform::call` owns prepared one-shot execution separately |
| Management history, controller handoff, installation, snapshots and signing envelopes | `management::{canister_info, canister_status, update_settings, upload_chunk, stored_chunks, clear_chunk_store, install_chunked_code, uninstall_code, start_canister, stop_canister, delete_canister, deposit_cycles, list_canister_snapshots, delete_canister_snapshot, raw_rand, ecdsa_public_key, sign_with_ecdsa}` |
| Host `icp/{candid,management}` and operator-mint transport | Same pure operations over the existing Host/Agent transport; no generic Candid encode/decode owner remains necessary |
| Operator mint/rate/top-up and ICP receipt requests | `cmc::{get_icp_xdr_conversion_rate, notify_top_up, notify_mint_cycles}` and `icp_ledger::{transfer, query_encoded_blocks}` |
| ICRC balances, fees, transfers and receipt readback | `icrc::{balance_of, fee, transfer, get_tip_certificate, get_blocks}` |
| Cycles creation/withdrawal and provider receipt blocks | `cycles_ledger::{create_canister, withdraw, get_blocks}` |
| Registry target-subnet lookup | `nns_registry::get_subnet_for_canister` |
| Native assets | `certified_assets::{get, get_chunk}` |
| ICP encoded-block archives | `archive_encoded_blocks::{encode, decode}`; **no fixed METHOD**: retain validated callback destination/method |

Cycles Ledger's pinned `icrc3_get_blocks` provider uses legacy `Value`, including
`Nat64`; select **both** `cycles-ledger-types` and `icrc-ledger-types` for this
operation. Its `cycles_ledger_types::{BlockWithId, ArchivedBlocks, GetBlocksResult}`
preserve that full schema and upstream callback identity. The existing standard
ICRC-3 `icrc_ledger_types` identities remain unchanged; do not use their narrower
`ICRC3Value` projection to decode this provider. No value/hash conversion occurs.

Replace Host's local generic codec helpers with these operations and its
external service parsing/extraction with existing `candid::service` and native
`candid::parser` APIs. Service text remains inspection of an external contract;
it does not choose an owned endpoint's codec. Do not generate owned Wire clients
from DID or restore retired JS/TS compiler exports.

Canic/Host still own destination/callback admission, signing authority, cycles,
fees, intent journals, original request custody, receipts/certificates, trust,
validation and retry/uncertainty. Encoding is usable before submission. Keep
retained request bytes unchanged on reconciliation/retry; re-encoding new DTOs
is not a migration for already-uncertain effects. Decode failure/timeout never
establishes no remote effect or authorizes resubmission. The supplying change
neither edits nor claims adoption in Canic or Host.

### Qualification and adoption

After full `make clippy`, run the standalone Platform-only profiles:
`bash scripts/ci/check-platform-consumer.sh candid-endpoint external-codecs external-contract-schema`.
They cover native/MSRV, Wasm where applicable, exact external schemas and native
service metadata. Independent management bytes and pinned provider DIDs anchor
compatibility; the provider `Nat64` regression is generated from the pinned DID.

The existing installed runner includes the endpoint cases for local sources:
`POCKET_IC_BIN=... bash scripts/ci/check-platform-runtime.sh --prepared-call`.
It checks valid and refused arguments, every budget, guard order, scalar/tuple/
zero/null results, synchronous rollback, composite query and management await,
alongside its existing callback/trap/upgrade subjects. Published historical
qualification remains available with `--published X.Y.Z`; request the new
subjects explicitly with `--external-adapters --published X.Y.Z` once the
supplying release is available. Native macOS, actual Canic migration tests and
installed provider/ledger acceptance are separate qualifications. No release,
publication, deployment or paid provider operation is performed by this change.

## DER framing admission (#72)

The maintainer approved one owned envelope check before the unchanged imported
canister-key parser. Truncated prefixes, inconsistent/noncanonical lengths, wrong
tags/OID/unused-bit framing and trailing data now return errors through
`CanisterSigPublicKey::try_from` and both verification functions. Valid existing
keys and signature vectors retain their bytes and results. No public type,
cryptographic implementation, encoder or seed/issuer policy changes.

The fixed-offset upstream parser's supported grammar remains short-form DER;
this does not add long-form support or repair the encoder's oversized-length
integer conversions. Consumers retain size/depth admission and authority checks.
Source checks require the single admission call at parser entry and reconstruct
the original fingerprints without changing their recorded hashes.

## External Candid nesting qualification (#73)

Source release **0.7.5** applies one fixed rule: bounded Platform decoders reject Candid
nested beyond **16 constructed levels** before upstream header construction or
decoding. Exactly 16 levels are allowed. Scalars have depth zero; an empty
container still uses one level. This applies to `decode_one`, `decode_args`,
`decode_exactly_one`, typed protocol operations and external endpoint adapters.
Call them as before; no policy object, nesting setting, macro option or manifest
change is required. The maintainer explicitly authorized this ceiling in place of
the earlier unpublished opt-in proposal.

```rust,ignore
let reply: ExternalReply = ic_platform::candid::decode_exactly_one(&bytes, limits)?;
// Existing endpoint declarations also receive the same fixed admission rule.
```

Byte, header and type-entry ceilings still apply. Admission work is capped by the
saturating sum of the existing decoding and skipping quotas; it charges consumed
bytes, graph roots/edges and visited values, including zero-byte vector elements.
These units count framing work, not upstream decoding work. Candid then receives
its original quotas unchanged. There is no additional configurable budget. Wide
DAG traversal consumes work, so shallow type graphs cannot amplify without bound.

The iterative framing walk checks every type definition, including unused ones,
and every value, including extra arguments and unknown record fields. Recursive
type declarations remain allowed; each simple constructed-type path and actual
recursive value must fit 16 levels. Large byte blobs are not interpreted as nested
option tags. Unsupported future constructors and opaque references fail closed.
Candid still owns semantic/type/UTF-8 validation. No bytes are rewritten, external
identities replaced, encoding modified, decoder forked or unsafe code introduced.

Existing function signatures, limit structs and exhaustive error matches remain
compatible. `DecodeError::nesting_error()` and
`ArgumentDecodeError::nesting_error()` expose the typed admission cause through
the existing Candid error variant. Entrypoint guards still run first, byte length
is checked before copying, and refusal precedes the handler. Retry, disclosure,
authorization and external protocol interpretation remain consumer-owned.

Upstream `candid::decoder` and `candid::contract` decoding APIs/macros retain
their original behavior. Their staged callers can explicitly run
`candid::nesting::check(&bytes, limits)?` before constructing a decoder. The
16-level wire check does not guarantee stack safety for arbitrary trusted expected
Rust schemas or custom Serde visitors. Owned application endpoints remain Wire.

The original installed reproduction sends 8,192 nested options in only 8,202
bytes. Candid 0.10.39 has no effective Wasm depth setter; the unchanged decoder
still has that defect. Qualification must show the boundary returns typed
refusals for that input and 17 levels, accepts 16, checks unused deep headers,
and preserves ordinary external DTO bytes. Standalone/native/MSRV/Wasm and local
installed qualification are separate from native macOS/live provider acceptance.
The preflight adds work; no speed or Wasm-size reduction is claimed.

The maintainer accepted the measured cost on 2026-10-29 and retained the fixed
16-level rule as implemented. Consumers own refusal/cost metrics and abuse
correlation; further optimization requires evidence from those measurements.

### Admission cost (2026-10-29)

Installed local Linux/PocketIC measurements use Rust 1.99.1, the default release
profile and Candid 0.10.39. The baseline is the original 0.7.4 exact-single decoder
with the same byte/config/arity checks and value drops; the guarded path adds the
new preflight. Identical encoded DTOs are passed to both in one Wasm. Each update
performs one decode. Seven post-warmup samples alternate baseline/guard order;
values below are medians. Message-buffer copying and reply encoding are outside the
instruction span; allocations/copies/drops performed by decoding are included.
Cycle charges include the complete fixture update.

| Payload | Wire bytes | Extra decoder instructions | Decoder-span increase | Extra charged cycles/update |
| --- | ---: | ---: | ---: | ---: |
| Management raw_rand (32 bytes) | 42 | 4,221 | 19.70% | 4,221 |
| Management history (9 changes) | 703 | 108,050 | 15.31% | 108,050 |
| ICRC-1 transfer | 157 | 20,434 | 15.92% | 20,434 |
| ICRC-10 (2 standards) | 151 | 8,224 | 19.68% | 8,224 |
| ICRC-21 consent | 202 | 42,313 | 17.68% | 42,313 |
| ICRC-3 (100 synthetic blocks) | 14,044 | 1,129,011 | 9.68% | 1,129,011 |
| HTTP body (64 KiB) | 65,624 | 12,407 | 4.19% | 12,407 |
| HTTP body (1 MiB) | 1,048,664 | 12,407 | 0.34% | 12,407 |

Percentages compare **decoder-span instructions**, not whole-call fees. Extra
cycles are observed local balance deltas, not a mainnet/provider price projection.
The management history uses the existing upstream-produced fixture. Other
messages use real external types with synthetic payloads; ICRC-3 blocks contain
nested transfer maps. No provider business logic or network latency is measured.
The snapshot preserves bytes/type identities; no cryptographic implementation is
changed or separately benchmarked.

The byte-length-only harness takes 76–78 instructions; the equivalent IC0 argument
length harness takes 579–581. These include harness/loop costs and remain constant
across payload lengths. HTTP body preflight adds about 12,400 instructions for both
64 KiB and 1 MiB: byte contents are skipped by length. More structured values cost
more; 100 nested ICRC-3 block records add about 1.13 million instructions. Ordinary
structured DTOs incur roughly 10–20% extra decoding work in this corpus. This cost
is real; fixed message fees must not be used to describe the guard as free.

Separate queries measure guard-only, original decoding and combined decoding,
with seven samples and 20 repeated operations per sample. The original 8,202-byte
trap input is refused in 7,654 combined-span instructions in that repeated-query
measurement; unsafe baseline decoding is excluded for the over-depth cases.
Wire endpoints never invoke this preflight. No memory/size improvement, native
macOS measurement, MSRV performance equivalence or live IC acceptance is claimed.

Source, locks, graphs, raw samples, CSV summaries and frozen Wasm are retained at
`/tmp/platform-candid-cost-20261029`. Wasm SHA-256: `6f18280c3ffa2facf941af7bdd0ef9b4e6a8e35473245da8727e5b5c272afffb`.
The frozen server SHA-256 is
`d244e1138910d18dfc65e04f79304995eed0deee52cfc8cbebfcfd45341bcc99`;
it is separate from the earlier runtime qualification server. Platform and both
native/Wasm benchmark Clippy checks pass with warnings denied before execution.

## Host gzip boundary

The maintainer requested that IC Profiler access gzip through Platform rather
than depend directly on `flate2`. Select optional `gzip` and use
`ic_platform::gzip::{encode_gzip, decode_gzip, GzipError}`. These explicit exports
delegate to published IC Tooling `ic-host-artifacts` 0.71.0; general compression
implementation remains in Tooling. No backend type, default feature, Candid,
executor, filesystem effect or automatic limit is added. The dependency and
exports are native-only, including when `gzip` is selected on Wasm.

The pending 0.10 release adopts Tooling 0.71.0 for Host Artifacts, Testkit and
the contributor release editor. `GzipError` retains its upstream identity;
consumers exchanging it with direct Host Artifacts APIs must align to 0.71.
Platform-only imports stay the same. No retained gzip artifact, external IC
payload or consumer retry policy is migrated. This is a breaking dependency
identity change; the earlier 0.8.2 patch exception does not apply.

The opt-in native Profiler profile retains its qualified Platform 0.8.3, Host
0.70.3 and Wire 0.7.23 Cargo/Bazel inputs. Its gzip facade requires Host 0.70,
so converging that profile on Host 0.71 requires the next published Platform
supplier and separate native assembly qualification. Do not mix Host families
or substitute the current checkout into its registry-pinned source recipe.

The runtime harness imports raw PocketIC identities from Platform and borrows
Testkit’s managed instance through `instance()`. Testkit 0.71 uses Platform 0.9
and Wire 0.8. The combined fixture’s maintained lock cannot yet refresh because
Timers 0.32.3 still selects Platform 0.8’s external executor. Keep the existing
lock until Timers has a compatible supplier; do not weaken singleton admission.
Standalone native driver compilation and Wire codec checks are separate evidence
from the blocked installed lifecycle matrix. No retry or lifecycle policy moves
to Platform.

Consumers own compression levels, compressed/decoded byte allowances and payload
interpretation. Decoding admits one complete member and rejects trailing bytes;
callers migrating permissive readers must qualify that admission change. Existing
compressed artifacts retain their recorded byte identities; deterministic headers
do not promise identical bytes across backend/feature selections.

Compatibility changes are handled in the affected producer/consumer issues.
Record agreed versions, identities and migration requirements in this contract
and the existing dependency inventory. There is no central approval queue or
mandatory cross-repository acknowledgement before independent work proceeds.

## External DTO additions for Canic

Published Platform 0.6.20 supplies optional CMC, legacy ICP, Cycles
Ledger, NNS Registry and native certified-asset contract modules. See the
[external contract and migration guide](external-contracts.md) for exact features,
import mappings, pinned schema/license provenance and qualification commands.
Registry archive identity is verified; Canic consumer adoption is not claimed. Required
external Candid remains; Canic retains effects, limits, retry bytes and policy.

## Non-cryptographic size qualification (0.7.1)

The compatible Principal input wrapper now delegates to one `&str` parser;
human-readable Serde writes canonical text directly from its stack buffer.
Principal/Account errors use mechanical `thiserror` derives, preserving messages,
source chains and variant identities. The `principal` feature consequently adds
the existing optional `thiserror` dependency. Defaults and executor-only profiles
still exclude application types and Candid. Account formatting retains canonical
spelling, checksum inputs and absent/explicit-zero semantics.

Runtime System API text still validates UTF-8 and panics on invalid bytes. The
panic retains its reason but omits `FromUtf8Error`'s rejected-byte debug dump.
No call submission, callback, cancellation or executor body changes.

Run `bash scripts/benchmarks/wasm-size.sh [PLATFORM_SOURCE [NEW_OUTPUT_DIRECTORY]]`
with prepared Rust 1.91/Wasm and `ic-wasm`. This manual, serial harness uses
independent workspaces/locks, normalizes embedded source/probe paths, preserves
source and tool identities, and records
raw/optimized artifacts and hashes for five recipes and four capabilities.
`WASM_BENCH_PROFILES` can select a subset; `WASM_BENCH_TARGET_DIR` can reuse a
dedicated target directory between serial runs. `WASM_BENCH_RECIPES` selects a
subset of the five documented recipes. Binary-only and self-authentication
controls are also available as `principal-binary` and `principal-self-auth`.
It installs nothing and is not a
routine CI gate. Generated probe endpoints are qualification fixtures, not new
application protocols or performance benchmarks.

The [size observations](benchmarks/0.7.1-wasm.tsv) compare baseline `e918d23` with
this batch on Linux, Rust 1.91.0, `ic-wasm shrink`, stripped symbols and aborting
panics. All reported deltas compare identical recipes and probe sources:

| Probe / recipe | Before optimized bytes | After | Difference |
| --- | ---: | ---: | ---: |
| Principal/Account, `3-fat-1` | 64,890 | 62,312 | −2,578 |
| Principal/Account, `z-fat-1` | 92,405 | 56,402 | −36,003 |
| Raw async calls, `z-fat-1` | 43,359 | 41,798 | −1,561 |
| External management Candid, `z-fat-1` | 426,731 | 426,731 | 0 |
| Signature verification, `z-fat-1` | 302,661 | 302,661 | 0 |

These are reachable-capability probes, not production canister measurements;
gains are not additive and no individual change's attribution is claimed.
Build settings dominate some comparisons: Canic and IcyDB already select fat
LTO and one codegen unit in the inspected release profiles. `z` is not always
smallest: the final Principal/Account probe is 54,311 bytes with `s-fat-1` versus
56,402 with `z-fat-1`. Actual application selections/artifacts remain unqualified.
Canic guards
service generation with `canic_export_candid` for its declaration pass, and its
Core ECDSA/Ed25519 declarations are development dependencies. No redundant
production capability was established by those declarations alone, so sibling
features, cryptographic code and required external Candid remain unchanged.
No new IC instruction/cycle, installed-canister or native macOS result is claimed.

The existing native Account harness was also run in two alternating before/after
process pairs on Linux x86-64, Rust 1.99.1: five internal trials per case/process,
500,000 operations per trial, with unchanged external controls. The
[raw observations](benchmarks/0.7.1-native-types.tsv) show formatting medians
0.5–6.7% lower across the six owner/subaccount cases, with parsing roughly
unchanged (−2.3% to +1.1%). These native timings are observations, not IC cost
predictions. Principal/Account/Subaccount layouts remain 30/63/32 bytes on that
target; they are not persisted ABI guarantees.

### Attribution and binary controls

The [isolated-change trials](benchmarks/0.7.1-attribution.tsv) apply one change at
a time to the same Platform-owned baseline, using the original text/JSON probe.
They do not compare DFINITY's types against Platform's:

| Isolated change | Optimized delta, `3-fat-1` | Optimized delta, `z-fat-1` |
| --- | ---: | ---: |
| One non-generic parser body | −2,288 bytes | −35,561 bytes |
| Stack-buffer human serialization | −276 bytes | −310 bytes |
| Account formatting dispatch | −18 bytes | −137 bytes |
| Mechanical error derives | 0 | 0 |

The results identify duplicated generic parsing as the dominant cost in this
probe/compiler combination. Deltas are not additive: optimizer interactions
explain the small difference from the combined result. This is not 36 KB of
unnecessary machinery in the raw types. The
[binary-only control](benchmarks/0.7.1-controls.tsv), excluding text/JSON code,
stays exactly 8,157 bytes (`3-fat-1`) and 7,631 bytes (`z-fat-1`).
That control covers raw Principal admission/byte replies through IC0 and an
optimizable Account conversion; it is not an Account Wire/Serde codec or a
persisted-format benchmark.

A further compatible scaffolding change passes `public_key.as_ref()` at the owned
Principal wrapper before delegating to `ExternalPrincipal::self_authenticating`.
It normalizes the generic upstream invocation to one slice type without changing
any upstream cryptographic body, key bytes, hashing, tag or vector. The focused
control exercises owned Vec, borrowed Vec and slice inputs: optimized bytes fall
18,612 → 17,788 (`3-fat-1`) and 17,898 → 17,099 (`z-fat-1`). Both vector identity
and exactly-once evaluation of a custom `AsRef` are qualified. No self-authentication
execution-time or application-Wasm saving is inferred. The final text/JSON probe
retains the combined sizes in the table above.

### Candid sharing experiments: implementations retained

Two isolated prototypes tested shared exact-one decoder setup/finalization and
storing the caller's const sequence limit as private visitor data. Neither is
retained in the consumer library: constructor sharing showed a native-runtime
tradeoff, and sequence sharing increased the single-limit control's size. The
retained code still uses its original decoder staging and const-specialized
visitor. A new regression proves that an enormous *admitted* hint does not cause
reservation from the hint.

The [size matrix](benchmarks/0.7.1-candid-experiments.tsv) compares the same
pre-experiment working tree with each isolated variant and their combination:

| Prototype / probe | Optimized delta, `3-fat-1` | Optimized delta, `z-fat-1` |
| --- | ---: | ---: |
| Shared setup/finalization; four management reply DTOs | −56,116 bytes | −1,421 bytes |
| Runtime visitor limit; four distinct const limits | −11,653 bytes | −7,339 bytes |
| Runtime visitor limit; one const limit | +150 bytes | +132 bytes |
| Both prototypes; four distinct const limits | −113,035 bytes | −71,731 bytes |

The management probe uses pinned CanisterStatus, HTTP, ECDSA public-key and ECDSA
signature result identities. Sequence probes use a dedicated synthetic external
`vec nat8` contract with limits 0/2/16/256, or just 2 for the single-limit control.
These are qualification probes, not owned application protocols or evidence of
application Wasm savings. Select `candid-exact`, `candid-sequence` or
`candid-sequence-one` through `WASM_BENCH_PROFILES`; they are opt-in, not part of
the default sweep.

Full Clippy and the focused Candid suites passed for each prototype on native and
Rust 1.91. A Rust 1.99.1 standalone corpus compared 780 value/arity/malformed-wire/quota
cases: decoded canonical bytes and compact diagnostics matched baseline exactly
for all three variants. Existing sequence tests cover deceptive/missing hints,
typed excess-element failure precedence and zero bounds. The final retained
implementation passes native/MSRV suites and its independent consumer proofs.

The [native observations](benchmarks/0.7.1-candid-native.tsv) used Rust 1.91,
`opt-level=3`, fat LTO and one codegen unit: four process rounds, five 10,000-call
trials per case, after 1,000 warmup calls. The separate scalar/blob controls showed
1.7–13.8% higher median time with shared setup/finalization; sequence-only medians
were roughly neutral to lower. These host observations do not predict IC
instruction/cycle cost. They do rule out claiming an unconditional performance
win from the Wasm reductions. No such tradeoff was silently adopted.

Frozen evidence includes the [exact-one patch](benchmarks/0.7.1-candid-exact.patch),
[visitor patch](benchmarks/0.7.1-candid-sequence.patch),
[differential driver](benchmarks/0.7.1-candid-corpus.rs.txt) and
[native timing driver](benchmarks/0.7.1-candid-perf.rs.txt).
They are historical experiment inputs, not additional maintained implementations;
the combined variant applies both patches. Full source/probe/artifact hashes,
tool versions, logs and standalone manifests remain at
`/tmp/platform-candid-experiments`. Separate build targets qualified the variants;
source-path-normalized Wasm inputs were checked before/after each sweep.

## Instrumented PocketIC source provider

The maintainer approved moving native ownership from IC Profiler into Platform.
The opt-in Git source profile [native/ic-profiler](../native/ic-profiler/README.md)
owns the pinned IC gitlink, patch series, native adapters, patched PocketIC client
and source/build receipts. Consumers select a full Platform commit and the
`instrumented-pocket-ic` profile. This is an explicit source-provider interface,
not a new default feature or a replacement for the published registry client.

Profiler retains its IC-free recorder, reports, retention, workloads and viewer.
The paired #90 cut also requires consumer-owned CLI/query-rendering files and
private PNG/range helpers listed in the [source contract](../native/ic-profiler/README.md#paired-product-source-contract).
Do not select this source revision with an older consumer that omits them.
The existing native integration module paths remain unchanged; no compatibility
copies remain in the provider.
The 0.9 source profile also removes the viewer API's recursive integer rewriting
and its blanket `format_version` / `integer_encoding` labels. Viewer producers
emit exact decimal counter strings; nested PocketIC DTOs use their canonical
Serde encoding, including numeric bounded identifiers and byte arrays. Consumer
code must use those fields' contracts rather than infer one document-wide integer
representation. Process generation/hash and conditional-clear guards remain.
Raw PocketIC trace exports and saved Profiler reports retain their own formats;
rebuild the paired viewer/provider together when adopting the new Git pin.
Its source injection is separate from provider-owned adapters; overlaps are
rejected and both origins enter the assembly digest. Native Cargo/Bazel build
inputs belong here. The consumer recorder's optional native `gzip` dependency
selects only the isolated Tooling codec facade; no IC/native feature or dependency
back to the provider is selected. Existing upstream type identities, external
Candid bytes and cryptographic bodies remain unchanged. The native workspace
and Bazel locks additionally supply published IC Metrics 0.17 with defaults
disabled for consumer-owned counters. This is distinct from internal replica
metrics 0.9; counter arithmetic transfers no runtime charging or report policy.

The native server selects the `ic-admin` library without CLI-only NNS setup;
`lifeline` and Motoko are excluded. The upstream administrative CLI keeps its
default feature. No dummy canister artifact or build-error suppression is admitted.

Platform 0.7.0's `pocket-ic` Cargo feature still supplies the registry client;
it does not expose the instrumented profile API. That registry contract remains
unchanged. The provider is distributed through the pinned Git source profile and
is excluded from default and Wasm package graphs.

Acceptance remains tracked in
[#65 — Provide the instrumented PocketIC boundary needed by IC Profiler](https://github.com/dragginzgame/ic-platform/issues/65):
qualify actual source identities, reproducible assembly, the native server and
all five `system_profiling` regressions against its recorded executable hash.
Source/fixture checks and historical receipts do not establish native or Bazel
qualification. Publishing a new Rust facade for instrumented clients is separate
from consuming this source profile.

## Optional BLAKE3 maintenance boundary

This surface is supplied by published Platform 0.6.18. Earlier releases do not
provide it; consumers must raise their minimum requirement when adopting it.

Select `ic-platform/blake3` and import `ic_platform::blake3::{hash, Hasher, Hash}`.
These are upstream identities, not wrapper types or a generic hashing API. The
workspace requirement is BLAKE3 1.8.7 with defaults disabled and `std` selected.
There are no default Platform features. Native and Wasm consumers can select
hashing independently of Principal, Candid, CDK, executor and host features.

Consumers may remove an exhausted direct/renamed BLAKE3 dependency after adopting
the supplying Platform release. Add their own declaration guard so first-party
usage has one owner, while retaining provider transitives. IcyDB's current
`blake3 = "1.8.5"` range can unify with this supplier; publication and actual
consumer graph checks still precede adoption. No sibling has been edited here.

`make check-primitives` admits default, executor, runtime and BLAKE3 combinations
on native and `wasm32-unknown-unknown`. It uses the shared Cargo graph checker
for at most one full rendered BLAKE3 identity (version plus source qualifier),
then reads Cargo's actual feature union and requires exactly `std` when present.
Repeated `(*)` nodes are not duplicates. The source qualifier is Cargo's graph
rendering, not artifact/source authentication. A real adversarial consumer shows
that an aliased direct dependency can enable `pure` on the same identity; feature
admission rejects it even though a duplicate-version check would pass.

For another explicit selection, use
`bash scripts/ci/check-blake3-graph.sh CONSUMER PACKAGE TARGET FEATURES_OR_DASH`.
It disables root defaults, uses normal/build edges and locked/offline inputs,
retains evidence under Platform's target directory and never compiles or fetches.
Consumers needing root defaults or a broader reviewed feature policy must use
their own actual build selection/policy rather than silently reusing this one.
The standard graph checker supplies count/feature/path mechanics; product roots,
reviewed exceptions and release features remain at the product owner.

Canic must apply governance to each actual deployable package/manifest, including
generated and independent workspaces, using its build's exact Cargo selection.
Its existing canonical-role crypto gate is not an inventory of arbitrary Apps.
Check version/source identities and effective features before expensive builds;
retain reviewed exceptions explicitly. Its active Wire batch defers builds/tests,
so no current Canic build, measurement or gate integration is claimed here.

Keep required IC hashes, Caffeine object hashes and existing signed/persisted
formats unchanged. This feature is not a migration of those contracts. Any new
owned format/domain policy belongs to its consumer. Compare optimized artifacts
under the same build recipe before claiming Wasm savings; host/test duplicates
and graph identity counts alone do not establish linked code size.

### Bounded Wasm size evidence

The retained Rust 1.91 freestanding streaming-hash probe uses one exported ABI,
`opt-level=z`, fat LTO, one codegen unit, aborting panics and symbol stripping,
followed by `ic-wasm 0.11.1 shrink`. Optimized sizes are: control 109 bytes,
BLAKE3 through Platform 23,340 bytes, direct BLAKE3 23,340 bytes and SHA-256
10,215 bytes. Direct/facade BLAKE3 outputs are byte-identical in this probe.
This demonstrates no facade overhead for the retained code path, not smaller
production canisters. BLAKE3 is 13,125 bytes larger than SHA-256 here. It does
not establish relative execution time or IC instruction/cycle cost.
Sources, selected lock, metadata and artifact hashes are retained at
`target/blake3-boundary.QmcewQ/wasm-probe`; `sizes.json` records the comparison.

## Browser SDK Core boundary

The published browser package is `@tokostudios/ic-platform` 0.1.1 under
`packages/browser`, backed by SDK Core exactly 6.1.1 as an ordinary dependency
for Blob's approved local 0.48 graph. This supersedes the unpublished SDK-5 and
`@dragginzgame/ic-platform` proposals. Initial 0.1.0 publication and archive
verification completed on 2026-10-26; evidence is recorded in
[Platform #57](https://github.com/dragginzgame/ic-platform/issues/57). This npm
release identity is independent of Rust package versions and `make publish`.
Published-package and installed consumer acceptance remain separate requirements.

The published 0.1.1 additive browser update for
[Platform #59](https://github.com/dragginzgame/ic-platform/issues/59) supplies
`Endpoint`, `SubmitRequestType`, `ReadRequestType` and `Expiry` through `/agent`.
Auth's signing-request fixtures may replace their `@icp-sdk/core/agent` imports
with `@tokostudios/ic-platform/agent` using a minimum requirement of 0.1.1,
after admission in Auth's own graph. Published 0.1.0
does not contain them. Keep request authorization, expiry selection and guarded
identity policy at Auth; Platform retains upstream identities and implementation.
Separate authorized npm publication completed on 2026-10-27. The public archive
matches the qualified tarball and a fresh registry-only strict TypeScript/runtime
consumer passes, with SDK Core still exactly 6.1.1. This does not qualify Auth's
guarded-identity application policy or installed consumer adoption.

The initial browser implementation is committed at `8d80630` (Rust 0.6.17). Browser CI at that
commit passes on Linux x86_64 and native macOS Intel/Apple Silicon, including
packed-provider and Chromium page/module-worker checks under the earlier
unpublished npm name. The maintainer selected the actual `tokostudios` npm scope;
final-name packed/browser checks pass locally on Linux. A fresh registry-only
consumer verifies archive integrity, Core identity and strict compilation/runtime
without declaring SDK Core directly. See the
[maintainer supply procedure](../packages/browser/README.md#initial-npm-supply).

| Current SDK import | Platform import |
| --- | --- |
| `@icp-sdk/core/principal` | `@tokostudios/ic-platform/principal` |
| `@icp-sdk/core/identity` | `@tokostudios/ic-platform/identity` |
| `@icp-sdk/core/identity/secp256k1` | `@tokostudios/ic-platform/identity/secp256k1` |
| `@icp-sdk/core/agent` | `/agent` for HttpAgent/request/reject APIs; `/certification` for Certificate/lookup |
| `@icp-sdk/core/candid` | `@tokostudios/ic-platform/candid` for required external protocols |

This is a narrow supported surface, not a blanket SDK namespace. Re-audit each
symbol against the package export map. Keep existing SDK class/type identities;
do not replace Principal, identity, Agent or certificate values with new structs.
Neither the npm package nor Rust adds a dependency from Wire back to Platform.
Wire still generates and encodes owned endpoint payloads. SDK ingress envelopes
and required Caffeine/system/ledger Candid remain external protocols.

Consumers adopting the published package replace their exhausted SDK Core
declaration with a regular Platform dependency and update locks/build inputs.
Blob retains its explicit SDK peer until its qualified migration is complete.
Keep its Caffeine dependency; Canic retains Auth for its session integration.
Qualify one compatible SDK Core 6.1.1 runtime across those provider edges rather
than bundling an SDK copy or distributing aliases/local patches.
After adoption, consumer CI must reject first-party direct SDK Core imports and
declarations while allowing the provider's transitive SDK edge. Platform's Cargo
boundary checker does not enforce JavaScript imports in sibling repositories.

Blob may replace its canonical representation checks with
`parseCanonicalPrincipal`; role/anonymous/range policy remains there. Its existing
identity JSON consistency check maps to `restoreEd25519Identity` or
`restoreSecp256k1Identity` with Blob's existing 512-byte import budget. Map typed
import refusals to the existing public bootstrap refusal without exposing keys.
No principal hashing, key derivation, encoding or signature implementation moves.

Use `createAgentOptions` before the existing SDK constructor/factory, including
Blob's HttpAgent subclass. Supply an admitted origin URL, exact selected identity
and root bytes, and explicit retry/query-verification/time-sync choices. The
helper snapshots trust bytes and disables automatic root fetching. Preserve each
caller's selected policy: Canic's existing defaults are not a new shared default,
and Blob retains zero SDK retries and its guarded fetch/intents transaction.
Keep response limits, deadlines, endpoint authorization and historical certificate
policy at their current owners. Required options must not select new network IO.

`queryRaw` preserves opaque argument/reply bytes and upstream rejection context.
Root and query snapshots own their selected bytes even for Node Buffer views;
non-byte query arguments are rejected before SDK invocation rather than coerced.
It adds no retries; underlying SDK retries/verification requests still apply.
Keep update submission, request-ID/envelope retention, polling, cancellation and
uncertain-effect authority explicit in consumers. Certificate lookup alone is
not verification. SDK verification and caller freshness/trust decisions remain
unchanged. No crypto or alternative transport implementation is introduced.

Qualification must include strict TypeScript through the packed public exports,
a consumer with no direct SDK Core declaration, exact provider Agent/Identity
compatibility, Principal-only bundle isolation, external Candid/signature/request
bytes, and real browser/module-worker checks. Disposable Blob/Canic source copies
provide import/helper/bundling evidence only; they do not qualify installed IC,
production authentication, durable intents or provider effects. Run those at the
owning consumers after adoption. New SDK versions require renewed qualification;
this batch follows Blob's approved 6.1.1 selection.

### Blob 0.48 and SDK 6

All current Blob browser SDK imports are covered by the explicit subpaths above;
no additional runtime exports are needed. Preserve `IntentAgent` and its guarded
fetch. Wrap its existing constructor inputs with `createAgentOptions`, supplying
`host: new URL(origin)`, `retryTimes: 0`, `shouldSyncTime: false` and the existing
`verifyQuerySignatures: true` policy. Retain exact envelope/request-ID handling,
certificate freshness/root policy, durable claims and cancellation fences.
Map checked restoration refusals to `BootstrapRefusal('signer')` with the existing
512-byte budget. Keep anonymous/role admission after canonical text parsing.

Caffeine 1.1.2 declares SDK `^5.3.0`; retain its reviewed integration patch and
Blob's qualified SDK-6 application override. When the direct SDK declaration is
removed, replace its `$@icp-sdk/core` override reference with the exact selection:

```json
"overrides": {
  "@caffeineai/object-storage": { "@icp-sdk/core": "6.1.1" }
}
```

Overrides in Platform's development manifest do not propagate to applications.
Do not use `--force`/legacy peer resolution, install duplicate SDKs or call this
provider-declared SDK-6 support. Check both locked packages and actual provider/
Platform Agent resolution. Preserve the direct-import ban requirement after
published adoption; the override is a transitive provider selection, not a new
first-party SDK dependency.

Canic's current Auth 8.0.3 declares SDK `^5`. The producer type fixture uses Auth
10.0.2 (`^6`) and Signer 6.1.0; it does not qualify Canic login/session behavior.
Keep Canic's current declarations until its coordinated Auth/Core migration and
consumer acceptance. Never force Auth 8 onto SDK 6 or infer fleet adoption from
Blob's tests. Native macOS and installed/live-provider acceptance remain separate.

## Owned Principal and Account Wire metadata

Platform requires Rust 1.91 for all features. Principal metadata originally shipped
in 0.6.9 and Account metadata in 0.6.12; Platform 0.6.15 adopted the Wire 0.7 family.
Platform 0.6.19 raised both Wire minimums to 0.7.16; Platform 0.10 adopted Wire 0.8.
The pending 0.11 release raises host metadata and tooling minima to Wire 0.9.0.
Qualify using a local Platform path until that supplier is published, then select
its released version with defaults disabled and `features = ["wire-schema"]`.
Host producers must also select `ic-wire = { version = "0.9.0", features = ["typescript"] }`
for their derives and generation. Wire 0.8 and 0.9 metadata traits are different
Rust identities; update both selections together. Wire 0.9 preserves codec bytes,
limits and accepted inputs; its optional endpoint integration uses Platform 0.10.
Package metadata remains 0.10.0 until the maintainer prepares the minor release.
No direct Candid dependency is needed:

```rust,ignore
#[derive(serde::Serialize, serde::Deserialize, ic_wire::Wire)]
struct Request {
    caller: ic_platform::Principal,
}
let schema = ic_wire::typescript::native_schema::<Request>()?;
```

`wire-schema` implies `principal`. Only native builds implement Wire's metadata
trait and select Wire/JSON dependencies. Canister builds keep the original
Principal Serde implementation even if a unified feature selection includes
`wire-schema`; no metadata trait implementation or metadata dependency enters
that target. Use ordinary `principal` in deployed consumer manifests.

The projection delegates to Wire's bounded byte-buffer schema with Principal's
existing 29-byte maximum. Generated TypeScript uses `Uint8Array`, not textual
principals or a Candid marker. It preserves empty and anonymous identities and
rejects longer values during encoding and decoding. External Candid DTOs still
use `ExternalPrincipal` with explicit conversions. Account metadata now describes
its existing owner and optional fixed 32-byte subaccount fields; no runtime codec,
cryptographic primitive, persisted format or retry policy changes.

The workspace alias `ic-wire-metadata` names the modern host metadata dependency.
The independent installed-runtime fixture also declares Wire 0.8.0 and Testkit
0.71.0. Its lock refresh remains blocked by the Timers executor conflict described
above. Installed qualification remains independent of host metadata generation.

[#55](https://github.com/dragginzgame/ic-platform/issues/55) records the original
Wire 0.6-to-0.7 supplying cut; older Platform 0.6.14 cannot satisfy a Wire 0.7
trait bound. After the supplying release is published and registry caches are prepared,
run `bash scripts/ci/check-principal-wire-schema.sh /path/to/tsc.js --published X.Y.Z`
with that exact released version.
This mode refuses a path/Git replacement and checks the exact registry version.
Local-path qualification is not evidence that the new artifact is published.

Blob Contracts must subsequently publish its paired Platform/Wire selection
before Canic consumes its Wire `Limits` and `GenerationError` identities. Do not
add cross-version adapters, consumer-local wrappers or a distribution Cargo patch.

Wire 0.7 decodes in one pass with only `Deserialize`; it no longer reserializes
the result to compare canonical bytes. Encoder bytes and frame identities are
unchanged, but maps, sets and custom visitors can normalize accepted input.
Consumers retain ordering/uniqueness admission and must keep original frame bytes
for exact retries, signatures and hashes. Do not reconstruct those bytes from a
decoded value. Required external Candid and existing persisted formats are outside
this adoption. The owned Principal/Account serializers and subaccount policy do
not change.

Remove exhaustive matches on Wire's retired `Noncanonical` error: nonminimal
varints and malformed character payloads now report `Malformed`. Configure encode
and decode allowances independently; decode no longer charges a second serializer
pass. Platform adds no automatic quotas, normalization or compatibility codec.
The existing runtime fixture budgets remain consumer-owned and are not increased.

### Account adoption

Account metadata was first supplied by published Platform 0.6.12. No new feature is required: host schema
producers select `wire-schema`, while canister consumers select `principal`.
Use `ic_platform::Account` for ICRC owner/subaccount identities in shared DTOs;
its enclosing DTO can derive `ic_wire::Wire` just like the Principal example.
Generated TypeScript represents the owner as `Uint8Array` and the subaccount as
`null | { readonly some: readonly number[] }`, with exactly 32 elements when present.
It is not a legacy ICP AccountIdentifier or a ledger/asset identifier.

`None` and `Some([0; 32])` compare, order and hash equally, but their serialized
representations remain distinct. The metadata and codec do not normalize them.
Consumers needing canonical bytes for signatures, request deduplication or content
hashes must define that policy explicitly before encoding; do not silently rewrite
existing stored data. Ledger identity, authorization, allocation and migration
policies remain consumer-owned. Convert to `account::ExternalAccount` only at an
external ledger DTO boundary, selecting `icrc-ledger-types` there. Conversions
preserve the absent/explicit-zero distinction. No Candid dependency is required
for the owned Account or its Wire metadata.

## Authenticated canister-signature time (pending 0.6.14)

The existing `signature_verification::verify_canister_sig` is unchanged, including
`Result<(), String>` and acceptance of valid signatures without a time leaf.
The additive `verify_canister_sig_with_time` uses the same four byte-slice inputs
and returns `Result<u64, CanisterSigTimeError>` under the existing
`signature-verification` feature. The u64 is the authenticated signing certificate's
Unix-epoch time in nanoseconds, never the subnet delegation certificate's time.

The new adapter calls the unchanged imported parse/check/BLS helpers in the same
order; it decodes the signature envelope and signing certificate once each. It
returns evidence only after certified-data, seed/message path, delegation and
signature checks succeed. The only imported-source change is one child-module
declaration; the source guard removes exactly it and recovers the original hash.
No cryptographic body, root/hash/domain bytes, key encoding or original vector changes.

`CanisterSigTimeError::Verification(String)` retains upstream diagnostics as opaque
text; `InvalidCertificateTime` distinguishes timestamp admission. Time must be a
unique visible top-level leaf and a complete u64 unsigned LEB128 value, at most ten
bytes. Missing/unknown/non-leaf time, duplicate visible labels, truncated/overflowing
values and trailing bytes are refused. Non-minimal complete values within ten bytes
remain accepted, matching Auth's current reader. Upstream sorted-tree assumptions
and all existing parser admission limitations remain.

Auth adoption replaces its verifier-plus-second-CBOR-decode sequence with the
additive call. Map `Verification(message)` to Auth's existing `InvalidSignature`
and `InvalidCertificateTime` to its existing time rejection. Keep the existing
inclusive freshness/future-skew arithmetic unchanged. Keep protected issuer/seed,
root and host time, byte/depth admission and canonical DER pre-validation in Auth.
No clock, application authorization, automatic quota or freshness policy is supplied.
Other Auth CBOR/LEB128 uses must be audited before retiring any dependencies.

Local implementation is not publication. Consumers await a supplying release;
no sibling checkout has been changed. The existing standalone consumer qualifies
both APIs without direct crypto/Candid dependencies. An isolated copy of Auth
0.15.2 adopts only the new API/error mapping and retains its actual policy tests.
Native macOS and installed-canister qualification remain separate from native,
MSRV and Wasm compilation. No timing or binary-size improvement is claimed.

## Authorized owned-endpoint Wire migration

Owned application endpoints and their Rust, host, test and browser clients use
Wire, including controlled canister-to-canister calls. Candid remains for required
external IC protocols: management, NNS/SNS, ledger and other pre-existing external
interfaces. Running on the IC does not itself select Candid. Existing persisted
and checkpoint bytes remain outside this transport migration.

The maintainer authorized the coordinated owned-endpoint and frontend cut under
[Canic #613](https://github.com/dragginzgame/canic/issues/613). This specifically
supersedes deferral of that migration; it does not authorize unrelated API cuts.
Canic owns endpoint-derived schemas, explicit method IDs/budgets, authorization
and the separate `wire_schema_sha256` artifact identity. Wire owns codecs and
TypeScript generation. [Tooling #225](https://github.com/dragginzgame/ic-tooling/issues/225)
owns Build composition after the concrete producer contract is qualified.

Platform [#38](https://github.com/dragginzgame/ic-platform/issues/38) retires
exhausted compiler/service exports after consumer adoption, not before it.
Remove corresponding generated imports, manifest selections and obsolete tests
together. Preserve external DTO bytes and malformed-input coverage, replacing
owned protocol fixtures with Wire equivalents. Qualify generated Rust/TS bytes,
installed query/update calls, authorization and frame refusals, and schema/module/
release mismatch admission before claiming completion. Report native macOS and
installed-canister evidence separately.

External DID inspection and textual external-call tooling may still require
`candid-parser`; required external service declarations may still require
`candid-service`. Neither feature is retired by this authorization alone. Removing
compiler exports does not remove the upstream parser's compiler dependencies.

## Stable-structures substrate

Pending 0.6.5 adds `memory_manager::{MemoryId, MemoryManager, VirtualMemory}`
under the same optional feature, retaining exact upstream 0.7.2 identities for
IC Persistence's Memory infrastructure. After the supplying release is qualified
and published, require Platform 0.6.5 or later and replace direct manager imports
with `ic_platform::stable_structures::memory_manager::{MemoryId, MemoryManager,
VirtualMemory}`. Existing `Vec`/`Log` exports cover local StableVec/StableLog
aliases; no additional alias exports are needed. Memory retains checked layout
reopening, growth preflight, allocation authority and exclusive leases. The
standalone vector-backed manager test qualifies upstream bytes and identity,
not production governance or installed upgrades. No manager implementation or
allocation policy moves into Platform.

Platform 0.6.2 adds optional `stable-structures` exports from upstream `=0.7.2`.
This completes [PR #32](https://github.com/dragginzgame/ic-platform/pull/32),
including Storage's review feedback: `storable::{Blob, BoundedVecN,
UnboundedVecN, TryFromSliceError}` accompany `Bound` and `Storable`.
Collection, Memory, iterator, log-error and backing types retain upstream
identities. No replacement types, formats, memory manager or storage
implementation is introduced. Defaults/executor-only graphs exclude the
dependency; substrate-only selection adds no Candid, parser, CDK, executor or
owned Principal. Upstream `ic_principal` remains a substrate dependency; this is
dependency control, not transitive removal or measured Wasm reduction.

The exact version preserves the trait identity requested by Memory/Storage and
Testkit. Changing it requires renewed interoperability and retained-byte checks.
Require Platform 0.6.2 or later with this feature and replace
the direct declaration/private alias with
`use ic_platform::stable_structures as substrate`. Keep governed constructors,
record/region/recovery policy in Storage and allocation/runtime handles in
Memory. Applications continue using governed Storage rather than raw backing.

Testkit's instrumentation selects `VectorMemory` explicitly for target-neutral
shared backing. `DefaultMemoryImpl` is target-dependent and is not a substitute.
Testkit retains counters, reset, overflow and fixture policy. No sibling changes
are made here. Actual adoption, native macOS and installed-upgrade qualification
remain separate from Platform export/identity and standalone reopening tests.

## Explicit external Candid entrypoints (0.6.2 qualification)

The 0.2 boundary removed implicit Candid entrypoint codecs. Owned application
endpoints use Wire; a feature-only update does not migrate pre-0.2 generators or
preserve deployed formats. Canic owns that coordinated protocol migration.
The coordinated owned-endpoint Wire migration is authorized above; unrelated
hard cuts remain deferred until Canic is live.

Required pre-existing external Candid interfaces remain supported. Select `cdk`
and `candid-service` (which implies `candid`). Supply `decode_with` and
`encode_with`, and explicitly register the external declaration with
`contract::candid_method` before calling
`contract::export_service!(ic_platform::candid::contract)`. Match original method
names, query/update modes and argument/result tuples in both declarations.

Zero-argument methods must still decode Candid: empty raw bytes are not an
encoded empty argument list. Use `decode_args::<()>` with the caller's five
finite `DecodeLimits` and retain its skipping policy. A zero skipping quota
refuses charged extra values. Encode one result with `encode_args((value,))`
and no results with `encode_args(())`; the unit-return encoder takes no Rust
arguments. Adapters retain encoding/decoding failure mapping and trap disclosure.

Guards run in declaration order before decoding and handler execution; a guard
error rejects without running codecs. Query/update context selection and callback
cleanup remain runtime-owned; do not install another executor. The standalone
`external-candid` profile declares only Platform and checks fixed request, reply
and service bytes, malformed/oversized refusal, native/MSRV compilation and linked
Wasm with canonical executor/IC0 identities. This is composition evidence, not
installed guard/trap execution or Canic/Blob adoption. Consumers retain their
actual endpoint/service goldens and installed qualification. No Wasm
`get_candid_pointer` helper is added; native service export remains separate from
compiled Wasm endpoint inspection.

## 0.6.2 strict reply helpers

Under the existing `candid` feature, use
`ic_platform::candid::sequence::deserialize::<D, T, LIMIT>` inside a caller-owned
Serde deserializer. It rejects excessive size hints before reading, never reserves
from hints, and reads at most one typed excess element. The caller supplies
expected-shape and overflow diagnostics; element errors propagate unchanged.
Application aggregate bounds and all five `DecodeLimits` remain caller-owned.

`ic_platform::candid::reserved::StrictReserved` preserves the Candid reserved
wire type and encoding. It accepts only the owned decoder marker `[3]`. Use
`DecodeLimits.skipping_quota = 0` for strict payload refusal; this type alone
does not impose a quota or replace variant-tag and schema-evolution policies.

Implementation and regression tests are preserved from Platform
[PR #26](https://github.com/dragginzgame/ic-platform/pull/26) at `c90004c`,
migrated from Testkit [PR #123](https://github.com/dragginzgame/ic-testkit/pull/123)
at `97df333a0aa57a056eb3bfe443a7365c4314b361`, originally extracted from Backup
0.30.0 (`3992e15`). Do not restore `ic-testkit-candid` or add a compatibility
package. Require at least Platform 0.6.2 with
`features = ["candid"]`, select the import paths above, and retain endpoint DTOs,
ceilings and tests in the consumer. No generator or existing import changes are
required. Publication and Backup adoption remain separately qualified; this
addition changes no existing API, dependency selection or external wire format.

## Pending 0.6 feature boundary

This supersedes older unconditional Principal/Account descriptions. No defaults
are introduced. Select `features = ["principal"]` for owned Principal, Account,
Subaccount, their errors and explicit external identity conversions. Existing
imports and bytes stay unchanged. Type-only consumers (including Wire schema
owners and IcyDB) must update inherited manifests and independent test/generator
manifests; do not rely on another package unifying this feature accidentally.
After publication, update the inherited Platform requirement to the 0.6 line
and do the same for independent fixtures/generators. A `0.5` requirement will
not select this breaking release. Timers keeps its two features but also needs
that version-requirement update and qualification against the published package.

`runtime` (therefore `cdk`), `candid` (therefore `candid-service`),
`canister-sig-creation` (therefore `signature-verification`), management and ledger
DTO features, `agent`, `pocket-ic`, `candid-parser` and `secp256k1` include
`principal`. Native-only client/parser dependencies remain absent on Wasm;
the explicitly implied portable principal types remain available there.
Certification, BLS, ECDSA and Ed25519 alone do not select application types;
add `principal` if a consumer uses both capabilities.

Raw Timers-style consumers keep `features = ["executor", "ic0"]`. Their public
executor/system imports and qualified registry identities remain unchanged.
Those production graphs exclude Principal, Serde, SHA-2 and text conversion.
Raw system helpers accept byte slices; construct application identities only in
profiles that select them. The standalone raw consumer checks graph isolation
and inability to import Principal, independently of workspace feature unification.

Signature-creation production no longer selects the unused delegation hash
helper or creation reference CBOR parser. Original bodies and tests are retained
under test-only attributes; verification keeps its independent original parser.
`ic-representation-independent-hash` becomes test-only for Platform. Other host
features can still bring it transitively. Certification still brings serde_bytes;
removing Platform's direct creation edge does not eliminate that package.
No cryptographic implementation, external byte format, storage, retry or caller
admission policy changes. A smaller dependency graph does not establish smaller
Wasm or faster execution. Package versions remain release-owned.

## Pending 0.5 retirement

The maintainer authorized a hard cut of unused compatibility surface after 0.4.1.
This section supersedes older checkpoint-retention descriptions below.

- Remove `hidden` from entrypoint attributes, including generated attributes and
  Testkit performance probes and IcyDB's handwritten endpoint test fixtures.
  It had no effect and now produces a targeted error.
  Other supported options retain their names; `manual_reply` and `composite`
  accept bare flags or boolean literals, not string-encoded booleans.
  Repeated `guard` options retain order; duplicate single-value options fail.
  `composite = true` is query-only; `manual_reply = true` is invalid on lifecycle
  hooks. These combinations previously had no effect and now fail explicitly.
  Handler and codec names no longer collide with generated local bindings;
  argument names and destructuring patterns need no migration.
  Native export symbols now use an injective UTF-8 hexadecimal encoding instead
  of punctuation replacement. Host tooling inspecting those symbols must update
  its expectations; ordinary Rust consumers and Wasm endpoint names are unchanged.
  Raw Rust handler identifiers such as `r#match` now compile. Their default
  endpoint name is `match`; an explicit `name = "..."` is always used verbatim.
  This repairs an expansion panic, not a change to a previously working raw-name
  endpoint. Conditional declarations remain compiler-controlled.
- Replace `.change_timeout(seconds)` with
  `.with_wait_policy(ic_platform::cdk::call::WaitPolicy::Bounded {
  timeout_seconds: seconds })`. Blob currently uses the old method in its client
  transport, replicated caffeine transport and service recovery. Keep each
  caller's existing timeout value. Select `WaitPolicy::Unbounded` explicitly
  when changing to unbounded waiting; neither option changes retry policy.
  Constructors retain their current defaults, zero remains bounded, and values
  are forwarded unchanged to the IC's own timeout admission.
- Raw-call/system libraries can replace `features = ["cdk"]` with
  `features = ["runtime"]`, retaining their `ic_platform::cdk::*` imports.
  Entrypoint owners keep `cdk`, which includes `runtime` and the generator.
  Canic Core's runtime usage is a candidate; Canic's entrypoint/start generator
  still requires `cdk`. If a workspace enables `cdk` centrally, move that
  selection to the packages that generate entrypoints before claiming a smaller
  library graph. Cargo feature unification may legitimately retain the generator
  in the final canister. Executor identity, callback behavior and codecs do not
  change. This separation removes neither the executor nor all proc macros.
- Service declaration generation now requires `features = ["candid-service"]`.
  That feature implies `candid` and exposes the existing `contract::candid_method`,
  `contract::export_service` and `contract::pretty` paths. No import or generated
  path changes are required. Plain `candid` still supports external DTO derives,
  arbitrary-precision values, reflection and configured decoding.
  Canic's adapter/start generators and Blob's standalone/service-export tests
  currently use these macros: add the feature at the workspace dependency and
  forward it from any optional consumer feature that generates declarations.
  Keep their existing generated declarations and contract tests during migration.
  Native parser users continue selecting `candid-parser`; parser selection does
  not implicitly expose Platform's service macros.
  To retire service generation entirely, consumers must first remove its actual
  generator and test uses. Removing Candid from management/ledger calls is not
  part of that migration.
- Remove `ic_platform::cdk::storage::{stable_save, stable_restore}`. There is no
  replacement Platform checkpoint module or compatibility package.
- Remove the unexported StableMemory/CanisterStableMemory abstraction,
  StableIO/StableWriter, whole-memory copying and its unsafe-code exception.
  Keep the existing raw stable grow/read/write signatures and error type.
  Remove `StableMemoryError::OutOfBounds`, which none of these functions returns:
  raw read/write bounds violations trap, while grow returns only `OutOfMemory`.
- Caller, canister and subnet identity APIs now copy into fixed 29-byte stack
  buffers. They still return the same owned Principal bytes; invalid system
  lengths trap. No executor, callback, timeout, retry or cryptographic code changes.

A source audit of current sibling Rust checkouts, including application and test
canister directories, found no calls to the checkpoint helpers outside Platform's
own fixture. Toko was excluded per the maintainer's existing scope. This is not a
claim about unknown downstream users or generated code unavailable in those trees.
External users of the removed API must remain on 0.4 while moving their existing
checkpoint codec into their application/storage owner, preserving its layout,
Candid argument encoding, error policy and restore behavior. In particular the
old restore decoder did not finalize the input; replacing it with a decoder that
rejects trailing stable-page bytes is not an equivalent migration. No automatic
data conversion or new persisted format is authorized.

The installed fixture now explicitly owns its fixed checkpoint record: it writes
the same Candid tuple bytes and decodes without finalization after upgrade. This
is test code, not a general checkpoint implementation offered to consumers.
Management/ledger DTOs, external Candid adapters and cryptographic dependencies
remain upstream. This cut reduces maintained source and temporary identity
allocations. Plain `candid` no longer selects the upstream printer; combined
graphs may still select it through Agent, PocketIC, parser or external DTO
packages. Feature separation alone establishes neither fleet-wide dependency
removal nor measured Wasm-size or execution-speed improvement.

IC Platform owns the controlled Candid/CDK dependency boundary. Consumers declare
`ic-platform`, not `candid`, `candid_parser` or `ic-cdk*` directly. IC Wire owns
protocols whose producers and consumers we control. External management-canister,
NNS/SNS and ledger interfaces retain their required Candid formats. Storage and
ic-memory changes are out of scope; no persisted representation is changed here.

## PocketIC maintenance boundary

The optional native-only `pocket-ic` feature exposes `ic_platform::pocket_ic`.
Platform owns the reviewed PocketIC 16.1.0 dependency and explicit client, error,
settings/snapshot and fixture REST identities. No replacement structs, runtime,
server launcher or automatic client/server acceptance policy are introduced.
Testkit retains provisioning, checksums, supported server versions, lifecycle,
transport, retry behavior and product qualification.

Published Testkit 0.55.2 uses Platform with `features = ["candid", "pocket-ic"]`
on native targets and forwards `ic_platform::pocket_ic` at its public module path.
After these fixture exports are published, raise Testkit's minimum Platform
requirement to the supplying release; an older compatible lock selection can
otherwise retain the incomplete surface. Do not restore a second direct upstream
declaration or DTO owner. Audit downstream use of the formerly blanket export
against this explicit surface. Consumers continue using Testkit workflows. Direct-dependency policies should allow upstream PocketIC
only in Platform, while deployment graphs must exclude PocketIC and Testkit.

The feature does not enable Platform Candid, executor or parser features, but
PocketIC brings Candid, certification and native HTTP dependencies transitively.
Its management DTOs are upstream 0.8.0 and must not be substituted with Platform
management-canister-types 0.11.0. PocketIC 16.1.0 also requires thiserror 2.0.18;
the lock selects that compatible version for existing 2.x consumers. These are
dependency-control changes, not transitive removal or Wasm-size claims.

Standalone checks cover building without a server; Testkit adoption, actual
client/server execution and native macOS qualification remain separate gates.

Backup, Canic and Blob's existing Testkit workflows additionally use placement,
effective-principal routing, pending messages, HTTP gateway configuration,
mock HTTP responses and chunked installation. The root `CanisterInstallMode`
retains PocketIC's own management 0.8 identity and `skip_pre_upgrade` flags;
Platform's independent management 0.11 type cannot replace that method parameter.
Their explicit root and REST exports retain the original
PocketIC identities, including method parameter/result compatibility. These
exports add no gateway startup, pending-call polling, retries or outcall policy.

## Owned Principal migration (0.3)

This is a breaking next-minor change. Package versions remain at the maintainer's
current release until the coordinated release process; do not publish this source
as a 0.2 patch. The root `Principal` and `PrincipalError` now belong to Platform.
The implementation is adapted from ic_principal 0.1.5 under Apache-2.0, with
provenance/license retained in `src/principal/`. It is not a clean-room rewrite.

Application and runtime identities use the owned type. Agent, PocketIC, management,
ledger, parser and cryptographic APIs retain their original parameter/DTO types.
`principal::{ExternalPrincipal, ExternalPrincipalError}` names those original
identities without requiring Candid; `candid::contract::Principal` names the same
external principal when Candid is selected. Both principal conversions use only
the original significant bytes and are infallible:

```rust
let application = ic_platform::Principal::anonymous();
let external: ic_platform::principal::ExternalPrincipal = application.into();
let restored = ic_platform::Principal::from(external);
assert_eq!(restored, application);
```

Use external principals in required Candid DTOs and existing Candid checkpoints;
convert the result into an owned principal afterwards. The owned type implements
no Candid trait, even with all features selected. Do not decode Candid's internal
Serde byte-buffer marker as application data. Native owned Serde accepts borrowed
and owned bytes identically, including a leading 0x02. Existing byte/text values,
length-first ordering and padding are preserved; no stored layout is rewritten.
Only decoder paths that previously depended on Candid's special visitor behavior
need explicit external DTO decoding. Consumer persistence and migration policy
remain with their owners.

Text parsing now rejects more than 63 input bytes as `TextTooLong` before any
normalization or allocation. Other normal-size spelling/checksum diagnostics are
retained. Parsing/Display use stack buffers; `to_text` allocates its result only.
Uppercase remains accepted, while missing/misplaced dashes remain an error. This
bound follows the identity grammar, not a caller-specific resource policy.
Use `try_from_slice` for untrusted bytes; `from_slice` retains its documented
panic above 29 bytes. Constructors accept opaque 0..=29-byte identities; successful parsing is
not authentication or proof of canister existence. Compare both length and bytes
when using fixed backing arrays; padding alone is not identity.

Generic `rangemap::StepLite`, upstream arbitrary generation and schema traits are
not inherited by the new type. Consumers needing ranges must define their valid
key domain rather than treating tag-skipping as a successor for all principals.
Do not change existing ordering to repair that integration. There is no second
executor, timer owner, transport or retry implementation.

Self-authentication delegates to registry ic_principal with its original SHA-224
implementation. This dependency remains both for cryptography and external type
identity; its features are explicit. No cryptographic primitive, key encoding or
hashing algorithm is rewritten. This change does not establish upstream removal
or a measured Wasm-size reduction.

Adoption must update API/error annotations, DTO construction and equality tests at
external boundaries. Qualify actual Wire frames, Candid bytes, persistence and
client calls with the published next-minor selection. Native macOS, installed
runtime and individual consumer acceptance remain separate from source tests.

## Qualified Candid surface

The `candid` feature supplies `decode_one`, `decode_exactly_one`, `decode_args`,
`DecodeLimits` and `DecodeError`. The `candid::contract` module explicitly exports
`CandidType`, `Principal`, `Nat`, `Int`, `Reserved`, `Error`, `ArgumentEncoder`,
`ArgumentDecoder`, `encode_one`, `encode_args`, Serde, and the machinery needed
by Candid derives. These are upstream identities, not independent codec types.
Arbitrary-precision integers enable Candid's `bignum` feature with `candid`;
the separate native parser also enables it through upstream Candid's `all` feature.
No default features, automatic limits or second executor are introduced.

`ic_platform::Principal` is Platform-owned since the 0.3
migration. The Candid contract keeps the original registry identity; see
[the migration below](#owned-principal-migration-03). Default builds remain
Candid/CDK-free, not dependency-free.

The decoder implementation and six tests match Testkit's current owner byte for
byte after excluding the added contract module. Reply length is admitted before
constructing the decoder. All five caller limits are forwarded: reply bytes,
decoding quota, skipping quota, type-table entries and header bytes. Zero is a
real limit. These limits do not promise heap, stack or elapsed-time bounds.

- `decode_one` requests one value using upstream compatibility rules: extra values
  are skipped within quota; an absent optional/null result may be synthesized.
- `decode_args` requests a top-level argument tuple, with the same compatibility
  and bounded skipping rules. `(T, U)` is two arguments; `((T, U),)` is one record.
- `decode_exactly_one` requires one actual wire argument, including for optional
  and null types, and rejects additional arguments. All paths reject trailing
  undecodable bytes and preserve malformed-input and quota errors.
- `ReplyTooLarge` retains actual/maximum lengths; `Candid` retains the upstream
  error as its typed source. Compact diagnostics are selected, but may disclose
  small inputs. Raw bytes, authentication, disclosure and aggregate budgets stay
  with callers.

## Consumer audit evidence

### Existing decoder and service contracts

`ic_platform::candid::decoder` explicitly exports upstream `DecoderConfig`,
`IDLDeserialize`, `decode_one_with_config` and `decode_args_with_config`. Preserve
caller byte admission before construction, exact optional/default ceilings,
full/compact diagnostics and error mappings at `new_with_config`, `get_value`,
`done` and canonical re-encoding. Callers can observe/drop the decoder between
stages and inspect its remaining configuration without a wrapper. These operations
do not impose the finite `DecodeLimits` helper's policy. Replay/checkpoint consumers
may change imports only while retaining existing behavior; this authorizes no
storage/schema change. See [staged decoding](https://github.com/dragginzgame/ic-platform/issues/3).

The Candid contract now explicitly supports `candid_method`, `export_service`,
`Encode`, `Decode`, `TypeEnv`, `idl_hash`, `types::internal::TypeContainer` and
`types::subtype::equal`, plus printer machinery required by service export.
From 0.5, `candid-service` enables upstream `printer` alongside `candid`'s
`bignum`; plain `candid` no longer provides service generation. Change
generators to emit `ic_platform::candid::decoder` for configured decoding and
`ic_platform::candid::contract::candid_method` for service registration. Invoke
`contract::export_service!(ic_platform::candid::contract)` with the explicit path;
upstream's no-argument form still emits `::candid`. Import `Encode` and `Decode`
by those names from the contract to preserve their recursive expansion. Root
`ic_platform::PrincipalError` preserves Canic's public Principal parse-error type.
Standalone Candid-only native/MSRV/Wasm qualification covers those paths, fixed DID,
reflection/equality, configured decoding, typed stage mapping and macro wire bytes.
See [Canic's contract](https://github.com/dragginzgame/ic-platform/issues/5).

The decoder module additionally exports upstream default `decode_one` and
`decode_args`. Replace `candid::decode_one` / `candid::decode_args` with
`ic_platform::candid::decoder::{decode_one, decode_args}` when preserving that
existing policy. They take no `DecodeLimits` argument and retain upstream default
quotas and diagnostics. The parent `candid::{decode_one, decode_args}` helpers
still require caller-selected finite limits and select compact diagnostics.
Do not switch between those policies merely to change imports.

Canic's reply projection checks can now use
`ic_platform::candid::contract::types::subtype::subtype` in place of the upstream
path, retaining its `HashSet`, `TypeEnv`, sent/received direction and typed error.
Subtyping permits valid narrower record projections that structural equality
rejects. Preserve Canic's surrounding selector/field compatibility policy and
its positive/invalid projection corpus; update generators emitting these paths.

Service registration alone does not fix upstream CDK entrypoint expansion.
The optional Platform-owned generator below now provides that path correction.
Qualify generated Fleet actors and the published runtime selection before
removing their direct CDK/Candid declarations; see the
[runtime boundary](https://github.com/dragginzgame/ic-platform/issues/1).

Read-only working-tree review on 2026-10-18 used the shared code-hygiene method at
`4eabe7f771907ccebfebceafffe10657b911c0eb` and this repository's AGENTS.md overlay.
Platform started at `9ada261`. Sibling HEADs below identify bases, not clean-source
claims: Testkit, Canic, Timers and Host have pending changes which were preserved;
Query was clean. No sibling builds or edits were performed.

| Consumer base | Actual use and adoption boundary |
| --- | --- |
| Testkit `55ac397` | `crates/ic-testkit/src/pic/calls.rs` requires generic argument encoder/decoder bounds; transport preserves decoder sources. `ic-testkit-dev` also consumes the decoder. The performance probe still uses CDK entrypoint macros. |
| Query `e90cb2ef` | `src/candid_call.rs` uses generic argument encoding; `src/candid_decode.rs` owns quota policy. Ledger DTOs use `Nat`/`Int`; governance projections use `Reserved`. The CLI also declares Candid and the old decoder. `nns/governance/source/canister.rs` uses CDK calls. Paths are under `crates/ic-query` unless noted. |
| Canic `5583496bf` | `canic-macros/src/adapter/mod.rs` generates absolute Candid paths; `canic/src/macros/start.rs` exports a service. `canic-core/src/infra/ic/call.rs` wraps raw CDK calls. Host paths parse/type-check DID, generate bindings and compare services. Paths are under `crates`. |
| Timers `194204d` | `crates/ic-timers/src/platform/{ffi,backend}.rs` use executor method handles, callbacks and trap recovery. Production uses raw IC system calls; unpublished testing packages also use Candid/CDK. |
| Host `c8cd434` | `crates/ic-host-tools/src/candid/service/mod.rs` parses service facts; pending `candid/decode/mod.rs` adds aggregate byte accounting. That accounting differs from Testkit and must survive consolidation. |

## Certification extension

The optional `certification` feature extends this boundary explicitly beyond
Candid/CDK. It selects `ic-certification` 3.2.0 with upstream Serde support and
exports only the observed consumer surface: `Certificate`, `Delegation`,
`HashTree`, `Label`, `LookupResult`, `empty`, `fork`, `labeled`, `leaf`, `pruned`
and `fork_hash`. These are direct upstream exports, with no replacement structs
or conversion wrappers. Inherent lookup/digest operations remain upstream.
Certification selects neither Candid nor the executor and is absent by default.
The same feature now exports `AsHashTree`, `RbTree` and `Hash` for Auth's existing
nested mutable signature tree. Root/witness generation and leaf modification
retain upstream identities; no store, expiry/pruning policy or publication owner
is introduced. The standalone certification consumer exercises nested insertion,
modification, deletion, full trees and root-preserving witnesses. See
[Auth's tree surface](https://github.com/dragginzgame/ic-platform/issues/17).

The follow-up read-only review used pending working trees at Host `c8cd434`,
Query `8982c2a4` and Canic `5583496bf`; all three were dirty. Host's
`crates/ic-host-tools/src/certificate/mod.rs` constructs owned upstream values;
Query's `crates/ic-query/src/certificate/mod.rs` passes those certificates to
Agent verification and checks witness digests. Canic's `certified_custody` and
`operator_mint/receipts` modules consume labels, lookups and certificates, while
`canic-core/src/ops/auth/signature_store/mod.rs` uses `fork_hash` for signature trees.

For adoption, add/inherit Platform and replace `ic_certification::` paths with
`ic_platform::certification::`, including tests. Host's `certificate` feature
must select Platform's `certification` while retaining `ciborium`. Query's
`certificate-host` and Canic's signature capabilities must enable the same
Platform feature in place of `dep:ic-certification`. If Platform is already
non-optional for Candid or Principal, use `ic-platform/certification` rather than
introducing a second dependency alias. Remove the direct upstream declaration
from each member and the unused workspace catalog; update locks and dependency
ownership rules. Do not remove transitive dependencies of Agent or other upstream
consumers, or claim full workspace isolation before runtime/parser migration.

Host retains its strict bounded CBOR decoder, errors, complete-input byte limit,
nesting limit, duplicate/unknown-field refusal, byte-string admission, exact tree
tag/arity checks and trailing-data refusal. Do not replace it with generic upstream
Serde decoding. Consumers retain combined budgets, trust roots, signature and
delegation verification, time policy, witness admission, retry semantics and
application leaf interpretation. No certificate CBOR, hash commitments or persisted
data change. Constructed or decoded values remain untrusted until authenticated.

The standalone certification consumer has no direct upstream dependency and tests
lookup outcomes, pruned witnesses, fork hashing, a fixed leaf digest and certificate
fields on native/MSRV with Wasm compilation. A disposable copy of Host's actual
decoder and tests changes only import paths; its retained lockfile and source hashes
bind that focused proof. It is not a migrated or built Host/Query/Canic workspace.
Source hashes are recorded under `target/certification-host/source.sha256` and
fixture evidence under `target/certification-host/`. Consumer qualification must
still run actual authentication/receipt/signature tests against the selected Agent
and protocol packages to confirm one upstream certification identity. Native macOS
and installed canisters are separate qualification. Dependency control alone
does not remove `ic-certification` or establish a smaller Wasm artifact.

## Canister signature and BLS extension

Three optional features serve Auth's reviewed signature-store and
verification consumers. They are supported on native and Wasm, have no defaults,
and do not enable Platform's Candid, executor, parser, Agent or certification
surfaces. Their actual transitive dependencies differ:

| Feature / module | Explicit upstream contract | Transitive boundary |
| --- | --- | --- |
| `canister-sig-creation` / `canister_sig_creation` | Imported 1.3.1 `CanisterSigPublicKey`, `hash_bytes`, `hash_with_domain`, `extract_raw_root_pk_from_der`, `IC_ROOT_PK_DER`, `IC_ROOT_PK_DER_PREFIX` | Certification; no Candid, ic0, BLS or executor |
| `signature-verification` / `signature_verification` | Imported 0.3.0 `verify_canister_sig` | Creation surface, certification and BLS; no Candid, ic0 or executor |
| `bls` / `bls` | 0.6.0 `PublicKey`, `Signature`, `PrivateKey`, `InvalidPublicKey`, `InvalidSignature`, `InvalidPrivateKey`, `verify_bls_signature` | Upstream alloc enabled; no creation, Candid, certification, ic0 or executor |

The signature implementation is packaged in Platform for 0.4; BLS remains a direct
upstream export. See [the identity migration](#runtime-codes-and-signature-packaging-04).
Public-key constructor/DER/raw methods and
Principal identities, root constants, domain/hash bytes, serialized BLS keys and
signatures, typed BLS errors and unit verification errors remain upstream.
`verify_canister_sig` retains four byte-slice arguments and `Result<(), String>`;
Platform adds no error projection, cryptography or resource/trust policy.
BLS defaults are disabled and only upstream `alloc` is selected, matching Auth's
fixtures and the IC verifier. Fixtures derive deterministic keys through
`PrivateKey::deserialize` and sign through its inherent methods. No RNG feature
or entropy source is selected. Consumer feature unification can still enable
upstream features; qualify the actual final graph and key-custody policy.

Read-only review used dirty Auth at base `2da4dc0e`. Its
`crates/ic-auth/src/{signature_store,canister_signature}.rs`, three corresponding
test subjects and qualification runner/canister consume these APIs. Captured
sources, manifests/locks, original-owner vector generator and before hashes are
under `target/auth-boundary/` and `target/auth-boundary-before/`. The sibling's
root manifest changed concurrently; proofs qualify the captured copy, not the
subsequently edited checkout. No sibling was edited or built in place.

For adoption, inherit Platform with empty defaults in Auth's catalog and member.
Wire `canister-signature-preparation` to `ic-platform/canister-sig-creation` and
`ic-platform/certification`. Wire `canister-signature-verification` to those same
two features plus `ic-platform/signature-verification`. Select `bls` through the
existing test dependency, separately from production preparation/verification.
Replace `ic_canister_sig_creation::`, `ic_signature_verification::`,
`ic_verify_bls_signature::` and `ic_certification::` with the corresponding
Platform modules in implementation, tests and qualification packages; update
generators that emit those paths. Remove the four obsolete direct declarations
and unused catalog entries, update locks and narrow the direct-boundary policy.
Other Serde/CBOR/SHA and consumer feature selections remain consumer-owned.

Keep Auth's byte/depth limits, canonical DER re-encoding, protected issuer/seed
and trust roots, original signed payloads/domains, certificate authentication,
freshness/future-skew policy, signature retention, clock/transaction behavior,
expiry/pruning and root publication. The #72 correction adds DER envelope
admission before the unchanged key parser; retain Auth's resource and canonicality
checks. Domain/DER encoding lengths still retain upstream integer-conversion
behavior, and long-form DER is unsupported. Do not move application checks into Platform or
interpret verifier success as application authority. No signature map, global
store, clock or certification side effect is exposed here.

The Platform-only consumers preserve original-owner DER/hash/BLS/certificate
vectors and valid/invalid results on native/MSRV/Wasm. A captured Auth copy changes
only dependency selections and paths; all 49 signature-store, canister-signature
and certified-proof cases pass on Rust 1.91, with the combined library features
compiling for Wasm. This does not qualify its runtime canister, real upgrade or
signed ingress. Published adoption must run those consumer workflows; native
macOS producer CI and installed-canister acceptance remain separate.

Upstream creation 1.3.1 uses `candid::Principal` for its only Candid API use.
`patches/canister-sig-principal.patch` proposes replacing that dependency/import
with `ic_principal`, against reviewed upstream
`3e5786bb06f94e898d3f8371ec088be23abec987`. It is not submitted or selected by
production. Upstream acceptance/release and subsequent Platform qualification
are required before changing the published dependency path. The candidate passes
all 24 upstream library tests on Rust 1.99/1.88 and Wasm compilation at 1.88.
Standalone creation/verifier probes preserve the same vectors at 1.88; both native
and Wasm graphs lose Candid only when the proposed source is explicitly selected
in those disposable probes. Raw ic0 remains 1.2.0. The current production features
centralize direct ownership without eliminating that Candid edge or proving a
Wasm-size reduction. See [key/hash ownership](https://github.com/dragginzgame/ic-platform/issues/16)
and [verification/BLS isolation](https://github.com/dragginzgame/ic-platform/issues/18).

## Native Agent extension

The optional `agent` feature exposes `ic_platform::agent` on native hosts only.
It currently locks `ic-agent` 0.49.2, matching Query's requirement and Canic's
exact selection, with upstream defaults disabled and PEM enabled for Canic's
exported identities. Agent and its module are absent on Wasm even with all
features; neither runtime nor parser features enable it. Agent does not enable
Platform's Candid, certification or parser features, though upstream Agent itself
depends on Candid and certification. Select `certification` for shared certificate
types and `candid` for DTOs/decoding when needed. No browser Agent is qualified.

The read-only Agent audit used working trees at Query `8982c2a4`, Canic
`5583496bf` and Host `8b458763`; these bases do not imply clean source. Copied
certificate/decoder source hashes bind the local integration proof below.

The explicit exports cover reviewed Query host transport/certificate paths and
Canic's ICP management/query and signed-receipt/capacity-import paths: `Agent`,
`AgentBuilder`, `AgentError`, `Identity`, `BasicIdentity`, `Secp256k1Identity`,
`Prime256v1Identity`, `TransportError`, `HttpErrorPayload`, `Envelope`,
`EnvelopeContent`, `SignedUpdate`, `RequestId`, `RequestStatusResponse`,
`RejectCode`, `RejectResponse`, `ReplyResponse`, `lookup_value`, `to_request_id`
and `signed_update_inspect`. They are upstream identities, not replacement
clients, request records or error projections. The certification surface also
explicitly supplies observed `SubtreeLookupResult` and `labeled_hash` operations.

The same native Agent feature also exports `CallResponse`,
`TransportCallResponse`, `AnonymousIdentity`, `DelegatedIdentity`, `Delegation`
and `SignedDelegation`, covering Backup's submission and anonymous fixtures,
Auth's delegated ingress fixtures and Blob's retained signed-update/certificate
responses. Flatten `ic_agent::agent::CallResponse`, `ic_agent::identity::{...}`
and `ic_agent::TransportCallResponse` to `ic_platform::agent::{...}`. Retain
single-dispatch response bounds and pending/lost-reply spending: `Poll` and
`Accepted` do not grant authority to poll or resubmit. Keep exact delegation
keys, targets, expiration, permissions, signatures and selected root keys;
unchecked construction deliberately leaves authentication to the receiving IC.
Consumers configure Reqwest/middleware through their own explicit compatible
Reqwest dependency, not an Agent blanket export. Keep no-redirect/proxy/retry
middleware settings and actual transport type identity unchanged.

For adoption, enable `agent` on the existing inherited Platform dependency in
host members and CLI packages. Product host features should select
`ic-platform/agent` in place of `dep:ic-agent`; test-only Agent use belongs in
dev-dependencies. Remove direct upstream member/catalog declarations, update
locks and boundary rules. Replace `ic_agent::` imports with
`ic_platform::agent::`, flattening `agent::AgentBuilder`,
`agent_error::{TransportError, HttpErrorPayload}`, `agent::RequestStatusResponse`,
`agent::signed::SignedUpdate`, identity and envelope paths to the explicit root
exports. Use `ic_platform::certification::Certificate`, `SubtreeLookupResult`
and `labeled_hash` instead of Agent's upstream certificate/hash-tree re-exports.
Preserve the same resolved Agent, Principal, certification and transport package
identities across Query, Canic, Agent and other protocol crates.

Keep endpoint parsing before upstream infallible URL-builder paths, selected
identities, PEM error mapping and secret handling, TLS/HTTP middleware, root-key
admission, query-signature verification, historical/live certificate policy,
response-body ceilings, ingress expiry and effective canister IDs unchanged.
Query's body/aggregate admission and outer logical retry categories remain
Query-owned; Canic retains exact signed envelope bytes, request IDs, issuance,
submission/reconciliation and effect retry policy. Upstream Agent's internal
HTTP retries still apply; do not add a Platform retry layer or reset identities
between attempts. Consumers may keep independent Reqwest/Tokio dependencies for
their own client/runtime configuration. Agent performs transport protocol work,
not parser or application ownership. `lookup_value` reads an unauthenticated tree;
`signed_update_inspect` checks envelope content without verifying its signature,
freshness or replay authorization.

The standalone Agent/certification consumer declares only Platform. It exercises
offline signing, exact raw payload/effective-ID preservation, canonical request
identity, mismatch errors, PEM identities, typed HTTP/transport/reject errors and
shared certificate types. Query's five certificate tests and Host's six decoder
tests run in a disposable combined copy with only import paths adapted. Evidence
and original source hashes are under `target/agent-query/`; this is not a migrated
or fully built sibling workspace. Real adoption must still run Query/Canic's
bounded HTTP, authentication, signed-request, retry and effect-reconciliation
tests. Native macOS, live gateway and installed-canister qualification remain
separate. Centralizing direct dependencies does not remove Agent's transitive
Candid/crypto/HTTP dependencies or establish any Wasm-size reduction.

## Testkit and Query migration

The Candid surface is usable for local integration qualification now. Registry
adoption requires an explicitly published, qualified Platform version; repository
creation or a local commit does not establish publication. The public GitHub remote
exists, but consumers have not been migrated by this work.

1. Declare the selected `ic-platform` version in each workspace dependency catalog
   with `default-features = false, features = ["candid"]`; child manifests inherit
   it with `workspace = true`. For local qualification only, select an explicit
   Platform path. Replace direct Candid and `ic-testkit-candid` dependencies in
   normal/dev/build/target tables, CLI and development packages too. Remove unused
   root catalog entries and regenerate each affected lockfile deliberately.
   Published Testkit 0.54.0 now consumes Platform's canonical limits/error
   identities directly. Upgrade the selected Testkit and qualify the actual
   driver before removing its old decoder declaration. Consumers still selecting
   0.52/0.53 must retain that narrow native dependency. Preserve tuple arity and all
   five limits without conversions, re-export packages or replacement transport
   ([published producer coordination](https://github.com/dragginzgame/ic-platform/issues/2)).
2. Replace `ic_testkit_candid` paths with `ic_platform::candid`. Import DTO types,
   encoders and generic argument bounds from `ic_platform::candid::contract`.
   Replace `candid::Result<T>` with `Result<T, contract::Error>`. Use
   `contract::serde::Deserialize` (and its `de` traits) where appropriate;
   independent Serde use need not move. Principal-only code uses `ic_platform::Principal`.
3. Add `#[candid_path("ic_platform::candid::contract")]` to every Candid-derived
   type. When deriving Serde through Platform, also add
   `#[serde(crate = "ic_platform::candid::contract::serde")]`. Preserve field and
   variant labels, rename attributes, optionality and numeric types. Update
   generators/templates and regenerate their DTOs, rather than patching output.
4. Replace `Encode!()` with `contract::encode_args(())`, `Encode!(&request)` with
   `contract::encode_args((&request,))`, and multi-argument macros with tuples.
   Replace test `Decode!` and unbounded decoders with the intended bounded API
   and explicit test-owned limits. Do not replace argument tuples with records.
   Existing configured/staged policies instead use `candid::decoder` unchanged;
   imported contract macros support existing macro contracts. No blanket crate
   alias or arbitrary new resource/diagnostic policy is required.
5. Preserve Query's `candid_decode.rs` policy: transport-owned byte admission,
   `min(bytes.len() * 32 + 1_000_000, 256_000_000)` using saturating arithmetic,
   skipping quota 100_000, 4_096 type entries and 64 KiB header limit. Preserve its
   existing error projection. Testkit keeps PocketIC calls/lifecycle, transport
   errors and caller-selected limits. Neither policy belongs in Platform defaults.
6. Qualify real consumer DTOs and generic call helpers with only Platform declared
   for Candid. Retain Testkit malformed/quota/arity/source cases and Query's large
   byte vector, recursive DTO, bounded skipping, ledger and governance tests.
   Compare encoded request bytes and external DID schemas; preserve retry/call
   identities, source authentication and family-owned error handling.
7. In the coordinated adoption, remove `crates/ic-testkit-candid`, its workspace
   membership, dependency catalog entries, package README/publication entries and
   release metadata. Update Testkit's Makefile, `scripts/release/metadata.sh`,
   `publish-workspace.sh`, package checks and release/publish guard fixtures.
   Migrate Query/CLI and other actual users (including Canic and Timers tests)
   before claiming retirement complete. Keep one decoder/test owner here, without
   a compatibility re-export package or permanent duplicate implementation.

Host-side Candid adoption is distinct from whole-workspace dependency isolation.
Testkit's `ic_testkit_perf_probe` still needs CDK macros; Query's `canister` feature
still needs `canister_self`, `Call` and `CallFailed`. The new optional `cdk`
surface supplies these paths, but consumer qualification against the published
selection remains required. Do not declare a workspace migrated while direct
manifests, generators or independent fixture workspaces remain.

## Runtime trace and required qualification

The existing `executor` feature explicitly exports upstream task/context APIs.
Platform now requires `ic-cdk-executor = "=2.0.0"`, preserving Timers' exact
qualified downstream constraint, not just this lockfile. A standalone Platform-only
fixture rejects a candidate 2.0.1 package and compiles on native/MSRV/Wasm. The
combined final dependency graph must still resolve
one executor package identity (version and source), including its method handles.

The independent `ic0` feature explicitly exports the raw IC 1.2.0 operations used
by Timers' platform/backend/FFI modules. It requires `ic0 = "=1.2.0"`, selects
neither Candid nor an executor, and preserves upstream safe/unsafe callback ABI.
Select both `executor` and `ic0` in Timers; replace every `ic0::` path with
`ic_platform::ic0::` and remove its direct ic0 and constraint-only executor
declarations after qualifying a published Platform selection. Timers retains
its sole global-timer/dispatch exports, callback environments, contexts, call
failure/retry and trap cleanup. `stable64_size` is observation, not storage or
allocation ownership. The exact-pin exceptions are recorded in AGENTS.md and
the declaration checker; changes require renewed combined installed qualification.

Testkit's Wasm performance module also needs `ic_platform::ic0::debug_print`,
which directly exports upstream's safe `fn(&[u8])` under the existing `ic0`
feature. Replace its sole remaining `ic0::debug_print` path and remove the member
and unused workspace declarations after published qualification. Keep
`format_marker`, prefixes, start/end labels, caller byte limits, instruction
counter 1, stable-page conversion and intentional traps unchanged. No logger,
formatting wrapper or new feature is introduced. The independent ic0-only
consumer compiles on native/MSRV/Wasm without Candid, parser or CDK frontend;
native compilation does not execute IC debug output. Testkit's private probe must qualify the owned CDK frontend below before
retiring its existing CDK/Candid declarations.
See [benchmark markers](https://github.com/dragginzgame/ic-platform/issues/11).

IcyDB's metrics/entropy and codec-query adapter additionally need direct upstream
`in_replicated_execution`, `msg_reply_data_append` and `msg_reject` exports;
IC Wire's controller path also needs `is_controller`. All four are now explicit
exports under the existing `ic0` feature, preserving IC 1.2.0's safe signatures:
the predicates return `u32`, and controller/reply/reject arguments are byte slices.
Replace only the corresponding `ic0::` paths with `ic_platform::ic0::`, remove
unused member/root declarations and their exceptions, then update locks and
ownership policies after a published selection is qualified. Generators emitting
these raw paths must change too. No new feature or Candid selection is needed.

Retain `!= 0` / `== 0` replicated/query predicates, existing entropy admission and
metrics decisions, Principal bytes, early 64-byte request/reply limits, rejection
bytes, append order and explicit reply finalization. Rejection is not a successful
empty reply. Authentication, byte limits, retry/effect authority and controller
policy stay in consumers. These direct raw identities do not supply CDK's
Principal/Vec convenience implementation or entrypoint/call/protected-spawn
integration; the optional CDK frontend below preserves them. Storage and allocation APIs
remain outside this work.

In particular, CDK 0.20.3's `api::is_controller(&Principal) -> bool` accepts
only raw 0/1 and panics on any other result. Wire and Auth still require that
checked frontend; replacing it with `!= 0` or `== 1` changes the failure contract.
The optional `cdk::api` surface retains that same checked helper, plus allocating `msg_arg_data` and Principal-returning `msg_caller`.
Raw IC exports do not replace those helpers. Consumer admission before argument
allocation and caller/controller interpretation remain consumer-owned.

Read-only review used IcyDB base `c93a7440` and IC Wire base `ce56d5eb`, including
dirty source; hashes are retained in `target/raw-boundary-before/consumers.sha256`.
The standalone ic0-only consumer compiles safe caller code natively and for Wasm
on Rust 1.88 without executing system calls on native hosts. A disposable copy of
IcyDB's actual codec-query adapter changes only raw-call imports and its manifest;
runtime qualification belongs to the retained `target/raw-runtime/` fixture.
This does not migrate the sibling workspace or qualify its entire database.

A disposable copy of the actual Timers library removes those two declarations
and changes only system-call paths. Its 160 native tests pass at Timers' Rust
1.91 floor; the production Wasm library compiles with one executor 2.0.0 and
ic0 1.2.0 identity and no Candid/CDK frontend/parser. Source hashes and evidence
are under `target/issues-timers/`; Platform-only resolver proof is under
`target/issues-executor/`. These are source-copy/native/Wasm proofs, not actual
sibling migration, publication or installed callback/trap/upgrade execution.

CDK 0.20.3's `futures/internals.rs` delegates entrypoint contexts to
`in_tracking_executor_context` and `in_tracking_query_executor_context`.
Canic's adapter emits those CDK context wrappers and async spawning. Its raw call
builder preserves bounded/unbounded waits, cycles and exact `with_raw_args` bytes.
Management adapters in `infra/ic/mgmt` and external ledger calls remain Candid.
No change here selects deadlines, retries, limits or protocol resets for them.

CDK `call.rs` retains a method handle when submitting a call, restores it in
`in_callback_executor_context_for` before waking the future, and uses
`in_trap_recovery_context_for` to cancel attached tasks plus the saved task after
callback rollback. Timers' `platform/ffi.rs` follows the same identity through
self-dispatch, response and cleanup; its own flight table, retry and rearm policy
stay in Timers. Replacing this with another executor would break that contract.

### Maintained runtime and Wire entrypoint adoption

The `cdk` feature now selects Platform-owned, non-cryptographic runtime adapters,
the owned generator, registry executor =2.0.0 and ic0 =1.2.0. It selects neither
Candid nor the full `ic-cdk`/`ic-cdk-macros` packages. Executor-only users retain
their independent feature. No executor implementation is copied or replaced.
The imported adapters retain their Apache-2.0 license and provenance in
`crates/ic-platform/src/cdk/upstream/NOTICE.md`; the combined consumer package is
MIT AND Apache-2.0. Unsafe code is confined to imported call ownership/callbacks
and the existing initialized stable-byte copy, under a workspace deny lint.

This is the **breaking 0.2.0 migration**. The workspace and generator declare
0.2.0; consumers must verify both supplying registry packages before adoption.
A local version declaration alone does not establish publication or qualification.

#### Protocol and feature selection

All application endpoints we own use Wire, including public endpoints called by
third parties. Here, a required external Candid interface means an independently
defined protocol such as management, NNS/SNS or ledger APIs, never simply a caller
outside our team or network. Candid is our interoperability adapter to those IC
services, not an application endpoint option. Platform owns IC export names,
sync/query/async contexts and protected spawning; Wire owns serialization,
method/direction frames and caller-selected byte/allocation/node/depth budgets.
Consumers own endpoint IDs, authentication, semantic validation and retry policy.
Platform has no Wire dependency and does not invent a default message codec.
Explicit hooks enforce codec selection, not Wire conformance; consumer tests must
verify Wire framing and bindings. Codec performance claims require measurements
for the stated workload and are not part of this compatibility contract.

Callable `query`/`update` macros require `decode_with`, including zero-Rust-argument
RPCs, and require `encode_with` unless the handler uses `manual_reply`. Lifecycle
methods with arguments also require explicit decoding; no-argument IC lifecycle
hooks may omit it. Use an admission guard to check argument size before copying.
Failed decoding must trap/reject before handler effects; response-encoding failure
must trap. There is no Candid fallback, dummy Candid service registration,
`export_candid!` macro or `candid = "path"` entrypoint option. Existing explicit
From 0.5, remove the retired no-op `hidden` option from Wire hooks and generators.

Automatic `encode_with` continues to require `Vec<u8>` so generic encoder
inference remains compatible. An existing `manual_reply` handler can pass a
borrowed slice or fixed array to `cdk::api::msg_reply` without allocating a reply
vector. Use this only for already encoded protocol bytes; Wire framing,
exactly-once replies and failure-before-effects remain consumer responsibilities.

```rust,ignore
#[ic_platform::cdk::update(
    guard = "codec::admit_request",
    decode_with = "codec::decode::<_, { Method::Start as u64 }>",
    encode_with = "codec::encode::<_, { Method::Start as u64 }>"
)]
fn start() -> bool { true }
```

Select `cdk,candid` only when the canister calls a required external Candid
interface, such as management/system canisters and standard NNS/SNS/ledger APIs.
The 0.4 runtime no longer exposes codec methods, even under that explicit Candid
selection. Use the separate Candid encoders/decoders as described in the
[transport migration](#transport-and-codec-separation-04).
The DTO features retain their original registry
identities and external bytes; they do not implicitly select Platform's codec API.
Native parser, Agent and PocketIC features stay separate from canister features.

Raw `Call` constructors now start with **empty transport bytes**, not implicit
Candid unit arguments. Always supply a Wire request frame, including unit requests,
using `with_raw_args`/`take_raw_args`. External no-argument Candid calls explicitly
encode `()` with `candid::contract::encode_args` and supply those raw bytes.
Preserve exact retained request/retry bytes, bounded/unbounded
wait choices, the bounded 300-second default, cycle amounts and reply error policy.

#### Runtime ownership and compatibility

The public paths remain `ic_platform::cdk::{api,call,futures,stable}`. Their explicit
reviewed helper surface remains available. The imported call state machine keeps
method/task handles, restores the same executor identity in callbacks, and cancels
attached/current tasks during callback-trap cleanup. Protected `futures::spawn`
retains its unfinished-task panic; raw `executor::spawn_protected` is not an
interchangeable policy. Timers retains its only global-timer entrypoint, queue,
rearm, cancellation and retry ownership.

Call/response/future, cost/performance/error and stable-error types are now
Platform-owned identities. Their corresponding re-exported identities are not
maintained through aliases, conversion packages or a second CDK dependency.
The 0.4 runtime RejectCode and signature-key identities are also Platform-owned.
ExternalPrincipal, external DTO, certification and BLS identities remain upstream.
Replace direct `ic_cdk` identity assertions with reviewed
behavior/signature tests and compile actual consumers against the new owner.

`stable::{stable_grow,stable_read,stable_write,StableMemoryError}` preserves raw
bytes, previous-page results, bounds traps and rollback. `api::stable_size` remains
an observation. Existing `storage::{stable_save,stable_restore}` requires explicit
`candid` solely to preserve already-persisted checkpoint bytes and restore behavior,
including its original lack of decoder finalization. This is a legacy data
compatibility exception, not permission to introduce new Candid application
messages or storage formats. Storage implementation/allocation work remains out
of scope. No layout, retry policy, trust root or cryptographic primitive changes.

#### Runtime maintenance and recovery

The maintained boundary must let us build, diagnose and update our applications
if upstream Rust tooling stops receiving releases. It still depends on the IC
protocol and established cryptographic implementations. Dependency isolation,
source availability and demonstrated recovery are separate claims.

| Component | Maintenance responsibility and qualification |
| --- | --- |
| Owned generator and runtime adapters | Platform reviews upstream CDK fixes for applicability and maintains entrypoints, call ownership and cleanup. Preserve Apache provenance and qualify installed behavior. |
| Embedded executor and IC0 (originally 2.0.0 / 1.2.0) | Platform maintains their source and Apache licenses, preserving one runtime owner and the System API ABI. Review executor fixes and IC System API changes; changes require combined entrypoint/call/Timers qualification. No second executor. |
| Candid, parser and required IC DTOs | Platform controls dependency exposure; retain external wire fixtures, bounded decoder behavior and host-only parser separation. Consumers own protocol selection and limits. |
| Agent and PocketIC | Platform controls explicit client/type exposure. Host/Query retain transport and verification policy; Testkit retains simulator provisioning and admission. Archive required binaries as well as Rust sources for recovery. |
| Certification and cryptographic implementations | Preserve upstream implementations, vectors and licenses. Platform may maintain scaffolding; algorithm or primitive changes require qualified external cryptographic expertise. |

For each runtime change, inspect relevant upstream fixes and IC interface changes
and record the source revision, applicability and validation in the PR. An unchanged
upstream version is not evidence that no fix is needed. Track unresolved findings
in GitHub issues rather than a second queue in this document.

The qualified call and protected-spawn fingerprints are admission controls, not
an instruction to freeze bugs forever. To intentionally change a protected region:

1. Explain the concrete defect and before/after behavior; review ownership,
   callback contexts, trap rollback and cancellation against the IC contract.
2. Preserve the original import revision/license. Record the reviewed deviation
   beside the import provenance; never relabel modified code as unchanged upstream.
3. Add behavior coverage for the defect and run the installed runtime suite with
   both Wire-only and required-Candid selections, including Timers and upgrade.
4. Run native/MSRV/Wasm consumer checks, graph identity checks and source-mutation
   refusal checks. Update the expected fingerprint only as part of that reviewed
   change, after behavioral evidence; changing the hash alone is not validation.

Recovery evidence must survive the development machine. For an authorized release,
retain the exact committed Platform/generator source, both maintained lockfiles,
dependency sources with checksums/licenses, toolchain and target identities,
prepared host tools, Testkit-admitted PocketIC binary and qualification logs/Wasm.
Include dependencies from the generated installed-runtime fixture, not just the
root workspace. Cargo source vendoring can capture the root and fixture manifests
in one operation; it does not capture toolchains, host utilities or the simulator.
Store the resulting artifact and its checksum in durable release storage. Exclude
credentials, Cargo configuration containing secrets and unrelated user caches.

A recovery drill uses an isolated copy, a fresh Cargo home and target directory,
restored source-replacement configuration and disabled network access. It must
build the standalone Wire and required-Candid consumers, check one executor/ic0
identity, and run installed callback/trap/cancellation/upgrade qualification using
the restored simulator. Use the existing runtime fixture's generated manifest and
lock at its recorded relative location; do not resolve new dependencies during the
drill. Also verify the pinned build tools and supported toolchains are recoverable.
Passing with an existing developer cache is not clean-cache recovery evidence.

Routine CI currently checks source fingerprints, mutation refusal and dependency
graphs; it does not provision a simulator or run the installed suite. Run
`make check-runtime` with a Testkit-admitted `POCKET_IC_BIN` or
`IC_TESTKIT_POCKET_IC_URL` for installed qualification. A remote server is useful
for behavior tests but cannot establish recovery of the simulator itself. Native
macOS and live IC results remain separately reported. The recovery procedure above
is an acceptance contract; no completed recovery drill is claimed here.

#### Consumer migration and qualification

1. Publish the coordinated next-minor Platform/generator packages before adopting
   their registry version. Verify registry availability rather than inferring it
   from the workspace version.
2. Select `cdk` for Wire canisters/probes; retain `executor,ic0` for executor-only
   libraries. Add `candid` explicitly only for required external calls or existing
   Candid checkpoints. Preserve the exact selected external DTO versions.
3. Update actual endpoint generators and manifests. Supply Wire decode/encode
   hooks, pre-copy admission and unit-frame validation. Remove Candid endpoint
   derives, service-export calls, generated DID registration and retired
   `candid` path options when they no longer serve an external interface. Generate
   Wire bindings at the consumer's canonical protocol owner. Avoid two readers.
4. Supply explicit bytes on every call; preserve request/reply identities and
   caller budgets. For required external Candid calls keep the existing DTOs,
   encoders/decoders and wire fixtures, adding explicit unit arguments where needed.
5. Remove exhausted direct full-CDK/macros declarations and obsolete ownership
   exceptions. Update typed error tests for Platform identities. Never add a
   compatibility re-export package, independent executor or local callback runtime.
6. Check standalone native/MSRV/Wasm consumers and actual normal/build graphs.
   Wire-only deployments must reject Candid/derive/parser/full-CDK packages.
   Canisters with external Candid calls permit Candid explicitly, still rejecting
   full CDK and host clients. Resolve one embedded Platform executor/IC0 owner across
   generated entrypoints, calls and Timers.
7. Qualify actual awaited/rejected/submission-failed/insufficient-cycle calls,
   protected/weak/migratory tasks, encoding and callback traps, cancellation,
   upgrade, external Candid bytes and persisted checkpoints. Native macOS,
   installed PocketIC and live IC qualification are separate evidence.

The maintained fixture uses Wire for every RPC and install/upgrade argument.
Its pure runtime graph is Candid-free; a separate `external-candid` selection
qualifies an outbound management call and original persisted checkpoint bytes.
The native Testkit transport's legitimate Candid graph is not a canister graph.
These changes remove the full CDK packages from the selected dependency graphs;
no measured Wasm-size reduction is claimed. See [the current handoff](status/current.md)
and [#31](https://github.com/dragginzgame/ic-platform/issues/31).

## Host parsing and aggregate admission

The optional `candid-parser` feature now exposes `ic_platform::candid_parser`
on native hosts, selecting upstream `candid_parser` 0.4.1. The dependency and
module are target-gated out of Wasm, including all-feature builds. No Candid,
executor or certification feature enables host parsing. Parser-only native
selection does pull transitive Candid with the parser's upstream `all` feature;
it does not select Platform's `candid` feature. This is dependency isolation,
not transitive removal or a Wasm-size measurement.

The explicit surface covers the reviewed Host service inspector and Canic's
external protocol argument helpers and service/type comparisons:
`IDLProg`, `check_prog`, `TypeEnv`, `Type`, `TypeInner`, `FuncMode`, `Dec`,
`Error`, `Result`, `IDLArgs`, `parse_idl_args`, `CandidSource`,
`instantiate_candid`, `service_equal`, `TypeContainer` and `type_equal`. These directly preserve upstream
identities; there is no blanket crate/module export or new parser implementation.
The pending 0.7.0 retirement below removes the earlier compiler exports and
their unused merged-program helper after audited Wire consumer adoption.

Native dynamic refusal fixtures additionally use explicit `IDLValue`, `IDLField`,
`TypeSerialize` and `ValueSerializer` exports from `ic_platform::candid_parser`.
Replace upstream `IDLValue`, `types::value::IDLField` and
`ser::{TypeSerialize, ValueSerializer}` imports with those flat paths. Existing
`IDLArgs` and contract `types::{Field, Label, Type, TypeInner}` remain the same
identities; select `candid` alongside `candid-parser` for contract derives and
configured decoding. Blob should retain these selections in native development
dependencies, remove its now-unused direct native Candid declaration, update the
lock and boundary policy, and rerun its existing old-field, oversized-header,
refusal/rollback fixtures. No hand-built type headers or independent serializer
are required. The module is still absent on Wasm; the production `candid` feature
alone does not activate upstream dynamic values or the host parser.

To adopt, enable `features = ["candid-parser"]` on the existing inherited
Platform dependency in host members, or `ic-platform/candid-parser` from an
optional product capability. Remove direct `candid_parser` declarations from
members and the unused root catalog; update locks and dependency-boundary rules.
Canister libraries that use parser helpers only in integration tests must select
the host feature through their dev-dependency rather than runtime features.
Replace `candid_parser::` imports with `ic_platform::candid_parser::` and flatten
`syntax::Dec` and `utils::{CandidSource, instantiate_candid,
service_equal}` imports. Use root `FuncMode`, `Type`, `TypeInner`, `TypeEnv`,
`TypeContainer` and `type_equal` for those host type operations. Owned binding
generation uses Wire's canonical endpoint metadata and generator through the
consumer's existing Build/frontend integration, not Candid compiler aliases.
Move ordinary DTO/encode/decode uses
of the parser's blanket Candid export to Platform's `candid` contract/decoder
instead; this host surface deliberately provides no `candid` crate alias.

Retain consumer source/output byte admission, method bounds, import rejection,
artifact hashes, expected service/type rules, aggregate budgets and error
projection. Parsing/type checking has no automatic stack, heap or work ceiling.
`check_prog` ignores imports rather than rejecting or resolving them; consumers
requiring self-contained text must inspect `Dec` first. `CandidSource::File`
reads files/imports and the upstream pretty loaders can print diagnostics;
`CandidSource::Text` and `service_equal` can print parse diagnostics too. Canic's
silent `parse_text` path must continue parsing `IDLProg` and using `check_prog`
directly. Do not replace it with `CandidSource::load`.

The standalone parser-only consumer requires no direct Candid/parser dependency.
It exercises constructor typing, malformed grammar,
undefined types, import detection, service drift, Rust type equality and a fixed
nat64 argument wire vector. Host's service inspector and existing tests are also
qualified from a disposable copy with only imports changed; source hashes and
fixture evidence are under `target/parser-before/host-source.sha256` and
`target/parser-host/`. These are focused proofs, not migrated sibling workspaces.
Actual Host/Canic adoption must still compare service inventories,
external argument bytes and existing protocol/effect/retry tests. Neither
consumer product code nor external Candid formats or persisted data change here.

Host's pending decoder first rejects oversized replies, then atomically subtracts
admitted bytes from the caller's remaining budget before decoding. Malformed
admitted replies still consume budget; failed admission does not. Consolidation
must preserve those semantics outside the per-reply Platform decoder. Host's
pending helper does not select compact diagnostics; adopting Platform changes
that diagnostic policy and needs explicit caller qualification.

Host's service inspector uses `IDLProg`, type checking and `TypeEnv` to project
bounded method facts, rejects imports, and leaves Wasm export identities to Host.
Canic retains textual external/init arguments and service equality;
method inventory alone cannot replace those uses. The new explicit host surface
covers those operations; Host and Canic retain the surrounding implementation.

### Pending 0.7.0 compiler-only retirement (#38)

The maintainer explicitly authorized completing the compiler retirement after
the current consumer re-audit. Remove imports/calls of
`ic_platform::candid_parser::{compile_javascript, compile_typescript, IDLMergedProg}`.
There is no replacement Candid generator alias, compatibility package or new
fallback codec. Owned endpoints and frontend binding artifacts use Wire metadata,
Wire's existing TypeScript generator and the consumer's reviewed JS/declaration
build path. Preserve method/direction/schema identities and artifact admission at
their existing owners. Required external Candid inspection and textual argument
encoding continue through the unchanged parser helpers above; `candid-service`
remains available for independently defined external service declarations.
After publication, move the inherited Platform requirement to the 0.7 line and
keep only the features each consumer still needs. Align the consuming framework
family rather than mixing Platform 0.6/0.7 owned Principal/Account identities in
one application graph. The coordinated macro package follows Platform's release.
Existing external Candid identities and owned Wire representations are unchanged;
those facts do not make different Platform package versions one Rust type.

Audit evidence: Canic default `aed32be2` has no references to the three removed
names under `crates`; its frontend ops uses retained Wire schemas and bindings.
Current Canic/Query/Host/Tooling/Blob/Auth/Backup/IcyDB/Jobs/Metrics/Persistence/Relay
crate sources likewise have no callers. Host still consumes parsing/type checking;
Canic's `fleet_ensure::ops::init_arguments` retains external/init textual argument
handling. Existing external service and dynamic refusal behavior is preserved.

| Retained parser surface | Actual external responsibility |
| --- | --- |
| `IDLProg`, `Dec`, `check_prog`, `TypeEnv`, `Type`, `TypeInner`, `FuncMode`, `Error`, `Result` | Host's bounded external service inspector and typed parser diagnostics |
| `CandidSource`, `instantiate_candid`, `parse_idl_args`, `IDLArgs` | Canic external/init arguments and external mint/receipt contract fixtures |
| `service_equal` | Blob's declared external installation/service contract comparison |
| `IDLValue`, `IDLField`, `TypeSerialize`, `ValueSerializer` | Native external refusal/old-field fixtures; existing serializer bytes stay upstream |
| `TypeContainer`, `type_equal` | External DTO/schema equality qualification, including Platform's pinned DID proofs |

Canic's owner records passing installed frontend handoff and generated-client
proofs in `docs/audits/working/0.112-push-readiness/owned-candid-cut.json` at
local `b9482ec526c5dfb9f5dd349d22b1f60190b2dfdf`; the inspected evidence file has
SHA-256 `989a4ce7bcd9f108fcff7fad602e554c97036e94c55c82267f7dca0389e5cc60`.
Its frontend admission proof passed in 275s; its later dedicated/embedded generated
browser-client proof is recorded as passed in 637s. These are the consumer owner's
retained observations, not new sibling execution or full #613/production rollout
acceptance by Platform. No sibling was edited or built in this task.

This removes supported Rust APIs, so use the next minor release (0.7.0), not a
0.6 patch. Own package versions remain unchanged until the maintainer executes
that release. No external Candid bytes, checkpoint/storage formats, retry policy,
runtime/executor identities or cryptographic implementations change.
The parser package remains `candid_parser` 0.4.1, including its unconditional
bindings/template scaffolding and Handlebars/derive-builder/Darling dependencies.
This is facade retirement only, not transitive dependency removal, a parser fork
or evidence of smaller Wasm or faster execution.

## Management-canister DTOs

The optional `management-canister-types` feature exports
`ic_platform::management_canister_types::{HttpRequestArgs, HttpRequestResult,
HttpMethod, HttpHeader, TransformContext, TransformFunc, TransformArgs}` directly
from `ic-management-canister-types = "=0.11.0"`. This explicit surface covers
Canic's observed `canisters/test/runtime_probe/src/perf_context/mod.rs` HTTP
request use and its nested/header/transform and response types. The read-only
review used Canic HEAD `5583496` and probe SHA-256
`4615d604316eaf4dbbc17baf9f985742c2b8533bfdcc50c8e04e1eb2c6d8616c`;
the working tree has pending changes, so HEAD is not a clean-source claim. Other upstream
management DTOs are not exported speculatively. No call facade is introduced.

Blob's recovery consumer additionally requires `CanisterInfoArgs`,
`CanisterInfoResult`, `ChangeDetails` and `CodeDeploymentMode`, now explicitly
exported by the same feature with their original 0.11.0 identities. Change only
those DTO imports and declarations when adopting. Retain Blob's bounded
`canister_info` call, 20 requested changes, timeout, reply/decode limits, version
checks across await, evidence expiry/completeness and recovery fences. The API
does not decide which deployment, snapshot, controller or rename changes prove
continuity; preserve that policy at Blob. Existing CDK calls remain subject to
the runtime exception. See [history DTOs](https://github.com/dragginzgame/ic-platform/issues/12).

Read-only review used Testkit `55ac397` performance source SHA-256
`cda2adcff73ff2618ed3760339c30442e23ae1668c5eba85f3a8011fa1b62e99`
and Blob `55d70cb` recovery source SHA-256
`0135a9c284e713a23880bfc3134d344a773c22122df2c63d5f8bc4eb1ab4168c`.
Both are working-tree observations; the commit IDs do not claim clean source.

Backup's current snapshot/status consumers additionally require 24 upstream
identities now exported by this same feature: `CanisterIdRecord`,
`CanisterStatusResult`, `CanisterStatusType`, `CanisterTimer`, `ChunkHash`,
`DefiniteCanisterSettings`, `LoadCanisterSnapshotArgs`, `MemoryMetrics`,
`OnLowWasmMemoryHookStatus`, `QueryStats`, `ReadCanisterSnapshotDataArgs`,
`ReadCanisterSnapshotDataResult`, `ReadCanisterSnapshotMetadataArgs`,
`ReadCanisterSnapshotMetadataResult`, `Snapshot`, `SnapshotDataKind`,
`SnapshotDataOffset`, `SnapshotMetadataGlobal`, `SnapshotSource`, `StatusVisibility`,
`TakeCanisterSnapshotArgs`, `UploadCanisterSnapshotDataArgs`,
`UploadCanisterSnapshotMetadataArgs` and `UploadCanisterSnapshotMetadataResult`.
These are explicit direct exports from exactly 0.11.0, not local DTOs or storage
operations. Read-only review used Backup's working tree at base `3317b4e6`;
`target/management-snapshot-before/backup.sha256` binds the reviewed source and
golden files. It is not a clean-source or released-consumer claim.

Canic's existing snapshot-delete call additionally requires the now-exported
`DeleteCanisterSnapshotArgs` from exactly 0.11.0. After published qualification,
replace `InfraDeleteCanisterSnapshotArgs` in `infra/ic/mgmt/snapshots/mod.rs`
with `ic_platform::management_canister_types::DeleteCanisterSnapshotArgs` and
remove that local request DTO from `types.rs`; no new feature/dependency is
needed. Update generators/tests that emit the old request name. Keep intentional
partial status/history response projections and their admission policies.
Preserve `Call::unbounded_wait`, the management target/method, original canister
and snapshot IDs, journal/authority checks, retry and recovery behavior.

Read-only Canic review used dirty source at base `5583496b`; reviewed inputs and
their hashes are retained under `target/delete-snapshot/`. A disposable copy of
its actual two-field DTO and the original upstream encoder produce identical
empty/nonempty snapshot-ID vectors and cross-decode in both directions. The
Platform-only management consumer preserves those vectors, truncation/arity/byte
refusals and zero-skipping admission on native/MSRV/Wasm. A separate native/Wasm
Rust 1.88 proof establishes bidirectional Rust identity with the upstream type.
No snapshot deletion or consumer lifecycle qualification is inferred from DTO
tests; Canic retains those operations. Published 0.1.9 supplies this addition;
the registry-only consumer passes native and Wasm checks at Rust 1.99/1.88.
Canic reports its local published-selection adoption under the
[management boundary](https://github.com/dragginzgame/ic-platform/issues/13).

To adopt in Backup, enable `management-canister-types` on its existing inherited
Platform dependency in Core and Agent/test selections, replace every
`ic_management_canister_types::` import with
`ic_platform::management_canister_types::`, including native tests and generated
imports, and remove direct root/member DTO declarations plus the obsolete local
exact-pin exception. Update the lock and direct-dependency policy; Platform still
constrains the same upstream package to `=0.11.0`. Keep exported public Rust types
as those upstream identities and retain existing Candid feature selections.

Retain capture/load/status/upload wire/hash goldens, exact arity, zero skipped
work, reserved-marker refusal, byte/work/type/header ceilings, controller/global/
chunk bounds, redacted errors, target IDs, original signed/byte/journal evidence,
finite spending and recovery classification. In particular, decoding the full
upstream metadata DTO with `Reserved` source payloads may consume skipping quota;
the new standalone vector test preserves zero-quota refusal. Do not replace
Backup's admitted projections or loosen quotas merely to change dependencies.
No snapshot execution, retry, polling, authority reset, settlement inference,
storage representation or allocation responsibility moves into Platform.
Registry-only adoption must rerun Backup's model/codec goldens and real Core/Agent
PocketIC snapshot/status/lifecycle journeys. Native macOS and installed-IC
acceptance are separate from Linux source-copy tests and Wasm compilation.
See [Backup's management boundary](https://github.com/dragginzgame/ic-platform/issues/13).

To adopt, add `management-canister-types` to the inherited Platform dependency's
features and replace `ic_management_canister_types::` with
`ic_platform::management_canister_types::` for these symbols. Select `candid`
separately when encoding/decoding through Platform. Remove the member's direct
management-types declaration and the workspace catalog entry once unused; update
the consumer lockfile and dependency-boundary rules. Update any generator that
emits the old DTO imports. Preserve pricing_version, is_replicated, response
ceilings, transform references/context, attached cycles, raw call semantics and
existing retries. No generated service format or persisted data changes here.

The feature is supported on native and Wasm and necessarily brings upstream
Candid; it does not enable Platform's Candid, executor, parser or Agent features.
Default/executor-only graphs still exclude both management DTOs and Candid.
The standalone management/Candid consumer declares only Platform. It compares
the existing probe request against a byte vector generated by the original
0.11.0 owner and exercises response/transform roundtrips and caller byte refusal.
History request/result vectors likewise come from the original upstream encoder;
the consumer declares only Platform and uses the real change records, without
copying DTO schemas. Tests cover every change variant, all deployment modes,
missing details, truncation, byte refusal and exact arity with Blob's limits.
Actual probe migration must still qualify its CDK call/entrypoint paths and
installed scheduling/cycle behavior; DTO qualification supplies no runtime proof.

## ICRC ledger types

The optional `icrc-ledger-types` feature provides an explicit flat
`ic_platform::icrc_ledger_types` module from upstream 0.2.0, with default features
disabled. It exposes `Account`, `MetadataValue`, legacy `Value`, `ICRC3Value`,
`TransferError`, `AllowanceArgs`, `Allowance`, `GetArchivesArgs`, `ICRC3ArchiveInfo`,
`QueryArchiveFn`, `ArchivedBlocks`, `BlockWithId`, `GetBlocksRequest`,
`GetBlocksResult`, `ICRC3DataCertificate` and `SupportedBlockType`. No replacement
DTOs or upstream module blanket exports are introduced. Inherent account/text,
value-hash and typed archive-callback behavior stays upstream. Other ledger APIs
and product-owned wire DTOs are not moved speculatively.

Read-only review used Query `8982c2a4` and Canic `5583496b` working trees. Query's
`src/icrc/ledger.rs`, live account/allowance/history paths, SNS token reporting and
ledger evidence consume these types. Canic's host release-account and operator
mint receipt/transport paths also use them, including transfer errors in tests.
Paths are below `crates/ic-query` and `crates/canic-host` respectively. Reviewed
source hashes are retained under `target/icrc-consumers.sha256`; these bases do
not imply clean sibling source.

To adopt, enable `icrc-ledger-types` on the inherited Platform dependency and
flatten the listed upstream imports into `ic_platform::icrc_ledger_types::{...}`,
including tests and any generators emitting those imports. Query's
`ledger-evidence-host`, `icrc-host` and `sns-host` features should select
`ic-platform/icrc-ledger-types` instead of `dep:icrc-ledger-types`. Remove the
direct optional member declaration and unused root catalog entry, then update
its lock and dependency-boundary rules. Canic's host member selects the same
Platform feature and removes its direct declaration; preserve test-only feature
selection where appropriate. Select `candid` separately for the codec and retain
separate Agent/certification features when actually used.

Preserve selected ledger/archives and effective canister IDs, request/reply and
aggregate bounds, amount/fee admission, hash-chain/certificate verification,
interpretation, deadlines, reconciliation and retry policy. Legacy `Value` has
`Nat64`; converting to `ICRC3Value` changes the wire variant, so do not normalize
existing protocols merely to change imports. Account equality treats absent and
zero subaccounts alike, while their encoded fields can differ. Do not rewrite
persisted data or local ICRC-21 contracts during dependency migration.

The feature brings transitive Candid, CBOR and other upstream dependencies;
explicit exports do not imply a smaller compiled dependency graph. The upstream
`storable` feature is not selected, and Platform's graph gate rejects
`ic-stable-structures`. No executor, parser or Agent is enabled by this feature.
Consumer graphs can still unify features through other dependencies; qualify the
actual final target/feature/lock selection rather than extrapolating this gate.
The standalone ledger/Candid consumer declares only Platform, preserving original
owner account/block wire vectors, value hashes, typed callbacks, certificate
fields, metadata variants and arbitrary-precision amounts on native/MSRV/Wasm.
Actual Query/Canic ledger, archive, receipt and certificate tests remain their
adoption qualification. No transitive Candid removal or Wasm-size reduction is claimed.

## Evidence and isolation

`make test` and `make msrv` exercise the retained six decoder cases and expanded
DTO fixture. `make check-consumer` builds separate workspaces with only Platform
as a direct dependency: native tests on current Rust and the declared 1.91 MSRV,
plus Wasm compilation
on both compilers for Candid, certification, management, raw-system and ledger
consumers, plus canister-signature creation, signature verification, BLS and the CDK
canister/service fixture (linked as a Wasm cdylib on both compilers).
The maintained `make check-runtime` harness separately installs the combined
CDK/Timers fixture, exercising callbacks, trap cleanup, cancellation and upgrade.
It uses published Timers/Testkit at their Rust 1.91 floor; Platform now also
declares that floor. See [runtime setup](../README.md#maintained-cdk-runtime-regression).
Parser/dynamic and Agent consumers are native-only. `make msrv` also
checks the all-feature library for Wasm at the declared MSRV. CI prepares the
Wasm target
for both toolchains; local validation never installs it. See
[Wasm MSRV enforcement](https://github.com/dragginzgame/ic-platform/issues/15).
It exercises recursive/renamed DTOs, generic argument helpers, `Reserved`, typed
error downcasting, and independent Candid wire vectors for nat 2^128 and int -65.
This is representative boundary evidence, not a build of migrated siblings.

`make check-features` checks no features, actual defaults, every single feature,
every pair, all features and explicit critical runtime/DTO/signature/host groups
on native/Wasm. `make check-features-exhaustive FEATURES=cdk,candid,executor`
checks the complete power set of 1–12 explicitly selected features as a separate
manual audit, outside routine CI/release validation. Missing, unknown and oversized
scopes fail before graph queries. Routine
coverage deliberately omits other higher-order combinations; this is not an
equivalent exhaustive proof. Combined CDK/Timers qualification remains separate.
It derives local feature closures from Cargo metadata and reuses graph queries
only within the same invocation. On Wasm, Agent/parser/secp256k1 leaves may share
a key only while each feature activates exactly its single optional native-only
dependency; a changed contract fails closed. Defaults keep a separate key.
All selection-specific admission checks still run in one validator process
per target, including target exclusion checks. Query counts depend on the declared
feature inventory and selected mode; full enumeration requires the explicit audit.
Target-specific TSV mappings and canonical graphs remain in each attempt directory.
Neither post-hoc package-list equality nor reuse of previous runs establishes
feature equivalence. The supported graph properties remain:

Candid/CDK-free defaults and certification-only selection, Candid-free executor
selection, optional certification isolation, explicit native-only parser/Agent and no
Testkit transport. Management DTOs are admitted only when selected, with an exact
optional dependency requirement. Empty default features are checked from Cargo metadata too.
Ledger types are likewise optional and excluded unless selected, with upstream
default/storage features disabled and storage packages refused in every selection.
`make wasm-check` compiles all features together while excluding parser and Agent. Consumer adoption
must also run the shared dependency-boundary checker on every workspace, including
independent fixtures and unused root catalog declarations; aliases are not escapes.

The declared feature inventory must match the reviewed matrix; adding a feature
requires an explicit matrix and isolation-policy update. Graphs admit the qualified CDK runtime and generators only with `cdk` selected;
executor-only builds reject those frontends. Every graph rejects additional
executor/system versions and alternate sources of those runtime identities. See
[graph admission](https://github.com/dragginzgame/ic-platform/issues/14).
The signature extension admits the reviewed BLS/certification identities only
through their selected or required transitive capabilities and rejects the retired
creation/verifier packages. Signature-only selections reject Candid and raw IC
calls. Default/executor and
certification-only isolation remain unchanged. These are workspace graph gates,
not guarantees about feature unification in every downstream application.

Direct dependency isolation does not remove transitive Candid from management DTOs, Agent, PocketIC,
protocol DTO packages. No Wasm-size reduction has been measured.
Native Linux, native macOS Intel/Apple Silicon, Wasm compilation and installed IC
behavior are separate evidence categories. Current delivered-source CI and
registry qualification are recorded in [the current handoff](status/current.md).
Those producer checks do not qualify consumer-installed callbacks/trap recovery
or establish adoption of every consumer workspace.

## Source provenance

The decoder and tests match the Testkit working tree at HEAD `55ac397`, reviewed
2026-10-18. Original MIT notices are retained. SHA-256 before adaptation:

- `crates/ic-testkit-candid/src/lib.rs`: `93d20eb5c28391f12df274993adbdc2de2ec2480175ff39cafa4b27bbe86996f`
- `crates/ic-testkit-candid/src/tests.rs`: `d69ba1128ee5ccc1d09c9ebbe6697089ff990f1cee18f65100c24f9a30d34b6c`

Working-tree provenance does not claim publication. Shared Tooling owns release
execution; the local adapter remains a thin consumer of its reviewed snapshot.

## Native secp256k1 public derivation

The optional `secp256k1` feature pins `ic-secp256k1 =0.3.0` at the maintainer's
request and explicitly exports `PublicKey`, `MasterPublicKeyId`,
`PocketIcMasterPublicKeyId` and `KeyDecodingError`. These are upstream identities,
including SEC1 encoding, chain codes, parsing errors and derivation methods.
Canic's `ops/config/chain_key` host operation uses mainnet `EcdsaKey1` and
PocketIC `EcdsaKey1`, `EcdsaTestKey1` and `EcdsaDfxTestKey` with Platform's existing
Principal identity. No private-key API or cryptographic implementation is copied.

Select `ic-platform` with `features = ["secp256k1"]` in the native consumer's
inherited dependency, replace `ic_secp256k1` imports with
`ic_platform::secp256k1`, then remove the exhausted upstream declaration from
member/root catalogs and update ownership guards. Preserve network selection,
allowed key IDs, path limits, root validation and existing-value mismatch checks.
Test the consumer's existing derivation and role-rendering behavior after the
paired release; source compilation here does not qualify the whole Canic workspace.

This surface is deliberately native-only, like Agent and the host parser.
Upstream 0.3.0 unconditionally selects Rand/getrandom and exposes no feature to
remove that dependency. Platform does not install a Wasm entropy backend or
substitute an RNG. Enabling this feature on Wasm exposes no module and adds no
secp256k1 dependency. Native selection stays Candid/CDK/executor-free; default
builds do not select it. It centralizes direct dependency ownership, without
claiming transitive dependency removal or measured Wasm savings.

## Qualified dependency-reduction experiments

The following patches are review experiments against exact registry sources,
not dependencies selected by Platform. They retain their upstream Apache-2.0
provenance and do not change the supported type-identity/runtime contracts above.
No new production fork, compatibility package or alternate executor is selected.

The maintainer subsequently excluded replacement of DFINITY-authored cryptographic
code. Retain upstream implementations and identities for derivation, signature
verification, key encoding/hashing and BLS. The public-derivation prototype below
is historical evidence, not an adoption candidate. Dependency/module trimming
may be assessed where it preserves that code; these experiments do not authorize
owned cryptographic replacements.

- `patches/canister-sig-no-store.patch` targets creation 1.3.1: use canonical
  `ic_principal` and omit the unused signature-map module and its `ic0` dependency.
  With verifier 0.3.0 selecting this candidate, all 17 retained creation and 12
  verifier library tests pass on Rust 1.88, as does Wasm checking. Its normal/build
  graph loses Candid and ic0. This does not qualify a production owner migration,
  altered malformed-DER behavior or Auth's installed application policies.
- `patches/secp256k1-public-only.patch` is a standalone public-derivation experiment
  from secp256k1 0.3.0, including its manifest and differential fixture. Rust 1.88
  native tests compare mainnet ECDSA key_1 and three PocketIC ECDSA keys across
  four Principals and four paths (64 pairs), including empty and 255-component
  paths; compressed/uncompressed outputs and chain codes match the original.
  Native and Wasm library checks pass. Its normal graph excludes upstream
  secp256k1, rand, rand_chacha, getrandom, PEM and simple_asn1; rand_core remains
  through curve arithmetic. The prototype omits broader methods/traits and has
  an inherited deprecated generic-array call: it is not public API parity or
  strict-lint qualification, and its owned types cannot silently replace current
  upstream identities. No new curve arithmetic or signing implementation exists.
- The CDK macro-gating proposal is retired by the maintained runtime adoption
  above. Its historical source-copy evidence did not establish a Candid-free
  published runtime; the new owner is qualified separately.

Each patch applies to its exact registry source. Disposable evidence is retained
under `/tmp/platform-signature-split`, `/tmp/platform-public-secp` and
`/tmp/platform-cdk-split`. These package-graph observations do not measure Wasm
size or wall-clock performance. Existing management/ledger DTOs remain selected:
owning their definitions would change Rust identity without removing external
Candid requirements. Public ownership changes require an explicit migration and
renewed consumer qualification; no supplying release is implied by these proofs.

## Scaffolding-only packaging qualification

The earlier CDK macro-gating proposal is superseded by the maintained runtime
above. Its patch and qualification branch are retired; the remaining proposals
below concern cryptographic dependency scaffolding only.

The maintained packaging proposals preserve upstream cryptographic function
bodies, constants and test vectors. They change feature selection, dependency
imports and module exposure only:

- `patches/canister-sig-optional-store.patch` retains the upstream signature map
  under a default-enabled `signature-map` feature. Disabling it removes ic0;
  canonical `ic_principal` replaces the Candid Principal import without changing
  that type. `patches/verifier-optional-store.patch` disables that unused store
  on the verifier's dependency edge; verifier source is unchanged.
- `patches/secp256k1-optional-scaffolding.patch` retains all original source lines
  and adds `full` feature gates. Defaults preserve the upstream API. Disabling
  `full` retains public derivation and SEC1/BIP340 serialization without private
  keys, signature verification, PEM/DER or RNG scaffolding. K256 remains 0.13.4,
  using its supported `arithmetic` and `sha2` features; full mode restores the
  original optimization/format/signature features. This changes inherited method
  availability and needs an explicit compatibility/source migration before adoption.

`make check-upstream-scaffolding` applies these patches to exact cached registry
sources in disposable directories. It compares every source file after removing
only the declared feature attributes and Principal import adjustment, verifies
license bytes, records source hashes and prepares an independent secp vector
workspace. It does not change the production dependency selections or publish
packages. Using the printed evidence directory as `proof_root`, rerun the
separate MSRV builds without unifying the upstream and trimmed crypto features:

```sh
cargo +1.88.0 run --manifest-path "$proof_root/vectors/Cargo.toml" -p vectors-original --frozen --offline > "$proof_root/original.out"
cargo +1.88.0 run --manifest-path "$proof_root/vectors/Cargo.toml" -p vectors-candidate --frozen --offline > "$proof_root/candidate.out"
cmp "$proof_root/original.out" "$proof_root/candidate.out"
```

Production source choice is separate: Cargo ignores dependency-local `[patch]`
tables. The disposable Platform consumer qualification uses an explicit root
patch for the verifier/creation edge; this proves behavior, not transparent
downstream distribution. Registry-source admission remains strict until an
explicitly supported source arrangement and any type-identity migration are
selected and qualified. No supplying release or installed runtime proof is implied.

## Standard Ed25519 maintenance boundary

The optional native/Wasm `ed25519` feature explicitly exports `Signature`,
`SignatureError`, `Signer`, `SigningKey`, `VerifyingKey` and `Scalar` under
`ic_platform::ed25519`. The owners remain ed25519-dalek 2.2 and curve25519-dalek
4.1.3 with defaults disabled. These are compatible registry requirements, not
new exact pins. No crypto implementation, RNG, Candid or executor is added.
Agent can independently bring these dependencies and broader unified features.

Auth can select `ic-platform/ed25519` through its existing portable-signature
feature, replace its two Dalek imports with this module, and remove exhausted
member/workspace declarations. Preserve canonical-S checks, weak-key rejection,
`verify_strict`, message hashing, budget charging and admission policy. Tests
can import `Signer` and `SigningKey` from the same module. Update generators,
locks and direct-dependency guards and rerun consumer acceptance tests.
The subsequent read-only audit found Auth and Canic selecting Platform's Ed25519
surface. Auth deliberately retains a direct Dalek dev dependency in its adversarial
feature-unification fixture; preserve that test. This controls direct ownership, not all transitive
versions, cryptographic policy or measured Wasm size.

## Standard secp256k1 ECDSA maintenance boundary

The optional native/Wasm `ecdsa` feature exposes original K256 0.13.4 identities
as `ic_platform::ecdsa::{Signature, SigningKey, VerifyingKey, SignatureError,
PrehashSigner, PrehashVerifier}`. The compatible registry requirement disables
upstream defaults and selects only `ecdsa`; no RNG backend, Candid or executor
is enabled. This is independent of the existing native-only `secp256k1` IC public
key derivation surface. Agent/public derivation may unify additional K256 features.

Auth's `chain-key-verification` should select `dep:ic-platform` and
`ic-platform/ecdsa` instead of `dep:k256`. Replace K256 imports in production,
tests and qualification tooling with the flat Platform module; select `ecdsa`
in the qualification runner too. Canic's K256 imports are test-only: select
Platform's `ecdsa` in its dev dependency and remove the K256 dev declaration.
Remove workspace catalog declarations only after all members migrate; update
consumer locks and dependency guards after publication. Retain Auth's explicit
high-S refusal, normalization operation, exact prehash bytes, typed errors,
malformed-key-first ordering, trust decisions and key custody.

An isolated copy of Auth's actual chain-key implementation and tests, changing
only its two imports, passes Rust 1.88 tests and Wasm checking against local
Platform. The separate Platform-only consumer covers signing, compressed and
uncompressed keys, malformed/wrong-message rejection and explicit high-S
normalization. These proofs do not establish published consumer adoption,
installed-canister behavior, macOS qualification or smaller Wasm.

## Owned account migration (0.3)

Use `ic_platform::{Account, Subaccount, AccountError}` with `principal` for owned
application identities. `Account` contains the owned `Principal` and
`Option<[u8; 32]>`; it has Serde but no Candid or storage/CBOR trait. Wire owns its
codec integration; this addition does not establish a Wire trait implementation.
No new feature, dependency or cryptographic implementation is added.

For external ledger calls, select the existing `icrc-ledger-types` feature and
convert with `.into()` between owned `Account` and
`ic_platform::account::ExternalAccount`. The latter is exactly the existing
`ic_platform::icrc_ledger_types::Account` registry DTO. Keep external DTO fields
and Candid derives on that type. Conversions preserve both owner bytes and the
original optional subaccount, including `None` versus explicit all-zero bytes.

Equality, ordering and hashing treat absent and zero subaccounts alike; Serde
preserves their distinct fields. Never use serialized-byte equality as account
identity equality. Display omits default subaccounts and retains the upstream
CRC32/Base32 and trimmed-hex text format. Parsing rejects more than 136 input
bytes before processing, and more than 64 subaccount hex digits before decoding.
Errors are owned bounded diagnostics; they do not retain arbitrary input or an
expected-checksum string. Principal parsing uses the owned bounded parser.

Change application imports and generated types deliberately; leave external
ledger DTO imports intact. Remove direct ledger dependencies only where no
remaining external use requires them. Test consumer Wire codecs, map/set identity,
text vectors and existing persisted records before adoption. No automatic storage
migration, account-identifier hashing or principal-to-subaccount mapping is supplied.

## Runtime codes and signature packaging (0.4)

The 0.4 candidate replaces the runtime `cdk::call::RejectCode` with a Platform
identity. Existing variant paths, integer casts, Display spelling and Serde enum
names are preserved. `TryFrom<u64>` retains unknown numbers in
`cdk::reject_code::UnknownRejectCode`; `CallRejected::reject_code()` still returns
its existing `UnrecognizedRejectCode`, and `raw_reject_code()` remains available.
Strum iteration and `From<ic_error_types::ErrorCode>` are not retained. External
Agent and PocketIC enums are unchanged. Compare documented numeric codes when
crossing those boundaries; do not cast an unrecognized number into a known enum.
Consumers retain retry classification, deadlines and best-effort outcome policy.

Signature features now select upstream-authored code packaged inside Platform.
This supersedes the signature portions of the packaging experiments above.
`canister_sig_creation::CanisterSigPublicKey` has a new Platform Rust identity;
its Principal field remains `principal::ExternalPrincipal`. Existing explicit
creation and verifier imports through Platform continue to work. Remove direct
registry creation/verifier declarations only after every consumer uses Platform's
surface, including tests, generated imports and helper signatures. No conversion
between duplicate creation-package identities is supplied. No root Cargo patch,
extra published crate, signature store or alternative cryptographic primitive is
introduced. The coordinated Platform package carries the supported implementation.

Both imported modules retain all upstream function bodies and test vectors.
Only the canonical registry Principal import, the sibling creation import and
omission of the unused store declaration differ. Exact source/license admission
runs with the runtime source check. The approved #72 exception adds only one
checked framing call at parser entry; removing that exact call reconstructs the
original source fingerprints. Domain/DER encoding length conversions remain
upstream; keep Auth's resource admission, canonical DER
re-encoding, issuer/seed, freshness and trust-root checks. BLS, certificates and
hash/domain separation use their existing upstream implementations.

Creation/verifier-only normal/build graphs no longer contain Candid, ic0 or the
two registry signature packages. CDK-only graphs no longer contain ic-error-types
or Strum. Host Agent/PocketIC and other selected interfaces can independently
bring Candid/error catalogues. These are graph observations, not measured Wasm
savings. Native/macOS, installed verification and actual consumer migration remain
separate qualifications; unchanged source does not prove all application policy.

## Transport and codec separation (0.4)

Raw `cdk::call::Call` now only configures and submits transport bytes. The Candid
feature no longer adds `Call::with_arg/with_args` or
`Response::candid/candid_tuple`. This avoids a second codec/error owner inside
the runtime. Awaiting returns `Result<Response, CallFailed>` as before; oneway
submission returns the now-explicit `OnewayError`. The existing
`UnrecognizedRejectCode` is also nameable through `cdk::call`.

For required external Candid calls, migrate deliberately:

| Previous runtime helper | Explicit boundary operation |
| --- | --- |
| `call.with_arg(arg)` | `contract::encode_one(&arg)`, then `call.take_raw_args(bytes)` |
| `call.with_args(&args)` | `contract::encode_args_ref(&args)`, then `call.take_raw_args(bytes)` |
| `response.candid::<T>()` | `decoder::decode_one::<T>(response.as_ref())` to preserve upstream defaults |
| `response.candid_tuple::<T>()` | `decoder::decode_args::<T>(response.as_ref())` to preserve upstream defaults |

Here `contract` and `decoder` are under `ic_platform::candid`. Existing callers
using finite limits continue using the parent module's bounded helpers, including
`decode_exactly_one` where exact arity is their policy. Do not silently replace
ordinary decoding with exact decoding, impose new budgets, change checkpoint
bytes or select a retry on codec failure. Encoding now exposes its ordinary typed
Result explicitly; callers decide whether to propagate failure or retain their
previous trap behavior. No automatic encoding panic is installed by the runtime.
Where a direct builder caller must preserve the former trap and diagnostic, use
`unwrap_or_else(|err| panic!("failed to encode args: {err}"))` on encoding. Canic's
existing fallible request builder already has its own error policy; retain it.

Remove imports and conversions for retired `cdk::call::{Error, CandidDecodeFailed}`.
Keep transport and codec errors distinct in the consumer's existing error enum.
The explicit decoder retains the upstream typed Candid cause rather than reducing
it to the old type-name/error strings. Preserve diagnostics disclosure policy.
No compatibility alias or second transport/codec union replaces those types.

The read-only Canic review found the affected layer at
`crates/canic-core/src/infra/ic/{call.rs,mod.rs}`: its request builder already uses
explicit encoding. Update response helpers and remove the obsolete combined-error
mapping. Agent's separate `with_arg` methods and external host flows are unaffected;
do not apply an unqualified global method-name replacement. Update consumer tests
and generators only where they emit these retired Platform call APIs, then run
external management/ledger request-byte, response-arity, error and retry tests.
Manifests continue selecting `cdk` and explicit `candid`; no new feature is needed.
Legacy `cdk::storage` Candid checkpoints are untouched and remain a compatibility
exception, not a new storage format.

## Canic external-contract closeout exports

Published Platform 0.6.21 supplies the narrow export patch prepared
by Canic's owner for its remaining external-contract mirrors. Published 0.6.20
already supplies CMC, ICP, Cycles Ledger, Registry and asset DTOs; these extra
management/ICRC identities are not present in that immutable supplying archive.
The 0.6.21 archive is non-yanked and verified against SHA-256
`f82b7556c1a6acf55a82b0bdc789fc6d4cb4f9c3e0113e911354817387b866e0`,
with VCS/default-main identity `43928b07c9d04a63b6a77f90b185a5c4afda89fa`.
Reviewed patch: Canic `b9482ec52`,
`docs/audits/working/0.112-push-readiness/platform-external-contracts.patch`,
SHA-256 `4ecfff50e4b450e51f4648406b99c89d1b48bb9e3615add239cc4597ed09c890`.
Only its reviewed export additions were adopted; obsolete release/supply prose
was replaced with the current versions and consumer obligations below.

Keep selecting `management-canister-types` and import the exact pinned 0.11.0
identities from `ic_platform::management_canister_types`:

- Lifecycle/settings: `CanisterSettings`, `LogVisibility`, `EnvironmentVariable`,
  `UpdateSettingsArgs`, `UploadChunkArgs`, `InstallChunkedCodeArgs`,
  `CanisterInstallMode` and `UninstallCodeArgs`.
- History: `Change`, `ChangeOrigin`, `FromUserRecord`, `FromCanisterRecord`,
  `CreationRecord`, `CodeDeploymentRecord` and `ControllersChangeRecord`.
- External ECDSA DTOs: `EcdsaCurve`, `EcdsaKeyId`, `EcdsaPublicKeyArgs`,
  `EcdsaPublicKeyResult`, `SignWithEcdsaArgs` and `SignWithEcdsaResult`.

These are types, not new calls, cryptographic implementations or execution
permission. The uninstall request with sender version maps to `UninstallCodeArgs`,
not `CanisterIdRecord`. Full settings/install types include optional fields and
variant payloads omitted by Canic's subset: do not assume identical Candid type
maps or request-cost quotations after adopting them. Encode/quote new operations
from the same canonical request and preserve original retained bytes when retrying
an uncertain operation. Keep controller, target and sender-version checks with
Canic. No call to the management canister is performed by this addition.

Keep selecting `icrc-ledger-types`; import `TransferArg` and `Memo` from
`ic_platform::icrc_ledger_types`. Use the explicit nested `icrc21` namespace for
`ConsentMessageMetadata`, `ConsentMessageRequest`, `ConsentMessageSpec`,
`DisplayMessageType`, `ConsentInfo`, `ConsentMessage`, `FieldsDisplay`, `Value`,
`ErrorInfo` and `Icrc21Error`. The consent `Value` identity is distinct from the
existing root ledger-generic `Value`; preserve that separation. Use upstream
`Memo::from(existing_bytes)` for retained raw memo bytes rather than introducing
a conversion bridge. Do not reinterpret those bytes through the integer
constructor or normalize them during an uncertain retry.

No feature, default, package dependency or upstream pin changes. There is no
blanket re-export or compatibility alias. Upstream Rust identities, derives and
external Candid encoding remain at the same registry suppliers. Canic retains
consent generation/admission, authorization, spending, reply bounds, journals,
retry/reconciliation, and native/installed acceptance. Require at least 0.6.21
before replacing these mirrors; supplier publication does not establish Canic
adoption or authorize rewriting any retained uncertain request bytes.

Qualification extends the existing standalone `management` and `icrc` consumer
profiles, with native/MSRV execution and Wasm compilation. Package-local identity
checks separately require exact registry type identity, including the two distinct
`Value` types. These checks do not execute cryptography or qualify payment effects.

## Prepared one-shot calls (issue #75)

The upcoming 0.10 line moves prepared calls from the separate `ic-call` package
into `ic_platform::call`, enabled by the existing `runtime` feature (also selected
by `cdk`). Remove the `ic-call` dependency and replace `ic_call::` imports with
`ic_platform::call::`. No compatibility crate is retained. The module owns generic
request custody and delegates every physical submission to Platform’s existing
raw runtime. No dependency or feature is added. No callback
body, executor, codec, timer, Relay dispatch or cryptographic implementation is
copied. Existing raw Platform callers remain supported.

Canic #700 demonstrated the gap against published Platform 0.7.1: a raw call
cannot be returned after dropping its dynamically selected method String.
Use the module to retire the local generic request/execute owner:

```rust
use ic_platform::call::{Call, Principal, WaitPolicy};

fn prepare(target: Principal) -> Call<'static, 'static> {
    let method = String::from("runtime-selected-method");
    Call::new(target, method, WaitPolicy::Bounded { timeout_seconds: 300 })
        .take_raw_args(vec![1, 2, 3])
}
```

Select `ic-platform` with defaults disabled and `features = ["runtime"]` in the
workspace production catalog and inherit it from the consumer. The module needs
the supplying 0.10 release before registry adoption; current package versions
remain 0.9.1 until the maintainer releases. Remove the local generic builder/execution wrapper,
replace its repeated attachment-plus-cost formula with `get_cost()`, and call
`execute().await`. Keep domain facades and stage-specific metrics with their
consumer; observing construction is different from observing actual submission.

There is no implicit codec or wait choice. Preserve each call site's existing
bounded timeout or unbounded policy explicitly. `String` owns the method;
`&str` borrows it. `take_raw_args(Vec<u8>)` owns bytes, and `with_raw_args(&[u8])`
borrows them independently. Last attachment/wait selection replaces the earlier
one. Empty raw bytes are not Candid unit bytes. Encode Wire frames for owned
endpoints; encode required external Candid separately through Platform before
constructing a request. Decode `Response::into_bytes()` with the selected codec
and consumer-owned budgets/arity/admission after handling transport failure.
No direct Candid/CDK dependency or generator change is needed for this API.
A codec failure after a reply does not prove no remote effect.

`CallFailed`, submission/balance errors, raw rejection codes and `Response` retain
Platform identities. `SysUnknown`, a logical timeout, future drop or a callback
trap does not establish physical settlement. No retry, funding, alternate route,
refund authority or universal codec/transport error is added. Capture refund facts
only in the matching reply/reject continuation, before another await; the new
execution wrapper adds no await after raw completion. Relay retains journals,
ACKs, retries and durable delivery; Timers retains its private dispatch machinery.

The read-only consumer inventory found no additional execution gap in Blob #108,
Query #108, IC Crates #19, Jobs #75 or Relay #14: their static/scoped method names
already fit raw calls. Blob's four adapters retain provider codecs, reply limits,
actor checks and financial/continuity policy. Query retains governance DTO/limit
policy; IC Crates retains attestation interpretation; Jobs and Relay retain
delivery policy. Timers #80 remains an infrastructure qualification. Codec
consolidation is separately tracked by Platform #78/#79; this does not restore
implicit Candid or retire consumer policy. Producer completion is independent
of later consumer adoption.

`make check-consumer` includes a standalone crate exercising raw calls with only
Platform `runtime`, then explicit Wire and external-Candid selections on the development
compiler and Rust 1.91, native and Wasm. `bash scripts/ci/check-platform-runtime.sh
--prepared-call` qualifies owned dynamic methods alongside the raw control in the existing installed lifecycle
fixture. Both modes use the same maintained lock; the prepared-call selection
adds no package or dependency edge. The original raw/published
fixture remains independently selectable. Local Linux/PocketIC evidence and
native macOS qualification must be reported separately. Neither graph isolation
nor this wrapper alone proves smaller application Wasm. Candid's unresolved Wasm
depth limitation (#73) remains for unguarded low-level upstream APIs.

The next paired native cleanup adds Profiler-owned query/cache/tool support under
`server/system_profile_product/`. Keep provider and consumer source pins paired;
older consumers retain their previous provider. HTTP/MCP admission, errors,
response fields, immutable query ownership and budgets stay unchanged. The
Platform gzip facade and native start/stop, IC metrics and pricing adapters stay
at their existing owner. Published Rust consumers need no import changes.
