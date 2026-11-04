# IC Platform

Platform re-exports canonical Principal/Account identities from ic-platform-types.
Platform (facade), Runtime (runtime, macros and exact guard), and Types have
independent release groups in one workspace. The native PocketIC client now
releases with Tooling’s Testkit. Each group
owns explicit versions and its canonical releases/ ledger. Unrelated runtime
releases retain the same compatible type supplier; see
[the migration contract](docs/migration.md#stable-owned-identity-package-118).

One controlled dependency boundary for IC runtime integration and required IC
Candid interfaces. All application endpoints we own use IC Wire, including public
endpoints called by third parties. Candid is the adapter for management canisters,
NNS/SNS, ledgers and other interfaces that require it; caller location does not
select a codec. Existing Candid checkpoints remain a legacy compatibility exception.

The separate [browser package](packages/browser/README.md) supplies the
`@tokostudios/ic-platform` 0.1.1 npm boundary over SDK Core 6.1.1 for Blob 0.48,
with checked identity restoration, explicit Agent configuration and raw query
integration. The package is published; existing browser SDK declarations remain
until qualified adoption. Use `make check-browser` and the separately prepared Chromium
gate `make check-browser-runtime`. This does not add SDK dependencies to Rust.

The released 0.5 hard cut removed the unused `cdk::storage` checkpoint helpers;
existing data stays with its consumer and raw stable-memory passthroughs remain.
See [the retirement contract](docs/migration.md#pending-05-retirement).

Platform has no default features. Its owned runtime is Candid-free; transport,
parsing and PocketIC are native opt-ins. Consumers retain their Wire schemas,
authentication, budgets and retry policy. See the [maintenance and recovery
contract](docs/migration.md#runtime-maintenance-and-recovery).

Select `principal` for owned Principal/Account bytes, text and Serde, without
`ic_principal`, hashing or Candid. Runtime includes this minimal capability;
executor/IC0-only builds omit application types. Select `principal-external` for
`principal::ExternalPrincipal` and `.into()` conversions at external boundaries.
This adds byte conversions without self-authentication. Required Candid/client
features select it automatically. Select `principal-self-authenticating` explicitly
for the unchanged imported derivation using upstream SHA-224, without registry
Principal. Select both features when external conversions are also needed. Default features
remain empty. See the [packaging migration](docs/migration.md#minimal-principal-and-selected-icrc-packaging).

The pending Types 0.18.1 correction makes `principal-external` self-contained for
upstream Serde. Published Types 0.18.0 alone does not provide those external
serialization traits. See the [passive contract audit](docs/audits/passive-contract-boundaries-2026-11-04.md)
for supplier availability and consumer-owned feature/import changes.

| Feature | Implemented contract |
| --- | --- |
| `blake3` | Upstream `hash`, `Hasher` and `Hash`; native/Wasm with `std`, no IC codecs, threading or memory mapping |
| `principal` | Owned Principal/Account bytes, text and Serde; no external Principal, hashing or Candid |
| `principal-external` | Registry Principal identity, byte conversions and upstream Serde; implies principal, no hashing or Candid |
| `principal-self-authenticating` | Unchanged imported self-authentication using upstream SHA-224; implies principal |
| `agent` | Native Agent transport/builders, PEM identities, typed errors and signed-request operations |
| `bls` | Upstream BLS cryptography with typed errors and identity-key admission; no embedded algorithms, production RNG or Candid |
| `candid` | Bounded single-result, exact-result and argument-list decoding; external DTO derives and encoding |
| `candid-service` | Candid service declaration macros and printer; implies `candid` |
| `candid-endpoint` | Explicit required-external-Candid query/update adapters with artifact-selected limits, empty-envelope policy and existing runtime; implies CDK and Candid, with separate service-printer selection |
| `runtime` | Raw calls, system and task adapters at `cdk::*`, without entrypoint macros |
| `candid-parser` | Native external Candid parsing/type checking, service equality and dynamic refusal fixtures; owned bindings use Wire |
| `canister-sig-creation` | Upstream signature key formats/constants/hashing, owned issuer adapter and DER admission; transitive Candid/ic0, no store API |
| `cdk` | Owned explicit-codec entrypoints, raw calls, system helpers and protected spawning; Candid-free runtime with one canonical Runtime executor |
| `certification` | Registry certificate/hash-tree and mutable tree identities, lookup and hashing; no Candid or executor |
| `executor` | Maintained task/context primitives with owned IC0 and SlotMap; no Candid |
| `ic0` | Raw IC 1.2.0 calls, including safe stable-memory size/grow/read/write, replicated/controller flags, reply/reject bytes and debug output; independent of Candid and executor |
| `icrc-ledger-types` | Selected maintained ICRC DTOs and unchanged upstream value hashing, plus ICRC-10; no registry ledger package, storage, CBOR or consent formatting |
| `management-canister-types` | Owned canister-used 0.11.0 DTOs on Wasm; native upstream DTOs for host tooling; Candid without implicit Wasm printer, no executor or transport |
| `cmc-types` | External CMC top-up, mint notification and certified rate DTOs; implies Candid, no payment policy |
| `icp-ledger-types` | Legacy ICP transfer and encoded-block/archive DTOs; implies Candid, no ledger engine or CDK |
| `cycles-ledger-types` | External Cycles Ledger creation and withdrawal DTOs, including its distinct creation settings; implies Candid |
| `nns-registry-types` | Exact optional-principal Registry subnet lookup DTOs; implies Candid, no topology policy |
| `certified-assets-types` | Native-only asset get/get_chunk DTOs; implies Candid, no verification or host transport |
| `signature-verification` | Imported canister-signature verification with optional authenticated time result; certification/BLS, no Candid/ic0, executor or host transport |
| `ecdsa` | Native/Wasm K256 secp256k1 ECDSA keys, signatures and prehash traits; no RNG or runtime |
| `ed25519` | Native/Wasm Dalek keys, signatures and canonical scalars; no RNG, Candid or runtime |
| `secp256k1` | Native offline public-key derivation from exactly 0.3.0; no Candid/CDK, excluded from Wasm |

The [IC inventory](docs/ic-inventory.md) maps upstream code to exposed APIs,
observed consumers and maintenance ownership. `make check-inventory` rejects
drift in its complete workspace feature and direct-dependency tables; consumer
usage evidence is dated and reviewed separately.

## ECDSA Wasm builds

For ECDSA Wasm builds, select RustCrypto's upstream 32-bit arithmetic backend.
This repository's `.cargo/config.toml` selects it only for Wasm; consuming
canister workspaces must set it in their own build configuration:

```toml
[target.wasm32-unknown-unknown]
rustflags = ['--cfg=cpubits="32"']
```

Merge this with existing flags. `RUSTFLAGS` or `CARGO_ENCODED_RUSTFLAGS` supplied
by a build tool override Cargo's configured flags; carry the backend selection
through that tool's flag composition too. A published library cannot impose
these flags on its dependencies or downstream callers. The qualified backend
keeps upstream crypto code and bytes while recovering most of the ECDSA upgrade's
Wasm growth; see [measurements](docs/benchmarks/upstream-2026-10-31/README.md).

## Development

Prepare the compiler selected in [rust-toolchain.toml](rust-toolchain.toml),
including `wasm32-unknown-unknown`, then prepare tools and locked dependencies:

```sh
make install-tools
make tools-check
cargo fetch --locked
```

For a change to Types, for example, select its focused checks:

```sh
cargo clippy --locked --offline -p ic-platform-types --lib -- -D warnings
cargo test --locked --offline -p ic-platform-types --lib
```

Use the [development and qualification guide](docs/development-and-qualification.md)
for browser setup, standalone consumers, feature matrices, installed runtime,
source-preservation checks and recovery. Aggregate Make targets and `make ci`
remain broad validation; they are not the default loop for ordinary edits.

## Releases

Select `RELEASE_GROUP=platform`, `runtime` or `types` explicitly for release,
resume and publication. No group is inferred from changed files. Root
[Cargo.toml](Cargo.toml) lists member versions; [CHANGELOG.md](CHANGELOG.md) links
the separate group ledgers and shared history. A Git release does not establish
registry availability. See the [release commands and recovery guidance](docs/development-and-qualification.md#development)
before executing an authorized delivery.

## Source licenses

Original Platform code is MIT. Imported non-cryptographic runtime adapters and
the generator retain Apache-2.0. The combined `ic-platform` package declares
MIT AND Apache-2.0; included source notices identify the upstream owner/version.
