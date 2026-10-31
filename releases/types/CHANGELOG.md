# Types changelog

This group covers ic-platform-types. Its current series continues from published
0.15.0. The coordinated 0.15.0 release belongs to [shared history](../history.md#0150---2026-10-31).
Initial 0.1 history remains below with its original scope. See the
[Platform ledger](../platform/CHANGELOG.md) and [release index](../../CHANGELOG.md).

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
