# ic-platform

Controlled Candid and executor dependencies. Enable `candid` for external
protocol DTOs and bounded replies; enable `executor` for the selected CDK executor.
`ic_platform::Principal` is unconditional; no features are enabled by default.

`ic_platform::candid::{DecodeLimits, DecodeError, decode_one, decode_args,
decode_exactly_one}` preserves Testkit's explicit limits and typed failures.
`decode_one` and `decode_args` allow bounded skipping of additional arguments;
`decode_exactly_one` requires one actual wire argument. All select compact Candid
diagnostics, which are not a redaction guarantee. Callers retain raw bytes,
aggregate accounting, trust, resource policy and domain errors.

Use `ic_platform::candid::contract` for Candid DTO derives and encoding. Add
`#[candid_path("ic_platform::candid::contract")]` to derived types, and
`#[serde(crate = "ic_platform::candid::contract::serde")]` when deriving Deserialize
through the boundary's Serde export. These are external protocol types, not an
application-wide requirement. Principal-only users import `ic_platform::Principal`
without selecting a feature or declaring `ic_principal` directly. The root and
Candid contract exports have the same upstream type identity.

Enable `certification` for `ic_platform::certification`: upstream `Certificate`,
`Delegation`, `HashTree`, `Label`, `LookupResult`, `empty`, `fork`, `labeled`,
`leaf`, `pruned` and `fork_hash`, with upstream Serde support. This feature is
independent of Candid and the executor. Host retains its strict bounded CBOR
decoder; consumers own trust roots, authentication, budgets and leaf meanings.
Neither construction nor deserialization authenticates a certificate. This
centralizes dependency control without removing upstream code or promising
smaller Wasm artifacts.

The contract also exports `ArgumentEncoder`/`ArgumentDecoder` for generic call
helpers, `Nat`/`Int` for arbitrary-precision external values, and `Reserved` for
partial projections. The `candid` feature includes upstream big-number support.
Use `Result<T, contract::Error>` for encoding errors and `DecodeError` for bounded
decoding failures. Existing wire labels and caller-owned limits remain unchanged.

`executor` exports the exact selected task/context types and operations. It owns
no independent task registry. Callers must preserve context placement, callbacks
and trap cleanup; native compilation does not qualify installed IC behavior.
No full CDK entrypoint macros, call facade, parser, scheduler or transport exists
in this initial package. Both upstream dependencies remain transitive dependencies.
