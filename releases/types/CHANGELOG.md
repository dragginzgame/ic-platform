# Types changelog

This group covers ic-platform-types. Its current series continues from published
0.15.0. The coordinated 0.15.0 release belongs to [shared history](../history.md#0150---2026-10-31).
Initial 0.1 history remains below with its original scope. See the
[Platform ledger](../platform/CHANGELOG.md) and [release index](../../CHANGELOG.md).

## [0.19.2] - 2026-11-11

### ♻️ Refactoring

- Remove the production `ic_principal` dependency. Optional self-authentication
  uses the shared upstream SHA-2 0.11 provider and appends the IC identity suffix,
  preserving arbitrary input admission, Principal bytes and canonical text.
  Base identity features remain free of hashing; registry comparisons stay dev-only.

### 🛠️ Tooling

- Adopt Toko Tooling 0.39.9's dependency-boundary correction while retaining
  Platform's stricter local PocketIC restriction.

## [0.19.1] - 2026-11-10

### ♻️ Refactoring

- Replace base Principal source and tests through a separate specification-only
  implementation context, preserving public identity, bounded text admission,
  ordering, error variants and Serde representations, including binary-format
  text admission. Record the protocol contract, observations and input boundary.
- Delegate optional self-authentication to registry `ic_principal` instead of
  retaining its derivation source and vectors. Base Types remains crypto-free;
  only this optional capability brings the supplier's SHA-2 0.10 dependency.
- Replace Account through the same specification-only process, retaining
  effective-zero equality/order/hash, text admission and exact optional-byte
  Serde representations.

### 🛠️ Tooling

- Align standalone consumer admission with the optional registry derivation
  provider; retain base Principal isolation and unrelated-client refusal checks.
- Adopt Toko Tooling 0.39.8 and audit product/qualification package placement.
  All five maintained crates already use the product layout; disposable fixture
  templates retain their documented locations; the native provider is retired.
  Production boundaries and the three release groups remain unchanged.

## [0.19.0] - 2026-11-09

### ⚠️ Breaking

- Remove `principal-external`, its registry aliases/conversions and the production
  `ic_principal` dependency. Canonical Principal bytes, text, Serde, ordering and
  self-authentication remain unchanged; registry comparisons are development-only.

### 🧭 Migration Notes

- Remove the retired feature selection and use canonical `Principal` /
  `PrincipalError` directly. Callers interacting with an independently selected
  native supplier must handle that supplier's identity at their own boundary;
  Types adds no replacement alias or conversion abstraction. Existing serialized
  identities and effective-zero Account behavior require no data migration.

### 🧪 Testing

- Retain independent registry text/byte/self-authentication oracles as dev-only
  dependencies and inherit published Toko Tooling 0.37.1.

### 🛠️ Tooling

- Remove downstream Timers composition from Platform release admission and
  qualification-lock custody. Retain standalone Runtime and singleton-refusal
  coverage; Scheduling owns scheduler compatibility tests.

- Adopt published Toko Tooling 0.37.1, migrating to `TOKO-TOOLING.md`,
  `.toko-tooling.snapshot`, `toko-tooling-*` commands and Toko LOC metadata.
  CI and publication adapters admit the renamed snapshot before effects;
  existing product release requirements remain unchanged.

## [0.18.3] - 2026-11-08

### 🛠️ Tooling

- Inherit the exact published `toko-tooling` 0.36.0 development selection with
  its source-bundle Machete exemption. Production dependencies, canonical type
  identities and serialization bytes are unchanged.

## [0.18.2] - 2026-11-06

### 🔧 Maintenance

- Select `alloc` for encoding and byte-Serde helpers without enabling their
  unnecessary `std` features. Principal/Account identities and bytes are unchanged.

## [0.18.1] - 2026-11-04

### 🐛 Fixed

- Select upstream Principal Serde with `principal-external`, so passive records
  using ExternalPrincipal no longer need Candid to supply its serialization
  traits. Preserve canonical identities, conversion behavior and bytes; keep
  defaults empty and self-authentication separate. No new production packages.

## [0.18.0] - 2026-11-02

### ⚠️ Breaking

- #138: encode present owned Account subaccounts as Serde bytes, retaining the
  fixed 32-byte length and omitted-field admission. Postcard Some gains a 0x20
  length prefix; CBOR uses bytes; None and explicit zero remain distinguishable.
  Require the next Types minor and coordinated Wire schemas, browser clients,
  fresh endpoint contract IDs and budget qualification. Preserve retained data
  bytes; there is no automatic persisted-data migration or legacy decoder.
  Logical identity and external ledger Candid contracts remain unchanged.

## [0.17.0] - 2026-11-02

### ⚠️ Breaking

- Require Rust 1.99.1, matching the pinned development/release compiler (#133).
  Preserve independent Types identity, Principal interoperability and native/Wasm proofs.

### 🔧 Changed

- Use RustCrypto SHA-2 0.11 for optional owned Principal self-authentication,
  without default provider features. Preserve the original derivation, vectors
  and external Principal identity. Wasm probe size improves by 386 bytes with
  size optimization but grows by 784 bytes at opt-level 3; no universal size
  reduction is claimed.

## [0.16.0] - 2026-11-01

### 🔧 Changed

- **Breaking feature selection:** self-authentication no longer enables registry
  Principal conversions. Retain the exact upstream derivation with upstream
  SHA-224, protected source/license/vector checks and unchanged Principal bytes.
  Enable `principal-external` explicitly when conversions are needed.

## [0.15.1] - 2026-10-31

### 🩹 Fixed

- Restore literal package-version ownership and an independent canonical ledger
  in the existing workspace. Preserve the published 0.15.0 numeric baseline,
  canonical definitions, external bytes and facade conversions.
- Keep the canonical group ledger under releases/types and require explicit group
  selection for release and publication; earlier history remains linked.
  [#118 — Independent identity releases](https://github.com/dragginzgame/ic-platform/issues/118).

## [0.1.0]

### ➕ Added

- Establish the canonical owned Principal, Account, Subaccount and identity errors
  independently of Platform runtime releases. Preserve existing representations,
  Serde, bounded text admission and account equality/order/hash semantics.
- Retain explicit registry Principal conversions and separately selected upstream
  self-authentication. Platform retains external ledger DTO conversions.
