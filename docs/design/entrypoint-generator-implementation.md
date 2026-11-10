# Entrypoint generator implementation record

On 2026-11-10 a separate implementer context authored the generator from
[entrypoint-generator-contract.md](entrypoint-generator-contract.md). Previous
macro sources, tests, fixtures, supplier implementations and Git/history were
excluded from this context.

## Inputs and separation

The only existing repository file newly read for this component was
`docs/design/entrypoint-generator-contract.md`, relative to
`/home/adam/projects/ic-platform`. Previously read `AGENTS.md`,
`TOKO-TOOLING.md` and `docs/principles/rust-code-hygiene.md` supplied policy without
another read. No existing macro implementation, fixture or test was inspected.
The owned external-Candid adapter was neither read nor edited.

The coordinator supplied these supplemental public/admission facts without
previous source:

- The generated wrapper is an ordinary private Rust function, not extern C.
- Native symbols use consumer-evaluated `concat!` and `env!("CARGO_CRATE_NAME")`.
- Only conditional-compilation attributes belong on the generated wrapper;
  arbitrary user/procedural attributes must not be copied there.
- Valid `impl Trait` arguments/returns require inferred generated bindings;
  explicit handler-type annotations on local bindings would reject them.
- Manual-reply return values remain in a named local until scope exit.
- Generated wrapper identifiers need a generated span; original handler spans
  remain on invocations. This avoids attributing generated-name lints to users.

A process-name check before editing found no Cargo/rustc process. Only the new
entrypoint module was written during the coordinator's baseline hold. After the
explicit release signal, lib/options/signature were replaced in full without
reading their previous contents. The signature test file was also overwritten
without reading its prior contents. The coordinator owns old-export retirement
and the mechanical external-Candid integration redirect.

The coordinator assigned public macro consumers to a separate specification-only
test author. This implementer did not delegate work or inspect those consumer
sources. Later parent-reported qualifications were not used as implementation
source or claimed as implementer-run tests.

## Authored files and behavior

This context wrote:

- `crates/ic-platform-macros/src/lib.rs`
- `crates/ic-platform-macros/src/entrypoint/mod.rs`
- `crates/ic-platform-macros/src/entrypoint/tests.rs`
- `crates/ic-platform-macros/src/options/mod.rs`
- `crates/ic-platform-macros/src/signature/mod.rs`
- `crates/ic-platform-macros/src/signature/tests.rs`
- `docs/design/entrypoint-generator-implementation.md`

One generator serves ordinary string-literal paths and the owned adapter's typed
paths. Paths are resolved only at their admission phase; manual reply never
parses an unused encoder path. Option parsing retains ordered repeated guards,
strict single-value duplication and boolean admission. Function/codec checks
use syntactic argument count and return shape, without alias resolution.

Generated locals and the wrapper identifier use mixed-site spans; user paths and
handler invocation retain their supplied spans. Decoder bindings are positional
and inferred, except the explicit unit check. Automatic encoder output retains
an explicit absolute Vec<u8> expectation. Sync guards run before context entry;
async guards run after entry and before one protected spawn. The required scoped
spawn lint allowance has its explicit consumer-configuration reason; no blanket
lint allowance was added.

Wasm exports retain exact protocol text. Native exports hex-encode every UTF-8
byte of that complete text and prefix the consumer crate name. The wrapper
retains only cfg and recursively filtered conditional cfg_attr metadata.
Ordinary expansions add no Candid registration, codec, scheduler or executor.
The ten public attributes and the owned adapter's private integration interfaces
follow the supplied contract. No dependency, feature or cryptographic change
was made.

Ten entrypoint tests and three signature tests were independently authored here.
They exercise option/path admission, error ordering, every export/context kind,
native-name encoding, raw/custom/reserved names, positional patterns, return
shape/encoding, guard/context structure, manual reply, lifecycle/function refusal,
typed paths and conditional attributes. These structural tests complement the
separately authored compiled/executed consumers; token checks alone are not
claimed to qualify runtime behavior.

## Validation and limitations

Rustfmt 1.99.1 succeeded on all authored Rust files using
`--edition 2024 --config-path /dev/null --config skip_children=true`. An initial
new-module quote-delimiter error was corrected before successful formatting.
The implementer ran no Cargo command, Clippy, compilation, consumer tests or
installed IC scenarios. Coordinator-owned checks and installed qualification are
separate evidence. No measured performance claim is made.

This record describes permitted inputs and actions. It does not provide legal
clearance, claim absence of model training exposure, or mathematically prove
independent authorship. Compatibility evidence alone cannot establish those
conclusions.
