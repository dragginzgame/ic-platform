# IC Platform

[Documentation](docs/README.md) · [Current handoff](docs/status/current.md)

Platform re-exports canonical Principal/Account identities from ic-platform-types.
Platform (facade), Runtime (runtime, macros and exact guard), and Types have
independent release groups in one workspace. The native PocketIC client now
releases with Tooling’s Testkit. Each group
owns explicit versions and its canonical releases/ ledger. Unrelated runtime
releases retain the same compatible type supplier; see
[the migration contract](docs/architecture/migration.md#stable-owned-identity-package-118).

One controlled dependency boundary for IC runtime integration and required IC
Candid interfaces. All application endpoints we own use IC Wire, including public
endpoints called by third parties. Candid is the adapter for management canisters,
NNS/SNS, ledgers and other interfaces that require it; caller location does not
select a codec. Existing Candid checkpoints remain a legacy compatibility exception.

The separate [browser package](packages/browser/README.md) supplies the
`@tokostudios/ic-platform` 0.2.0 npm boundary over SDK Core 6.1.1 for Blob 0.48,
with checked identity restoration, explicit Agent configuration and raw query
integration. The package is published; existing browser SDK declarations remain
until qualified adoption. Use `make check-browser` and the separately prepared Chromium
gate `make check-browser-runtime`. This does not add SDK dependencies to Rust.

The released 0.5 hard cut removed the unused `cdk::storage` checkpoint helpers;
existing data stays with its consumer and raw stable-memory passthroughs remain.
See [the retirement contract](docs/architecture/migration.md#pending-05-retirement).

Platform has no default features. Its owned runtime is Candid-free; Agent and
IC packet clients are native opt-ins. PocketIC belongs to Tooling. Consumers retain their Wire schemas,
authentication, budgets and retry policy. See the [maintenance and recovery
contract](docs/architecture/migration.md#runtime-maintenance-and-recovery).

Select `principal` for owned Principal/Account bytes, text and Serde, without
`ic_principal`, hashing or Candid. Runtime includes this minimal capability;
executor/IC0-only builds omit application types. The owned codec and maintained
DTOs use the same Principal. The external aliases and `principal-external` feature
are removed. Select `principal-self-authenticating` explicitly for derivation
using upstream SHA-224 plus the IC suffix. Native supplier APIs retain their own
identities and may require explicit byte conversion at the boundary. See the
[hard-cut contract](docs/architecture/migration.md#owned-principal-hard-cut-unreleased).

| Feature | Implemented contract |
| --- | --- |
| `principal` | Owned Principal/Account bytes, text and Serde; no external Principal, hashing or Candid |
| `principal-self-authenticating` | Owned identity framing using upstream SHA-224; implies principal |
| `agent` | Native Agent transport/builders, PEM identities, typed errors and signed-request operations |
| `ic-protocol` | Native typed IC packet codecs with caller-selected byte budgets; upstream transport identity without Agent or runtime |
| `ic-protocol-fixtures` | Explicit native development helpers for IC query, certificate, controller and subnet-metrics fixtures |
| `bls` | Upstream BLS cryptography with typed errors and identity-key admission; no embedded algorithms, production RNG or Candid |
| `candid-codec-endpoint` | Owned-codec external query/update adapters; explicit decode/reply limits, arity and empty policy; existing runtime, no upstream Candid or service registration |
| `runtime` | Raw calls, system and task adapters at `cdk::*`, without entrypoint macros |
| `candid-args` | Dependency-free encoding of empty, unit/null and single-text external argument lists |
| `candid-codec` | Owned typed binary codec, canonical Principal and arbitrary integers; no registry Candid/parser |
| `raw-rand` | Sole typed management `raw_rand` codec, dependency-free; transport and seed policy remain caller-owned |
| `canister-sig-creation` | Owned signature key framing/constants with external SHA-256, canonical Principal and strict DER admission; no Candid, IC0 or store API |
| `certification` | Registry certificate/hash-tree and mutable tree identities, lookup and hashing; no Candid or executor |
| `certification-cbor` | Byte-bounded standard certificate/signature CBOR; caller-selected budgets, no verification or runtime |
| `executor` | Maintained task/context primitives with owned IC0 and SlotMap; no Candid |
| `ic0` | Raw IC 1.2.0 calls, including safe stable-memory size/grow/read/write, replicated/controller flags, reply/reject bytes and debug output; independent of Candid and executor |
| `icrc-ledger-types` | Selected ICRC DTOs with owned binary codec and unchanged upstream value hashing, plus ICRC-10; no registry ledger package, storage, CBOR or consent formatting |
| `management-canister-types` | Owned management 0.11.0 DTOs and codec on native/Wasm; native snapshot transfer types retain their existing scope. No production registry management, Candid or runtime dependency. |
| `cmc-types` | CMC top-up, mint notification and rate DTOs with the owned codec; no upstream Candid or payment policy |
| `icp-ledger-types` | ICP transfer/archive DTOs with owned codec and typed query-reference admission; retained account derivation, no upstream Candid or CDK |
| `cycles-ledger-types` | Cycles Ledger creation/withdrawal DTOs with owned codec; adding `icrc-ledger-types` supplies owned archive-callback codecs |
| `nns-registry-types` | Registry subnet lookup DTOs with owned binary codec and canonical Types Principal; no upstream Candid or topology policy |
| `certified-assets-types` | Native-only asset get/get_chunk DTOs with owned codec; no upstream Candid, verification or host transport |
| `signature-verification` | Owned canister-signature protocol verification and authenticated time; external BLS/certified-tree primitives, no Candid, registry Principal or runtime |
| `ecdsa` | Native/Wasm K256 secp256k1 ECDSA keys, signatures and prehash traits; no RNG or runtime |
| `ed25519` | Native/Wasm Dalek keys, signatures and canonical scalars; no RNG, Candid or runtime |

The [IC inventory](docs/architecture/dependency-inventory.md) maps upstream code to exposed APIs,
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

Platform supports binary Candid for required external IC interfaces. Textual DID
export, inspection and sidecar tooling are retired; there is no replacement
service-export API. See [the hard cut](docs/architecture/migration.md#did-retirement-pending-030).

Prepare the compiler selected in [rust-toolchain.toml](rust-toolchain.toml),
including `wasm32-unknown-unknown`, then prepare tools and locked dependencies:

```sh
make install-tools
make tools-check
cargo fetch --locked
```

Shared tooling is pinned in each workspace's `[workspace.dependencies]` and
inherited as a dev-dependency by every member. Keep the product root and private
`qualification/Cargo.toml` selections identical. After changing both exact pins
and their documented exceptions in `ci/dependency-pinning-exceptions.json`, run
`make deps-update DEPENDENCY_PACKAGE=toko-tooling`, then `make toko-tooling-update`
to install the selected archive. Retain both Cargo.lock files and the snapshot. Use
`make toko-tooling-install` to reinstall that Cargo selection, and
`make toko-tooling-status` to compare selected and installed identities offline.

For general dependency maintenance, `make deps-update` runs `cargo update -v`
sequentially in the root and qualification workspaces. It changes lockfiles;
review the changes and qualify the affected graphs. Select `DEPENDENCY_PACKAGE`
and, optionally, `DEPENDENCY_PRECISE` for a scoped update. The command preserves
normal Cargo output and does not perform a separate registry freshness audit.

For a change to Types, for example, select its focused checks:

```sh
cargo clippy --locked --offline -p ic-platform-types --lib -- -D warnings
cargo test --locked --offline -p ic-platform-types --lib
```

Use the [development and qualification guide](docs/development/qualification.md)
for browser setup, standalone consumers, feature matrices, installed runtime,
source-preservation checks and recovery. Aggregate Make targets and `make ci`
remain broad validation; they are not the default loop for ordinary edits.

## Releases

Select `RELEASE_GROUP=platform`, `runtime` or `types` explicitly for release,
resume and publication. No group is inferred from changed files. Root
[Cargo.toml](Cargo.toml) lists member versions; [CHANGELOG.md](CHANGELOG.md) links
the separate group ledgers and shared history. A Git release does not establish
registry availability. See the [release commands and recovery guidance](docs/development/qualification.md#development)
before executing an authorized delivery.

## Source licenses

See the canonical [licence](LICENSE.md) and [contribution policy](CONTRIBUTING.md).
Retained imported components keep their applicable notices and source records;
the [dependency inventory](docs/architecture/dependency-inventory.md) identifies those boundaries.
