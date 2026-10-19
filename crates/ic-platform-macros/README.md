# ic-platform-macros

Implementation support for `ic-platform::cdk`; consumers declare `ic-platform`
and select its `cdk` feature. This is a maintained fork of the IC CDK 0.20.3
entrypoint generator, not an executor or runtime implementation.

The default runtime path is `::ic_platform::cdk`; the default Candid path is
`::ic_platform::candid::contract`. Explicit `crate = "module::path"` and
`candid = "module::path"` overrides retain parsed path validation. Use the
matching Candid path in `export_candid!(module::path)` when overriding it.
No-argument `export_candid!()` uses the Platform contract.

Argument/reply handling, export names, guards, lifecycle restrictions, custom
codecs, manual/hidden methods, CDK contexts and spawning preserve upstream
0.20.3 behavior, including skipping quota 10,000. `IC_CDK_DEBUG` retains the
upstream opt-in expansion diagnostics. Caller resource and disclosure policy
remain outside the generator. Inherent runtime dependencies stay upstream.
`hidden = true` with custom codecs removes generated Candid operations, not the
`cdk` feature's transitive Candid dependency. There is no Candid-free entrypoint
bundle under the current registry-runtime contract.

Imported source: `dfinity/cdk-rs` release revision
`624606f7fd1ecd6668e9608594946486aec29167`, package `ic-cdk-macros` 0.20.3.
The Apache-2.0 license is retained. Platform maintains the path correction and
public entrypoints locally; upstream acceptance is not a delivery prerequisite.
The paired `ic-platform` dependency requires the coordinated package
version. Publish both packages through the repository's workspace workflow.
