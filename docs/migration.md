# Candid and runtime ownership

## Native external-service export (unreleased)

`candid-codec` now supplies native-only `service::{Method, Mode, export}` from
the actual owned schemas. Consumers emit descriptors from the same catalogue
as mounted external endpoints and use the pinned native `didc check --strict`
against their frozen DID. No registration, generic forwarding or parser
dependency returns. See [the exact consumer migration and evidence](native-candid-service.md).
This resolves [#153 — Document the native external-service schema/export owner](https://github.com/dragginzgame/ic-platform/issues/153).

## Per-field decoder admission (0.29)

The `candid-codec` feature adds `decoder::Reader::bounded_vec<T>(wire, maximum)`:
the element count is checked before reserving vector storage or reading any
element. A consumer's `Decode` wrapper selects its own cap for each field;
nested wrappers can independently cap inventory length, ID bytes and other blobs.
Global work/allocation charges and the fixed 16-level ceiling still apply.
`Reader::strict_option<T>(wire)` admits only an actual wire option and propagates
inner errors, without bare-value promotion or incompatible-value fallback.
Missing fields remain the consumer mapping's `Decode::missing` decision.

`decoder::decode_one_with_skip_limit(bytes, limits, skipping)` and
`decode_args_with_skip_limit(bytes, limits, arity, skipping)` add a separate
cumulative skipped-work budget. Reads/traversal during validated skipping debit
both this budget and total work once, including nested skips, unknown fields,
surplus compatible arguments, Reserved and optional fallback. Zero refuses all
skipped values, even zero-byte ones. These are owned codec units, not upstream
quota units. Existing decoder entrypoints keep their current behavior.

Backup can select its existing per-field caps and zero skipping without an
upstream codec or a separate framing parser. Its strict source-marker adapter
must admit the actual reserved wire type (`-16`) rather than use the permissive
`Reserved` mapping, which deliberately skips any supplied value. Required fields,
minimum lengths, exact shapes, duplicates, authority and recovery remain its
responsibility. The public consumer test demonstrates nested 1024/256 caps and
unchanged bytes; this is not Backup product or recovery qualification.

## Owned management DTOs on all targets (unreleased)

The maintainer authorized removing Platform's production
`ic-management-canister-types` dependency. Native and Wasm use the same maintained
management DTOs, canonical Principal, owned Nat/Reserved and explicit Candid codec.
The previous 58 native exports and 44 Wasm exports remain available on their
respective targets. Fourteen native-only snapshot declarations and their
`StoredChunksResult` dependency alias are maintained from 0.11.0 separately;
this does not add host-only snapshot transfer APIs to the Wasm surface.

The Tooling client extension adds native-only `InstallCodeArgs`,
`ProvisionalCreateCanisterWithCyclesArgs`, `CanisterLogRecord` and
`FetchCanisterLogsResult`. These bring the native surface to 62 names; Wasm
remains at 44. Tooling uses Principal, ChunkHash and Vec<ChunkHash> directly
instead of adding aliases. Independent source fingerprints preserve the
original snapshot selection and the four additional registry 0.11.0 declarations.

Method names, Candid schemas, variants, payload bytes and Serde attributes remain
unchanged. Native Rust type identity changes: replace upstream Candid trait-based
encoding/decoding with `candid_codec`, and supply the owned Principal and Nat.
Callers must qualify explicit byte/header/type/work/allocation budgets; upstream
decoding and skipping quotas are not the owned codec's counters. Storage,
transport, retries, authorization and snapshot lifecycle stay consumer-owned.

Registry management 0.11.0 remains a dev-only schema/byte oracle. Existing native
suppliers such as Testkit may still depend on it independently. The production
management-only graph excludes registry management, Candid and registry Principal
on both targets. Native management can no longer admit those through the runtime
graph exception. Signature suppliers retain their separately qualified transitive
Candid capability. No cryptographic implementation changes.

This supersedes older native management registry-identity constraints below.

## Owned Principal hard cut (unreleased)

Platform-owned DTOs, ledger accounts, function callbacks and certified-controller
results now use the canonical `ic-platform-types::Principal`. The owned Candid
codec encodes/decodes that identity directly, including its wire marker; Principal
Serde remains marker-free. Remove `principal-external` and use `Principal` /
`PrincipalError`; the external aliases and conversion impls are removed without
compatibility features. This is a breaking change in Platform and Types.

Types has no production `ic_principal` dependency. Its dev-only registry oracle
retains independent text/byte/self-authentication checks. Native Agent, transport
and management APIs retain their supplier identities; signature adapters copy
validated Principal bytes into inferred supplier types at their private boundary.
Those suppliers can still bring `ic_principal` and Candid transitively. Cryptographic
bodies, domains, hash inputs and upstream verification remain unchanged.

Owned DTOs no longer derive CandidType even in unit tests. Frozen DIDs and wire
fixtures remain independent expectations. Oracle schema projection is compared
against those DIDs, and upstream parsing/re-encoding checks the owned corpus;
that corpus check alone is not an independently constructed value oracle. Explicit
Principal/registry value cases and the existing callback/primitive comparisons
provide independent value encoding checks. No Candid trait is added to Types.

This section supersedes older external-Principal and DTO test-bridge wording below.

## Legacy Candid surface removed (unreleased)

The public `candid`, `candid-parser` and `candid-service` features and modules
are removed, including protocol forwarding, generic upstream derives, dynamic
values, reflection, native service registration and parser/printer exports.
There is no compatibility alias. This supersedes historical bridge instructions
in earlier release sections below.

Use `candid-codec` plus the required DTO feature, `encode_one(&value, reply_bytes)`
or `arguments::encode_args(&tuple, reply_bytes)`, and
`decoder::{decode_one, decode_args}` with explicit `Limits` and `Arity`.
Use `candid-codec-endpoint` for required external query/update interfaces.
Callers retain method names, transport, authorization, freshness and retry policy.
Owned application protocols remain Wire.

Owned DTOs no longer implement upstream CandidType or expose conversions to
upstream Nat/Int/Func. Construct their owned carriers directly; integer conversion,
public magnitudes, numeric ranges and Serde/Candid bytes remain qualified.
Native management DTO identities remain unchanged; owned DTO Principal fields
follow the hard cut above.
Consumers that need textual parsing must select a native parser owner explicitly;
Platform supplies no text-parser replacement.

Candid and candid_parser are development-only independent byte/schema oracles.
They do not enter normal/build graphs from Platform's codec or DTO bridge.
Native Agent/transport/management and upstream signature suppliers still bring
transitive Candid. Cryptographic algorithms and checks are unchanged; complete
all-feature/lockfile elimination requires separate supplier changes.

## Owned binary codec (unreleased)

Select `candid-codec` for `ic_platform::candid_codec`, without selecting `candid`,
`candid-parser` or `candid-service`. It supplies typed encoding, bounded decoding,
explicit argument arity, record/unit/payload/named-variant mapping macros, canonical Principal
and arbitrary Nat/Int through num-bigint. It does not select runtime, Agent,
external Principal or cryptography. It is not a Serde/CandidType compatibility
layer or a textual parser. `candid_args` remains the dependency-free choice for
empty/unit/text requests; raw_rand retains its separately qualified codec.

`nns-registry-types` selects the owned codec and canonical Principal. Its external
wire schema and Serde representation remain unchanged. Use `candid_codec::encode_one` or
`arguments::encode_args`, and `decoder::{decode_one, decode_args}` with explicit
`Limits` and `Arity`. Request/response bytes are checked against upstream Candid.
Independent frozen schemas and development-only registry oracles qualify bytes;
owned DTOs do not implement upstream CandidType.

`cmc-types`, `cycles-ledger-types` creation/withdrawal and native
`certified-assets-types` now also select the owned codec without upstream Candid.
CMC/Cycles now use canonical Principal. Their Nat fields now use
`candid_codec::Nat`, a Platform-owned carrier around num-bigint's `BigUint`.
This is a Rust source/type-identity change: replace annotations using registry
`candid::Nat` with the owned carrier; the old upstream conversion bridge is removed.
Integer `.into()` construction, the public `.0` magnitude, decimal `FromStr`,
grouped decimal display and numeric comparisons remain available; arithmetic
uses `.0`. The parse error is num-bigint's error, not Candid's diagnostic type.
The unpublished prototype's `Nat = BigUint` alias is replaced by this carrier;
direct BigUint codec implementations remain supported.

Nat remains unbounded and preserves the established Serde newtype/limb-sequence
representation and nonnegative integer, decimal-string and tagged-byte admission.
JSON/CBOR parity and boundary values through 8,192 bits are tested against the
registry type. Empty tagged buffers now return an error instead of panicking.
No cryptographic arithmetic is implemented here; all magnitude operations use
num-bigint. The legacy protocol/derive bridge is removed. The ICRC and Cycles provider archive migration below extends this change to
recursive values and typed callback signatures without narrowing their schemas.

`icp-ledger-types` now selects the owned codec for transfer and encoded-block
archive DTOs, including query function references. The existing
`QueryEncodedBlocksFn::new` constructor now takes canonical Principal and String;
its `.0` payload fields remain. Its payload carrier is now `candid_codec::references::Func`,
not registry `candid::types::reference::Func`; the upstream conversion and CandidType bridges are removed. The
ordinary function payload bytes and tagged Serde decode representation remain.
Opaque/malformed references fail closed. The reference module is available with
`candid-codec`; it does not enable upstream Candid or execution.

Reference admission checks argument contravariance, result covariance, function
modes, record/variant/service width, sequence omission and optional compatibility.
An iterative pair worklist handles recursive signatures within the caller's work
and logical-allocation budgets. These checks do not authorize callback principals,
method names or calls; callers retain those decisions. The 16-level incoming wire
ceiling still applies, including unused table entries. The imported ICP account
derivation body and its supplier/license fingerprints remain unchanged; only its
Principal import now names the canonical owned identity.

The new reader always validates unused types, skipped values and trailing data,
and retains the 16-level constructed-wire ceiling. Its work and cumulative
logical-allocation budgets are not the upstream decoding/skipping quotas; do not
copy quota numbers without qualifying the actual consumer workload. Custom Rust
mappings are trusted code. Standalone typed service values, complete recursive
Rust mappings, broad fuzzing and installed instruction qualification remain open
before migration of interfaces needing those capabilities. Existing management,
ICRC and endpoint adapters retain their qualified codec until migrated.

This does **not** remove Candid from the entire all-feature graph. Registry
canister-signature suppliers still depend on it on Wasm; native Agent, transport,
management DTO and parser suppliers also retain it. Cryptographic implementations
are unchanged. No supplier fork or sibling migration is included.

## Small external argument encoders (Platform 0.27.0)

Select `candid-args` on published Platform 0.27.0 for dependency-free external
Candid encoding on native and Wasm targets. The deliberately small `candid_args`
module supplies:

```rust,ignore
candid_args::empty();        // zero arguments: ()
candid_args::units(2);       // two unit/null arguments: ((), ())
candid_args::text("hello");  // one text argument: ("hello",)
```

The empty envelope is a static six-byte slice; unit/text encoders return owned
bytes. Text lengths count UTF-8 bytes and counts/lengths use minimal unsigned
LEB128. Empty text, no arguments and one null remain distinct. There is no text
parser, schema machinery, runtime or new dependency here. Callers retain input/
output budgets and method selection. Owned application protocols still use Wire;
other external argument shapes retain the explicit general Candid capability.

Both `raw-rand` and `candid-codec-endpoint` select this module. The entropy request, zero-argument
external protocol encoders and legacy-empty admission reuse `empty()`. Additional
supported shapes can be added here only when a concrete external caller needs
them, with independent upstream byte checks. No sibling is edited here.

<!-- Published Platform 0.27.0's crate README uses this anchor. -->
<a id="candid-free-management-entropy-platform-027-unreleased"></a>

## Candid-free management entropy (Platform 0.27.0)

Select `ic-platform` with `default-features = false, features = ["raw-rand"]`
for `ic_platform::raw_rand::{METHOD, encode, decode, DecodeError}`. Encoding
returns the static six-byte empty Candid argument tuple; decoding returns
`[u8; 32]` without allocation or any general Candid dependency. Select execution
separately through `ic-platform-runtime/runtime` (or its `cdk` entrypoints).
The facade's existing `runtime` feature can also be composed with `raw-rand`.

This is a deliberately restricted management-producer profile, not a general
Candid decoder: one `vec nat8` type at index zero, exactly one result and 32
payload bytes, with no trailing bytes. The vector/nat8 opcodes use their one-byte
encodings. The four count/index/length fields may use minimal or zero-padded
LEB128, capped at nine bytes each; the entire reply is capped at 74 bytes. Other
valid Candid forms, including unused type definitions or a different result type
index, are refused. The
replica's `raw_rand` producer creates a 32-byte vector and replies using Candid
`Encode!(&buffer)`; the reference-byte and installed checks qualify that shape.
This bounded external-protocol admission does not impose application seed policy.

[Canic #749 — Adopt Platform's Candid-free raw_rand codec](https://github.com/dragginzgame/canic/issues/749)
can replace its general `candid::protocols::management::raw_rand`
callers with this module and remove the adapter's `candid` feature selection.
Use `encode().as_slice()` with the existing raw call, then pass response bytes to
`decode`. Retain the current wait, callback/cancellation, retries and all-zero
seed refusal in the consumer. The old
`candid::protocols::management::raw_rand` module is removed without an alias:
`ic_platform::raw_rand` is Platform's sole typed entropy codec, independently
selected even when Candid is already enabled. `encode()` is now infallible;
`decode()` returns `[u8; 32]` and takes no general Candid budgets or arity option.
Other Candid protocols keep their signatures and caller-selected budgets.

Generic Runtime/CDK/Agent transports and general Candid encoders are still
method-agnostic; callers can construct arbitrary external requests manually.
The single-owner rule does not install method-name filters or an alternative
runtime. All maintained Platform entropy callers use the typed codec; the
size/reference fixtures alone construct general Candid as a test baseline.
Published Platform 0.27.0 supplies this API. Select a requirement accepting 0.27.0;
the 0.26 line does not supply it. No Canic or IcyDB source is changed here.

## Canonical ordinary entrypoints (Runtime 0.20 / Platform 0.25)

Ordinary entrypoint macros belong only to `ic-platform-runtime`, selected with
`default-features = false, features = ["cdk"]`. Import
`ic_platform_runtime::cdk::{query, update, init, pre_upgrade, post_upgrade,
heartbeat, inspect_message, on_low_wasm_memory}`. These names remain conventional;
the implementation macro package now has the same single family, defaulting to
`::ic_platform_runtime::cdk`. The prefixed `runtime_*` exports are removed.

Remove the facade's `cdk` feature selection and ordinary macro imports before
adopting Platform 0.25. Raw facade adapters remain under its `runtime` feature;
execution-only consumers should depend on Runtime directly. Native/scaffold and
benchmark generators must emit the direct Runtime dependency/imports too. Renamed
Runtime dependencies retain the explicit `crate = "renamed::cdk"` override.
One generator, executor, IC0 ABI, guard ordering, codec hooks, method names,
reply/async/lifecycle behaviour and canonical Types identities remain unchanged.
No compatibility feature or second executor is introduced.

Required external Candid adapters stay in the facade (the original
`candid-endpoint` route is superseded by the owned adapter below).
The original feature selected `runtime`, `candid` and the coordinated macro
supplier; new consumers use the owned codec selection documented below. Its typed bridge retains an explicit facade adapter path, so it does
not force ordinary macro consumers through Platform or require consumers to name
private dependencies. Candid/Serde bytes and all verification policy are unchanged.
The CDK consumer integration proof moves to Runtime; external-Candid proof stays
with the facade. Types and browser groups do not advance for this cut.

## Full release qualification only

The selected-stage release pilot is retired. All three groups require all local
stages; there is no host allowlist or source-path exclusion classifier. Immutable
source/evidence receipts, release identity, metadata-only projection and recovery
refusals remain. See [release validation](platform-release-validation.md).
Historical pilot text below is superseded by this contract; retained attempts
must use their own supplying source and tooling.

## Explicit Candid decoder configuration (Platform 0.24.0)

The unconfigured `candid::decoder::{decode_one, decode_args}` exports are
removed. Use `candid::{decode_one, decode_args}` with caller-selected
`DecodeLimits`, or the existing `decoder::{decode_one_with_config,
decode_args_with_config}` with an explicitly selected `DecoderConfig`.
Staged `IDLDeserialize` and its upstream identity remain available.

This is an API cut, not a codec or resource-policy replacement. The bounded
helpers retain their existing admission rules and closed errors; the configured
and staged APIs retain upstream behaviour, optional quotas and diagnostics.
Preserve external bytes, integer ranges, optional/subtyped values, extra-value
skipping versus exact arity, and consumer-owned error/transport/retry policy.
An explicit `DecoderConfig::new()` retains the former default behaviour when
that is the consumer's deliberate selection; production budgets remain theirs
to choose. Do not replace an ordinary decoder with `decode_exactly_one` without
reviewing that operation's reply contract.

Read-only caller migration issues: [Canic #740](https://github.com/dragginzgame/canic/issues/740)
(production host/history paths and tests), [Blob #119](https://github.com/dragginzgame/ic-blob-storage/issues/119)
(gateway test canister), and [IC Crates #25](https://github.com/dragginzgame/ic-crates/issues/25)
(managed HTTP tests). No sibling code or release is changed here. Platform's
own consumer and installed-runtime fixture sources use explicit configuration
while preserving their prior decoding choices. Runtime, Types and browser groups
are unchanged.

## Uninitialized stable reads (Runtime 0.19.4)

Runtime's `ic0::stable64_read_uninit(&mut [MaybeUninit<u8>], offset)` exposes
the existing safe imported IC0 helper. A successful call initializes the whole
buffer; invalid stable ranges trap. It permits Storage to retain ICSS's direct
uninitialized-read path without adding another system-call binding. The export
adds no dependency, executor, Types capability or allocation authority. Existing
initialized reads are unchanged. Registry consumers require Runtime >=0.19.4
with `default-features = false` and `features = ["ic0"]`. Native/Wasm registry
consumer proofs retain the exact supplying package and dependency-free graph;
Storage retains its own installed IO, initialization and upgrade qualification.

## ICP account identifier (Platform 0.23.3)

Select `icp-ledger-types` and call
`ic_platform::icp_ledger_types::account_identifier(owner, &subaccount)` with the
canonical owned Principal and `[u8; 32]` subaccount. It returns `[u8; 32]` in the
standard ICP format, distinct from ICRC Account text. The selected constructor
is unchanged from Apache-2.0 ic-ledger-types 0.16.0, with its SHA-224/domain and
CRC32 bytes intact. The new API does not add a transfer, address validator,
account-identity abstraction, subaccount policy or runtime dependency.

Platform 0.23.3 supplies this operation on crates.io. Canic can replace its mint/receipt helper,
preserving the retained `bdc4ee05...680a938` vector, transfer bytes and receipt
checks. Remove its direct crc32fast dependency only after confirming no other
consumer use. Existing SHA-256 contracts are unaffected. No sibling adoption or
installed-canister acceptance is claimed by the focused native/Wasm compile proofs.

## Management create_canister request (0.23.2)

Select `management-canister-types` for
`management_canister_types::CreateCanisterArgs { settings, sender_canister_version }`.
It is the exact management 0.11.0 request, distinct from the Cycles Ledger creation
request. Its optional settings use the existing management `CanisterSettings`;
the optional version retains the full `u64` range. Native exports retain registry
identity; Wasm uses the maintained declaration with equal Candid schema/bytes.

Also select `candid` for
`candid::protocols::management::create_canister::{METHOD, encode, decode}`.
The reply is the existing `CanisterIdRecord`. The decoder retains caller budgets,
16-level admission and explicit reply arity. No transport or runtime is selected.
Canic can delete `StubCreateCanisterArgs` and construct this public request
directly, retaining its deadline, attached cycles, recovery and fixture policy.
Published Platform 0.23.2 supplies this API. Select a requirement accepting 0.23.2;
published 0.23.0 and the 0.22 line do not expose this request. The interrupted
0.23.1 attempt was never published. Consumer adoption remains separately owned.

## Optional subnet metrics (0.23.0)

The native `ic-protocol` decoder's `decode_subnet_metrics` result is now
`Result<Option<SubnetMetrics>, CborError>`, matching Testkit's existing IC leaf
contract. Handle `Some(metrics)` and CBOR null's `None` explicitly. All metrics
fields retain upstream identity and the complete u128 cycle range. Empty,
malformed and trailing data still fail, and complete-input byte limits apply
before decoding even null. Missing tree paths, authentication and freshness
remain consumer-owned. The correction is published in Platform 0.23.0. Consumers
must update their Platform requirement from `0.22` to `0.23`; `cargo update` alone
does not cross that minor boundary. Published 0.22.3 returns a required metrics object.

## Bounded certificate and signature CBOR (0.22.3)

Select `certification-cbor` for `certification::cbor::{decode_certificate,
decode_canister_signature, encode_certificate, encode_canister_signature}`.
Every call takes a caller-selected byte budget. Input length is admitted before
upstream Serde decoding; serialization refuses an oversized write before growing
the output buffer. The APIs preserve standard CBOR and upstream certificate/tree
identities. Byte budgets are not total heap limits; upstream recursion admission
remains unchanged. Decode is not verification, including for delegation bytes.
Consumers retain trust/freshness, certified-root binding, witness construction
and signature-store policy. No Runtime, Candid or cryptographic primitive is
added to the certification-only graph. Auth can adopt the published 0.22.3 facade
addition; consumer qualification and releases remain separately owned.

## Typed IC packet and fixture boundary (0.22.3)

Platform supplies protocol-specific operations rather than a generic CBOR
serializer, decoder or Value API. The default, identity and raw Runtime profiles
do not enable these capabilities. Cryptography, signature verification,
delegations, hash/domain behavior, certificate freshness and external bytes are
unchanged. Decoding or constructing a fixture never authenticates it.

`certification-cbor` additionally supplies `encode_certificate_untagged` and
`encode_hash_tree`. Use the untagged certificate operation where retained packet
bytes or hashes were produced by upstream `serde_cbor::to_vec`; existing
`encode_certificate` retains self-described framing. The `strict` submodule
contains the Host certificate/tree visitors with their existing `DecodeLimits`,
`DecodeError` and `ParseError` contract: unknown/duplicate fields, non-byte
payloads, malformed tree shapes, trailing data and upstream nesting limits still
refuse. This strict decoder is distinct from upstream certificate deserialization;
consumers must retain the decoder matching their current admission contract.

The native-only `ic-protocol` module exposes upstream transport identities and
bounded envelope encoding/decoding, strict call-content inspection, call and
read-state response decoding, controllers-leaf decoding and subnet-metrics
decoding. It uses transport types directly without selecting Agent or Runtime.
The transport supplier does select Candid and its printer on native hosts;
neither is introduced into the certification-only Wasm graph. Strict call-content
inspection rejects unknown content fields; its returned sender-info remains a
consumer admission decision. Never replace original signed bytes with an encoded
projection or infer submission/retry authority from a decoded response.

Select `ic-protocol-fixtures` only in native development dependencies for typed
query replies/rejections, read-state responses, controller/range/metrics leaves
and deliberate unknown/duplicate certificate fields. There is no arbitrary
Serde or CBOR value encoder. All Rust codec operations take caller-selected byte
limits; those limits are not total heap quotas or universal stack-safety claims.

Required consumer adoption after supplier publication:

| Consumer | Selection and migration | Consumer responsibilities retained |
| --- | --- | --- |
| Auth | `certification-cbor` for certificate/signature preparation and profiling; native fixture selection for protocol tests | Certified-root binding, witness/store policy, limits, signature/delegation/freshness verification |
| Query | `certification-cbor::strict` in production; `ic-protocol-fixtures` in native tests | Agent authentication, combined evidence budgets, exact callback arguments/replies, rejection metadata and malformed/deep fixture coverage |
| Blob native tests | `ic-protocol` packet readers and untagged certificate encoding; fixture selection where needed | Saved envelope/request-ID correlation, raw v4 certificate bytes, forged-proof refusal, recovery and revocation checks |
| Canic | Native protocol readers/fixtures and certificate encoding; use strict call inspection where unknown fields currently refuse | Sender-info refusal, expected call bindings, controller limits/uniqueness, original-envelope custody, paid-effect authority; Fleet/release-build commitments keep their existing bytes |
| Tooling | Move the strict decoder API's consumers to Platform; Testkit selects the native packet capability for read-state/metrics | Server lifecycle, deadlines, existing authentication behavior and imported-client provenance; remove the duplicate Host implementation without a forwarding owner |

Publish Platform before registry adoption and qualify each consumer's affected
contracts afterward. Source-only preparation or local overrides do not establish
published-package or installed consumer acceptance. No consumer release follows
automatically from this supplier implementation.

## Passive contract consumers (Types 0.18.1)

Types' `principal-external` now explicitly selects upstream Principal Serde.
This removes a hidden dependence on Candid feature unification for passive DTOs;
the owned and external identities, conversion behavior and bytes remain unchanged.
The correction is published in Types 0.18.1. Existing facade feature forwarding is
sufficient; Candid and external-protocol ownership remain in the facade.
See the [consumer boundary audit](audits/passive-contract-boundaries-2026-11-04.md)
for completed Platform work, required sibling feature/import migrations, old
IC Crates identity-cohort constraints and focused evidence. No sibling adoption
or release is implied.

## Upstream cryptographic dependency cut (0.22)

The embedded BLS curve/verifier, certification certificate/hash/tree code and
canister-signature creation/verifier bodies are removed, together with their
algorithm suites, vectors, provenance payloads and four reconstruction guards.
Use registry ic-verify-bls-signature 0.6, ic_bls12_381 0.10, ic-certification 3.2,
ic-canister-sig-creation 1.3.1 and ic-signature-verification 0.3. No cryptographic
primitive, key encoding, domain/hash or signature algorithm is implemented here.
Certification stays at 3.2 to share the native Agent/signature suppliers' identity.

Certificate/tree, BLS Signature and BLS error types now have registry identity;
consumers exchanging those Rust types must align with the Platform minor. The
native Agent accepts the same certificate directly, removing recursive copying
conversions. Principal/Account and Runtime identities remain unchanged.
CanisterSigPublicKey retains owned Principal/seed fields through a delegating
adapter; all DER/raw encoding and hash operations call upstream. PublicKey and
PrivateKey likewise delegate BLS operations. Public-key construction performs a
second upstream point parse and caches identity admission for repeated verification.

Keep #72 short-form DER framing admission before the upstream key parser. Registry
BLS 0.6.0 lacks #127's identity-key fix, so canister verification additionally
rejects identity roots and identity delegated subnet keys, using upstream point
operations. Integration regressions cover direct and both delegated levels.
No successful result or signing time is returned until all checks pass.

The adapter calls the upstream verifier first, then decodes the same envelope
and certificate bytes for delegated-key admission and optional signing-time
extraction. This repeats CBOR decoding because upstream exposes no evidence API. It retains
u64/nonminimal-encoding behavior and refuses duplicate, missing or malformed time;
delegation time is not signing time. Consumers retain budgets, trust and freshness.

Returning creation/verifier ownership upstream restores their transitive Candid
printer, raw IC0, CBOR, representation-independent hashing and lazy root-key cache.
These are packaging costs; Platform does not expose the upstream store or cache.
The unused direct development-only representation-independent-hash 4.x dependency
is removed. Raw Runtime/executor/IC0, certification-only and standalone BLS retain
their codec/host isolation. The extra upstream raw IC0 supplier in signature graphs
contains no executor; the exact Runtime singleton requirement is unchanged.

This is a maintenance reduction, not a measured Wasm-size or performance win.
Algorithm tests stay upstream; Platform retains byte/admission/integration tests.
Earlier embedded-source sections below are historical and superseded by this cut;
removed provenance links point to the preceding release. Runtime and Types do not
advance for this facade change.

The published 0.22.1 correction removes signature admission's redundant key
serialization/reparse through the same upstream checked point parser. Its envelope
projection mirrors both upstream fields, preserving packed-map and sequence forms
while skipping the already authenticated witness. Verification, identity-key
refusal and authenticated-time/freshness boundaries are unchanged. The
[matched Canic measurement](audits/platform-022-canic-wasm-2026-11-05.md) records
the remaining modest 0.22 overhead and focused Auth requalification. Those sizes
describe the pre-release local candidate; no size parity with 0.21 is claimed.

## Account byte-adapter cut (#138)

Types 0.18.0 changed Account’s optional subaccount to the standard
`serde_bytes` adapter. Rust fields remain owner: Principal followed by
subaccount: Option<[u8; 32]>. Principal remains bounded to 29 bytes. Postcard
None stays 00; Some changes from 01 plus 32 payload bytes to 01 20 plus those
bytes. JSON retains an array or null; CBOR now emits a byte string for Some.
Omitted optional map fields still decode to None; 31/33-byte subaccounts refuse.
Logical equality/ordering/hash/text and raw None versus explicit zero remain
distinct concepts. External ledger Candid DTOs and their bytes do not change.

This is an incompatible binary Serde contract, without a legacy lane or
automatic migration. Matching Wire schema must use optional bytes, min=max=32,
and generated browser fields must use Uint8Array. Requalify frame, allocation,
node and depth limits. Affected endpoints and clients require fresh unique
contract IDs through the existing endpoint id setting: default method-name IDs
and a package minor alone do not distinguish these payloads. An old Account
whose subaccount starts with 20, followed by a blob, can decode successfully as
a different Account and blob under the new layout. A regression retains this
collision; do not rely on malformed-input rejection to fence old messages.

Consumers resolve retained/in-flight operations under their existing recovery
policy and preserve original retry, signature and encoder-hash bytes. Endpoint
IDs do not fence unframed persisted Account data: inventory and handle those
stored formats separately. Platform does not own storage migration or Wire IDs.

The published Types 0.18 line supplies this cut; its current release is 0.18.2.
Align the facade/Runtime dependency selections where
the canonical Types identity changes, retaining one Types identity and one exact
Runtime guard. Do not advance unrelated release groups mechanically. Producer
completion does not require every consumer rollout; Wire owns its subsequent
schema/client adoption and registry-only installed qualification. Current local
Linux/Wasm proofs do not establish consumer rollout or macOS coverage.

## Supported compiler (#133)

Current source requires Rust 1.99.1 for every member of the independent Platform,
Runtime and Types groups. The minimum and pinned development/release toolchain
must agree; compiler admission checks and records actual Cargo/rustc identities.
Historical Rust 1.91 results remain dated evidence, not a support promise for
this source. Published versions are recorded in the group ledgers and handoff.

| Existing obligation | Current qualification |
| --- | --- |
| All five packages, tests, proc macros and doc tests | All-feature native tests on 1.99.1 |
| All-feature facade Wasm | `wasm-check` on 1.99.1 |
| Isolated capability, Principal/ledger/schema and raw/executor consumers | Same feature/target selections, once on 1.99.1 |
| Prepared calls | Raw, Wire and external Candid; native tests and Wasm checks |
| Installed Runtime owner | Raw/prepared × Wire/external Candid, driver tests and lifecycle assertions; downstream composition belongs to Scheduling |
| Archive/publication and release groups | Existing package, source, singleton and release gates retained |
| Native hosts | Linux, macOS Intel and macOS ARM retained |

`make msrv` delegates to the same native/Wasm targets rather than compiling them
again in CI. New qualification reads the workspace policy; it does not change
old benchmark receipts or the external IC source profile's pinned compiler.

## Independent Runtime owner (#131)

The workspace retains three independent groups, defined by
[the release policy](../ci/cargo-release-groups.json): Runtime (runtime, macros
and exact guard), Platform (facade), and Types. Published versions belong to their
ledgers and [current handoff](status/current.md), not a second version table here.
PocketIC's native client belongs to Testkit; it has no Platform package or group.

Runtime owns the sole IC0/executor/CDK implementation and prepared raw calls.
Its empty defaults and capability boundaries remain explicit: IC0 has no
application dependencies; executor adds SlotMap and the exact guard; runtime adds
raw calls and canonical Types; cdk adds the ordinary entrypoint generator.
Candid, crypto, certification, native clients and storage allocation stay outside
this owner. Preserve imported licenses, source guards, IC ABI and executor
semantics, including cancellation, cleanup and destruction outside borrows.

The facade's raw adapters delegate to Runtime. Ordinary macros belong only to
Runtime; the facade retains its explicit external-Candid bridge. Both use one
generator/executor and preserve fixed Wasm export names, guard ordering and codec
hooks. Renamed Runtime dependencies use the explicit crate-path option.

Select Runtime directly for execution, Types directly for identities, and the
facade for required external protocols/crypto/native clients. Timers and Wire
Endpoint select the same Runtime owner. The exact guard refuses incompatible
owners and an external executor beside the enabled embedded owner.

Downstream transfer is tracked in
[Scheduling #91 — Own Timers composition qualification against Platform Runtime](https://github.com/dragginzgame/ic-scheduling/issues/91).
Platform qualifies only its standalone Runtime, using its maintained lock and
prepared caches. Scheduling owns Timers composition against its selected Runtime;
Tooling owns its own consumer adoption. No scheduler dependency or downstream
version range gates Platform publication. Testkit remains native test infrastructure,
never a normal/build dependency of the shipped packages.

The standalone qualification lock participates in all three groups' transactional metadata
preparation/recovery. Every local package must match its owning group; missing,
duplicate or incompatible owner records refuse. Saved journals retain their own
policy/source identity. See [release validation](platform-release-validation.md)
for the current full gate and separate native/installed evidence obligations.

## IC0 stable memory infrastructure

Pending Runtime 0.18.1 completes the existing `ic0` feature with the safe
`stable64_grow(u64) -> u64`, `stable64_read(&mut [u8], u64)` and
`stable64_write(&[u8], u64)` exports beside `stable64_size`. Read/write take
the byte slice before the offset. The imported implementations, System API ABI,
bounds traps and `u64::MAX` growth-refusal result remain unchanged. The facade
delegates the same exports. No storage allocator, executor, application types,
new feature or second implementation is selected by this infrastructure boundary.
Production application storage remains governed by Storage.

The external IC0-only consumer compiles all four operations on native/Wasm and
requires exactly itself plus Runtime in its normal/build graph. Installed owner
qualification uses these public wrappers for growth, round trips, refused growth,
read/write bounds traps and rollback, while retaining CDK checkpoint paths.
Run `bash scripts/ci/check-runtime-ic0.sh`; after publication its
`--published X.Y.Z` mode additionally requires the exact registry owner.

## Selected canister management DTOs

Platform 0.18 introduced the canister-used subset of management 0.11.0.
With the 0.23.2 creation request, the same module/feature supplies
44 explicit public names now refer to Platform-owned types, backed by 52 complete
upstream declarations. The selection follows non-test Canic Core/control-plane
and Blob source references, plus the maintained Canic runtime-probe canister's
HTTP outcall contract. All supporting records, enum variants, optional fields,
Serde attributes and Candid function signatures are retained. This adopts no
transport, storage, signing implementation or application policy.

Native builds retain all 58 registry 0.11.0 exports and their identities.
Fourteen snapshot-transfer exports used only by native Backup/Testkit/tooling are
not available on Wasm. Do not assume registry/owned Rust identity across targets;
external Candid bytes and schemas remain compatible. The original exact supplier
constraint below continues to apply to native DTOs and is superseded only on Wasm.
Sibling source checkouts and release versions remain unchanged.

The owned definitions live in `management_canister_types/maintained` inside the
sole consumer library. Provenance records the exact source revision, selected
names and hashes of the audited sibling files. Apache-2.0 terms are retained.
Native tests compare all 52 selected schemas to the pinned original; independent
management wire fixtures and installed Wasm checks cover HTTP, history, status,
snapshot list/delete and lifecycle/signing envelopes. Native transfer tests retain
their original registry identities. No consumer adoption or live IC acceptance is
implied by this qualification.

The original 0.18 packaging selected Candid bignum/serde_bytes explicitly,
without the registry DTO package or its default-enabled printer. The final owned status-codec probe saves 55,386
bytes (6.6%) of stripped Wasm; the earlier manifest-only investigation saved
about 63 KB. Neither measures a Canic production binary.
The original packaging retained the same decoder and admission rules; some diagnostic type descriptions
use Debug formatting. Native tools retain the printer for diagnosis. Another
dependency explicitly selecting Candid's printer can re-enable it; graph checks
reject its implicit selection through the management-only Wasm profile.

## Current contracts for the historical migrations

The historical references below preserve old anchors for callers and provenance.
Their earlier ownership/version language is not the current contract:

- **Identity:** Types owns Principal and Account bytes, Serde and conversion
  behavior. Principal ordering is length-first; Account equality/order/hash use
  the effective zero subaccount while serialization preserves absent versus
  explicit zero. External Candid identities remain upstream. Wire owns schemas.
- **Runtime:** Runtime owns IC0, the executor, raw/prepared calls and ordinary
  entrypoints. Exact guard identity refuses multiple executor owners. Preserve
  the IC ABI, task generations, wake routing, cancellation, callback-trap cleanup
  and destructor behavior. Raw executor/IC0 selections remain Candid-free.
- **Calls and entrypoints:** arguments/replies are explicit raw bytes. Codecs,
  retries, timers, budgets and lifecycle policy remain consumer-owned. Prepared
  calls are inert until polled and consumed once; dropping a future does not
  prove a remote effect was cancelled. Ordinary macros require explicit codecs;
  external Candid macros remain a separately selected facade adapter.
- **Candid admission:** bounded decoders and external endpoints refuse more than
  16 constructed wire levels before upstream decoding, including unused type
  definitions/skipped values. Retain existing byte/work/header/type quotas,
  exact versus compatible arity, numeric ranges and typed errors. Configured
  low-level upstream APIs retain their behavior. This is not a Candid fork or a
  universal claim about arbitrary trusted Rust schemas/custom visitors.
- **Certification and signatures:** implementations and identities belong to
  registry suppliers. Keep no embedded BLS/curve/verifier bodies or algorithm
  suites. Preserve DER admission, identity-key refusal at root and delegated
  keys, upstream verification and signing-time extraction only after successful
  verification from the signing certificate. Consumers own trust/freshness.
- **Crypto adapters:** upstream BLS selects alloc/groups without RNG/defaults.
  Standard ECDSA/Ed25519 stay upstream. Preserve the ECDSA bigint security floor
  and the qualified Wasm backend; do not alter algorithms, encodings or domains.
  Native Agent retains its upstream graph and advisory #50 remains deferred.
- **CBOR and IC packets:** expose typed codecs, preserving strict versus ordinary
  admission and caller byte limits. Preserve framing, trailing-data rejection,
  complete u128 metrics and upstream certificate/tree identities. Decoding alone
  grants no authentication, retry, polling or submission authority.
- **External DTOs:** preserve external Candid/Serde bytes, fields, numeric ranges
  and function references. Wasm management uses the maintained 0.11 selection;
  native management retains registry identities. Ledger declarations retain
  their notices and protected value-hash bodies. No storage owner is added.
- **Native and browser owners:** Testkit owns the PocketIC client and generic
  gzip belongs to Host Artifacts in Tooling. Profiler's opt-in provider retains
  its pinned source contract and separate native qualification. The browser SDK
  uses explicit subpaths/upstream identities; consumers own authentication and
  effect policy. No generic CBOR export is provided for callers.
- **Qualification:** use the current compiler and full selected-group release
  gate. Keep immutable input/receipt identities, recovery custody and separate
  installed/Linux/macOS evidence. Old receipts qualify only their supplying
  source. Historical pilot stages are not a release shortcut.

The [inventory](ic-inventory.md), [external contracts](external-contracts.md),
[development guide](development-and-qualification.md) and API/provenance sources
provide current selections. The following links retain historical rationale and
migration evidence without maintaining another active implementation contract.

## Candid codec packaging investigation

Historical rationale/evidence: [record](history/migrations-through-0.26.md#candid-codec-packaging-investigation).

## Runtime-only Cargo singleton guard (#103)

Historical rationale/evidence: [record](history/migrations-through-0.26.md#runtime-only-cargo-singleton-guard-103).

## Embedded IC0 and executor

Historical rationale/evidence: [record](history/migrations-through-0.26.md#embedded-ic0-and-executor).

## Minimal Principal and selected ICRC packaging

Historical rationale/evidence: [record](history/migrations-through-0.26.md#minimal-principal-and-selected-icrc-packaging).

## Shared ICRC-10 discovery record (since 0.7.5)

Historical rationale/evidence: [record](history/migrations-through-0.26.md#shared-icrc-10-discovery-record-since-075).

## External endpoint and protocol codecs (#78, #79)

Historical rationale/evidence: [record](history/migrations-through-0.26.md#external-endpoint-and-protocol-codecs-78-79).

### Canic external entrypoints

Historical rationale/evidence: [record](history/migrations-through-0.26.md#canic-external-entrypoints).

### Canic and Host external calls

Historical rationale/evidence: [record](history/migrations-through-0.26.md#canic-and-host-external-calls).

### Qualification and adoption

Historical rationale/evidence: [record](history/migrations-through-0.26.md#qualification-and-adoption).

## DER framing admission (#72)

Historical rationale/evidence: [record](history/migrations-through-0.26.md#der-framing-admission-72).

## External Candid nesting qualification (#73)

Historical rationale/evidence: [record](history/migrations-through-0.26.md#external-candid-nesting-qualification-73).

### Admission cost (2026-10-29)

Historical rationale/evidence: [record](history/migrations-through-0.26.md#admission-cost-2026-10-29).

## Host gzip boundary

Historical rationale/evidence: [record](history/migrations-through-0.26.md#host-gzip-boundary).

## External DTO additions for Canic

Historical rationale/evidence: [record](history/migrations-through-0.26.md#external-dto-additions-for-canic).

## Non-cryptographic size qualification (0.7.1)

Historical rationale/evidence: [record](history/migrations-through-0.26.md#non-cryptographic-size-qualification-071).

## Instrumented PocketIC source provider

Historical rationale/evidence: [record](history/migrations-through-0.26.md#instrumented-pocketic-source-provider).

## Retired BLAKE3 facade (Platform 0.26)

Historical rationale/evidence: [record](history/migrations-through-0.26.md#retired-blake3-facade-platform-026).

## Browser SDK Core boundary

Historical rationale/evidence: [record](history/migrations-through-0.26.md#browser-sdk-core-boundary).

### Typed browser call-ingress inspection

Historical rationale/evidence: [record](history/migrations-through-0.26.md#typed-browser-call-ingress-inspection).

### Blob 0.48 and SDK 6

Historical rationale/evidence: [record](history/migrations-through-0.26.md#blob-048-and-sdk-6).

## Owned Principal and Account Wire metadata

Historical rationale/evidence: [record](history/migrations-through-0.26.md#owned-principal-and-account-wire-metadata).

### Preserved schema and byte contract

Historical rationale/evidence: [record](history/migrations-through-0.26.md#preserved-schema-and-byte-contract).

### Wire transfer evidence and release order

Historical rationale/evidence: [record](history/migrations-through-0.26.md#wire-transfer-evidence-and-release-order).

## Authenticated canister-signature time (pending 0.6.14)

Historical rationale/evidence: [record](history/migrations-through-0.26.md#authenticated-canister-signature-time-pending-0614).

## Authorized owned-endpoint Wire migration

Historical rationale/evidence: [record](history/migrations-through-0.26.md#authorized-owned-endpoint-wire-migration).

## Stable-structures substrate

Historical rationale/evidence: [record](history/migrations-through-0.26.md#stable-structures-substrate).

## Explicit external Candid entrypoints (0.6.2 qualification)

Historical rationale/evidence: [record](history/migrations-through-0.26.md#explicit-external-candid-entrypoints-062-qualification).

## 0.6.2 strict reply helpers

Historical rationale/evidence: [record](history/migrations-through-0.26.md#062-strict-reply-helpers).

## Pending 0.6 feature boundary

Historical rationale/evidence: [record](history/migrations-through-0.26.md#pending-06-feature-boundary).

## Pending 0.5 retirement

Historical rationale/evidence: [record](history/migrations-through-0.26.md#pending-05-retirement).

## PocketIC maintenance boundary

Historical rationale/evidence: [record](history/migrations-through-0.26.md#pocketic-maintenance-boundary).

## Owned Principal migration (0.3)

Historical rationale/evidence: [record](history/migrations-through-0.26.md#owned-principal-migration-03).

## Qualified Candid surface

Historical rationale/evidence: [record](history/migrations-through-0.26.md#qualified-candid-surface).

## Consumer audit evidence

Historical rationale/evidence: [record](history/migrations-through-0.26.md#consumer-audit-evidence).

### Existing decoder and service contracts

Historical rationale/evidence: [record](history/migrations-through-0.26.md#existing-decoder-and-service-contracts).

## Maintained certification (pending 0.16)

Historical rationale/evidence: [record](history/migrations-through-0.26.md#maintained-certification-pending-016).

## Certification extension

Historical rationale/evidence: [record](history/migrations-through-0.26.md#certification-extension).

## Owned Principal in canister signatures (pending 0.16)

Historical rationale/evidence: [record](history/migrations-through-0.26.md#owned-principal-in-canister-signatures-pending-016).

## Maintained BLS curve (pending 0.16)

Historical rationale/evidence: [record](history/migrations-through-0.26.md#maintained-bls-curve-pending-016).

## Canister signature and BLS extension

Historical rationale/evidence: [record](history/migrations-through-0.26.md#canister-signature-and-bls-extension).

## Native Agent extension

Historical rationale/evidence: [record](history/migrations-through-0.26.md#native-agent-extension).

## Testkit and Query migration

Historical rationale/evidence: [record](history/migrations-through-0.26.md#testkit-and-query-migration).

## Runtime trace and required qualification

Historical rationale/evidence: [record](history/migrations-through-0.26.md#runtime-trace-and-required-qualification).

### Maintained runtime and Wire entrypoint adoption

Historical rationale/evidence: [record](history/migrations-through-0.26.md#maintained-runtime-and-wire-entrypoint-adoption).

#### Protocol and feature selection

Historical rationale/evidence: [record](history/migrations-through-0.26.md#protocol-and-feature-selection).

#### Runtime ownership and compatibility

Historical rationale/evidence: [record](history/migrations-through-0.26.md#runtime-ownership-and-compatibility).

#### Runtime maintenance and recovery

Historical rationale/evidence: [record](history/migrations-through-0.26.md#runtime-maintenance-and-recovery).

#### Consumer migration and qualification

Historical rationale/evidence: [record](history/migrations-through-0.26.md#consumer-migration-and-qualification).

## Host parsing and aggregate admission

Historical rationale/evidence: [record](history/migrations-through-0.26.md#host-parsing-and-aggregate-admission).

### Pending 0.7.0 compiler-only retirement (#38)

Historical rationale/evidence: [record](history/migrations-through-0.26.md#pending-070-compiler-only-retirement-38).

## Management-canister DTOs

Historical rationale/evidence: [record](history/migrations-through-0.26.md#management-canister-dtos).

## ICRC ledger types

Historical rationale/evidence: [record](history/migrations-through-0.26.md#icrc-ledger-types).

## Evidence and isolation

Historical rationale/evidence: [record](history/migrations-through-0.26.md#evidence-and-isolation).

## Source provenance

Historical rationale/evidence: [record](history/migrations-through-0.26.md#source-provenance).

## Retired native secp256k1 public derivation (Platform 0.24.0)

Historical rationale/evidence: [record](history/migrations-through-0.26.md#retired-native-secp256k1-public-derivation-platform-0240).

## Qualified dependency-reduction experiments

Historical rationale/evidence: [record](history/migrations-through-0.26.md#qualified-dependency-reduction-experiments).

## Scaffolding-only packaging qualification

Historical rationale/evidence: [record](history/migrations-through-0.26.md#scaffolding-only-packaging-qualification).

## Standard Ed25519 maintenance boundary

Historical rationale/evidence: [record](history/migrations-through-0.26.md#standard-ed25519-maintenance-boundary).

## Standard secp256k1 ECDSA maintenance boundary

Historical rationale/evidence: [record](history/migrations-through-0.26.md#standard-secp256k1-ecdsa-maintenance-boundary).

## Owned account migration (0.3)

Historical rationale/evidence: [record](history/migrations-through-0.26.md#owned-account-migration-03).

## Runtime codes and signature packaging (0.4)

Historical rationale/evidence: [record](history/migrations-through-0.26.md#runtime-codes-and-signature-packaging-04).

## Transport and codec separation (0.4)

Historical rationale/evidence: [record](history/migrations-through-0.26.md#transport-and-codec-separation-04).

## Canic external-contract closeout exports

Historical rationale/evidence: [record](history/migrations-through-0.26.md#canic-external-contract-closeout-exports).

## Prepared one-shot calls (issue #75)

Historical rationale/evidence: [record](history/migrations-through-0.26.md#prepared-one-shot-calls-issue-75).

## Stable owned identity package (#118)

Historical rationale/evidence: [record](history/migrations-through-0.26.md#stable-owned-identity-package-118).

### Release command routing

Historical rationale/evidence: [record](history/migrations-through-0.26.md#release-command-routing).

## Maintained PocketIC Rust client

Historical rationale/evidence: [record](history/migrations-through-0.26.md#maintained-pocketic-rust-client).

### Recursive codec mappings and signed carrier (unreleased)

The owned record/variant macros now resolve stable schema anchors at runtime,
allowing recursive Rust values through vectors and maps without compile-time
constant cycles. Custom `Type::Deferred` resolvers must always return the same
static, non-deferred schema; chained/self-returning anchors fail closed. This is
trusted mapping code, not input-controlled schema execution. Wire registration
and function-signature comparison resolve the same anchors. Existing incoming
16-level admission and caller budgets remain in force.

`candid_codec::Int` is now an owned newtype around `num_bigint::BigInt`, matching
`Nat` rather than aliasing BigInt. Use `.0` for magnitude arithmetic; BigInt itself
still implements the codec. Primitive conversions, decimal parsing/display and
optional explicit registry conversions are provided. Wire and Serde serialization
match the registry carrier, including negative and greater-than-128-bit values.
Serde admission retains upstream signed/natural byte tags; empty tagged buffers
return an error. Arrays use vector wire types with exact-length admission; maps
use vectors of key/value records and retain Serde's last-duplicate-wins behavior.
Allocation accounting remains cumulative logical storage, not measured heap use.
The ICRC adoption below uses these mappings and preserves its upstream hash-input encoding.

### ICRC and Cycles archive adoption (unreleased)

`icrc-ledger-types` now selects `candid-codec` and external Principal conversions,
without selecting registry Candid. ICRC-1/2/3, ICRC-10 and ICRC-21 DTOs, recursive
Value/ICRC3Value and typed query archive callbacks all implement the owned codec.
Adding `cycles-ledger-types` supplies its distinct recursive provider archive
schema, preserving legacy `Value::Nat64`; it does not convert blocks to ICRC3Value.

Use owned `candid_codec::{Nat, Int}` for DTO field annotations. Magnitudes remain
unbounded and `.0` retains num-bigint identity. `QueryArchiveFn` keeps its fields,
constructor, ordering and serialized representation, with no mandatory CandidType
bound; its reference conversion uses owned `references::Func`. Upstream trait/Func bridges now compile only for unit-test oracles. There is
no production legacy feature or protocol forwarding surface. This is a Rust
carrier-identity migration, not an external protocol or trust-policy change.

The owned writer merges repeated type declarations without unfolding recursive
graphs. The selected ICRC fixture corpus matches upstream encoding byte for byte,
and both readers cross-decode. This is not a universal canonicalization guarantee
for arbitrary independently declared recursive Rust schemas. Decoder budgets,
16-level admission, callback signature checks and consumer authority remain intact.

Ledger hash methods retain their original bodies and vectors. Their natural-number
hash input uses the unchanged Candid 0.10.40 encoder, now a narrow attributed import
with an independent source/license guard. It calls registry leb128 and num-bigint;
it does not call our new binary writer. SHA-256, account checksums and formatting
suppliers remain legitimate ICRC dependencies. No new cryptographic algorithm is
implemented. See the ICRC NOTICE and provenance for the exact supplier revision.

### Management owned-codec adoption (unreleased)

The management-only Wasm graph now selects `candid-codec`, external Principal
conversions and serde_bytes, without registry Candid or its derive/printer stack.
Native builds deliberately retain the 58-name registry surface and original type
identities. No native-only snapshot-transfer types are added to Wasm.

On Wasm, Nat and Reserved fields use `candid_codec::{Nat, Reserved}`. Nat retains
unbounded magnitude, `.0` and Serde representations. Reserved encodes the Candid
top type and validates/skips any incoming value under the existing byte, work,
allocation and 16-level admission limits; it is not unchecked byte disposal.
Missing reserved record fields remain accepted. Native field carriers remain
registry types. Consumers must choose their codec explicitly for the target;
existing native tooling is not redirected onto a new decoder.

The Wasm `TransformFunc` keeps `new(ExternalPrincipal, String)` and its `.0`
principal/method payload, now carried by owned `references::Func`. The reader
checks the full query signature; it does not authorize the callback. The legacy CandidType and external Func conversion bridges are removed.
No HTTP transport, retries, signing, management policy or freshness behavior is
added or changed.

All 52 maintained schema type tables are compared byte for byte against registry
0.11.0. Independent history/status/snapshot fixtures, creation values above u128,
and HTTP transform payloads cross-decode and reject truncated inputs. The import
source guard reconstructs the pre-adoption declarations by reversing the explicit
carrier imports, conditional derives and callback macro extraction; it normalizes
only whitespace/comments and optional trailing commas in derive attributes.
Original source and license digests remain recorded. Earlier installed evidence
covers the former codec; new installed qualification remains separate.

### Explicit owned external endpoints (unreleased)

Select `candid-codec-endpoint` and use `candid_codec::endpoint::{query, update}`
for required pre-existing external Candid interfaces. This selection enables the
owned codec, small argument framing, the existing Runtime and the shared macros;
it does not enable upstream Candid, its derives/parser/printer, or another executor.
Owned application endpoints continue to use Wire. This is not a transport, retry,
authorization or schema-publication owner.

```rust,ignore
use ic_platform::candid_codec::{decoder::Limits, endpoint};
const LIMITS: Limits = Limits {
    bytes: 1024, header: 512, types: 32, work: 10000, allocation: 10000,
};
const REPLY_BYTES: usize = 1024;
#[endpoint::query(
    limits = "LIMITS", reply_bytes = "REPLY_BYTES",
    arity = "exact", empty_args = "reject"
)]
fn external(value: u64) -> u64 { value }
```

These numbers are illustrative, not recommended defaults. Consumers choose owned
work/allocation budgets, reply size and arity explicitly; upstream decoding and
skipping quotas cannot be translated mechanically. `compatible` retains typed
missing arguments and validated surplus skipping; `exact` refuses argument-count
changes. `unit` treats absent bytes as the zero-argument envelope, not one null,
and still enforces arity, work and header budgets. Scalar byte admission occurs
before copying the request. Guards retain their order before admission/copying;
argument and reply codec failures trap through the existing runtime. Consumers
retain guard policy and all callback/authority/freshness decisions.

The attribute supports the same name, guard, composite-query and renamed-platform
paths. A zero Rust return encodes zero arguments, a tuple return encodes multiple
arguments, and a scalar/unit alias encodes one value. Async execution and replies
delegate to the existing generator. Native service registration is deliberately
absent from this selection; maintain checked external schemas separately. The legacy `candid-endpoint` feature, `candid::endpoint` module and
`candid_query`/`candid_update` generator exports are removed. Callers must change
imports and select every owned budget explicitly; no endpoint silently switches
codecs. The separate legacy codec/DTO bridge is also removed.

This batch changes Platform and the macros in Runtime's coordinated release group;
Types and the runtime implementation are unchanged. Source consumer checks prove native behavior and Candid-free Wasm compilation.
Installed Linux PocketIC qualification passes at target/cdk-runtime.8ZP502:
guard ordering, malformed/budget refusal, replies, composite queries, rollback,
management awaits and existing checkpoint bytes. The installed external Wasm
fixture now excludes Candid; native management reference tests retain registry
identities. Consumer-selected budgets remain required for adoption.
