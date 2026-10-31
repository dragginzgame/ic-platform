# Current handoff

## Pending 0.15: one coordinated Rust workspace version

After publishing Platform 0.14.0 and the initial types 0.1.0 supplier, the
maintainer explicitly chose the same version for all four Rust packages.
`ic-platform-types` now inherits `workspace.package.version`; the facade's local
registry requirement follows it. All development packages resolve at 0.14.0.
The root changelog selects pending 0.15.0 for the changed type dependency and
release policy. Existing registry artifacts, tags and journals are unchanged.
Do not re-upload the modified source as the already published facade 0.14.0.

The sole `platform` group includes facade, macros, runtime guard and types. Normal
release commands update all four and their local requirements; normal publication
selects all four in Cargo dependency order. The separate `types` group is retired.
Explicit `PACKAGE` subsets remain for reconciled partial-publication recovery.
The root changelog owns future release notes; the types ledger retains its initial
publication history. Browser npm versioning/publication remains separate.

This supersedes #118's independent-version policy. One canonical type definition
and explicit facade re-exports remain, with unchanged bytes, Serde, bounds,
account semantics, conversion ownership, source bodies and provenance. Shared
version numbers do not preserve Rust identity across incompatible minors; matching
Wire/schema/caller adoption can be needed again. Wire #36 owns that integration.
The maintained identity fixture checks one release's actual facade/type identity,
real Wire trait source, ledger conversions and single-runtime profiles on current
and Rust 1.91 native/Wasm. It makes no cross-minor stability claim.

Shared remains canonically exported from committed
`ef14942393871e3e2f0c094532dd2a0ea835485a`. Its commit title is 0.27.15 while VERSION
records 0.27.14; the maintainer accepted that discrepancy. Preserve the snapshot's
actual version/revision. IC Build 0.73.0 is the admitted metadata editor. The
platform group's exact file plan retains the runtime qualification lock and guard
pin exception. Runtime metadata updates include types when the qualified lock
actually records that owner; historical locks are not resolved or invented.

Full Clippy passes. Focused qualification logs are `/tmp/platform-align-*.log`:
admission, routing, runtime lock, identity/schema and actual coordinated release
preparation/retry/local delivery/resume all pass. The real disposable release
proof is `/tmp/platform-group-preparation.JHBtsy`. Disposable tests substitute tool
installation and full CI; they do not establish production release acceptance.
No real-checkout staging, commit, tag, push, upload or cleanup is authorized here.

## Retained constraints and evidence

The maintained combined installed-runtime lock still selects Timers 0.32.3 and
its older Platform/executor graph. Its existing singleton/qualification conflict
remains separate from this packaging change. Preserve all release/runtime gates;
source and graph fixtures do not establish new installed, macOS or live IC proof.

Native Profiler's qualified profile retains published Platform 0.12 gzip,
Host Artifacts/Process/FS 0.72.1, Wire 0.11 and arithmetic-only Metrics 0.22;
its source remains the qualified provider `26ce82a78759f626fc177259eb6f73b75eafc526`.
Recorded server SHA-256:
`10c53a828fb12a99cf5bb41cbe8802be2f7e3209c2ca320cf7081d7c12ed42c7`.
Profiler owns recorder/viewer and shared PNG/range product mechanics; Platform
owns native integration/state adapters. No generic gzip facade is restored.

#50 remains unresolved: retained RustSec evidence at
`target/issue50-20261030/evidence.json` found crypto-bigint 0.5.5 without a complete
compatible correction; yanked checks were explicitly excluded. Keep protected
crypto bodies, DER exception, fingerprints and vectors unchanged. Candid's fixed
16-level external admission and legacy checkpoint compatibility remain required.

The rejected lean Candid reader saved 7,631 probe bytes while adding about 3k
maintained source lines; registry Candid remains selected. Historical runtime Vec
and selected management DTO evidence is retained under `target/vec-runtime-20261030`
and `target/management-adoption`; do not infer fresh acceptance from those receipts.
The migration contract remains the detailed owner/provenance/qualification source.
Earlier installed receipts and consumer-adoption context remain in the
[0.14 release handoff](https://github.com/dragginzgame/ic-platform/blob/7b4ed265804a60b796c4680880cb7a4e66e53d1e/docs/status/current.md).
