# EngineView JSON compatibility

The public projection derives Serde independently of Candid decoding. Its
qualification owner is `qualification/engine-view-consumer/src/tests/serde.rs`.
Run `make qualify-engine-response` for native execution, Clippy and Wasm test
compilation. All checks are offline and locked after dependency preparation.

## Contract and fixtures

Literal JSON supplies the expected representation; the projection's Candid
schema and encoder do not construct the inputs. Each input has an explicit
accept/reject expectation. Native tests also deserialize it through published
Platform 0.33.7, require equal admission and compare successful serialized values.
The portable tests retain their independent expectations without that oracle.

- Twelve nested combinations cover signed timestamps at i64 minimum, -1, zero
  and maximum, and unsigned state sizes at zero, one and u64 maximum. Serialized
  values must contain exact integer tokens and deserialize without value loss.
- Management and anonymous Principal text remains canonical. Root node order
  and duplicates survive; the nested list is independently preserved. Opaque
  Unicode/NUL/whitespace text and the empty configured-version marker survive.
- Omitted and explicit-null optional root fields are equivalent. Nested state
  size omission/null remains None; a present subnet stays present. Unknown JSON
  fields retain the published ignore policy.
- Thirty-two refusal cases cover missing/null required fields, malformed
  Principal text/checksums, byte arrays in human-readable identity positions,
  integer overflow/underflow and noninteger values, malformed present optional
  records, nonboolean halt flags, duplicate selected fields (including null
  optional values and nested fields), and trailing JSON values.

JSON admission is a separate contract: accepting an unknown JSON field does not
remove Candid validation of skipped values. Exact Rust JSON integers also do not
establish lossless JavaScript Number consumption. Binary serializers, browser
adapters and every future protocol DTO remain separate qualification work.

## Observed result — 2026-11-15

All three new tests passed on Linux x86-64, bringing the public consumer suite to
12 passing tests. No disagreement with published Platform was observed. Clippy
with warnings denied and Wasm test compilation passed. These tests have not been
executed on Wasm or macOS. No production source or dependency selection changed.

The run used Rust 1.100.0, the default test profile, working Protocol 0.1.0,
registry Platform 0.33.7, registry Types 0.20.4 and serde_json 1.0.154.
Real Query adoption and the first projection's overall gate remain open.

Inputs at observation:

- Product lock SHA-256: `bc82b53beb5a552bd71eaec7fb07f2062805485e732b5738da236c9633a0df81`.
- Qualification lock SHA-256: `e67d5bc6c2405a4109f47694828c42c8a98c0a77a070d0ca3c5ae591533f1856`.
- Product/consumer file-inventory SHA-256: `76f49ddf9d796b4dd1ea8b8f826c322f877f33c4299c95ea9cb5755b1e159597`.
  Reproduce from the repository root with
  `rg --files crates/ic-platform-protocol qualification/engine-view-consumer | sort | xargs sha256sum | sha256sum`.
  This binds working manifests and authored source/tests rather than a commit.
