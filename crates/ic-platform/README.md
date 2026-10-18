# ic-platform

Controlled Candid and executor dependencies. Enable `candid` for external
protocol DTOs and bounded replies; enable `executor` for the selected CDK executor.
`ic_platform::Principal` is unconditional; no features are enabled by default.

Enable `icrc-ledger-types` for `ic_platform::icrc_ledger_types`, selecting upstream
0.2.0 without default/storage features. The explicit flat surface is `Account`,
`MetadataValue`, `Value`, `ICRC3Value`, `TransferError`, `AllowanceArgs`, `Allowance`,
`GetArchivesArgs`, `ICRC3ArchiveInfo`, `QueryArchiveFn`, `ArchivedBlocks`,
`BlockWithId`, `GetBlocksRequest`, `GetBlocksResult`, `ICRC3DataCertificate` and
`SupportedBlockType`. These preserve upstream identities, external Candid formats,
account/text semantics and value hashes. Select `candid` separately for Platform
encoding/decoding; the ledger dependency itself brings transitive Candid. Native
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
These retain the external Candid format and upstream type identity on native and
Wasm targets. This feature brings transitive Candid; select Platform's `candid`
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

Use `ic_platform::candid::contract` for Candid DTO derives and encoding. Add
`#[candid_path("ic_platform::candid::contract")]` to derived types, and
`#[serde(crate = "ic_platform::candid::contract::serde")]` when deriving Deserialize
through the boundary's Serde export. These are external protocol types, not an
application-wide requirement. Principal-only users import `ic_platform::Principal`
without selecting a feature or declaring `ic_principal` directly. The root and
Candid contract exports have the same upstream type identity.

`ic_platform::PrincipalError` is also unconditional and retains the upstream
Principal parse-error identity used by public host APIs.

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
`contract::types::subtype::subtype` preserves upstream structural subtyping for
narrowed reply projections; equality is a different relation.
For service generation, write `contract::export_service!(ic_platform::candid::contract)`;
the upstream macro accepts this explicit generated path. Update generators too.
Import `contract::{Encode, Decode}` by those names so their recursive macro calls
resolve without a direct Candid crate. The `candid` feature selects upstream
`bignum` and `printer`; it does not enable an executor or host parser.

Enable `certification` for `ic_platform::certification`: upstream `Certificate`,
`Delegation`, `HashTree`, `Label`, `LookupResult`, `empty`, `fork`, `labeled`,
`leaf`, `pruned` and `fork_hash`, with upstream Serde support. This feature is
independent of Candid and the executor. Host retains its strict bounded CBOR
decoder; consumers own trust roots, authentication, budgets and leaf meanings.
Neither construction nor deserialization authenticates a certificate. This
centralizes dependency control without removing upstream code or promising
smaller Wasm artifacts.

Enable `candid-parser` on native hosts for `ic_platform::candid_parser`. It
explicitly exports `IDLProg`, `check_prog`, `TypeEnv`, `Type`, `TypeInner`,
`FuncMode`, `Dec`, `IDLMergedProg`, `Error`, `Result`, `IDLArgs`, `parse_idl_args`,
`CandidSource`, `instantiate_candid`, `service_equal`, `TypeContainer`, `type_equal`,
`compile_javascript` and `compile_typescript`. These retain upstream identities
and semantics. The feature does not enable Platform's `candid` or `executor`;
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

`executor` exports the exact selected task/context types and operations. It owns
no independent task registry. Callers must preserve context placement, callbacks
and trap cleanup; native compilation does not qualify installed IC behavior.
Its published dependency constraint is `ic-cdk-executor = "=2.0.0"`.

Enable `ic0` for `ic_platform::ic0`: the explicit IC 1.2.0 calls used by Timers
for time/version/counters/stable page count, one timer, self/caller/argument
copying, replies/reject codes, cycles/costs and call submission/cleanup, plus
Testkit's `debug_print(&[u8])`. Debug output retains upstream byte/trap behavior;
marker formatting, labels and byte limits belong to the caller. The exact
`=1.2.0` dependency preserves upstream safe/unsafe callback signatures and traps.
It selects neither Candid nor an executor and installs no exports, timer or storage
owner. Timers must select both `executor` and `ic0`; changing either qualified
identity requires renewed combined callback/trap/cancellation/upgrade evidence.

Until the full CDK frontend is qualified, existing canisters/probes may retain
narrow direct `ic-cdk` 0.20.3 and macro-required Candid declarations for entrypoint,
lifecycle, call/system and spawn APIs ([runtime contract](https://github.com/dragginzgame/ic-platform/issues/1)).
Typed/default CDK expansion still emits absolute Candid paths. Re-exporting its macros
or changing imports does not fix that expansion. CDK `futures::spawn` adds an
unfinished-task drop panic outside trap recovery; `executor::spawn_protected`
alone does not preserve that policy and must not replace it.
Existing raw CDK 0.20.3 methods with zero Rust arguments, `manual_reply = true`
and `hidden = true` avoid generated Candid references. `hidden` preserves callable
exports but omits Candid service registration; retain supported DID metadata when
required. This subset still uses direct CDK and is not a Platform runtime facade.

Published Testkit 0.52/0.53 call helpers still own distinct limits/error types.
Existing native tests may retain `ic-testkit-candid` solely for that API until a
published Testkit adopts Platform's identities and consumers qualify it
([producer coordination](https://github.com/dragginzgame/ic-platform/issues/2)).
Do not add conversion/re-export packages or replacement simulator transport.
CDK entrypoint macros and a full CDK call facade remain unsupported.
Selected upstream dependencies remain transitive dependencies.
