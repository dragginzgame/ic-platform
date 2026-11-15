# Types self-authentication and public consumer

The unpublished Types 0.21.0 candidate now retains the opt-in
`principal-self-authenticating` feature. It delegates SHA-224 to upstream
`sha2` 0.11.1 with default features disabled, matching the published 0.20.4
supplier requirement. Only the protocol frame is owned here: digest of the
exact supplied bytes followed by byte `0x02`. No key parsing, key validation,
proof of possession or authorization is implied. No hash implementation or
published test source is copied.

## Independent consumer

`qualification/crates/types-consumer` is a private 0.1.0 package outside product
release groups. Its normal dependency selects the local Types path and exact
0.21.0 version. The `candidate-types` alias distinguishes source selection, not
a wrapper or a second identity in its production graph. Public assignments
between directly imported Principal and Account.owner compile and execute.

Registry Types 0.20.4 is a native-only development oracle. It is compared through
bytes, text and JSON, never by pretending the two sources have the same Rust
type. Native comparison covers all 30 Principal lengths, four subaccount states,
every single-position `!` mutation of the resulting account text, and twelve
self-authentication input lengths. This is focused compatibility evidence, not
an exhaustive proof of equivalent malformed-input handling.

Literal SHA-224 answers for empty input, `abc`, and 55/56/64/65/1024 bytes of
`0xa5` were calculated separately using Node/OpenSSL. They cover padding/block
boundaries and the final identity tag. The same independent consumer corpus
executes natively and in standalone Wasm, in separate default and
self-authentication artifacts. It also exercises all Principal lengths and
Account Serde presence. The Wasm runner requires no imports and reports each
artifact SHA-256; these results do not establish installed IC execution.

## Reproduction and boundaries

Run `make qualify-types qualify-types-consumer check-types-graphs` after preparing
locked sources. The first target checks product tests, strict Clippy/Rustdoc and
Wasm test compilation under both feature selections. The consumer target runs
native comparison tests, Clippy and both standalone Wasm artifacts under Node.
The graph target checks normal/build closures with exactly one Types identity,
no hash supplier in the default selection, and one SHA-2 supplier when enabled.
Override `GRAPH_TARGET` for a native triple. CI runs these on each supported host.

Protocol still selects registry Types. This fixture qualifies an independent
public consumer, not a migration of Query, Protocol or a future Runtime cohort.
Those actual crossings still need coordinated source selection and qualification.
Publication remains disabled and no sibling is modified.

## Executed evidence — 2026-11-15

Local Linux x86-64, Rust 1.100.0 and Node 24.21.0 passed both product feature
selections, both native consumer selections, strict Clippy/Rustdoc, formatting,
workflow lint and default/enabled normal/build graph checks on Linux and Wasm.
The native consumer runs two default tests and three enabled tests. Both
standalone Wasm corpora execute successfully with these artifact hashes:

| Selection | SHA-256 |
| --- | --- |
| Default | `9c24cc15a0022110f399698d044ef7ec92ace2e58986ba0f0a7693fe5871d224` |
| Self-authentication | `dbcd1df49eb0ee40b747aa0a825ac908319cf562bd075e92815e3abfb8bd0a0f` |

CI includes these checks on all three supported hosts. Hosted/macOS results for
this change remain pending. No Wasm size acceptance threshold is introduced.
