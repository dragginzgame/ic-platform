# PocketIC client changelog

This retired group covered `ic-platform-pocket-ic`; it is no longer a release
selection. The implementation and pending socket fix below moved to Testkit
under Platform #139 / Tooling #324. Published versions and this ledger remain.
Its initial publication is recorded in the [Platform ledger](../platform/CHANGELOG.md).

## [0.20.1] - Unreleased

### 🧪 Testing

- Explicitly select blocking I/O for accepted mock-server sockets on macOS.
  Preserve bounded reads and the no-replay assertions; production client code
  and the published 0.20.0 behavior are unchanged.

## [0.20.0] - 2026-11-03

### ✨ Added

- [#136](https://github.com/dragginzgame/ic-platform/issues/136): add bounded
  in-memory snapshot transfer with explicit sender, chunk and decoding budgets.
  Preserve timer/hook state and retain partial-upload identity for caller-owned
  cancellation and reconciliation; no shared files or automatic cleanup.

### ⚠️ Breaking

- Add `ClientError::ResponseTooLarge` for explicitly bounded snapshot exchanges;
  update exhaustive error matches. Existing operations retain their selected limits.

## [0.19.0] - 2026-11-02

### 🧹 Maintenance

- Consolidate base64 on 0.23.1, shared with Reqwest. Preserve standard padded
  JSON bytes and malformed-input rejection; imported client bodies stay exact.

### ⚠️ Breaking

- Return structured ClientError failures from sync/async network operations and
  builder execution. Preserve sources, server status and accepted operation IDs;
  never replay uncertain submissions. Use monotonic deadlines and async polling.
  Remove I/O-performing Default; explicit deletion now retains state on failure
  and prevents automatic Drop replay. Management/protocol bytes remain unchanged.

- Update Strum and its macros to 0.28. Exposed enum iteration now uses that
  trait identity; consumers importing IntoEnumIterator must select 0.28.
  Preserve imported client bodies, enum catalog values and protocol bytes.

## [0.18.0] - 2026-11-02

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
