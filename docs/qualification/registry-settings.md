# Registry subnet settings projection

The opt-in `registry-settings` feature implements a selected protobuf decoder in
`ic-platform-protocol`. It accepts an already extracted SubnetRecord. Registry
transport, envelope decoding, authentication and Query reporting remain outside
this slice. This is partial progress on [Platform #169](https://github.com/dragginzgame/ic-platform/issues/169),
not completion of the consumer requirement.

## Contract and provenance

Field selection follows the replica's
[subnet.proto at 00e4d8e3ee6081c1862e6119d63cffa76bd4af3d](https://github.com/dfinity/ic/blob/00e4d8e3ee6081c1862e6119d63cffa76bd4af3d/rs/protobuf/def/registry/subnet/v1/subnet.proto).
This is a new implementation from the wire contract; no upstream implementation,
generated source or old Platform tests are copied. The independent test fixtures
spell out protobuf bytes directly without a production encoder.

| Message | Selected field numbers |
| --- | --- |
| SubnetRecord | 23 features, 24 max_number_of_canisters, 34 resource_limits |
| SubnetFeatures | 3 http_requests, 9 optional sev_enabled |
| ResourceLimits | 1 maximum_state_size, 2 maximum_state_delta, 3 maximum_query_instructions, 4 maximum_query_walltime_seconds |

Absent nested messages differ from present empty messages. Optional scalars
retain absent versus explicit false/zero. HTTP defaults to protobuf false within
a present feature message, irrespective of the replica's
[Rust Default implementation](https://github.com/dfinity/ic/blob/00e4d8e3ee6081c1862e6119d63cffa76bd4af3d/rs/registry/subnet_features/src/lib.rs).
Raw default-selection sentinels are not resolved into effective limits. SEV is a
configured value, not attestation.

## Resource and refusal semantics

Callers supply complete input bytes and a shared field-occurrence allowance.
Every selected nested and unknown field consumes that allowance; entering a
nested message does not reset it. Input bytes are checked once, including skipped
content. The fixed selected graph has only the root and one nested-message level.
All outputs are inline values. There are no collection allocations, so this
projection needs neither an allocation allowance nor a collection-item allowance.
This does not replace the broader Registry envelope decoder's limits.

Unknown varint, fixed64, length-delimited and fixed32 fields are skipped after
validating their framing. Unknown length-delimited content remains opaque.
Groups and wire types 6/7 refuse. Field numbers must be nonzero and fit protobuf's
29-bit range. Varints must fit u64 within ten bytes; bounded non-minimal varints
are accepted. Selected booleans accept only zero or one. Duplicate selected
singular fields refuse even when empty, false or zero; repeated unknown fields
are permitted. Selected fields with the wrong wire type refuse.

## Current evidence and remaining gates

`make qualify-registry-settings` passes seven literal-fixture tests, strict Clippy,
Rustdoc and wasm32 compilation locally on Linux x86-64 with Rust 1.100.0. Tests
cover presence, order, all selected duplicates, strict booleans/types, varint and
tag bounds, truncation, unknown framing, exact byte bounds and shared nested field
bounds. `make check-protocol-graphs` admits only Protocol and Serde's closure for
this feature; it admits no Types, Runtime, Platform or cryptographic supplier.
The feature is included in the three-host CI matrix, but the new revision's
hosted results are not yet established.

Independent public consumer/Serde fixtures, executed Wasm admission, measured
zero-allocation evidence and actual Query reporting integration remain pending.
Existing EngineView evidence does not qualify these new surfaces. Publication
stays disabled and the package remains at its initial unpublished 0.1.0 version.
