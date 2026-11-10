# ic-platform-macros

Implementation support for Runtime entrypoints and Platform’s external-Candid
bridge. The generator and shared syntax helpers now have independent implementations
from a reviewed protocol/API contract, with separate public consumer tests.
The retired imported generator and its source records were removed together. See the
[qualification record](../../docs/design/entrypoint-generator-qualification.md).
The entrypoint core implements no executor or implicit codec.

The owned external-Candid bridge is available through the facade's
`candid-codec-endpoint` feature. Explicit `limits`, `reply_bytes`, `arity` and
`empty_args` are required. It delegates to this same runtime generator; guards
run before byte admission/copying. It adds no registry Candid or native service
registration. The legacy `candid_query`/`candid_update` exports are removed.
Owned Wire endpoints keep their selected codec.
See the [migration](../../docs/architecture/migration.md#explicit-owned-external-endpoints-unreleased).

The single ordinary attribute family defaults to `::ic_platform_runtime::cdk`
and is re-exported by Runtime. The former `runtime_*` family and ordinary facade
exports are removed. The external-Candid bridge supplies its explicit adapter
path. All use one generator and
support the parsed `crate = "module::path"` override for renamed dependencies. Every query/update explicitly supplies
`decode_with` and `encode_with` (or `manual_reply`); even unit RPCs run their
explicit decoder. Lifecycle argument decoding is explicit. Wire owns controlled
application encoding and framing, including public endpoints called by third
parties. Consumers own identities, guards and budgets. Explicit hooks do not prove
Wire conformance: consumers must qualify their generated bindings and wire bytes.
No implicit Candid endpoint codec, dummy service registration, Candid path option
or service-export macro is retained. The no-op `hidden` option is retired in 0.5;
remove it from declarations and generators. Options are parsed directly with Syn:
string literals for names/paths, boolean literals or bare flags for
`manual_reply`/`composite`, ordered repeated `guard` options, and no duplicate
single-value options. String-encoded booleans are rejected.
`composite` is query-only; lifecycle hooks cannot select `manual_reply`.
Generated bindings use positional names and mixed-site hygiene so they do not
capture handler names or user-supplied codec paths. Generated standard-library
types use absolute paths and do not depend on the consumer's prelude imports.
Native build symbols encode the full IC export name as UTF-8 hexadecimal with
an `__ic_export_` prefix, avoiding punctuation collisions across host object
formats. Wasm exports retain the original IC names. Native symbols are build
scaffolding, not an IC endpoint or a supported native invocation interface.
Raw Rust identifiers are supported: `fn r#match` defaults to endpoint name
`match`. Explicit `name` strings remain verbatim.

Export names, lifecycle restrictions, guard order, tuple/unit custom-encoder
arity and sync/async/query contexts retain the reviewed generator behavior.
Async entries use the preserved protected-spawn wrapper and the sole
`ic-platform-runtime` executor. `IC_CDK_DEBUG` retains opt-in expansion diagnostics.
Automatic `encode_with` retains its `Vec<u8>` expectation, including inference for
generic encoders. For a constant or fixed-size encoded response, `manual_reply`
can pass a borrowed slice or array directly to `ic_platform_runtime::cdk::api::msg_reply`.
It does not select a codec: the bytes must still satisfy the consumer's explicit
Wire or required external protocol. Consumers own the exactly-once reply and
encoding-failure behavior on this path.
The support package, Runtime and exact guard form the Runtime release group.
The facade selects that supplier independently of its crypto release version.

The facade's `candid-codec-endpoint` selection exposes the owned-codec adapters.
They require `limits`, `reply_bytes`, `arity = "exact" | "compatible"` and
`empty_args = "reject" | "unit"`. The adapter shares the same generator and
ordered guard/copy/execution paths. It does not emit native Candid method
registration or select Candid/Serde derives. External schema publication remains
consumer-owned. The legacy adapter's work quotas and behavior are unchanged;
there is no automatic quota conversion when moving to the owned codec.
