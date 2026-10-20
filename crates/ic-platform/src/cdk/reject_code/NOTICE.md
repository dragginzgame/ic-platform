# Runtime reject code provenance

The six variants, numeric assignments, Serde names and Display spellings derive
from ic-error-types 0.2.0 under its retained Apache-2.0 LICENSE. Platform owns this
runtime enum. Conversion rejects unknown numbers while retaining their u64 value;
CallRejected continues exposing its original raw u32 and fallible getter.
Source: dfinity/ic revision `6bea63e3244b1aeaab8d16d1c7cab7f48643aa47`,
`packages/ic-error-types/src/lib.rs`.

The unrelated ErrorCode catalogue, Strum iteration and ErrorCode conversion are
not imported. Agent and PocketIC types retain their external owners; compare raw
protocol numbers explicitly when crossing those boundaries. This is a breaking
Rust identity/trait change for 0.4, not a change to reject codes or retry policy.
