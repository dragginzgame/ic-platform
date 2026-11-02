# PocketIC client changelog

This independent group covers `ic-platform-pocket-ic`.
Its initial publication is recorded in the [Platform ledger](../platform/CHANGELOG.md).

## [0.18.0] - Unreleased

### ⚠️ Breaking

- Use management-canister types 0.11, shared with Platform's native management
  surface. Settings and status types now expose status visibility and minimum
  incoming-call cycles. Consumers exchanging 0.8 DTOs must adopt the new identity.
- Require Rust 1.99.1, matching the workspace's supported compiler (#133).

### 🔧 Changed

- Release and publish the native client independently with `RELEASE_GROUP=pocket-ic`.
  Native consumers can select `ic-platform-pocket-ic` directly; the optional
  `ic_platform::pocket_ic` facade continues to expose that same implementation.
  Client releases no longer increment the facade package version.

## [0.17.0] - 2026-11-01

### 📝 Summary

- First published in the Platform group; see the original
  [0.17.0 notes](../platform/CHANGELOG.md#0170---2026-11-01).
