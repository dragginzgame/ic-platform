# ic-platform

Controlled Candid and executor dependencies. Enable `candid` for external
protocol DTOs and bounded replies; enable `executor` for the selected CDK executor.
No capabilities are enabled by default. This initial package is unpublished.

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
application-wide requirement. Principal-only users can select `ic_principal`.

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
