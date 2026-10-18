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

Enable `candid-parser` on native hosts for `ic_platform::candid_parser`. It
explicitly exports `IDLProg`, `check_prog`, `TypeEnv`, `Type`, `TypeInner`,
`FuncMode`, `Dec`, `IDLMergedProg`, `Error`, `Result`, `IDLArgs`, `parse_idl_args`,
`CandidSource`, `instantiate_candid`, `service_equal`, `TypeContainer`, `type_equal`,
`compile_javascript` and `compile_typescript`. These retain upstream identities
and semantics. The feature does not enable Platform's `candid` or `executor`;
the upstream parser itself depends on Candid with full type/value support.
On Wasm this module and parser dependency are absent even with all features.
Callers own source/output limits, imports and diagnostics. `check_prog` ignores
imports; inspect `Dec` first when requiring self-contained source. Parse `IDLProg`
and call `check_prog` for silent in-memory checking. `CandidSource::File` reads
files/imports; its loaders and service comparison can print parser diagnostics.

The contract also exports `ArgumentEncoder`/`ArgumentDecoder` for generic call
helpers, `Nat`/`Int` for arbitrary-precision external values, and `Reserved` for
partial projections. The `candid` feature includes upstream big-number support.
Use `Result<T, contract::Error>` for encoding errors and `DecodeError` for bounded
decoding failures. Existing wire labels and caller-owned limits remain unchanged.

`executor` exports the exact selected task/context types and operations. It owns
no independent task registry. Callers must preserve context placement, callbacks
and trap cleanup; native compilation does not qualify installed IC behavior.
No full CDK entrypoint macros, call facade, scheduler or transport exists.
Selected upstream dependencies remain transitive dependencies.
