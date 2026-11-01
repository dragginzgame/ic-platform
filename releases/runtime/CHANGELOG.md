# Runtime changelog

This group owns ic-platform-runtime, ic-platform-macros and
ic-platform-runtime-guard. Earlier macro/guard history remains in
[history](../history.md) and [Platform](../platform/CHANGELOG.md).

## [0.17.1]

### ⚠️ Breaking

- Move IC0, executor, CDK adapters and prepared calls into the sole canonical
  runtime owner. Timers and Wire Endpoint must adopt this owner directly.
  Preserve callback/trap/upgrade behavior, guarded source and canonical Types.
- Advance macros/guard together to distinguish this owner from the previously
  embedded executor. The exact guard refuses mixed owners; facade versions that
  delegate to the same runtime can coexist without duplicate executors.

### 🧹 Cleanup

- Keep Candid, crypto/certification and native clients outside this release group.
  Direct runtime macros default to the runtime path; facade entrypoints retain
  their existing paths and delegate to the same implementation.
