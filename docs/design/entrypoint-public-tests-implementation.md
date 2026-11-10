# Independent public entrypoint consumers

This author used `docs/design/entrypoint-generator-contract.md` and previously
admitted policy. No existing or replacement generator bodies, generator tests,
link fixtures, supplier implementations, Git history or diffs were read. Earlier
work in this context included independently authored executor/task-adapter tests
and implementation, and explicitly authorized exposure to the newly written raw
call transport; it did not expose entrypoint-generator implementation details.

Fresh sources are `crates/ic-platform-macros/tests/entrypoint_consumer.rs`,
`tests/entrypoint_native_link.rs`, and the ordinary Rust fixture files beneath
`tests/fixtures/native_link/`. The consumers call the contract-specified private wrapper
spelling in its defining module solely for native observations. This does not
establish a public native invocation API.

The fake runtime records guard, context, argument copy, decoding, handler,
encoding and reply events. Its spawn helper polls an immediately-ready fixture
future exactly once with safe standard pinning and a no-op Waker, and refuses
Pending. It is an observation seam, not a replacement executor or IC simulation.

Consumers cover every ordinary entrypoint kind, zero/scalar/multiple and
single-tuple argument shapes, scalar/tuple/unit/aliased-unit results, typed
patterns, synchronous and asynchronous ordering, ordered guards and refusal,
manual reply with an unused invalid encoder path, generic codec paths and
encoder inference, raw handler identifiers, prelude/path hygiene and cfg after
the macro attribute. There is only one of each lifecycle export in each crate.
Lifecycle decoder alternatives use separate fixture crates.

The native-link harness creates a disposable standalone Cargo workspace under
the repository's `target/`, with two role libraries and a binary linking both.
It copies whole maintained fixture files without transforming or injecting Rust
source, generates only fixture manifests, runs Cargo offline against the selected
local macro package and uses its own target directory. It then uses native `nm`
to check the exact crate-prefixed hexadecimal export names, including every
lifecycle kind, composite queries, raw identifiers, empty/Unicode names, and
punctuation distinctions repeated across both role crates. Failure preserves
the fixture and captured Cargo/nm output; success removes the disposable tree.
The coordinator explicitly authorized this generated-fixture workflow.

The coordinator reports that the original eight consumer cases and native-link
harness passed against the old generator. After that baseline, review prompted
two additional consumer cases: an `impl Trait` parameter and return with a
concrete decoder and generic encoder, and collisions with the generated names
`__ic_argument_0` and `__ic_result`. The coordinator supplied those two spellings
as supplementary collision facts; no generator source was read. These additions
are review regressions for the replacement, not old-owner baseline evidence.

Also after the baseline, the native-link child command was changed to remove
`CARGO_BUILD_TARGET` and set `IC_CDK_DEBUG` to the empty string. It checks captured
stderr for one known generated export literal, without asserting debug output
decoration. Environment changes apply only to the child command; there is no
global process-environment mutation. This exercises opt-in by variable presence
while ordinary consumer compilation retains its default debug setting. These
harness additions likewise await replacement-owner coordinator execution.

The author ran only rustfmt 1.99.1 with an empty configuration and child traversal
disabled. The coordinator owns product manifest registration, Clippy, execution
against the old and replacement generators, and installed Runtime qualification.
These successful-consumer sources do not independently qualify compile-failure
admission diagnostics or the unchanged external-Candid adapter. Native linking
and the fake runtime do not establish Wasm export or installed IC behavior.
No legal-clearance or performance claim is made.
