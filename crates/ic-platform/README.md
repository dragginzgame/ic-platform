# ic-platform

Platform re-exports canonical Principal/Account identities from ic-platform-types.
Platform (facade), PocketIC (native client), Runtime (runtime, macros and exact
guard), and Types have independent release groups in one workspace. Each group
owns explicit versions and its canonical releases/ ledger. Unrelated runtime releases retain the
same compatible type supplier; see [the migration contract](../../docs/architecture/migration.md#stable-owned-identity-package-118).

Platform 0.11 removed `wire-schema` and the production Wire dependency. Principal
and Account retain their existing bytes and Serde; published Wire 0.10 supplies
their schema trait integration through opt-in `platform-types`. Host schema
consumers select that feature alongside Platform `principal`.
Generic gzip codecs are selected directly from IC Tooling.

Select `certification-cbor` for typed certificate/signature codecs and the
existing strict certificate/tree admission contract. Native `ic-protocol`
supplies bounded IC packet codecs without Agent or Runtime; select
`ic-protocol-fixtures` separately in native development dependencies. No generic
CBOR value/Serde encoder is exposed. Consumers retain byte budgets, verification,
freshness, request bindings and effect authority. See the
[packet migration contract](../../docs/architecture/migration.md#typed-ic-packet-and-fixture-boundary-pending-0223).

Required external Candid endpoints select `candid-codec-endpoint` and
`candid_codec::endpoint::{query, update}` with explicit decode limits, reply bytes,
arity and empty-envelope policy. Ordered guards and the existing generator/executor
are retained. The legacy endpoint feature is removed; native service registration
is not supplied by the owned adapter. Raw/Wire endpoints retain their codecs.

Select the required DTO features and use `candid_codec` explicitly. Method names,
transport, reply arity, budgets and retries remain caller-owned.

Use `ic-host-artifacts` with its `gzip` feature for native bounded gzip codecs.
Platform has no compression feature or Tooling production dependency.

Published 0.6.20 provides optional `cmc-types`,
`icp-ledger-types`, `cycles-ledger-types`, `nns-registry-types` and native-only
`certified-assets-types` surfaces. These retain external Candid and add no service
engine or policy. See the [exact contract and migration guide](../../docs/protocols/external-contracts.md).
Consumer adoption and qualification remain separate from supplier publication.

Published 0.6.21 extends `management-canister-types` with
explicit lifecycle/settings/ECDSA/history identities and `icrc-ledger-types` with
transfer/memo and `icrc21` consent DTOs. They keep the existing pins and feature
selections; they add no calls, signing implementation or policy. Require at least
0.6.21 and qualify the consumer before replacing those remaining mirrors.

Platform 0.18 removes the unused `stable-structures` feature and module,
including their upstream dependency. Storage and allocation remain consumer-owned;
Runtime's raw stable-memory system calls remain available. See the
[retirement contract](../../docs/architecture/migration.md#stable-structures-substrate).

Select `principal` for owned Principal/Account bytes, text and Serde without
external Principal or hashing. The owned codec, maintained DTOs and callbacks use
this same identity. The external aliases and `principal-external` feature are
removed; `principal-self-authenticating` separately adds unchanged upstream
self-authentication. Native supplier APIs retain their registry identities. See
the [hard cut](../../docs/architecture/migration.md#owned-principal-hard-cut-unreleased).

Select `runtime` for prepared one-shot requests through
`ic_platform::call::{Call, WaitPolicy}`. Methods and argument bytes can be owned
or borrowed independently; execution delegates to the existing raw transport.
See the [prepared-call contract](../ic-platform-runtime/src/call/README.md). This replaced the separate
`ic-call` package in 0.10.0.

For raw calls and system/task adapters, select `runtime` and keep the existing
`ic_platform::cdk::{api, call, futures, stable}` imports. When the
package owns entrypoints, depend directly on `ic-platform-runtime` and select
its `cdk` feature. Both use the same executor and remain Candid-free.

Select `raw-rand` for `ic_platform::raw_rand::{METHOD, encode, decode}` when a
consumer needs only management entropy. The codec adds no dependency or runtime:
`encode()` returns the static empty Candid tuple and `decode(bytes)` returns
`Result<[u8; 32], DecodeError>` under its documented bounded producer profile.
Select raw execution separately. Callers retain seed admission, including any
all-zero refusal, and wait/retry/cancellation policy. See the
[entropy contract](../../docs/architecture/migration.md#candid-free-management-entropy-platform-027-unreleased).
The former `candid::protocols::management::raw_rand` helper is removed in 0.27;
this module is the sole typed entropy codec. Generic raw transports remain
method-agnostic.

Select `candid-codec` for the owned typed binary codec in `candid_codec`, without
registry Candid or its parser. It supports explicit argument sequences, bounded
decoding, record/payload-variant mappings, Principal and arbitrary integers.
Registry, CMC, ICP transfer/archive, Cycles creation/withdrawal and native asset DTOs use it directly,
preserving external Principal identity and wire bytes. Migrated Nat fields use
`candid_codec::Nat` with the established unbounded range and Serde representation;
registry Nat callers need an explicit conversion. ICP references validate their
query signatures without upstream Candid; destination and method admission stay
consumer-owned. Other adapters are not automatically migrated; see the
[migration contract](https://github.com/dragginzgame/ic-platform/blob/main/docs/architecture/migration.md#owned-binary-codec-unreleased)
for supported mappings and distinct resource-budget semantics.

Select `candid-args` for the small dependency-free argument encoders:
`candid_args::empty()`, `candid_args::units(count)` and `candid_args::text(value)`.
These create empty arguments, unit/null arguments and one UTF-8 text argument.
The `raw-rand` and `candid-codec-endpoint` features select this shared framing automatically.
Other supported binary schemas use `candid-codec`.

All owned application endpoints use IC Wire, including public endpoints called by
third parties. Select Runtime’s `cdk` feature for explicit-codec entrypoints,
or the facade’s `executor` feature for the shared executor alone. Use the owned codec only for
required management/system, NNS/SNS, ledger and other Candid interfaces, or legacy
persisted checkpoints. Caller location does not select Candid.
Select `principal` for Principal, Account and Subaccount. Runtime, Candid,
signature creation and external DTO/client/parser profiles include it.
No features are enabled by default; executor/IC0-only builds omit application
types and their Serde, text and cryptographic dependencies.

Enable `icrc-ledger-types` for `ic_platform::icrc_ledger_types`, maintained from
selected upstream 0.2.0 sources without the registry ledger package. The explicit flat surface is `Account`,
`MetadataValue`, `Value`, `ICRC3Value`, `TransferError`, `AllowanceArgs`, `Allowance`,
`GetArchivesArgs`, `ICRC3ArchiveInfo`, `QueryArchiveFn`, `ArchivedBlocks`,
`BlockWithId`, `GetBlocksRequest`, `GetBlocksResult`, `ICRC3DataCertificate` and
`SupportedBlockType`. These have Platform identities and preserve external Candid formats,
account/text semantics and value hashes. Use the owned codec for encoding/decoding. ICRC-10 remains a passive external
discovery record; no endpoint or standards list is automatic. Native
and Wasm are supported, without selecting an executor, parser, Agent or storage
implementation. Consumers retain amount/fee policy, byte/work limits, ledger and
archive admission, certificate verification, interpretation, deadlines and retries.
Hashing a block or decoding certificate bytes does not authenticate either.

Enable `management-canister-types` for `ic_platform::management_canister_types`:
upstream `HttpRequestArgs`, `HttpRequestResult`, `HttpMethod`, `HttpHeader`,
`TransformContext`, `TransformFunc` and `TransformArgs` from exactly 0.11.0.
The same feature explicitly exports Blob's history types: `CanisterInfoArgs`,
`CanisterInfoResult`, `ChangeDetails` and `CodeDeploymentMode`.
Backup's snapshot and status contract also exports `CanisterIdRecord`,
`CanisterStatusResult`, `CanisterStatusType`, `CanisterTimer`, `ChunkHash`,
`DefiniteCanisterSettings`, `LoadCanisterSnapshotArgs`, `MemoryMetrics`,
`OnLowWasmMemoryHookStatus`, `QueryStats`, `ReadCanisterSnapshotDataArgs`,
`ReadCanisterSnapshotDataResult`, `ReadCanisterSnapshotMetadataArgs`,
`ReadCanisterSnapshotMetadataResult`, `Snapshot`, `SnapshotDataKind`,
`SnapshotDataOffset`, `SnapshotMetadataGlobal`, `SnapshotSource`, `StatusVisibility`,
`TakeCanisterSnapshotArgs`, `UploadCanisterSnapshotDataArgs`,
`UploadCanisterSnapshotMetadataArgs` and `UploadCanisterSnapshotMetadataResult`.
Canic's snapshot-delete request additionally uses the same feature's explicit
`DeleteCanisterSnapshotArgs` export, with the original canister/snapshot fields.
Native and Wasm targets use independently authored Platform DTOs and the owned
codec: 62 native exports and 44 shared Wasm exports. Native-only snapshot
transfer, install/create and log additions stay absent from Wasm. All 52 shared
schemas, nested fields, traits and external bytes remain compatible with the
registry 0.11.0 development oracle. See the [management migration](../../docs/architecture/migration.md#selected-canister-management-dtos)
and [shared qualification](../../docs/design/management-shared-qualification.md).
Neither production management graph selects upstream Candid, the registry DTO
crate, executor, Agent or parser.
Consumers retain HTTP bounds, cycle pricing, transforms, call execution and retry
policy. History consumers retain bounded calls, reply/work limits, change counts,
instance-version checks and recovery fences. Decoding history does not establish
continuity. Other management DTOs and call helpers are outside this explicit surface.
Snapshot/status consumers retain target IDs, exact arity, controller/global/chunk
bounds, reserved-marker refusal, journal evidence, finite spending and restore
safety. These DTOs add no snapshot execution or storage/allocation owner. Upstream
`Reserved` source payloads can consume skipping work: preserve zero-quota refusals
and existing bounded projections instead of increasing quotas to decode a full DTO.

Enable `agent` on native hosts for `ic_platform::agent`: upstream `Agent`,
`AgentBuilder`, `AgentError`, `Identity`, `BasicIdentity`, `Secp256k1Identity`,
`Prime256v1Identity`, `TransportError`, `HttpErrorPayload`, `Envelope`,
`EnvelopeContent`, `SignedUpdate`, `RequestId`, `RequestStatusResponse`,
`RejectCode`, `RejectResponse`, `ReplyResponse`, `lookup_value`, `to_request_id`
and `signed_update_inspect`, with PEM support. The module also exposes
`AnonymousIdentity`, `DelegatedIdentity`, `Delegation`,
`SignedDelegation`, `CallResponse` and `TransportCallResponse`. These retain
upstream signed-chain validation and immediate/pending/rejected response formats.
Unchecked delegation construction does not authenticate a chain or enforce
expiry/target policy. Pending responses do not authorize polling or resubmission.
The module and Agent dependency are absent on Wasm, including all-feature builds.
No other Platform feature
enables Agent, and Agent does not enable other Platform features. Its upstream
Candid/certification dependencies remain transitive. Consumers own runtime,
endpoint admission, TLS/client configuration, identity/root-key selection,
query verification, byte limits, expiry, retries and durable effect policy.
Use `certification` for certificate/hash-tree types, including
`SubtreeLookupResult` and `labeled_hash`. Certificate lookup does not authenticate;
signed-update inspection compares envelope fields without verifying signatures
or authorizing replay. Existing Agent internal HTTP retries remain upstream.

Use `candid_codec` for binary external Candid with explicit decode and reply budgets.
The legacy decoder, trait bridge, parser, printer and protocol-forwarding modules
are removed. Upstream Candid remains a dev-only compatibility oracle.

Enable `certification` for `ic_platform::certification`: upstream `Certificate`,
`Delegation`, `HashTree`, `Label`, `LookupResult`, `empty`, `fork`, `labeled`,
`leaf`, `pruned`, `fork_hash`, `AsHashTree`, `RbTree` and `Hash`, with upstream Serde support and registry type identities. This feature is
independent of Candid and the executor. Host retains its strict bounded CBOR
decoder; consumers own trust roots, authentication, budgets and leaf meanings.
Neither construction nor deserialization authenticates a certificate. This
centralizes dependency control without removing upstream code or promising
smaller Wasm artifacts.

Enable `canister-sig-creation` for `ic_platform::canister_sig_creation`:
`CanisterSigPublicKey`, `hash_bytes`, `hash_with_domain`,
`extract_raw_root_pk_from_der`, `IC_ROOT_PK_DER` and `IC_ROOT_PK_DER_PREFIX`
are independently implemented protocol APIs using canonical Platform Principal.
DER/raw bytes and public key constants retain their compatibility contract;
SHA-256 remains an external implementation. Strict short-form DER admission
remains in place. There is no signature store or automatic trust-root selection.

Enable `signature-verification` for
`ic_platform::signature_verification::verify_canister_sig`, with the same four
byte-slice arguments and Result<(), String>. Platform owns certificate/delegation,
range and witness checks; external packages own BLS, curve operations and tree
hashing. Identity keys are rejected at both the root and delegated subnet.
`verify_canister_sig_with_time` extracts the unique authenticated signing time
without decoding the signing certificate again. Delegation time is never returned.
Consumers retain byte/depth limits, roots, freshness and authority.

Both production signature profiles exclude Candid, registry Principal, the old
signature packages, raw IC0 and an executor. DFINITY creation/verification
packages are dev-only byte/behavior/benchmark oracles. See the
[implementation contract](../../docs/design/canister-signatures/contract.md) and
[comparison report](../../docs/benchmarks/canister-signatures-2026-11-10.md).

Enable `bls` independently for `ic_platform::bls::{PublicKey, Signature,
PrivateKey, InvalidPublicKey, InvalidSignature, InvalidPrivateKey,
verify_bls_signature}` from upstream 0.6.0. This retains typed deserialization
errors, serialized bytes and verification results. Upstream defaults are off
and alloc is on, matching the IC verifier and Auth fixtures. Deterministic keys
can be deserialized from caller-supplied bytes and used for signing; no RNG feature
or entropy source is selected. Key generation/custody and domain/trust policy
stay with callers. BLS alone brings neither Candid, creation, certification,
CDK/executor nor host tooling. Native and Wasm are supported by all three features.

`executor` re-exports Runtime-owned task/context types and operations from the
single maintained task registry. Callers must preserve context placement, callbacks
and trap cleanup; native compilation does not qualify installed IC behavior.
The implementation is maintained from ic-cdk-executor 2.0.0, with protected-query
wake routing, wake coalescing, cached wakers and task-bookkeeping reclamation.
The Cargo singleton guard rejects old registry executors and incompatible Runtime
owners; coordinate Timers/application adoption of the same Runtime identity.
The internal, dependency-free `ic-platform-runtime-guard` package is selected
only by `executor` and exactly matches the Runtime version. Native default and
IC0-only consumers select no guard and can coexist with unrelated executors.
The executor implementation lives in `ic-platform-runtime`; the guard exports no API.

Enable `ic0` for `ic_platform::ic0`: the explicit IC 1.2.0 calls used by Timers
for time/version/counters/stable page count, one timer, self/caller/argument
copying, replies/reject codes, cycles/costs and call submission/cleanup, plus
Testkit's `debug_print(&[u8])`. Debug output retains upstream byte/trap behavior;
marker formatting, labels and byte limits belong to the caller. The embedded
1.2.0 implementation preserves upstream safe/unsafe callback signatures and traps.
It selects neither Candid nor an executor and installs no exports, timer or storage
owner. Timers must select both `executor` and `ic0`; changing either qualified
identity requires renewed combined callback/trap/cancellation/upgrade evidence.

The same feature explicitly exports `in_replicated_execution() -> u32`,
`is_controller(&[u8]) -> u32`, `msg_reply_data_append(&[u8])` and
`msg_reject(&[u8])`. These are the upstream safe signatures and integer results.
Pass existing Principal bytes to the controller operation; append preserves
slice order and still requires `msg_reply()` to finalize. Keep rejection bytes,
early size refusal and authentication ordering unchanged. These exports do not
replace CDK's Principal/Vec convenience helpers, macros or call/task lifecycle.

Select `ic-platform-runtime/cdk` for explicit-codec entrypoints, raw calls,
system helpers, protected spawning and raw stable-byte operations. Use
`ic_platform_runtime::cdk` for these imports; the facade has no ordinary
entrypoint macros or `cdk` feature. Its `runtime` feature retains raw adapters. Full `ic-cdk` and upstream macros are not dependencies. The embedded
executor and IC0 remain shared with Timers; neither registry package is required.

All controlled endpoints use Wire. Query/update macros require `decode_with`
(including unit RPCs) and `encode_with` or `manual_reply`. Use a guard for
pre-copy byte admission. Caller code retains endpoint IDs, decoder budgets,
authentication and retry policy. No Candid endpoint fallback, service-registration
macro or Candid path option is supplied. Lifecycle hooks with arguments require
explicit decoding; no-argument hooks may omit it. A renamed dependency can use
`crate = "runtime::cdk"`. Remove the retired no-op `hidden` option.
`Call::change_timeout(seconds)` is retired; use
`Call::with_wait_policy(call::WaitPolicy::Bounded { timeout_seconds: seconds })`.
`WaitPolicy::Unbounded` explicitly selects unbounded waiting. Neither choice
configures retries; bounded constructors still default to 300 seconds.

Select Runtime’s `runtime` or `cdk` feature and the facade’s `candid-codec` feature
for required external management/system-canister calls. Encode with the owned
`encode_one` or `arguments::encode_args` and an explicit reply-byte budget,
then supply `with_raw_args` or `take_raw_args`. Decode `Response::as_ref()` through
the separate Candid decoder, choosing existing admission/arity policy explicitly.
Raw constructors start with empty transport bytes; Wire callers always supply
a request frame. External no-argument Candid calls encode `()` explicitly.
`CallFailed` and `OnewayError` describe transport failures only. Bounded calls retain
their 300-second default and original cycle/timeout/error behavior.

Call/response/future, cost/performance/error and stable-error types now have
Platform identities; upstream direct-type equality is not retained. Principal
and runtime RejectCode are owned too. Management and selected ledger DTOs use
Platform identities and canonical Types Principal; BLS and certification types
retain their explicit upstream identities. The 0.4 release changes signature-key identity as
documented above.
The generator and Runtime adapters now use specification-only implementations.
Retained ledger value/hash and ICP derivation keep their source records. No executor or
cryptographic implementation is forked.

Published Testkit 0.54.0 call helpers use Platform's canonical limits/error types.
Upgrade and qualify actual drivers before removing the old decoder declaration.
Consumers still selecting Testkit 0.52/0.53 need its distinct limits/error types.
Those native tests may retain `ic-testkit-candid` solely for that older API until
their published Testkit upgrade is qualified
([producer coordination](https://github.com/dragginzgame/ic-platform/issues/2)).
Do not add conversion/re-export packages or replacement simulator transport.
Selected upstream dependencies remain transitive dependencies.

Runtime’s `runtime` feature includes explicit typed call failures, signing/HTTP/creation
costs, balances, certificate publication, counters and `println!`, plus the
reviewed `stable::{stable_grow, stable_read, stable_write, StableMemoryError}`
passthroughs. The 0.5 hard cut retires unused `cdk::storage` checkpoint helpers.
Consumers own existing checkpoint bytes, layouts, limits, retries and certification
policy. Use `api::global_timer_set` (the upstream 0.20.3 name) only under the
consumer's timer ownership, and `api::msg_method_name` in `inspect_message`.
The base runtime removes full CDK and Candid from its normal/build graph.
There is no replacement checkpoint implementation or automatic data migration.

The maintained PocketIC client moved to `ic_testkit::pocket_ic`. Select Testkit
directly for its client, DTOs, errors and lifecycle helpers; Platform provides no
client feature or forwarding module. See the [migration](../../docs/history/pocket-ic-client.md).

Enable `ed25519` for native/Wasm `ic_platform::ed25519::{Signature,
SignatureError, Signer, SigningKey, VerifyingKey, Scalar}`. These are original
Dalek identities with upstream defaults disabled; no RNG is selected. Consumers
retain strict verification, canonical-scalar checks, message construction and key
custody. This feature is independent of Candid, executor and host features.

Enable `ecdsa` for native/Wasm secp256k1 ECDSA through
`ic_platform::ecdsa::{Signature, SigningKey, VerifyingKey, SignatureError,
PrehashSigner, PrehashVerifier}`. K256 retains type identities and cryptography;
consumers retain prehash construction, high-S policy, normalization and custody.
No RNG backend is selected. Offline IC public-key derivation is not provided.

The `icrc-ledger-types` selection now uses the owned binary codec for ledger,
archive and consent DTOs, including recursive callbacks. The legacy trait/protocol bridge is removed. Nat/Int fields use the owned unbounded
carriers and canonical Types Principal; protocol/Serde bytes and imported hash
bodies remain unchanged. CMC/ICP/Cycles/assets passive declarations and their
selected DID fixtures now use independent implementations; see
[external contract qualification](../../docs/design/external-dtos-qualification.md).

Management DTOs on all targets use the owned codec, including typed HTTP transform
callbacks and budgeted `reserved` skipping. Principal, Nat and Reserved use owned
carriers. Native snapshot transfer declarations retain their existing scope and
external bytes. The registry package is a dev-only oracle; native callers migrate
from upstream Candid traits to explicit owned codec budgets. Cryptographic operations,
transport, callback authority and application policy remain with their owners.

`candid-codec-endpoint` exposes `candid_codec::endpoint::{query, update}` through
the same runtime generator. Supply `limits`, `reply_bytes`, `arity` and `empty_args`
explicitly. Owned decoder budgets are not upstream Candid quotas: callers must
select them anew. The legacy endpoint adapter is removed, including its implicit native service
registration; keep checked external schemas separately.
