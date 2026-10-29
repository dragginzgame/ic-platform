# ic-platform-macros

Implementation support for Platform's optional `cdk` feature. This maintained
Apache-2.0 generator derives from IC CDK 0.20.3 release revision
`624606f7fd1ecd6668e9608594946486aec29167`; its complete original license remains.
Its entrypoint core implements no executor or implicit codec and requires no
upstream change.

The external-Candid bridge is available only through the facade's
`candid-endpoint` feature. It supplies explicit codec functions and native Candid
registration, then delegates to this same entrypoint generator. `limits` and
`empty_args` are mandatory; argument decoding automatically refuses more than
16 constructed wire levels. Caller guards retain their ordering and execute
before byte admission/copying. No additional nesting option is needed. This
preserves existing query/update macros' explicit-codec contract and keeps owned
Wire endpoints on their selected codec.
See the [migration](../../docs/migration.md#external-endpoint-and-protocol-codecs-78-79).

The default runtime path is `::ic_platform::cdk`; a parsed `crate = "module::path"`
override supports renamed dependencies. Every query/update explicitly supplies
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
Async entries use Platform's preserved protected-spawn wrapper and the sole
registry executor. `IC_CDK_DEBUG` retains opt-in expansion diagnostics.
Automatic `encode_with` retains its `Vec<u8>` expectation, including inference for
generic encoders. For a constant or fixed-size encoded response, `manual_reply`
can pass a borrowed slice or array directly to `ic_platform::cdk::api::msg_reply`.
It does not select a codec: the bytes must still satisfy the consumer's explicit
Wire or required external protocol. Consumers own the exactly-once reply and
encoding-failure behavior on this path.
The support package and facade require coordinated publication.
