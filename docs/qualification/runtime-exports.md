# Generated IC update exports — Runtime 0.23.3

The implemented generator composes the qualified manual update adapter.
The published macro identity is `ic-platform-macros`,
not `ic-platform-runtime-macros`. The official crates.io sparse index checked on
2026-11-15 reports non-yanked 0.22.5 with checksum
`9de8495d8ca3ff74e9d458b89674cc7a916460fdd1b5c6764bcc03760cd8c65c`.
The replacement retains that name and joins the coordinated Runtime 0.23.3
group with an exact dependency. It reuses the already selected proc-macro2,
quote and syn suppliers. No old generator source or fixture was copied.

## Approved exception

The maintainer approved extending the existing IC0 exception to generated IC
export attributes. Rust requires `unsafe(export_name = "canister_update …")`
on these functions. The generator applies one justified item-scoped
`expect(unsafe_code)` to a fixed-signature wrapper. The macro implementation and
wrapper body contain no unsafe operations. Qualification uses workspace deny;
all pre-existing non-export targets now also have crate-root unsafe forbids.
The dedicated `runtime-exports` package and export UI fixtures admit only this
generated attribute boundary.

This restores an integration capability already present in the old macros; the
export attribute itself is not a new correctness improvement. Qualification is
for this replacement's adapter and ownership contract.

## Explicit declaration

Enable Runtime's opt-in `entrypoint-macros` feature. A declaration is explicit:

```rust,ignore
#[candidate_runtime::update(
    runtime = candidate_runtime,
    name = "echo",
    guard = admit,
    decode = decode,
    max_bytes = 4,
    on_error = fail,
    manual_reply
)]
async fn echo(input: Vec<u8>) {
    candidate_runtime::system::reply(&input).unwrap();
}
```

The handler must be a safe async nongeneric function taking one decoded input
and returning unit. `on_error` receives the exact `UpdateError` and must return
`!`; failure behavior is supplied by the application, not inferred by Runtime.
The fixture chooses to trap. The wrapper has no Wasm parameters or results and
calls `update_manual`, dropping only its observational task handle after admission.

Names must be nonempty and contain no NUL; the generator always supplies the
`canister_update ` prefix. Runtime paths are explicit and support renamed
dependencies. Direct cfg and cfg_attr that conditionally applies cfg gate both
handler and export. Other export/instrumentation attributes, query options,
automatic encoders, duplicate options and unsupported signatures are refused.
This first slice does not select a codec or send a reply automatically.

## Executable evidence

`make qualify-runtime-exports` runs six generator tests, strict Clippy/Rustdoc,
builds the dedicated cdylib and invokes its actual exports. The host requires
exactly two update functions, zero parameters/results, and only context,
argument and reply imports. Binary and empty echo replies pass; denied guard,
oversized input, decoder failure and non-replicated entry trap at their expected
stage. Each trapping case uses a fresh instance. Disabled cfg/cfg_attr handlers
with deliberately unresolved symbols produce no exports.

An independent Cargo build driver compiles one positive control and five
maintained negative declarations. Unsupported options and signatures, invalid
decoder types and returning failure handlers fail with expected diagnostics.
Duplicate update names fail during real Wasm code generation; this is not merely
a parser test. The driver checks structured compiler results, generates only
temporary manifests, and never injects or replaces Rust source.

The Wasm production graph contains exactly one local Runtime, guard, macros
package and Types identity. Guard conflict probes and input-lifetime compile
checks still pass. Local Rust 1.100.0 / Linux / Node 24.21.0 validation also passes
25 Runtime tests and the existing IC0, argument, manual-update and awaited
callback corpora. Hosted/macOS validation remains pending for this increment.

Generated export artifact SHA-256:
`382f0354a134248b54910b87dc09469dcfb699417460da866ecf91ac05ab274c`.

Installed canister execution and rollback remain next. The generated-export
fixture currently completes immediately; suspension/cleanup runs through the
same manual adapter in a separate fixture. Query/lifecycle exports, automatic
reply modes, actual consumer migration and protection against unguarded or
modified competing executors are not established by these tests.
