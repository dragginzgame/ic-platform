# Query hard-cut candidate — Platform 0.34.5 / Protocol 0.2.4

This candidate supplies the Query read boundary from issue #4 without selecting
legacy Platform, Agent or signing identities. Defaults remain empty. Platform
owns native HTTP and authenticated evidence; Protocol owns service bytes and
projections. Neither product depends on the other. Query owns observation,
placement policy, pagination and whole-operation deadlines.

## Native contract and provenance

The opt-in native `ReadClient` requires an admitted endpoint, trusted root,
independently trusted root-subnet identity, byte ceiling, HTTP timeout and
certificate traversal limits. Every operation requires explicit ingress expiry
and signing/delegation time windows. Redirects, proxies and transport retries
are disabled. Dropping its future cancels the operation; callers retain the
whole-operation deadline. The byte ceiling is an input/output bound, not a claim
of exact heap allocation or total execution work.

Anonymous query responses require node signatures. Request IDs and response
signable bytes come from the original `ic-transport-types` 0.49.3 implementation;
HTTP/TLS comes from original `reqwest` 0.13.5 with Rustls and empty defaults.
These registry sources and checksums are pinned in Cargo.lock (Apache-2.0 and
MIT/Apache-2.0 respectively). No Agent code or cryptographic algorithm is copied.
Node identity, exact Ed25519 key framing, weak-key refusal, strict signatures,
request-ID binding, timestamp bounds and duplicate signatures are checked.
An authenticated canister-scoped certificate establishes routing authority;
a separately authenticated selected-subnet certificate supplies node keys.
A node-authenticated query is not a consensus-certified response.

Selected-subnet reads preserve exact certificate bytes, selected subnet, certified
time, raw optional type (including unknown strings), all four optional metrics
and node key bytes. Direct certificates require the explicit root-subnet binding;
delegated certificates must authorize that exact subnet. Missing/pruned/mistyped
paths refuse where proof is required. Self-reported ranges never supply routing
or effective-canister authority. Existing strict certificate decoding, range,
delegation, signature and time admission remain the verifier.

## Protocol contract

`registry` supplies bounded get_value requests/responses, latest-version decoding
and borrowed subnet-list, subnet-record, node, node-operator and data-center
projections. Presence, empty values, version and timestamp remain distinct;
application error code/reason/key are retained. Chunked values explicitly refuse.
Repeated selected values charge item and logical allocation budgets; unknown
fields still charge framing work, nested errors share the field budget and
selected singular duplicates refuse. Separate decoding operations have separate
budgets; no end-to-end heap quota is claimed.

`registry-placement` supplies exact target argument encoding and bounded
Ok/Err/optional-subnet decoding through the existing Candid parser. CloudEngine
also supplies bounded empty arguments. The owned projection macro was moved into
the shared private Candid module; no legacy decoder or encoder is selected.

Independent literals follow the public DFINITY declarations observed 2026-11-16:
[Registry transport](https://github.com/dfinity/ic/blob/master/rs/registry/transport/proto/ic_registry_transport/pb/v1/transport.proto),
[subnet](https://github.com/dfinity/ic/blob/master/rs/protobuf/def/registry/subnet/v1/subnet.proto),
[node](https://github.com/dfinity/ic/blob/master/rs/protobuf/def/registry/node/v1/node.proto),
[node operator](https://github.com/dfinity/ic/blob/master/rs/protobuf/def/registry/node_operator/v1/node_operator.proto),
and [data center](https://github.com/dfinity/ic/blob/master/rs/protobuf/def/registry/dc/v1/dc.proto).
Tests use independent upstream Candid schemas for placement and existing
CloudEngine responses. No implementation or generated schema code is copied.

## Actual consumer evidence

`make qualify-query-hard-cut` freezes the read-only `QUERY_SOURCE` checkout,
verifies the revision and hashes of patch inputs, applies the retained
`qualification/query-hard-cut/consumer.patch` only inside ignored target storage,
and checks the resulting Cargo graph, all topology/CLI tests and Clippy. The
patch targets actual Query source at `12844b628d96bfd203ffbd900f10d02c4a896988`
plus its recorded local migration. Its source license remains the Toko Studios
Network Licence 1.0. The source inventory includes dirty bytes, not just HEAD.
The fixture refuses changed patch inputs instead of silently skipping adaptation.
`qualify-query-settings` now delegates to this complete current-cohort runner.

The native Linux run at `target/query-hard-cut/1794852912499/verified.json` passed
184 tests (22 + 26 CLI tests and 136 topology tests) and all-target Clippy. It
selects exactly one local Platform 0.34.5, Protocol 0.2.4 and Types 0.21.1, with
no Agent or legacy Platform. Source inventory SHA-256:
`67eda7081598869d43aebfb02a5c614cdd1dc64f420258f99a58674dab91798d`;
patch SHA-256: `a223ec379dedc14faa13046a6d4f35ef80eb042f53aa01437bb647cc242d1c8c`;
lock SHA-256: `fc7a65fa39fc1eba0529e9ed105c06f362a88310ad76c1cbe8a66bacf7901e3d`.
The private independent schema package is a dev-only fixture dependency. This
is local consumer qualification, not a registry-only consumer delivery.

Product synthetic tests use real independently constructed BLS/Ed25519 proofs
and reject changed requests/replies, absent/duplicate/unknown/bad signatures,
wrong subnet, stale time and corrupted certificates. Local HTTP tests reject
both declared and streamed oversize bodies and malformed CBOR. An explicit live
v3 probe authenticated mainnet root-subnet type `system`, 40 node keys and all
four metrics, Registry latest version and CloudEngine fee 0.25. Its root and
root-subnet binding are supplied independently by qualification. Run it with:

```sh
cargo run --locked --manifest-path qualification/Cargo.toml \
  -p platform-certificates --features native-read --bin native-read -- --live
```

## Remaining gates

No macOS execution, production sibling migration or registry-only publication is
claimed. Siblings remain unchanged. Auth installed proof is recorded separately
in [Auth qualification](auth-crypto.md). These local results are not issue closure
or evidence that every design gate is complete.

Fresh RustSec review used revision
`02e2517f457a359fc6d3440ea4560c8ea188fee8`. Root, qualification and migrated Query
locks had no known vulnerabilities. Existing unmaintained-package warnings are
not suppressed. This does not clear issue #4's separate Agent signing/derivation
obligations: current ic-agent 0.49.3 still requires elliptic-curve 0.13,
k256 0.13.4 and p256 0.13.2; ic-secp256k1 0.3.0 still requires k256 0.13.4.
Those upstream paths remain blocked on a compatible correction. No fork,
incompatible override or risk exemption is selected.

## Focused final checks

Platform all-feature tests passed 45 unit, eight integration and 12 compile-fail
contract cases. Protocol passed 41 unit cases; the independent consumer passed
25 library, two HTTP and one placement schema cases. Product and consumer Clippy,
product Rustdoc with denied warnings, both product all-feature Wasm checks and
workspace formatting checks passed. Registry settings heap admission and the
extended standalone Wasm Registry envelope corpus passed; its Wasm SHA-256 is
`c737a1b853348fe890fa42bf98b02f0cc785df59eaec2a8dce8a9cfe9546ff8a`.

Cargo dependency lints passed in these selected builds. The supplemental
cargo-machete scan reported `serde_bytes` in Platform and Types; both are used
by `#[serde(with = "serde_bytes")]` (certificate construction and account
serialization respectively). These are inspected attribute-use false positives,
not removed edges or added lint exemptions.
