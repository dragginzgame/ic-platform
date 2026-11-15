# Registry subnet settings projection

The opt-in `registry-settings` feature implements a selected protobuf decoder in
`ic-platform-protocol`. It accepts an already extracted SubnetRecord. Registry
transport, envelope decoding, authentication and Query reporting remain outside
this slice. This is partial progress on [replacement #3](https://github.com/dragginzgame/ic-platform-new/issues/3),
migrated from [Platform #169](https://github.com/dragginzgame/ic-platform/issues/169),
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

Four independent public consumer tests now run in the existing qualification
consumer under an opt-in `registry-settings` feature. They check literal
protobuf-to-JSON presence, exact u64::MAX tokens in all five integer slots,
omitted/null/unknown JSON fields, mistyped and overflowing values, and duplicate
selected JSON fields (including null). JSON requires nonoptional scalar fields;
protobuf absence supplies their defaults. These formats intentionally have
different omission rules. The public consumer tests, Clippy and Wasm compilation
pass locally through `make qualify-registry-settings`. The combined consumer
graph is also checked without widening the existing supplier allowlist.

Four further native-only tests compose published Platform 0.33.7's Registry
envelope decoder with this settings decoder. The published `nns-registry-types`
feature is enabled only through the qualification consumer's native development
dependency; no published Platform edge enters the product or Wasm graph. The
qualification lock adds its existing thiserror edge without changing versions.
The fixture retains mutation version/timestamp and borrowed application-error
code/reason/key, rejects application errors before interpreting inline settings,
distinguishes missing content from present empty bytes, and keeps envelope and
payload refusals distinct. Chunked values remain unsupported, including responses
containing both inline and chunked content. Literal frames cover duplicate and
malformed envelope/payload fields and exact/exceeded field allowances.

This models the existing Query configuration caller's two decoder stages. Each
stage has a separate 64 KiB byte / 4,096-field allowance; nested selected settings
share the payload stage's allowance. Neither public API exposes remaining budget
to the other, so composition does **not** establish a single 4,096-field budget
over the envelope plus payload. Envelope allocation/item allowances remain with
the published owner; this settings projection allocates no collections. A future
combined-budget contract would require an explicit API and qualification change.

Read-only review of Query found `subnet::settings::fetch` and its CLI still at the
design stage. The composition helper exists only under qualification tests; it
does not implement that report, verify a query signature, select an endpoint,
bind a subnet key or create authenticated evidence. It imports no Query source
or tests and makes no sibling changes or network requests.

`make qualify-registry-runtime` now executes independent Registry Wasm admission
and a separate native heap-test process. Both pass locally on Linux x86-64 with
Rust 1.100.0. The heap test uses DHAT 0.3.3 in release mode and constructs inputs
before profiling. Seven workloads each record zero total allocation blocks and
bytes: empty input, all selected fields, a 65,536-byte opaque unknown field,
duplicate and malformed fields after selected values, and field/byte exhaustion.
This is measured evidence for those workloads, not a universal allocation proof
or a performance comparison. No Serde operation is included in heap measurement.

The standalone Wasm corpus verifies absence/presence, explicit false/zero,
u64::MAX, root-boundary versus mid-field truncations, shared field and byte bounds,
duplicates, invalid booleans, unsupported groups and overflowing varints. A
length of 2^32 must refuse on Wasm rather than truncate to zero. The Node runner
requires an import-free module and successful main completion. Node 24.21.0 /
V8 13.6.233.17-node.53 executed artifact SHA-256
`0ea28e6e1a300fdf72615c6b53fc5a0c647ce3036cbd3ac460f7c84fd0c692d0`.
The probe reuses the existing qualification package and therefore selects its
EngineView/Types dependency closure; the Registry-only product graph remains
separately restricted to Serde. This is standalone Wasm evidence, not installed
IC execution, a size acceptance decision or an authenticated Registry response.

Actual Query reporting integration and hosted execution of these new checks
remain pending. Existing EngineView evidence does not qualify these new surfaces. Publication
stays disabled and the package remains at its initial unpublished 0.1.0 version.
