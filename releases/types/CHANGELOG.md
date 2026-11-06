# Types changelog

This group covers ic-platform-types. Its current series continues from published
0.15.0. The coordinated 0.15.0 release belongs to [shared history](../history.md#0150---2026-10-31).
Initial 0.1 history remains below with its original scope. See the
[Platform ledger](../platform/CHANGELOG.md) and [release index](../../CHANGELOG.md).

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
