# Current handoff

## Compatible cleanup after 0.6.15

This local batch starts at released Platform 0.6.15 (`6dcf54a`). Pending notes use
0.6.16 because the changes preserve the public API, feature selections, external
wire bytes and persisted formats. Package versions remain 0.6.15. No staging,
commit, push, release, publication or sibling edit is part of this batch.

The incoming uncommitted root lock selects Wire/derive 0.7.5 and is preserved.
The manifest still declares Wire 0.7.2 ranges; the independent maintained runtime
lock still selects 0.7.2. This cleanup does not upgrade or reconcile either lock.
Shared Tooling 0.26.8 remains selected at `0b07c2a`; snapshot payloads are unchanged.

Implemented cleanup:

- Consumer features, source/assets and Wasm commands have one reviewed TSV profile
  table. Unknown profiles, graph admission, Cargo failure propagation and the
  parsed final-loop exit retain their behavior.
- Published consumer, metadata and runtime proofs share exact registry identity
  admission. Generator presence is required for runtime qualification; an optional
  generator, when present elsewhere, must also match the facade's exact version.
  Runtime qualification retains older registry facades through Timers while
  refusing replacements and requiring the exact supplier/generator pair.
- Runtime manifests are explicit templates with shared workspace metadata. Setup,
  graph admission and compilation retain their ordering and separate lock owner.
- The reviewed feature inventory and native-only leaf dependency policy share one
  JSON input. Cargo still supplies structural facts; graph permission/refusal
  checks and adversarial fixtures remain independently specified.
- Formatting write/check commands share their extra source list. Installed-runtime
  CI selects common host/Rust setup without release-editor or frontend tooling;
  standard contributor setup and CI still retain the complete frontend proof.
- Entrypoint validation resolves paths and reply/context decisions before token
  emission. Guard order, explicit unit codecs, mixed-site locals, symbols and
  sync/async executor selection are preserved.
- Principal deserialization directly preserves format errors. Bounded grammar,
  visitors, binary bytes and external conversions are unchanged.
- The experimental scaffolding checker now follows its sole supported secp path;
  active source/license preservation and attribute-location checks are unchanged.

## Qualification

Full root all-target/all-feature `make clippy` passes before longer tests. Macro
and Principal unit tests pass on Rust 1.99.1 and 1.91.0, including structured
format-error preservation. An isolated before/after comparison finds identical
emitted token strings and diagnostic text in 2,208 cases across all eight macros.
This comparison does not independently establish span hygiene or installed behavior.
The proof retains both source copies under `target/macro-equivalence.z1SoQI`.

Consumer admission stubs cover all 22 profiles and their real source/assets,
unknown-profile rejection, runtime graph refusals, final Cargo failure and live
script edits. Registry admission and native-leaf closure/graph refusal tests pass.
All 22 standalone profiles pass native tests on both compilers and applicable
Wasm checks/builds, without warnings or package-cache lock waits. All 240 routine
selections pass graph admission per target; the selection helper's exhaustive
completeness test also passes. Source/license, formatting, boundary, inventory,
documentation-link and snapshot checks pass; all-feature MSRV Wasm compiles.

The runtime fixture passes native/Wasm graph admission for both protocols,
all-target/all-feature canister Wasm Clippy and four release Wasm builds
(Wire/external Candid, Rust 1.99.1/1.91.0). Runtime evidence and preserved artifacts
are in `target/cdk-runtime.ZSVYbe`; feature graphs are in
`target/feature-graphs/attempt.3uyWPo`. Logs are `/tmp/platform-cleanup-*.log`.

Native macOS and installed-canister qualification have not been rerun for this
batch; no admitted Testkit server is configured locally. Full release CI remains
maintainer-owned. No runtime timing or Wasm-size improvement is claimed.

## Retained contracts and retirement gate

Executor =2.0.0 and ic0 =1.2.0 retain their registry identities. Callback/trap and
cryptographic implementations, licenses, source guards and vectors are unchanged.
No default features, alternative executor, codec fallback or new allocation owner.
Required external IC Candid interfaces and consumer-owned retry/resource policies
remain intact. The [migration contract](../migration.md) and
[IC inventory](../ic-inventory.md) remain authoritative for the public boundary.

Compiler retirement remains gated by
[#38](https://github.com/dragginzgame/ic-platform/issues/38) and
[Canic #613](https://github.com/dragginzgame/canic/issues/613). Canic's working
frontend uses Wire bindings, but committed `cf2d3d1fb` still imports/calls
`compile_javascript` and `compile_typescript`. Its frontend file is dirty; do not
retire exports or claim committed adoption from that working source. External DID
inspection, typechecking and textual call tooling still require their parser
surface. Removing compiler exports alone would not remove upstream Handlebars.

Published 0.6.15 supplies the Wire 0.7 Principal/Account metadata integration;
prior detailed qualification is preserved at `6dcf54a:docs/status/current.md`.
Wire producers must align metadata identities; storage/allocation, schema admission,
trust and interpretation remain with their consumers. The crypto-bigint advisory
and advisory-index preparation remain separately tracked by
[#50](https://github.com/dragginzgame/ic-platform/issues/50) and
[#56](https://github.com/dragginzgame/ic-platform/issues/56).
