# Candid and runtime ownership

IC Platform owns the controlled Candid/CDK dependency boundary. Consumers declare
`ic-platform`, not `candid`, `candid_parser` or `ic-cdk*` directly. IC Wire owns
protocols whose producers and consumers we control. External management-canister,
NNS/SNS and ledger interfaces retain their required Candid formats. Storage and
ic-memory changes are out of scope; no persisted representation is changed here.

## Stable-structures substrate

The optional `stable-structures` feature exposes `ic_platform::stable_structures`
on native and Wasm targets. It explicitly exports the upstream collection,
`Memory`, `Storable`, default backing, restricted-memory view, iterator and
log-error identities used by IC Storage. There are no replacement types,
formats, manager exports or collection implementations.

The exact `=0.7.2` selection matches IC Memory 0.48.2's substrate and preserves
its shared trait and persisted-format identities. Changing it requires renewed
Memory/Storage interoperability and retained-byte qualification. Defaults and
executor-only graphs exclude this dependency. Selecting this feature adds no
Candid, parser, CDK or executor dependency.

IC Storage retains governed collection constructors and its private substrate
alias; applications continue using Storage. IC Memory retains allocation and
committed runtime-handle authority. Default backing and restricted views are
infrastructure/testing interfaces, not permission to bypass governed handles.
After publication, Storage can replace its direct substrate declaration with
Platform plus this feature and change its private import to
`use ic_platform::stable_structures as substrate`. Preserve its existing record
codecs, recovery and region behavior. Consumer adoption and installed upgrade
qualification remain separate from Platform export qualification.

## PocketIC maintenance boundary

The optional native-only `pocket-ic` feature exposes `ic_platform::pocket_ic`.
Platform owns the reviewed PocketIC 16.1.0 dependency and explicit client, error,
settings/snapshot and fixture REST identities. No replacement structs, runtime,
server launcher or automatic client/server acceptance policy are introduced.
Testkit retains provisioning, checksums, supported server versions, lifecycle,
transport, retry behavior and product qualification.

Testkit should replace its native `pocket-ic` declaration with Platform plus
`features = ["pocket-ic"]`, and import `ic_platform::pocket_ic`. Its public
`pocket_ic` module can forward the Platform module; do not retain a second direct
upstream declaration. Its current blanket upstream export becomes this explicit
surface, so downstream usages must be audited before delivery. Consumers continue
using Testkit workflows. Direct-dependency policies should allow upstream PocketIC
only in Platform, while deployment graphs must exclude PocketIC and Testkit.

The feature does not enable Platform Candid, executor or parser features, but
PocketIC brings Candid, certification and native HTTP dependencies transitively.
Its management DTOs are upstream 0.8.0 and must not be substituted with Platform
management-canister-types 0.11.0. PocketIC 16.1.0 also requires thiserror 2.0.18;
the lock selects that compatible version for existing 2.x consumers. These are
dependency-control changes, not transitive removal or Wasm-size claims.

Standalone checks cover building without a server; Testkit adoption, actual
client/server execution and native macOS qualification remain separate gates.

## Qualified Candid surface

The `candid` feature supplies `decode_one`, `decode_exactly_one`, `decode_args`,
`DecodeLimits` and `DecodeError`. The `candid::contract` module explicitly exports
`CandidType`, `Principal`, `Nat`, `Int`, `Reserved`, `Error`, `ArgumentEncoder`,
`ArgumentDecoder`, `encode_one`, `encode_args`, Serde, and the machinery needed
by Candid derives. These are upstream identities, not independent codec types.
Arbitrary-precision integers enable Candid's `bignum` feature with `candid`;
the separate native parser also enables it through upstream Candid's `all` feature.
No default features, automatic limits or second executor are introduced.

`ic_platform::Principal` is available unconditionally from `ic_principal` 0.1.5,
with its standard conversion, self-authenticating and Serde support. Consumers
can remove direct `ic_principal` declarations and use this root export without
selecting Candid or executor features. The existing contract export uses the
same type. Default builds are Candid/CDK-free, not dependency-free.

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
The `candid` feature enables upstream `printer` alongside `bignum`. Change
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

Three independent optional features serve Auth's reviewed signature-store and
verification consumers. They are supported on native and Wasm, have no defaults,
and do not enable Platform's Candid, executor, parser, Agent or certification
surfaces. Their actual transitive dependencies differ:

| Feature / module | Explicit upstream contract | Transitive boundary |
| --- | --- | --- |
| `canister-sig-creation` / `canister_sig_creation` | 1.3.1 `CanisterSigPublicKey`, `hash_bytes`, `hash_with_domain`, `extract_raw_root_pk_from_der`, `IC_ROOT_PK_DER`, `IC_ROOT_PK_DER_PREFIX` | Candid, certification and raw ic0; no BLS or executor |
| `signature-verification` / `signature_verification` | 0.3.0 `verify_canister_sig` | Creation, Candid, certification, raw ic0 and BLS; no executor |
| `bls` / `bls` | 0.6.0 `PublicKey`, `Signature`, `PrivateKey`, `InvalidPublicKey`, `InvalidSignature`, `InvalidPrivateKey`, `verify_bls_signature` | Upstream alloc enabled; no creation, Candid, certification, ic0 or executor |

These are direct upstream exports. Public-key constructor/DER/raw methods and
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
expiry/pruning and root publication. Upstream short public-key DER can panic;
Auth's pre-parser length admission must remain. Domain/DER lengths also retain
upstream integer-conversion behavior. Do not move these checks into Platform or
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
The optional `cdk::api` surface directly exposes that same upstream checked
helper, plus allocating `msg_arg_data` and Principal-returning `msg_caller`.
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

### Maintained CDK generator and adoption

Platform's optional `cdk` feature selects `candid`, `executor`, `ic0`, the published runtime
`ic-cdk = "=0.20.3"` and the coordinated `ic-platform-macros` version.
Activating the executor and raw capabilities enforces their exact downstream
constraints even for CDK-only consumers. The helper is a maintained Apache-2.0 fork of published macros 0.20.3 at revision
`624606f7fd1ecd6668e9608594946486aec29167`. It generates Platform runtime and
Candid paths by default. It is implementation support for the single consumer
library, not an alternative executor or consumer compatibility package.
Upstream acceptance is no longer required to deliver this generator. The old
CDK contribution patch is retired in favor of this one maintained owner; the
separate Principal-only crypto proposal remains unselected.

Published upstream macros accept a runtime `crate` override but hardcode
`::candid` in typed/default expansion. Re-exporting those unchanged macros or
changing imports cannot solve that. Platform's generator parses both runtime
and Candid paths and uses the selected Candid path in encoding, configured
decoding, service registration and service extraction. Defaults are
`::ic_platform::cdk` and `::ic_platform::candid::contract` respectively.

`ic_platform::cdk` explicitly exports query/composite-query, update, init,
pre/post-upgrade, heartbeat, inspect-message and low-Wasm-memory macros plus
`export_candid!`. Its `api`, `call` and `futures` modules expose only the
reviewed upstream helpers, `Call`/`CallFailed`/`Response`, spawning and the
exact executor-context wrappers used by generated code. There are no blanket
upstream modules or replacement convenience implementations. Runtime APIs retain
the original upstream identities and CDK's controller 0/1/panic behavior.

#### Remaining consumer CDK helpers

The requests in [#19](https://github.com/dragginzgame/ic-platform/issues/19),
[#20](https://github.com/dragginzgame/ic-platform/issues/20),
[#21](https://github.com/dragginzgame/ic-platform/issues/21) and
[#22](https://github.com/dragginzgame/ic-platform/issues/22) extend this same `cdk`
feature with explicit upstream identities. No extra feature is needed.

- `storage::{stable_save, stable_restore}` preserves the original Candid tuple
  format, error types and restore behavior, including no decoder finalization.
  Timers retains its `(bool, bool, u64, u64)` checkpoint and upgrade policy.
  This narrow passthrough adds no storage implementation, layout or limits.
- `stable::{stable_grow, stable_read, stable_write, StableMemoryError}` retains
  `u64` pages/offsets, grow's previous-page result, bounds traps and rollback.
  Wire retains page admission and its rollback test. `api::stable_size` supplies
  Canic's existing observation without transferring memory ownership.
- `api::{certified_data_set, data_certificate}` retains generic `AsRef<[u8]>`
  input and `Option<Vec<u8>>` certificates. Auth owns tree roots, certification
  policy and Wire payloads. Passive `certification` remains independent of CDK.
- `call::{Error, CandidDecodeFailed, CallPerformFailed, CallRejected, RejectCode,
  InsufficientLiquidCycleBalance}` preserves typed matching and conversions.
  Canic keeps rejection classification, bounded/unbounded waits and retry policy.
- `api::{cycles_burn, cost_sign_with_ecdsa, SignCostError, canister_version,
  canister_cycle_balance, canister_liquid_cycle_balance, cost_create_canister,
  cost_http_request, subnet_self, root_key, msg_method_name, performance_counter,
  PerformanceCounterType, global_timer_set, debug_print}` and `cdk::println!`
  preserve upstream signatures and execution-context restrictions. In particular,
  use `msg_method_name` in `inspect_message`. The 0.20.3 name is
  `global_timer_set`, not `set_global_timer`; no speculative alias is supplied.
  Setting a timer does not install another timer entrypoint: Timers retains its
  sole runtime export. Existing instruction/context counters remain available.

For adoption, replace these paths with `ic_platform::cdk`, enable `cdk` in each
actual canister/probe manifest, and remove exhausted direct CDK declarations and
feature edges. Update generated paths and negative ownership tests as described
below. Auth's hidden/custom-Wire canister needs a narrowly reviewed dependency
gate allowance for this runtime bundle: `cdk` includes Candid transitively even
when the application protocol is Wire. Preserve its custom codecs and hidden
exports; do not introduce Candid protocol fallback. Executor-only libraries keep
their independent feature selection. Host parser and Agent stay host-only.

Qualify original checkpoint bytes, growth/write trap rollback, certificate tree
lookup and typed error identity, then run the consumer's own installed tests.
Captured consumer compilation is evidence about those captured sources, not
published adoption. The maintained standalone and PocketIC fixture exercise
these paths; native system-call execution is not simulated by a library test.

#### Existing cycle helpers

Blob's funding and resource consumers additionally use the explicit
`cdk::api::{msg_cycles_available, msg_cycles_accept, msg_cycles_refunded,
instruction_counter, call_context_instruction_counter}` exports. These are the
unchanged CDK 0.20.3 functions: cycle amounts are `u128`, instruction counters are
`u64`, and the selectors remain owned by CDK. No wrapper, new feature, cycle policy
or counter implementation is introduced. See the
[consumer report](https://github.com/dragginzgame/ic-platform/issues/1#issuecomment-6250276514).

Replace only these `ic_cdk::api` imports in Blob core and its funding, admission,
authority, gateway-source and storage probes. Keep their current `cdk` selection;
remove the exhausted direct CDK member/catalog declarations and narrow the
dependency allowlists. Preserve exact attached/accepted/refunded amounts, zero-cycle
admission and counter attribution across awaits. Refunds belong to the matching
reply/reject callback, before decoding or another await; submission failures have
no callback refund, and callback traps still leave application settlement uncertain.
Consumer restore fences, retry authority, Wire/CBOR/provider Candid and decoder
limits are unchanged. The maintained fixture checks those upstream helper paths;
actual published consumer funding/rollback tests remain adoption qualification.

The generator preserves upstream export naming, guards and their order,
lifecycle restrictions, async/sync contexts, protected spawning, tuple/unit
reply shape, custom codecs, manual/hidden behavior and skipping quota 10,000.
Zero Rust arguments skip default decoding. Hidden methods remain callable Wasm
exports but are omitted from DID registration. Do not use hidden methods to
silence Candid paths on supported public services. The finite `DecodeLimits`
helpers are not substituted into upstream generator policy. Consumers keep
argument byte admission, budgets, disclosure policy and custom codec behavior.
CDK `futures::spawn` preserves its unfinished-task drop panic; raw
`executor::spawn_protected` is not a policy-equivalent replacement.

Adopt after the paired packages are published and qualified:

1. Add `cdk` to the inherited Platform feature selection in the actual canister
   member and any independent probes. Keep host parser/Agent selections in host
   or development dependencies. Executor-only Timers users retain `executor`
   and `ic0`, without `cdk` or Candid.
2. Replace the reviewed `ic_cdk::{api,call,futures}` paths and entrypoint/service
   macros with `ic_platform::cdk` paths. Preserve bounded/unbounded waits,
   timeouts, cycles, exact raw arguments, original reply bytes and failure
   classification. `Call` convenience Candid methods keep upstream defaults;
   consumers with finite reply budgets keep their admitted configured decoder.
3. Update Canic/Fleet and any other generators, including generated contexts and
   spawning, at their source. Replace generated `::candid` paths with the
   Platform contract; DTO derives still use `#[candid_path("ic_platform::candid::contract")]`
   and `#[serde(crate = "ic_platform::candid::contract::serde")]`.
   Regenerate and compare DID and external request/reply bytes. Do not rely on
   handwritten import changes to repair generated code.
4. Normally use `#[ic_platform::cdk::update]` and
   `ic_platform::cdk::export_candid!()`. Renamed Platform dependencies require
   explicit paths on each macro, for example
   `#[platform::cdk::update(crate = "platform::cdk", candid = "platform::candid::contract")]`
   and `platform::cdk::export_candid!(platform::candid::contract)`;
   update DTO derive/Serde paths to the same alias.
5. Remove direct Candid/CDK/macros declarations only after every actual use is
   covered, including generators, tests and independent workspaces. Remove
   unused root catalog entries, update lockfiles and dependency-ownership rules,
   and preserve consumer-owned protocol types and policies. Unsupported upstream
   APIs require a real usage review and explicit extension, not blanket exports.
6. Run native/MSRV and Wasm builds, standalone no-direct-dependency tests, exact
   service/wire comparisons and production-target graph admission. Resolve one
   executor 2.0.0 and ic0 1.2.0 source/version identity across entries, calls and
   Timers. Qualify reply/reject/submission failures, awaited/protected/migratory
   tasks, callback trap cleanup, cancellation and upgrade in the actual consumer.
   Linux/PocketIC, native macOS and installed IC are separate gates.

The support package inherits the workspace release version; the facade declares
that compatible version requirement. Release metadata checks cover both packages and update the
paired requirement together. Publication selects both packages and Cargo owns
the dependency order. Reconcile both exact registry identities after interruption;
partial upload is not transactional and must not be retried blindly. Local
packaging/dry-run evidence does not prove delivery or deployed consumer behavior.

Focused generator, standalone, graph and simulator evidence for this source is
recorded in [the current handoff](status/current.md). CDK's published runtime
still depends on its original macros and on Candid transitively. This route
removes consumers' direct dependency requirement; it neither removes those
transitive packages nor establishes measured Wasm-size reduction. The remaining
consumer adoption and installed acceptance are tracked in
[the runtime boundary](https://github.com/dragginzgame/ic-platform/issues/1).

## Host parsing and aggregate admission

The optional `candid-parser` feature now exposes `ic_platform::candid_parser`
on native hosts, selecting upstream `candid_parser` 0.4.1. The dependency and
module are target-gated out of Wasm, including all-feature builds. No Candid,
executor or certification feature enables host parsing. Parser-only native
selection does pull transitive Candid with the parser's upstream `all` feature;
it does not select Platform's `candid` feature. This is dependency isolation,
not transitive removal or a Wasm-size measurement.

The explicit surface covers the reviewed Host service inspector and Canic's
frontend bindings, protocol argument helpers and service/type comparisons:
`IDLProg`, `check_prog`, `TypeEnv`, `Type`, `TypeInner`, `FuncMode`, `Dec`,
`IDLMergedProg`, `Error`, `Result`, `IDLArgs`, `parse_idl_args`, `CandidSource`,
`instantiate_candid`, `service_equal`, `TypeContainer`, `type_equal`,
`compile_javascript` and `compile_typescript`. These directly preserve upstream
identities; there is no blanket crate/module export or new parser implementation.

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
`syntax::{Dec, IDLMergedProg}` and `utils::{CandidSource, instantiate_candid,
service_equal}` imports. Use root `FuncMode`, `Type`, `TypeInner`, `TypeEnv`,
`TypeContainer` and `type_equal` for those host type operations. Replace
`bindings::javascript::compile` with `compile_javascript` and
`bindings::typescript::compile` with `compile_typescript`, preserving the merged
program supplied to TypeScript generation. Move ordinary DTO/encode/decode uses
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
It exercises constructor typing, generated browser bindings, malformed grammar,
undefined types, import detection, service drift, Rust type equality and a fixed
nat64 argument wire vector. Host's service inspector and existing tests are also
qualified from a disposable copy with only imports changed; source hashes and
fixture evidence are under `target/parser-before/host-source.sha256` and
`target/parser-host/`. These are focused proofs, not migrated sibling workspaces.
Actual Host/Canic adoption must still compare service inventories, generated
binding bytes, argument bytes and existing protocol/effect/retry tests. Neither
consumer product code nor external Candid formats or persisted data change here.

Host's pending decoder first rejects oversized replies, then atomically subtracts
admitted bytes from the caller's remaining budget before decoding. Malformed
admitted replies still consume budget; failed admission does not. Consolidation
must preserve those semantics outside the per-reply Platform decoder. Host's
pending helper does not select compact diagnostics; adopting Platform changes
that diagnostic policy and needs explicit caller qualification.

Host's service inspector uses `IDLProg`, type checking and `TypeEnv` to project
bounded method facts, rejects imports, and leaves Wasm export identities to Host.
Canic additionally consumes parser bindings, textual arguments and service equality;
method inventory alone cannot replace those uses. The new explicit host surface
covers those operations; Host and Canic retain the surrounding implementation.

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
as a direct dependency: native tests on Rust 1.99 and 1.88, plus Wasm compilation
on both compilers for Candid, certification, management, raw-system and ledger
consumers, plus canister-signature creation, signature verification, BLS and the CDK
canister/service fixture (linked as a Wasm cdylib on both compilers).
The maintained `make check-runtime` harness separately installs the combined
CDK/Timers fixture, exercising callbacks, trap cleanup, cancellation and upgrade.
It uses published Timers/Testkit at their Rust 1.91 floor; it does not raise
Platform's 1.88 floor. See [runtime setup](../README.md#maintained-cdk-runtime-regression).
Parser/dynamic and Agent consumers are native-only. `make msrv` also
checks the all-feature library for Wasm at Rust 1.88. CI prepares the Wasm target
for both toolchains; local validation never installs it. See
[Wasm MSRV enforcement](https://github.com/dragginzgame/ic-platform/issues/15).
It exercises recursive/renamed DTOs, generic argument helpers, `Reserved`, typed
error downcasting, and independent Candid wire vectors for nat 2^128 and int -65.
This is representative boundary evidence, not a build of migrated siblings.

`make check-features` checks no features, actual defaults, every single feature,
every pair, all features and explicit critical runtime/DTO/signature/host groups
on native/Wasm. `make check-features-exhaustive` retains the complete power set
as a separate manual audit, outside routine CI/release validation. Routine
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
The signature extension admits the reviewed creation/verifier/BLS/certification
identities only through their selected or required transitive capabilities. BLS
alone cannot admit creation, Candid or raw IC calls; the two canister-signature
features explicitly retain those upstream dependencies. Default/executor and
certification-only isolation remain unchanged. These are workspace graph gates,
not guarantees about feature unification in every downstream application.

Direct dependency isolation does not remove transitive Candid from management DTOs, Agent, PocketIC,
protocol DTO packages or full CDK. No Wasm-size reduction has been measured.
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
- `patches/cdk-optional-macros.patch` targets CDK 0.20.3, preserving its default
  macro API while permitting Platform's owned generator to disable that feature.
  Three standalone Platform CDK consumer tests pass on Rust 1.99/1.88, with linked
  Wasm on both. The selected Wasm graph omits upstream ic-cdk-macros and retains
  executor 2.0.0, ic0 1.2.0 and Candid. This is not a Candid-free raw-call runtime
  or new callback/trap/timer qualification; those runtime contracts stay intact.

Each patch applies to its exact registry source. Disposable evidence is retained
under `/tmp/platform-signature-split`, `/tmp/platform-public-secp` and
`/tmp/platform-cdk-split`. These package-graph observations do not measure Wasm
size or wall-clock performance. Existing management/ledger DTOs remain selected:
owning their definitions would change Rust identity without removing external
Candid requirements. Public ownership changes require an explicit migration and
renewed consumer qualification; no supplying release is implied by these proofs.

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
