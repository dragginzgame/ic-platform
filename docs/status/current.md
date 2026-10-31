# Current handoff

## Shared 0.27.19: symmetric groups and delegated delivery

Canonical adoption selects committed Shared revision
54fc49e12f439dbe18d232f622a6d78e61773353 (Shared Tooling 0.27.19).
Exported payloads are never edited in place. The new rules require
explicit versions for every independent-group member, symmetric releases/ ledgers,
a root changelog index and explicit command selection.

At adoption, numeric versions and dependency selections are unchanged: Platform,
macros and exact runtime guard are 0.15.1; Types is 0.15.0. Root Cargo has
no workspace.package.version. Every member owns a literal matching its group's
primary, with the same crate paths, common metadata, catalog and one lockfile.
The root Cargo comment map and ci/cargo-release-groups.json record both groups.

releases/platform/CHANGELOG.md owns Platform notes and released 0.15.1;
releases/types/CHANGELOG.md owns Types notes and its initial 0.1 history.
Coordinated releases through 0.15.0 are preserved once in releases/history.md;
relative links are rebased to keep their targets. Root CHANGELOG.md is an index.
The old Types ledger path is a short pointer and is excluded from crate archives.
Delivery selects Platform 0.15.2 and Types 0.15.1 for compatible contributor
tooling/layout changes; runtime APIs and type bytes are unchanged. Adoption alone
does not change numeric versions; delegated release preparation owns those edits.

Every release, resume and publication requires RELEASE_GROUP=platform or types.
Explicit environment selection works; missing/empty/unknown values refuse before
setup/validation/effects. PACKAGE may select only a subset of that group. When
both groups change, publish supplier and consumer groups separately in order.
The former implicit platform default and mixed publication batches are retired.

Common runtime-lock and platform-only exact guard-pin custody are preserved.
Version readers, metadata transactions and fixtures use each primary package.
A read-only resume guard rejects unfinished plans when their recorded policy
blob differs; Shared's runner still owns all recovery authorization and effects.
Older completed observations remain subject to its existing exact-source checks.
No release journals or artifacts are removed. Current platform 0.14/0.15/0.15.1
journals were complete at adoption; older legacy push-phase journals remain
retained and must be reconciled in their original source/policy context if needed.

Full Clippy, current/MSRV native/Wasm canonical identity/schema and singleton
proofs pass. Public Make/CLI refusal and roster tests, runtime lock tests, both
real group transaction/retry/local delivery/resume proofs and final governance
checks are retained under /tmp/platform-groups-018-*.log. The complete group proof
is /tmp/platform-groups-018-preparation-final.log; its disposable repositories are
retained at /tmp/platform-group-preparation.L80Dns. Fixtures substitute tool
installation/full CI, while retaining the actual editor, metadata,
staged/committed checks and local Git effects. No production release, publication
or new installed-runtime/macOS qualification is claimed.

The only open native adoption PR at the start was #122, updated at 2fb05d2.
It preserves qualified native inputs at 6c054c8 and uses pending root-ledger notes;
after this layout migration is delivered, its notes need the Platform ledger.
The maintainer now requested adoption and deployment. Shared's delegated-delivery
rules authorize the necessary scoped source commit, Types patch release and
crates.io publication, followed by Platform patch release and publication through
the existing adapters. Both groups have packaging changes; npm is unaffected.
The complete release gates and exact metadata/source admission remain required.
Observe Git-owned release journals and registry state for actual delivery status;
this plan alone is not a release or upload receipt. Preserve failed evidence and
reconcile uncertain effects before recovery. Sibling checkout edits stay excluded.

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
