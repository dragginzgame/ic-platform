# ic-platform

Pending 0.11 removes `wire-schema` and the production Wire dependency. Principal
and Account retain their existing bytes and Serde; their schema trait integration
is moving to Wire’s planned opt-in `platform-types` feature. Host schema consumers
need the coordinated supplying Wire release; no compatibility feature is retained.
IC Tooling 0.71 remains the gzip supplier.

Source release 0.7.4 adds `candid-endpoint` and `candid::endpoint::{query, update}` for
required external Candid interfaces with explicit `limits`, `empty_args` and
ordered caller guards. It reuses the existing generator/executor; select
`candid-service` separately for native DID printing. Raw/Wire endpoints retain
their explicit codecs. `candid::arguments` also supplies pure argument admission.

Select `candid` and the required DTO features for transport-free
`candid::protocols` operations: explicit method, request encoding and bounded
reply decoding with caller-selected arity. Cycles provider block codecs require
both `cycles-ledger-types` and `icrc-ledger-types` to preserve legacy `Nat64`.
Verify registry availability before downstream adoption; see the
[migration and qualification guide](../../docs/migration.md#external-endpoint-and-protocol-codecs-78-79).
Bounded external codecs automatically reject more than 16 constructed wire levels
before Candid runs (see #73).

Enable `gzip` for native `ic_platform::gzip::{encode_gzip, decode_gzip, GzipError}`.
These are IC Tooling's existing bounded codecs, without backend types or a second
implementation. Consumers retain compression levels, byte limits and artifact
admission. The optional dependency is absent from Wasm and default graphs.

Published 0.6.20 provides optional `cmc-types`,
`icp-ledger-types`, `cycles-ledger-types`, `nns-registry-types` and native-only
`certified-assets-types` surfaces. These retain external Candid and add no service
engine or policy. See the [exact contract and migration guide](../../docs/external-contracts.md).
Consumer adoption and qualification remain separate from supplier publication.

Published 0.6.21 extends `management-canister-types` with
explicit lifecycle/settings/ECDSA/history identities and `icrc-ledger-types` with
transfer/memo and `icrc21` consent DTOs. They keep the existing pins and feature
selections; they add no calls, signing implementation or policy. Require at least
0.6.21 and qualify the consumer before replacing those remaining mirrors.

Enable `stable-structures` for explicit upstream 0.7.2 collection, Memory,
Storable/codec, iterator, restricted, vector-backed and memory-manager identities. Storage owns
governed constructors; Memory owns allocation; Testkit owns instrumentation.
This optional capability adds no Candid or executor and implements no storage.
See the [substrate contract](../../docs/migration.md#stable-structures-substrate).

Select `principal` for owned Principal/Account bytes, text and Serde without
external Principal or hashing. `principal-external` adds registry identity and
byte-copy `.into()` conversions; `principal-self-authenticating` separately adds
unchanged upstream self-authentication. External protocol/client features select
conversions automatically. The selected ledger DTOs now have Platform identities
and omit the upstream package, storage/CBOR adapters and consent formatting. See
the [packaging migration](../../docs/migration.md#minimal-principal-and-selected-icrc-packaging).

Select `runtime` for prepared one-shot requests through
`ic_platform::call::{Call, WaitPolicy}`. Methods and argument bytes can be owned
or borrowed independently; execution delegates to the existing raw transport.
See the [prepared-call contract](src/call/README.md). This replaced the separate
`ic-call` package in 0.10.0.

For raw calls and system/task adapters, select `runtime` and keep the existing
`ic_platform::cdk::{api, call, futures, stable}` imports. Select `cdk` when the
package also owns generated entrypoints; it includes `runtime`. Both use the
same executor and remain Candid-free. This avoids the Platform entrypoint
generator in runtime-only graphs, not every transitive proc-macro dependency.

Enable `secp256k1` for native offline public-key derivation through
`ic_platform::secp256k1::{PublicKey, MasterPublicKeyId, PocketIcMasterPublicKeyId,
KeyDecodingError}`. The dependency is exactly `ic-secp256k1 =0.3.0`; types and
derivation bytes remain upstream. This feature is independent of Candid/CDK and
excluded from Wasm because upstream unconditionally selects native RNG support.
Consumers retain network/key choice, path validation and authorization policy.

All owned application endpoints use IC Wire, including public endpoints called by
third parties. Enable `cdk` for explicit-codec entrypoints and the Candid-free
runtime, or `executor` for the shared executor alone. Enable `candid` only for
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
account/text semantics and value hashes. Select `candid` separately for Platform
encoding/decoding. Pending 0.7.5 also supplies
`icrc_ledger_types::icrc10::SupportedStandard { name, url }`, Platform's passive
external discovery record. Its Candid derive uses a direct dependency on the same
Candid already selected transitively by the ledger crate; it does not select the
Platform `candid` codec feature. No endpoint or standards list is automatic. Native
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
Native targets retain this registry 0.11.0 surface and its type identities.
Wasm targets use 43 Platform-owned exports selected from sibling canister use;
the 14 host-only snapshot-transfer exports are absent. Complete nested types and
external Candid bytes are retained. See the [management migration](../../docs/migration.md#selected-canister-management-dtos)
and `src/management_canister_types/maintained/provenance.json` for the exact lists.
The Wasm graph excludes the registry DTO crate and implicit Candid printer.
This feature brings Candid; select Platform's `candid`
feature separately for encoders/decoders. It selects no executor, Agent or parser.
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

`ic_platform::candid::{DecodeLimits, DecodeError, decode_one, decode_args,
decode_exactly_one}` preserves Testkit's explicit limits and typed failures.
`decode_one` and `decode_args` allow bounded skipping of additional arguments;
`decode_exactly_one` requires one actual wire argument. All select compact Candid
diagnostics, which are not a redaction guarantee. Callers retain raw bytes,
aggregate accounting, trust, resource policy and domain errors.

The bounded helpers, protocol decoders and external endpoint adapters reject more
than **16 constructed wire levels** automatically. No extra limit object or macro
option is needed. The check includes unused type entries and skipped values; large
byte blobs remain valid. Existing decoding/skipping quotas also cap admission
work, without changing the quotas subsequently passed to Candid. Typed failures
are available through `error.nesting_error()`. Low-level upstream APIs remain
unchanged: staged callers can run `candid::nesting::check(&bytes, limits)` first.
This bounds wire nesting, not arbitrary trusted expected Rust schemas/custom
visitors. See the [contract](../../docs/migration.md#external-candid-nesting-qualification-73).

Use `ic_platform::candid::contract` for Candid DTO derives and encoding. Add
`#[candid_path("ic_platform::candid::contract")]` to derived types, and
`#[serde(crate = "ic_platform::candid::contract::serde")]` when deriving Deserialize
through the boundary's Serde export. These are external protocol types, not an
application-wide requirement. Principal-only users import `ic_platform::Principal`
with the `principal` feature, without declaring `ic_principal` directly. In the released
0.3 migration the root type and `PrincipalError` are Platform-owned. Required
Candid DTOs use `candid::contract::Principal`, which remains the original registry
type. Selecting `principal-external` supplies `principal::{ExternalPrincipal, ExternalPrincipalError}` for that
explicit external identity without requiring Candid. Convert principals with
`.into()` at the client/DTO boundary; errors are distinct types.

Owned parsing rejects text longer than 63 bytes before copying, retains uppercase
acceptance and dash validation, and formats using stack buffers. Binary Serde
uses identical raw bytes for borrowed and owned buffers. No generic range-step
implementation or Candid trait/marker handling is provided. Explicitly select
`principal-self-authenticating` for the original upstream implementation;
the base owned type and runtime do not acquire hashing.

Use `candid::decoder::{DecoderConfig, IDLDeserialize, decode_one_with_config,
decode_args_with_config, decode_one, decode_args}` for existing low-level configuration and staged
decoding. These are direct upstream identities: optional/default ceilings,
diagnostics, construction/value/finalization/drop boundaries and typed errors
are unchanged. Callers retain pre-decoder byte admission and canonical re-encoding;
the finite `DecodeLimits` helpers continue selecting compact diagnostics.
The decoder module's `decode_one` and `decode_args` use upstream defaults,
including diagnostics, and take no Platform `DecodeLimits` argument. Preserve
existing caller byte admission; do not confuse them with the parent helpers.

The contract explicitly exports `candid_method`, `export_service`, `Encode`,
`Decode`, `TypeEnv`, `idl_hash`, reflection/type equality/subtyping and printer machinery.
It also exports `DecoderConfig` and `decode_args_with_config` with their original
upstream identities for generated-code contracts. These preserve upstream defaults
and selected configuration, independently of the finite-limit parent helpers.
Published upstream CDK macros 0.20.3 still hardcode `::candid`. Platform's optional
`cdk` surface uses its own maintained generator to resolve these paths.
`contract::types::subtype::subtype` preserves upstream structural subtyping for
narrowed reply projections; equality is a different relation.
For service generation, write `contract::export_service!(ic_platform::candid::contract)`;
the upstream macro accepts this explicit generated path. Update generators too.
Import `contract::{Encode, Decode}` by those names so their recursive macro calls
resolve without a direct Candid crate. The `candid` feature selects upstream
`bignum`; it does not enable an executor or host parser. Service declaration
macros (`candid_method`, `export_service`) and their printer require the separate
`candid-service` feature. External DTO encoding/decoding needs only `candid`;
native parsing still selects `candid-parser`.

Enable `certification` for `ic_platform::certification`: upstream `Certificate`,
`Delegation`, `HashTree`, `Label`, `LookupResult`, `empty`, `fork`, `labeled`,
`leaf`, `pruned`, `fork_hash`, `AsHashTree`, `RbTree` and `Hash`, with upstream Serde support. This feature is
independent of Candid and the executor. Host retains its strict bounded CBOR
decoder; consumers own trust roots, authentication, budgets and leaf meanings.
Neither construction nor deserialization authenticates a certificate. This
centralizes dependency control without removing upstream code or promising
smaller Wasm artifacts.

Enable `canister-sig-creation` for `ic_platform::canister_sig_creation`:
`CanisterSigPublicKey`, `hash_bytes`, `hash_with_domain`,
`extract_raw_root_pk_from_der`, `IC_ROOT_PK_DER` and `IC_ROOT_PK_DER_PREFIX`
from the unchanged upstream 1.3.1 implementation packaged inside Platform.
CanisterSigPublicKey now has a Platform identity; its Principal field, DER/raw
methods, domain separators and `String` errors preserve upstream behavior.
The unused store and Candid import are omitted. No signature store, root
publication or clock API is exposed, and no Candid/ic0/executor is selected.

Enable `signature-verification` for
`ic_platform::signature_verification::verify_canister_sig`, preserving upstream
0.3.0's four byte-slice arguments and `Result<(), String>`. This checks supplied
signature/certificate bytes against the supplied root. It does not select a
trusted issuer, seed, network root, freshness window, resource ceiling or grant
application authority. Keep those checks in the consumer. Owned framing admission
rejects truncated/noncanonical short-form public-key DER before the unchanged
parser on both public-key and verification paths. Long-form DER is unsupported;
encoder/hash/domain length conversion behavior remains upstream.
The verifier selects creation, certification and BLS, exposing the creation
surface without enabling Candid, ic0, CDK, parser or Agent. Both imported source
modules and their vectors/licenses are protected by exact fingerprints.

Enable `bls` independently for `ic_platform::bls::{PublicKey, Signature,
PrivateKey, InvalidPublicKey, InvalidSignature, InvalidPrivateKey,
verify_bls_signature}` from upstream 0.6.0. This retains typed deserialization
errors, serialized bytes and verification results. Upstream defaults are off
and alloc is on, matching the IC verifier and Auth fixtures. Deterministic keys
can be deserialized from caller-supplied bytes and used for signing; no RNG feature
or entropy source is selected. Key generation/custody and domain/trust policy
stay with callers. BLS alone brings neither Candid, creation, certification,
CDK/executor nor host tooling. Native and Wasm are supported by all three features.

Enable `candid-parser` on native hosts for `ic_platform::candid_parser`. It
explicitly exports `IDLProg`, `check_prog`, `TypeEnv`, `Type`, `TypeInner`,
`FuncMode`, `Dec`, `Error`, `Result`, `IDLArgs`, `parse_idl_args`,
`CandidSource`, `instantiate_candid`, `service_equal`, `TypeContainer` and `type_equal`
with their upstream identities and semantics. The pending 0.7.0 cut removes
`compile_javascript`, `compile_typescript` and compiler-only `IDLMergedProg`;
owned frontend bindings must use Wire. The feature does not enable Platform's
`candid` or `executor`;
the upstream parser itself depends on Candid with full type/value support.
On Wasm this module and parser dependency are absent even with all features.
Callers own source/output limits, imports and diagnostics. `check_prog` ignores
imports; inspect `Dec` first when requiring self-contained source. Parse `IDLProg`
and call `check_prog` for silent in-memory checking. `CandidSource::File` reads
files/imports; its loaders and service comparison can print parser diagnostics.
Native refusal fixtures can also import `IDLValue`, `IDLField`, `TypeSerialize`
and `ValueSerializer` from this module. Select `candid` too when using the
contract's derives/reflection or configured decoders. These exports impose no
automatic bounds and do not expose dynamic fixture APIs on Wasm.

The contract also exports `ArgumentEncoder`/`ArgumentDecoder` for generic call
helpers, `Nat`/`Int` for arbitrary-precision external values, and `Reserved` for
partial projections. The `candid` feature includes upstream big-number support.
Use `Result<T, contract::Error>` for encoding errors and `DecodeError` for bounded
decoding failures. Existing wire labels and caller-owned limits remain unchanged.

`executor` exports Platform-owned task/context types and operations from its
single maintained task registry. Callers must preserve context placement, callbacks
and trap cleanup; native compilation does not qualify installed IC behavior.
The implementation is maintained from ic-cdk-executor 2.0.0, with protected-query
wake routing, wake coalescing, cached wakers and task-bookkeeping reclamation.
The Cargo singleton guard rejects old registry executors and multiple new Platform
runtime versions; coordinate Timers/application adoption of the same Platform minor
line. The internal, dependency-free `ic-platform-runtime-guard` package is selected
only by `executor` and exactly matches the facade version. Native gzip-only and
IC0-only consumers select no guard and can coexist with unrelated executors.
The executor implementation stays here; the helper exports no API.

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

Enable `cdk` for the Candid-free `ic_platform::cdk` runtime: explicit-codec
entrypoints, raw calls, system helpers, protected spawning and raw stable-byte
operations. Full `ic-cdk` and upstream macros are not dependencies. The embedded
executor and IC0 remain shared with Timers; neither registry package is required.

All controlled endpoints use Wire. Query/update macros require `decode_with`
(including unit RPCs) and `encode_with` or `manual_reply`. Use a guard for
pre-copy byte admission. Caller code retains endpoint IDs, decoder budgets,
authentication and retry policy. No Candid endpoint fallback, service-registration
macro or Candid path option is supplied. Lifecycle hooks with arguments require
explicit decoding; no-argument hooks may omit it. A renamed dependency can use
`crate = "platform::cdk"`. Remove the retired no-op `hidden` option.
`Call::change_timeout(seconds)` is retired; use
`Call::with_wait_policy(call::WaitPolicy::Bounded { timeout_seconds: seconds })`.
`WaitPolicy::Unbounded` explicitly selects unbounded waiting. Neither choice
configures retries; bounded constructors still default to 300 seconds.

Select `cdk,candid` explicitly for required external management/system-canister
calls. Encode with `candid::contract::{encode_one,encode_args,encode_args_ref}`,
then supply `with_raw_args` or `take_raw_args`. Decode `Response::as_ref()` through
the separate Candid decoder, choosing existing admission/arity policy explicitly.
Raw constructors start with empty transport bytes; Wire callers always supply
a request frame. External no-argument Candid calls encode `()` explicitly.
`CallFailed` and `OnewayError` describe transport failures only. Bounded calls retain
their 300-second default and original cycle/timeout/error behavior.

Call/response/future, cost/performance/error and stable-error types now have
Platform identities; upstream direct-type equality is not retained. Principal
and runtime RejectCode are owned too. Management DTO/Principal, BLS and certification
identities remain upstream; the selected ledger DTOs have Platform identities. The 0.4 release changes signature-key identity as
documented above.
The generator and imported non-cryptographic runtime source retain Apache-2.0
with provenance; the combined package is MIT AND Apache-2.0. No executor or
cryptographic implementation is forked.

Published Testkit 0.54.0 call helpers use Platform's canonical limits/error types.
Upgrade and qualify actual drivers before removing the old decoder declaration.
Consumers still selecting Testkit 0.52/0.53 need its distinct limits/error types.
Those native tests may retain `ic-testkit-candid` solely for that older API until
their published Testkit upgrade is qualified
([producer coordination](https://github.com/dragginzgame/ic-platform/issues/2)).
Do not add conversion/re-export packages or replacement simulator transport.
Selected upstream dependencies remain transitive dependencies.

The same `cdk` feature includes explicit typed call failures, signing/HTTP/creation
costs, balances, certificate publication, counters and `println!`, plus the
reviewed `stable::{stable_grow, stable_read, stable_write, StableMemoryError}`
passthroughs. The 0.5 hard cut retires unused `cdk::storage` checkpoint helpers.
Consumers own existing checkpoint bytes, layouts, limits, retries and certification
policy. Use `api::global_timer_set` (the upstream 0.20.3 name) only under the
consumer's timer ownership, and `api::msg_method_name` in `inspect_message`.
The base runtime removes full CDK and Candid from its normal/build graph.
There is no replacement checkpoint implementation or automatic data migration.

Enable `pocket-ic` for explicit native client, typed error and fixture DTO exports
through `ic_platform::pocket_ic`, including `nonblocking::PocketIc`. Testkit retains
server installation, compatibility admission and lifecycle policy. This feature
is absent on Wasm; default and executor-only builds do not select PocketIC.
PocketIC's management DTOs retain their own upstream 0.8.0 identity.

PocketIC fixture exports include root `CreateCanisterPlacement` and
`CanisterInstallMode`, plus REST `RawEffectivePrincipal`, `RawMessageId`,
`InstanceHttpGatewayConfig`, `CanisterHttpReply`, `CanisterHttpResponse` and
`MockCanisterHttpResponse`. The install mode retains PocketIC's management 0.8
identity, including upgrade flags, independently of Platform's management 0.11
surface. Testkit forwards these identities without owning another client or DTO.

Enable `ed25519` for native/Wasm `ic_platform::ed25519::{Signature,
SignatureError, Signer, SigningKey, VerifyingKey, Scalar}`. These are original
Dalek identities with upstream defaults disabled; no RNG is selected. Consumers
retain strict verification, canonical-scalar checks, message construction and key
custody. This feature is independent of Candid, executor and host features.

Enable `ecdsa` for native/Wasm secp256k1 ECDSA through
`ic_platform::ecdsa::{Signature, SigningKey, VerifyingKey, SignatureError,
PrehashSigner, PrehashVerifier}`. K256 retains type identities and cryptography;
consumers retain prehash construction, high-S policy, normalization and custody.
No RNG backend is selected. This is independent of native IC key derivation
under `secp256k1`.
