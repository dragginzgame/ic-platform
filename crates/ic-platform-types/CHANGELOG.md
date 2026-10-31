# Changelog

The Types group covers ic-platform-types. Its independent release series continues
from the existing published 0.15.0 identity. The coordinated history through 0.15.0
is preserved once in the [root ledger](../../CHANGELOG.md#0150---2026-10-31), with
its original scope. Initial 0.1 history is retained below; it is not a new release.

## [0.15.1]

### 🩹 Fixed

- Restore literal package-version ownership and an independent canonical ledger
  in the existing workspace. Preserve the published 0.15.0 numeric baseline,
  canonical definitions, external bytes and facade conversions.
  [#118 — Independent identity releases](https://github.com/dragginzgame/ic-platform/issues/118).

## [0.1.0]

### ➕ Added

- Establish the canonical owned Principal, Account, Subaccount and identity errors
  independently of Platform runtime releases. Preserve existing representations,
  Serde, bounded text admission and account equality/order/hash semantics.
- Retain explicit registry Principal conversions and separately selected upstream
  self-authentication. Platform retains external ledger DTO conversions.
