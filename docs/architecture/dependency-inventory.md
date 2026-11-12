# Platform dependency and ownership inventory

This describes the current checkout, including the pending source replacements.
[Migration contracts](migration.md) own compatibility details; the
[current handoff](../status/current.md) records release state and blockers.
The [earlier inventory](../history/dependency-inventory-2026-11-10.md) retains
historical selections and consumer observations, not current dependency claims.

## Package boundaries

| Owner | Responsibility | Deliberate exclusions |
| --- | --- | --- |
| Types | Canonical Principal, Account/Subaccount, bytes, text and Serde | Candid, Agent, execution and crypto in the base selection |
| Runtime | Sole executor, explicit registry IC0 exports, raw/prepared calls, CDK adapters | Codecs, native clients, crypto, timers and storage allocation |
| Runtime macros and guard | Explicit-codec entrypoints and exact-version executor singleton | Alternative executors or implicit Candid selection |
| Platform facade | Owned external Candid codec/DTOs, certificate/protocol adapters, crypto boundaries and native clients | Consumer trust roots, freshness, authorization, retries and orchestration |
| Browser package | Explicit SDK Core subpaths and typed ingress inspection | Generic CBOR export or application policy |

The root workspace retains five product packages and three Rust release groups.
No package has default features. Runtime macros/guard release with Runtime;
Types and the facade release independently. IC Tooling's Testkit re-exports its
upstream PocketIC client; Platform fixtures select Testkit 0.101.0. Its native
snapshot helpers select registry Platform 0.31.2 with only the owned management
codec/DTO features; this infrastructure does not enter the canister graph or
replace the local implementation under qualification. The former native Profiler
provider is retired here.

## Current external boundaries

| Selection | External owner or dependency | Current use |
| --- | --- | --- |
| Base Types | crc32fast, data-encoding, Serde, thiserror | Identity representation; no registry Principal in this selection |
| Types self-authentication | sha2 0.11 | Optional upstream SHA-224 plus owned IC suffix; no production registry Principal |
| Runtime IC0 | ic0 1.2 | Explicit bindings; no embedded binding source or second executor |
| Runtime executor | SlotMap, registry IC0, exact owned guard | One task/method owner; Vec task lists |
| Candid codec and DTOs | num-bigint 0.5, Serde and selected byte helpers | Owned codec and canonical Types; no upstream Candid/parser in this production selection |
| Native agent | ic-agent | Registry client, identities and EffectiveId; Candid may remain transitive here |
| Native IC protocol | ic-transport-types | Typed ingress/read-state/metrics adapters; same registry packet identities as Agent |
| Certificates | ic-certification | External tree/certificate/hash identities; typed CBOR admission stays in Platform |
| BLS | ic_bls12_381, sha2 0.11, digest 0.10 traits | Owned checked wrapper over external primitives; caller-owned preparation and identity-key refusal |
| Canister signatures | Certificate/BLS suppliers and SHA-2 | Owned protocol composition; trust/freshness policy stays consumer-owned |
| ECDSA / Ed25519 | k256 / ed25519-dalek and curve25519-dalek | External primitives; crypto-bigint security floor remains explicit |
| Browser SDK | @icp-sdk/core 6.1.1 | Browser 0.2.0 has typed call-ingress inspection, no generic Cbor export |

The native `certification::verify` composition requires both `agent` and
`certification-cbor`. It preserves Agent authentication, delegation and freshness
checks under the caller-selected root and binds a witness to the exact canister's
certified data. Callers supply byte allowances and retain application leaf policy;
the helper adds no dependency or Wasm feature edge.

Upstream binary Candid and management/signature packages also remain as explicit
**development oracles**. An all-features development tree is not a production
Wasm dependency list. Use a target-specific normal/build graph for an application;
this inventory makes no fresh fleet or whole-canister size claim.

The qualified [BLS wrapper](../design/bls-wrapper/qualification.md) now lives in
the facade. Its standalone normal graph excludes `ic-verify-bls-signature`, SHA-2
0.10, `hex` and `lazy_static`; the curve still requires older Digest/field/group
dependencies. Certificate hashing independently retains SHA-2 0.10. The old
verifier remains a development oracle and native Agent transitive. Qualification
contains only the matched published-baseline/current-source probe, not another
implementation or product package.

The additive prepared-root signature APIs accept caller-owned BLS pairing data;
they retain the same `signature-verification` feature closure and every certificate,
delegation, range, witness and authenticated-time check. No trust or freshness cache
is introduced. Owned codec bulk-byte encoding and schema-buffer reuse add no
dependency or feature. Twenty-two management declarations now share their Rust/codec
definition; fields, variant order and Serde representations are unchanged.

## Source maintenance

Principal/Account, executor, call/System API adapters, macros, passive ICRC DTOs
and all selected management declarations have specification-only replacements
with records under [design](../README.md#design-and-qualification). The 52 shared
schemas and public exports remain unchanged: 44 names on Wasm and 62 on native.
The four CMC/ICP/Cycles/assets DTO families and their selected DID fixtures also
have independent replacements, preserving 33 declarations, six aliases and 14
method schemas. Value/ICRC3Value non-cryptographic helpers also have independent
implementations. The final ICRC value-hash and ICP account-framing replacements
now use specification-only authors and independent tests. The original source,
copied tests and fingerprints are retired after 782 frozen-output comparisons;
see [ledger qualification](../design/ledger-composition-qualification.md).
There are no remaining identified embedded DFINITY implementation areas in the
maintained Rust source. External cryptographic algorithms remain registry-owned;
this is a source-maintenance inventory, not a change to external package terms.

All textual DID fixtures and export tooling are retired. Independent
`tests/fixtures/*-schema.json` files preserve binary schema evidence;
`binrw` reads the oracle headers only in tests and adds no new transitive package.
Original/replacement equality evidence is recorded in
[external qualification](../design/external-dtos-qualification.md).
External Cargo/npm suppliers retain their own metadata independently.

## Checked manifest facts

Run `bash scripts/ci/check-platform-inventory.sh --print` after reviewing manifest
changes. Replace only the block below with its output. The checker uses locked,
offline Cargo metadata and verifies every workspace feature/dependency plus the
facade README feature table. Package release versions are represented as
`workspace-owned`. Transitive graphs and source integrity have separate checks.

<!-- BEGIN MANIFEST INVENTORY -->
| Package | Feature | Enables |
| --- | --- | --- |
| ic-platform | agent | dep:ic-agent, principal |
| ic-platform | bls | dep:digest_10, dep:ic_bls12_381, dep:sha2_11 |
| ic-platform | candid-args |  |
| ic-platform | candid-codec | dep:num-bigint, num-bigint/serde, principal |
| ic-platform | candid-codec-endpoint | candid-args, candid-codec, dep:ic-platform-macros, runtime |
| ic-platform | canister-sig-creation | certification, dep:sha2_11, principal |
| ic-platform | certification | dep:ic-certification |
| ic-platform | certification-cbor | certification, dep:serde, dep:serde_bytes, dep:serde_cbor, dep:thiserror |
| ic-platform | certified-assets-types | candid-codec |
| ic-platform | cmc-types | candid-codec, principal |
| ic-platform | cycles-ledger-types | candid-codec, principal |
| ic-platform | default |  |
| ic-platform | ecdsa | dep:crypto-bigint, dep:k256 |
| ic-platform | ed25519 | dep:curve25519-dalek, dep:ed25519-dalek |
| ic-platform | executor | ic-platform-runtime/executor, ic0 |
| ic-platform | ic-protocol | certification-cbor, dep:ic-transport-types, principal |
| ic-platform | ic-protocol-fixtures | ic-protocol |
| ic-platform | ic0 | dep:ic-platform-runtime, ic-platform-runtime/ic0 |
| ic-platform | icp-ledger-types | candid-codec, dep:crc32fast, dep:sha2_11, principal |
| ic-platform | icrc-ledger-types | candid-codec, dep:crc32fast, dep:data-encoding, dep:hex, dep:num-bigint, dep:num-traits, dep:serde_bytes, dep:sha2_11, principal |
| ic-platform | management-canister-types | candid-codec, dep:serde_bytes, principal |
| ic-platform | nns-registry-types | candid-codec, principal |
| ic-platform | principal | dep:ic-platform-types, dep:serde |
| ic-platform | principal-self-authenticating | ic-platform-types/principal-self-authenticating, principal |
| ic-platform | raw-rand | candid-args |
| ic-platform | runtime | executor, ic-platform-runtime/runtime, ic0, principal |
| ic-platform | signature-verification | bls, canister-sig-creation, dep:serde_bytes, dep:serde_cbor, dep:thiserror |
| ic-platform-runtime | cdk | dep:ic-platform-macros, runtime |
| ic-platform-runtime | default |  |
| ic-platform-runtime | executor | dep:ic-platform-runtime-guard, dep:slotmap, ic0 |
| ic-platform-runtime | ic0 | dep:ic0 |
| ic-platform-runtime | runtime | dep:ic-platform-types, dep:pin-project-lite, dep:serde, dep:thiserror, executor |
| ic-platform-runtime-guard | default |  |
| ic-platform-types | default |  |
| ic-platform-types | principal-self-authenticating | dep:sha2_11 |

| Owner | Direct dependency | Requirement | Kind | Target | Optional | Defaults | Features |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ic-platform | binrw | ^0.15.2 | dev | all | false | false | std |
| ic-platform | candid | ^0.10.38 | dev | all | false | false | serde_bytes, value |
| ic-platform | crc32fast | ^1.5 | normal | all | true | true |  |
| ic-platform | crypto-bigint | ^0.7.10 | normal | all | true | false |  |
| ic-platform | curve25519-dalek | ^5.0.0 | normal | all | true | false |  |
| ic-platform | data-encoding | ^2.11 | normal | all | true | false | alloc |
| ic-platform | digest | ^0.10.7 | normal | all | true | false |  |
| ic-platform | ed25519-dalek | ^3.0.0 | normal | all | true | false |  |
| ic-platform | hex | ^0.4.3 | normal | all | true | true |  |
| ic-platform | ic-agent | ^0.49.2 | normal | cfg(not(target_family = "wasm")) | true | false | pem |
| ic-platform | ic-canister-sig-creation | ^1.3.1 | dev | all | false | false |  |
| ic-platform | ic-certification | ^3.2.0 | normal | all | true | false | serde |
| ic-platform | ic-management-canister-types | =0.11.0 | dev | all | false | true |  |
| ic-platform | ic-platform-macros | workspace-owned | normal | all | true | true |  |
| ic-platform | ic-platform-runtime | workspace-owned | normal | all | true | false |  |
| ic-platform | ic-platform-types | workspace-owned | normal | all | true | false |  |
| ic-platform | ic-signature-verification | ^0.3.0 | dev | all | false | false |  |
| ic-platform | ic-transport-types | ^0.49.1 | normal | cfg(not(target_family = "wasm")) | true | true |  |
| ic-platform | ic-verify-bls-signature | ^0.6.0 | dev | all | false | false | alloc |
| ic-platform | ic_bls12_381 | ^0.10.1 | normal | all | true | false | alloc, experimental, groups, pairings |
| ic-platform | k256 | ^0.14.0 | normal | all | true | false | ecdsa |
| ic-platform | num-bigint | ^0.5.1 | normal | all | true | true |  |
| ic-platform | num-traits | ^0.2.19 | normal | all | true | true |  |
| ic-platform | serde | ^1.0 | normal | all | true | false | alloc, derive |
| ic-platform | serde_bytes | ^0.11 | normal | all | true | false | alloc |
| ic-platform | serde_bytes | ^0.11 | dev | all | false | false | alloc |
| ic-platform | serde_cbor | ^0.11.2 | normal | all | true | true |  |
| ic-platform | serde_cbor | ^0.11.2 | dev | all | false | true |  |
| ic-platform | serde_json | ^1.0 | dev | all | false | true |  |
| ic-platform | sha2 | ^0.11.1 | normal | all | true | false |  |
| ic-platform | thiserror | ^2.0.18 | normal | all | true | true |  |
| ic-platform | toko-tooling | =0.39.15 | dev | all | false | true |  |
| ic-platform-macros | proc-macro2 | ^1.0.106 | normal | all | false | true |  |
| ic-platform-macros | quote | ^1 | normal | all | false | true |  |
| ic-platform-macros | syn | ^3 | normal | all | false | true | full |
| ic-platform-macros | toko-tooling | =0.39.15 | dev | all | false | true |  |
| ic-platform-runtime | ic-platform-macros | workspace-owned | normal | all | true | true |  |
| ic-platform-runtime | ic-platform-runtime-guard | workspace-owned | normal | all | true | false |  |
| ic-platform-runtime | ic-platform-types | workspace-owned | normal | all | true | false |  |
| ic-platform-runtime | ic0 | ^1.2.0 | normal | all | true | false |  |
| ic-platform-runtime | pin-project-lite | ^0.2.17 | normal | all | true | true |  |
| ic-platform-runtime | serde | ^1.0 | normal | all | true | false | alloc, derive |
| ic-platform-runtime | serde_test | ^1.0.177 | dev | all | false | true |  |
| ic-platform-runtime | slotmap | ^1.1.1 | normal | all | true | true |  |
| ic-platform-runtime | thiserror | ^2.0.18 | normal | all | true | true |  |
| ic-platform-runtime | toko-tooling | =0.39.15 | dev | all | false | true |  |
| ic-platform-runtime-guard | toko-tooling | =0.39.15 | dev | all | false | true |  |
| ic-platform-types | crc32fast | ^1.5 | normal | all | false | true |  |
| ic-platform-types | data-encoding | ^2.11 | normal | all | false | false | alloc |
| ic-platform-types | ic_principal | ^0.1.5 | dev | all | false | false | convert |
| ic-platform-types | postcard | ^1.1.3 | dev | all | false | false | alloc |
| ic-platform-types | serde | ^1.0 | normal | all | false | false | alloc, derive |
| ic-platform-types | serde_bytes | ^0.11 | normal | all | false | false | alloc |
| ic-platform-types | serde_cbor | ^0.11.2 | dev | all | false | true |  |
| ic-platform-types | serde_json | ^1.0 | dev | all | false | true |  |
| ic-platform-types | serde_test | ^1.0.177 | dev | all | false | true |  |
| ic-platform-types | sha2 | ^0.11.1 | normal | all | true | false |  |
| ic-platform-types | thiserror | ^2.0.18 | normal | all | false | true |  |
| ic-platform-types | toko-tooling | =0.39.15 | dev | all | false | true |  |
<!-- END MANIFEST INVENTORY -->
