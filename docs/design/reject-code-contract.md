# Runtime rejection catalogue contract

This specification contains protocol facts and the existing public Rust API
requirements only. It is input for an implementation that has not inspected the
previous implementation, its patches or its tests. It does not itself establish
ownership clearance or certify a completed clean-room replacement.

## Protocol source

The public [IC HTTPS interface, reject codes](https://docs.internetcomputer.org/references/ic-interface-spec/https-interface/#reject-codes),
observed 2026-11-10, assigns these protocol values. These names and numbers
describe interoperability, not an implementation algorithm.

| Numeric value | Rust variant | Display string |
| --- | --- | --- |
| 1 | SysFatal | SYS_FATAL |
| 2 | SysTransient | SYS_TRANSIENT |
| 3 | DestinationInvalid | DESTINATION_INVALID |
| 4 | CanisterReject | CANISTER_REJECT |
| 5 | CanisterError | CANISTER_ERROR |
| 6 | SysUnknown | SYS_UNKNOWN |

`SysUnknown` means a best-effort call's outcome is unknown. This catalogue must
not infer failure, replay permission or retry policy from it. Zero is not a
rejection code; neither are integers outside the six entries above.

## Public compatibility

- Supply `RejectCode`, a fieldless Rust enum with the listed variant names and
  numeric casts. Preserve `Clone`, `Copy`, `Debug`, `Eq`, `PartialEq`, `Hash`,
  `serde::Serialize` and `serde::Deserialize`.
- Supply `UnknownRejectCode(pub u64)` with `Clone`, `Copy`, `Debug`, `Eq` and
  `PartialEq`. It preserves the exact rejected input, including `u64::MAX`.
- `TryFrom<u64>` for `RejectCode` returns its matching entry, or
  `UnknownRejectCode(input)`. It must never truncate the input or admit an
  unsupported value.
- `Display` returns the exact protocol label from the table. `Debug` uses the
  Rust variant name. Neither conversion needs to allocate.
- Serde represents a unit variant with enum name `RejectCode`, the Rust variant
  name and its zero-based declaration index. Do not serialize the protocol
  numeric cast in place of the established enum representation.
- The API resides in Runtime's existing rejection catalogue module; callers
  reach it through the existing CDK call surface. Do not add a crate, feature,
  dependency, public variant, extra conversion or compatibility alias.
- Do not promise a new stable C layout or introduce unsafe numeric conversion.
  The protocol representation and Serde representation are separate contracts.

## Independently written qualification

Write new tests from this contract, without consulting prior tests or source:

- Verify every supported numeric value, cast, display label and Debug name.
- Reject zero, values immediately above the catalogue, values around 32-bit
  boundaries and `u64::MAX`; verify exact error payload preservation.
- Verify the Serde unit-variant tokens for all six entries and refusal of an
  unknown variant. Existing `serde_test` is available as a dev dependency.
- Check the declaration remains usable through the existing public call module.

The integrating reviewer separately runs affected Runtime Clippy and consumer
checks. Passing tests prove these behaviors, not independent authorship by
themselves. Record the implementer's allowed inputs and exposure statement
separately from this contract.
