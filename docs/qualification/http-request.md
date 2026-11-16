# Bounded management HTTP GET arguments — issue #5

Protocol 0.2.3 adds `http-request`, an opt-in capability with no dependencies.
`http_request::encode_http_get_args(url, body, response_bytes, replicated, max_bytes)`
encodes exactly one management request. The method is GET; headers are empty;
transform and pricing version are absent. Response limit and replication are
explicit `Some` values. Absent and empty bodies are distinct. UTF-8 URL and body
bytes remain unchanged, including whitespace, NUL and non-ASCII text.

The schema follows the [IC management interface](https://docs.internetcomputer.org/references/ic-interface-spec/management-canister/),
checked 2026-11-16. The fixed type table uses a GET-only variant and `opt empty`
for the absent transform, both valid Candid subtypes. It includes `pricing_version:
opt nat32` with an absent value. No generated codec or historical implementation
was copied. No Principal crosses this narrow API, so no identity dependency is
needed. Transport, URL validation, cycles and HTTP policy stay with the consumer.

`make qualify-http-request` checks native lint/docs, Wasm compilation and the
one-package production graph. Two independent decoders accept the full argument:
upstream Candid 0.10.40 with management types 0.8.0, and the legacy Platform 0.33.7
consumer schema including pricing version. The test matrix covers absent/empty
bodies, arbitrary bytes, UTF-8 and 127/128/16383/16384-byte LEB128 boundaries,
both replication choices, full u64 response limits, exact-cap success and
cap-minus-one refusal. A held-reply slot case verifies the little-endian u64 body,
256-byte response bound and exactly one complete argument; appended bytes refuse.

Length arithmetic is checked before `try_reserve_exact`; exceeding the cap or
integer range yields `EncodeError::OutputLimit`, reservation failure yields
`EncodeError::Allocation`. No partial output is returned. The cap bounds the
encoded result rather than peak process memory; allocator-failure injection is
not part of this evidence.

## Testkit adoption

The retained requirement comes from `ic-tooling` source
`632eb713296c50f6a0fa5696b73fb0d607c61493`,
`crates/ic-testkit/src/held/module.rs`. Its consumer can replace the request struct
and broad codec call with:

```rust,ignore
let body = (index as u64).to_le_bytes();
let encoded = encode_http_get_args(
    &format!("{prefix}{index}"), Some(&body), 256, true, module_bytes,
)?;
```

Select Protocol's `http-request` feature and map the new structured `EncodeError`
into the consumer's error type. The encoder requires no legacy Platform edge.
This repository has not changed Testkit, published Protocol 0.2.3, or run Testkit's
installed held-reply suite against it. Those remain delivery/adoption obligations;
the schema tests alone do not close the issue's consumer gate.
