# IC dependency and maintenance inventory

This is Platform's boundary inventory, not a fleet adoption tracker. Manifest
facts below are checked by `make check-inventory` in CI. Requirements are declared
ranges/pins, not resolved versions; Cargo.lock and the existing feature-graph
checks own resolution and isolation. No default features are enabled.

The [individual export audit](audits/export-usage-2026-10-21.md) adds dated
per-symbol evidence for 393 facade exports and owned methods/constants. It
distinguishes sibling references, generated-code requirements, public contracts
and exports with only Platform fixture evidence or no resolved consumer.
Upstream type members are not individually covered; this is not removal approval.

## APIs, consumers and maintenance ownership

Pending 0.7.5 adds `icrc_ledger_types::icrc10::SupportedStandard` under existing
`icrc-ledger-types`. Platform maintains the named external discovery record from
ICRC protocol facts; it is not a registry Rust identity. The feature now names
`dep:candid` for its derive, without selecting the facade's codec feature.
The existing registry ledger edge already supplies that Candid identity/features;
native/Wasm dependency identities and effective-feature sets are unchanged.
Default/executor isolation remains. Canic owns opt-in mounting, advertised
standards and consent policy; its [integration issue #706](https://github.com/dragginzgame/canic/issues/706)
tracks adoption after supplying publication. No consumer adoption or size saving
is inferred from this export.

Pending 0.7.5 also adds a fixed 16-level admission ceiling in bounded Candid
helpers, protocol decoders and external endpoint adapters, plus
`candid::nesting::{MAX_DEPTH, NestingError, check}` and typed error accessors.
Platform owns this bounded nonrecursive framing admission; Candid still owns
decoding, subtyping and external identities. Work admission uses existing caller
quotas; there is no separate policy object or macro option. No dependency/feature
is added. Default, executor and runtime isolation remains unchanged. See the
[bounded wire contract](migration.md#external-candid-nesting-qualification-73).

Source release 0.7.4 adds `candid-endpoint` for explicitly external query/update codec
adapters over the existing generator/executor. It selects `cdk` and `candid`,
never a parser or service printer. Native DID generation selects `candid-service`
separately. Consumers own guards, names, limits and lifecycle; Platform owns only
the shared codec/metadata mechanics. Existing raw/Wire entrypoints are unchanged.
The `candid::protocols` surface is transport-free and uses only selected DTO
features. Cycles Ledger's provider block schema requires both Cycles and ICRC
features to preserve legacy `Value::Nat64` and upstream callback identities.
No consumer adoption, depth protection or dependency/Wasm reduction is claimed.

`ic-call` is the authorized #75 consumer crate for prepared one-shot requests.
It depends only on `ic-platform` with `runtime`, has no default or codec features,
and owns method/argument custody. Physical calls, response/error identities,
cost calculation, callbacks and executor remain with Platform. Canic #700 needs
owned dynamic methods; other audited callers may keep the raw API. Codecs, metrics,
budgets and durable delivery stay consumer-owned. See the
[call contract](../crates/ic-call/README.md) and
[migration](migration.md#prepared-one-shot-calls-issue-75).

The optional native `gzip` surface explicitly exports `encode_gzip`, `decode_gzip`
and `GzipError` from published IC Tooling `ic-host-artifacts` 0.69.1 for IC Profiler.
Tooling retains codec implementation; callers retain levels, limits and payload
admission. No backend types, default/Wasm dependency or IC runtime edge is added.

The published 0.6.20 external DTO additions are maintained from pinned protocol
schemas: `cmc_types`, `icp_ledger_types`, `cycles_ledger_types`,
`nns_registry_types` and native-only `certified_assets_types`. Each has its own
optional `*-types` feature selecting Candid, without service engines, host
transport, executor or storage dependencies. Canic Core/Host are the requesting
consumers; no sibling adoption is claimed. Complete source/license provenance,
intentional projection differences and migration mappings are in the
[external contract guide](external-contracts.md). These are Platform-maintained
Rust DTO identities, not re-exported upstream Rust structs. Required external
Principal/Nat/function and wire identities remain Candid's.

The separate ESM package `packages/browser` owns the browser SDK Core boundary
under published npm identity `@tokostudios/ic-platform` 0.1.1. Its sole production
dependency is exactly `@icp-sdk/core` 6.1.1 for Blob's local 0.48 graph.
Canic's SDK-5/Auth-8 handoff needs a coordinated migration before adoption.
Explicit subpaths expose Principal, Ed25519/secp256k1
identities, Agent/request APIs, certificates and required external Candid.
The published 0.1.1 #59 browser extension adds explicit upstream `Endpoint`,
`SubmitRequestType`, `ReadRequestType` and `Expiry` identities under `/agent`;
published 0.1.0 does not supply them. No SDK selection or crypto body changes.
Owned non-cryptographic adapters cover canonical text, checked key-pair import,
explicit trust/Agent options and raw query errors. SDK crypto, hashing, key
encoding and classes remain upstream; trust, limits, intents and application
policies remain consumer-owned. Wire retains owned codecs and generators.

`make check-browser` checks the npm selections, real provider types and packed
consumer/bundle contract; `make check-browser-runtime` adds Chromium page/worker
qualification. The Rust-only manifest block below does not inventory npm packages.
Its separate lock also includes TypeScript, bundler, browser and Caffeine/Auth
test dependencies; none are Rust or production browser dependencies. npm supply
is verified; sibling adoption is not claimed. See the
[browser migration contract](migration.md#browser-sdk-core-boundary).
Browser CI at `8d80630` passes on Linux and native macOS Intel/Apple Silicon.
The maintainer selected the `tokostudios` scope; final-name packed/browser checks
pass locally on Linux. Public registry metadata/archive and a fresh registry-only
consumer were verified on 2026-10-26. Prior macOS CI used the earlier unpublished
name with unchanged runtime code. These producer checks do not qualify
installed IC, live provider effects or consumer adoption.

The optional native-only `wire-schema` feature adds the upstream Wire metadata
trait to owned Principal and Account. Principal delegates to Wire's bounded
byte-buffer schema; Account describes its ordered owner and optional fixed
32-byte subaccount fields, preserving absent versus explicit zero.
Platform #48's requested DTO schema integration uses the same Principal identity;
consumer adoption is not yet claimed. The production alias `ic-wire-metadata`
selects Wire 0.7.19 metadata independently of the tooling-only Wire 0.7.19
runtime fixture and its separate qualification lock. Wire owns schema/TypeScript generation; Platform owns this
projection of its existing serialization. Wasm, default and executor-only graphs
exclude the added Wire/JSON dependency edges. All features now require Rust 1.91.

IC Profiler selects the opt-in Git source profile at `native/ic-profiler`, which
owns the instrumented IC pin, adapters, patches and patched PocketIC client.
The recorder/viewer remain consumer-owned. Native workspace/Bazel inputs supply
published IC Metrics 0.17 counters with defaults disabled, distinct from internal
replica metrics 0.9. These inputs add no Platform Cargo feature or default/Wasm
dependency. The registry `pocket-ic` feature is
unchanged; no instrumented Rust export is added to that feature. The
[migration contract](migration.md#instrumented-pocketic-source-provider) records
the source-provider interface and the remaining native acceptance under
[#65 — Instrumented PocketIC boundary for IC Profiler](https://github.com/dragginzgame/ic-platform/issues/65).

Paths are relative to `crates/ic-platform/src` unless otherwise stated. Each
module's explicit exports are authoritative; this table summarizes capabilities,
not every symbol. Additions to exports require reviewing this table and the
[migration contract](migration.md). The manifest check does not detect semantic
API changes or automatically establish whether sibling source still uses an API.

| Upstream / owned code | Public surface and source | Observed consumers | Maintenance responsibility |
| --- | --- | --- | --- |
| `blake3` | optional `blake3::{hash,Hasher,Hash}` | IcyDB currently uses upstream directly for predicate fingerprints; adoption remains consumer-owned | Platform selects unchanged upstream hashing with `std` only. Consumers own domain/format policy; no SHA-256 protocol replacement or size saving is implied. |
| `ic_principal`; owned Principal/Account | `principal`, `account`, root Principal/Account/Subaccount in their `mod.rs` files | IcyDB schema/guards; Auth protocol types; Relay; most runtime clients | Platform owns parsing, formatting, account semantics and conversions. External principal identity and self-authentication crypto remain upstream. |
| `ic0`, `ic-cdk-executor`; imported CDK adapters and owned generator | `ic0`, `executor`, `cdk::{api,call,futures}`; `crates/ic-platform-macros/src` | Timers `src/platform/{ffi,backend}.rs`; Canic entrypoints/core; Blob client transport; IcyDB runtime counters; Relay canisters | Platform maintains non-crypto adapters/generator. Registry executor and system ABI stay pinned; callback/trap semantics require installed qualification. No full `ic-cdk` dependency. |
| `candid` | `candid::{contract,decoder}` and owned bounded decoder helpers | Query external DTOs; Backup management calls; Canic system calls; Tooling Host/Testkit | Upstream codec/derive identities; Platform owns narrow exposure and decoder adapters. Consumers own limits, arity and retry policy. Owned endpoints use Wire. |
| `candid_parser`; Candid printer | native external parsing/typechecking and optional `candid-service` exports | Tooling Host's external service inspection; Canic external arguments/testing | 0.7.0 removed JS/TS compiler and merged-program exports after Wire adoption. Upstream parser/generator scaffolding remains transitive; no size/removal claim. |
| `ic-agent` | native `agent` builders, identities, request/error types | Query `src/agent.rs`; Tooling Host `src/agent`; Backup Agent; Blob CLI; Canic Host | Upstream transport implementation, no fork. Consumers own retries/authentication. Platform owns explicit exposure only. |
| `pocket-ic` | native `pocket_ic` client/DTO identities | Tooling Testkit `src/{lib.rs,pic/transport.rs}` | Upstream simulator client; Testkit owns server provisioning, compatibility, lifecycle and transport policy. |
| `ic-management-canister-types` | `management_canister_types` lifecycle/settings/ECDSA/history/snapshot/status DTOs | Canic Core/Host/Control Plane; Backup snapshot/submission modules | Upstream exact DTO identity and Candid format. Platform pin does not transfer IC protocol ownership. |
| `icrc-ledger-types` | `icrc_ledger_types` account/metadata/transfer/consent/archive DTOs | Query `src/icrc/{ledger.rs,live}`; Canic Host | Upstream external ledger formats, distinct from owned Account. No ledger/storage policy in Platform. |
| `ic-certification` | `certification` certificate/hash-tree types and operations | Tooling Host certificate decoder; Query certification; Auth signature store/verification; Canic | Upstream tree/hash implementations. Host owns bounded CBOR decoding; consumers own trust roots and verification policy. |
| `ic-verify-bls-signature` | `bls`; imported verifier dependency | Auth and Canic test keys; Auth canister verification | Upstream cryptography unchanged. No in-house primitive replacement. |
| `k256`, `curve25519-dalek`, `ed25519-dalek` | `ecdsa`, `ed25519` | Auth `src/{chain_key_signature,portable_signature}.rs`; Canic tests | General-purpose upstream crypto, not DFINITY-only code. Platform controls selected identities/features, not algorithms. |
| `ic-secp256k1` | native `secp256k1` public derivation/types | Canic Core native chain-key configuration and public-key fixtures | Upstream key derivation and encoding remain unchanged. Native-only explicit boundary. |
| Imported `ic-canister-sig-creation` / `ic-signature-verification` | `canister_sig_creation`, `signature_verification`; private `signature_upstream` | Auth `src/{signature_store,canister_signature}.rs` | Platform owns packaging, DER framing admission, evidence extraction and import scaffolding. The additive verifier returns authenticated signing time; consumers own freshness. Original fingerprints reconstruct after removing the single admitted framing call; crypto bodies, vectors and licenses remain protected. |
| `ic-stable-structures` | `stable_structures` collections, Memory/Storable and `memory_manager::{MemoryId,MemoryManager,VirtualMemory}` | Persistence Storage `src/lib.rs`; Memory runtime and stable-cell modules | Upstream substrate implementation. Memory owns layout validation, growth preflight, allocation and exclusive leases; Storage owns adapters. |
| `ic-representation-independent-hash` | development dependency, signature reference tests only | Platform's own tests | Upstream comparison oracle; not a normal dependency. |

Generic support dependencies (Serde, SHA-2, encoding/checksum helpers, error and
proc-macro libraries) are also included in the manifest inventory so additions
cannot escape review by having a non-IC name. SHA-2 and upstream cryptographic
primitives remain outside our implementation authority. Tooling-only fixture
workspaces additionally use owned Wire, Timers and Testkit; their locks and runtime
graph admission remain separate from the two-package workspace inventory.

Imported-source ownership is recorded beside the code in
[signature notices](../crates/ic-platform/src/signature_upstream/NOTICE.md),
[runtime notices](../crates/ic-platform/src/cdk/upstream/NOTICE.md),
[Principal notices](../crates/ic-platform/src/principal/NOTICE.md) and
[Account notices](../crates/ic-platform/src/account/NOTICE.md).
Existing signature/runtime fingerprint checks enforce their respective admitted
source changes. Do not refresh fingerprints to approve cryptographic changes.

## Consumer evidence and its limits

Read-only source/manifest review on 2026-10-21. Revisions identify HEAD; **dirty**
means observed working-tree contents can differ from that commit. These are
usage examples, including tests and optional paths, not frequency measurements,
proof of deployment, or a claim that every downstream consumer was discovered.

| Checkout | HEAD | State at inspection |
| --- | --- | --- |
| ic-tooling (current Host/Testkit owner) | e5b40951 | dirty |
| ic-persistence | fbe1208 | dirty; inspected local Memory/Storage adoption |
| ic-auth | 5d7da69 | dirty |
| ic-query | d5ea7b77 | clean |
| ic-timers | 6c3aa21 | dirty |
| canic | c63bdca65 | dirty |
| icydb | 2a3e1efa5 | dirty |
| ic-backup | 35f2d03 | clean |
| ic-blob-storage | 0050490 | clean |
| ic-relay | d7c1915 | clean |

Also inspected Jobs `7d1f808` (Principal in canister tests), Metrics `b6bd4e3`
and Wire `83f90a4` (no Platform imports found in their searched crate sources).
The older Host checkout `80cc30a` is dirty and still declares Platform 0.5;
it is not evidence that current Tooling requires that old selection. Toko was
excluded as previously requested. No sibling was built, edited or qualified here.

Repeat consumer review with `rg -n 'ic-platform|ic_platform::'` over the chosen
root/member Cargo.toml and Rust source files, recording HEAD and dirty state.
Review aliases, forwarded feature selections and test-only tables before calling
an API unused. Repository CI checks only local facts; periodic sibling re-audits
remain necessary and should update this dated evidence rather than imply live
fleet monitoring. GitHub remains the only follow-up issue tracker.

## Checked manifest facts

Run `bash scripts/ci/check-platform-inventory.sh --print` after reviewing a
manifest change and replace only the marked block. Check mode is read-only and
uses offline, locked, no-dependency Cargo metadata. It covers both workspace
packages, every feature edge and every direct dependency's requirement, kind,
target, optional/default status and selected features. It also checks that the
README feature table lists every non-default feature exactly once. Local package release
versions are deliberately represented as `workspace-owned` to avoid release-only
document churn. This is not a transitive dependency, source-integrity or Wasm-size
report; retain the existing graph, pin and imported-source gates.

<!-- BEGIN MANIFEST INVENTORY -->
| Package | Feature | Enables |
| --- | --- | --- |
| ic-call | default |  |
| ic-platform | agent | dep:ic-agent, principal |
| ic-platform | blake3 | dep:blake3 |
| ic-platform | bls | dep:ic-verify-bls-signature |
| ic-platform | candid | candid/bignum, dep:candid, principal |
| ic-platform | candid-endpoint | candid, cdk |
| ic-platform | candid-parser | dep:candid_parser, principal |
| ic-platform | candid-service | candid, candid/printer |
| ic-platform | canister-sig-creation | dep:ic-certification, dep:sha2, principal |
| ic-platform | cdk | dep:ic-platform-macros, runtime |
| ic-platform | certification | dep:ic-certification |
| ic-platform | certified-assets-types | candid |
| ic-platform | cmc-types | candid |
| ic-platform | cycles-ledger-types | candid |
| ic-platform | default |  |
| ic-platform | ecdsa | dep:k256 |
| ic-platform | ed25519 | dep:curve25519-dalek, dep:ed25519-dalek |
| ic-platform | executor | dep:ic-cdk-executor |
| ic-platform | gzip | dep:ic-host-artifacts |
| ic-platform | ic0 | dep:ic0 |
| ic-platform | icp-ledger-types | candid |
| ic-platform | icrc-ledger-types | dep:candid, dep:icrc-ledger-types, principal |
| ic-platform | management-canister-types | dep:ic-management-canister-types, principal |
| ic-platform | nns-registry-types | candid |
| ic-platform | pocket-ic | dep:pocket-ic, principal |
| ic-platform | principal | dep:crc32fast, dep:data-encoding, dep:ic_principal, dep:serde, dep:thiserror |
| ic-platform | runtime | dep:pin-project-lite, dep:thiserror, executor, ic0, principal |
| ic-platform | secp256k1 | dep:ic-secp256k1, principal |
| ic-platform | signature-verification | canister-sig-creation, dep:ic-verify-bls-signature, dep:serde_bytes, dep:serde_cbor |
| ic-platform | stable-structures | dep:ic-stable-structures |
| ic-platform | wire-schema | dep:ic-wire-metadata, dep:serde_json, principal |

| Owner | Direct dependency | Requirement | Kind | Target | Optional | Defaults | Features |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ic-call | ic-platform | workspace-owned | normal | all | false | false | runtime |
| ic-platform | assert_matches | ^1.5 | dev | all | false | true |  |
| ic-platform | blake3 | ^1.8.7 | normal | all | true | false | std |
| ic-platform | candid | ^0.10.38 | normal | all | true | false |  |
| ic-platform | candid_parser | ^0.4.1 | normal | cfg(not(target_family = "wasm")) | true | false |  |
| ic-platform | crc32fast | ^1.5 | normal | all | true | true |  |
| ic-platform | curve25519-dalek | ^4.1.3 | normal | all | true | false |  |
| ic-platform | data-encoding | ^2.11 | normal | all | true | true |  |
| ic-platform | ed25519-dalek | ^2.2 | normal | all | true | false |  |
| ic-platform | ic-agent | ^0.49.2 | normal | cfg(not(target_family = "wasm")) | true | false | pem |
| ic-platform | ic-cdk-executor | =2.0.0 | normal | all | true | true |  |
| ic-platform | ic-certification | ^3.2.0 | normal | all | true | false | serde |
| ic-platform | ic-host-artifacts | ^0.69.1 | normal | cfg(not(target_family = "wasm")) | true | false | gzip |
| ic-platform | ic-management-canister-types | =0.11.0 | normal | all | true | true |  |
| ic-platform | ic-platform-macros | workspace-owned | normal | all | true | true |  |
| ic-platform | ic-representation-independent-hash | ^3.0 | dev | all | false | true |  |
| ic-platform | ic-secp256k1 | =0.3.0 | normal | cfg(not(target_family = "wasm")) | true | true |  |
| ic-platform | ic-stable-structures | =0.7.2 | normal | all | true | false |  |
| ic-platform | ic-verify-bls-signature | ^0.6.0 | normal | all | true | false | alloc |
| ic-platform | ic-wire | ^0.7.19 | normal | cfg(not(target_family = "wasm")) | true | false | typescript |
| ic-platform | ic0 | =1.2.0 | normal | all | true | true |  |
| ic-platform | ic_principal | ^0.1.5 | normal | all | true | false | convert, self_authenticating, serde |
| ic-platform | icrc-ledger-types | ^0.2.0 | normal | all | true | false |  |
| ic-platform | k256 | ^0.13.4 | normal | all | true | false | ecdsa |
| ic-platform | pin-project-lite | ^0.2.17 | normal | all | true | true |  |
| ic-platform | pocket-ic | ^16.1.0 | normal | cfg(not(target_family = "wasm")) | true | true |  |
| ic-platform | serde | ^1.0 | normal | all | true | false | alloc, derive |
| ic-platform | serde_bytes | ^0.11 | normal | all | true | true |  |
| ic-platform | serde_bytes | ^0.11 | dev | all | false | true |  |
| ic-platform | serde_cbor | ^0.11.2 | normal | all | true | true |  |
| ic-platform | serde_cbor | ^0.11.2 | dev | all | false | true |  |
| ic-platform | serde_json | ^1.0 | normal | cfg(not(target_family = "wasm")) | true | true |  |
| ic-platform | serde_test | ^1.0.177 | dev | all | false | true |  |
| ic-platform | sha2 | ^0.10 | normal | all | true | true |  |
| ic-platform | thiserror | ^2.0.18 | normal | all | true | true |  |
| ic-platform-macros | proc-macro2 | ^1.0.106 | normal | all | false | true |  |
| ic-platform-macros | quote | ^1 | normal | all | false | true |  |
| ic-platform-macros | syn | ^2 | normal | all | false | true | full |
<!-- END MANIFEST INVENTORY -->
