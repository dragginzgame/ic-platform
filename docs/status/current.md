# Current handoff

## #118: restore independent Platform and Types groups

Latest approval is the implementation section of Platform #118. It supersedes the
synchronized 0.15 policy: keep one workspace, dependency catalog and root lockfile,
with platform (facade/macros/exact runtime guard) and types (ic-platform-types)
versioned independently. Both numeric baselines remain published 0.15.0. Types
now owns a literal version; root requirements and numeric package versions stay
unchanged. No existing release is rewritten or re-uploaded. Pending group notes
select 0.15.1 separately; root retains coordinated history through 0.15 intact.

Root Cargo comments map every member, canonical version source and ledger; JSON
is executable policy. Platform releases preserve Types' selection/requirement and
ledger; Types releases select their own version/notes and affected requirements.
Default publication excludes unchanged Types. Explicit group and PACKAGE batches
retain full gates, retained attempts and supplier-first Cargo ordering.

Shared release commit 0.27.17 is canonically exported from revision
39ba9836a269ee7ea7f073a3f23d9cefe3c7d03c, including Shared PR #330's reviewed common
metadata custody. All 117 payloads verify; none was edited in place. Top-level
shared_metadata_files declares one common lock, retaining duplicate/reserved-path
rejection and the existing transaction/receipt/runner. IC Build 0.73.0 remains the
admitted published editor. The supplier custody gap is cleared. The supplier
VERSION still says 0.27.16; snapshot metadata preserves that supplied value,
following the maintainer's earlier instruction to ignore the version-label wrinkle.

The combined runtime lock is common to both groups; the exact guard pin remains
platform-only. Consumer preparation renders all selected metadata in disposable
committed source and installs it through Shared's transaction. Failed rendering
must leave real metadata byte-identical, so exact resume safely retries.
Runtime semantic checks treat the optional recorded Types owner independently;
historical locks without it are preserved rather than resolved or invented.

Full Clippy, current/MSRV native/Wasm cross-minor identity/schema profiles,
independent runtime-lock and both real group preparation/resume tests pass.
Failed consumer rendering preserves all originals; wrong-group resume refuses.
Evidence is /tmp/platform-group-preparation.9w8ueo. The identity fixture uses actual Wire 0.12.1
trait/implementation source with only its Types dependency aligned to local 0.15;
published Wire 0.12.1 originally selected Types 0.1, so published adoption is not
claimed. Wire #36 owns that product change. Disposable release qualification uses
the committed PR snapshot; all
117 payload hashes and release-policy/adapter inputs match the merged adoption.
It substitutes tool installation/full CI, retaining real editor, metadata and
local Git effects. Production release and installed-runtime acceptance are separate.
Release evidence: /tmp/platform-118-independent-preparation.log. Other focused
logs use /tmp/platform-118-*.log. No normal-checkout staging, commit, tag, push,
upload or cleanup was performed. The Shared fix is delivered through merged #330.

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
