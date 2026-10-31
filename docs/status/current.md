# Current handoff

## Delivered Shared and Rust groups

Shared Tooling 0.27.19 is adopted at committed revision
54fc49e12f439dbe18d232f622a6d78e61773353 through the canonical exporter.
Snapshot payloads remain intact. Platform/macros/runtime guard 0.15.2 and Types
0.15.1 are released and published. Types delivered first at 668e44f, then Platform
at 3587cd2; GitHub main/tags, registry visibility, unyanked versions, archive
checksums and embedded clean source identities were verified. Both complete
release gates passed. Publication evidence is retained under target/publication/
and /tmp/platform-deploy-*-publication.log; release plans are Git-owned.

Every package has a literal version, with no workspace.package.version. Root
Cargo keeps common metadata/dependencies and the group comment map. One lockfile
and existing crate paths remain. Group policy is ci/cargo-release-groups.json.
Canonical notes live in releases/platform/CHANGELOG.md and
releases/types/CHANGELOG.md. Root CHANGELOG.md is an index. Coordinated history
through 0.15.0 is preserved once in releases/history.md, with links rebased.
Initial Types 0.1 history stays in its own ledger; the old crate ledger is a
navigation pointer excluded from archives.

Release, resume and publication require explicit RELEASE_GROUP=platform or types,
including environment selection. Missing/empty/unknown groups refuse before
setup/validation/effects. PACKAGE only narrows that group; publish changed groups
separately in supplier order. Common runtime-lock and Platform-only exact guard
pin custody remain transactional. A read-only guard rejects unfinished saved
attempts under changed group policy; Shared retains recovery/effect ownership.
Older legacy push-phase journals remain retained and require their original
source/policy context. Never reinterpret or discard their evidence implicitly.

## Native Profiler PR #122

PR #122 removes native Profiler's forwarding-only registry Platform dependency
and selects Host Artifacts 0.72.1/gzip directly. Host Process/FS/Artifacts 0.72.1,
core Wire 0.11 and arithmetic-only Metrics 0.22 remain selected. Consumer product
imports, levels, strict gzip framing, 128 MiB API and 256 MiB recorder ceilings,
ICPROF03 bytes and source-pin qualification obligations are unchanged. Older
consumers keep their qualified provider; no new generic facade is restored.

Current released main is merged into the PR without rewriting its history.
The root changelog remains the index; published group entries and Types metadata
are preserved. Pending Platform 0.15.3 collects this compatible native packaging
change, without changing any package version. The user requested this PR's merge;
that instruction does not select another release/publication operation.

The native manifest and dependency patch remain byte-identical to qualified
provider 6c054c86bb10c2630e61ecd5b9b24d8a06de3902. Patch SHA-256 is
572c0a88309f6990cfd487f494909b484f63452433a607b58efd262f7adf0aac.
Retained qualification includes strict native/standalone Clippy, 83 recorder
tests on current Rust and 1.91, 114 server tests, six installed scenarios, actual
Bazel repin/alias loading and reconstruction of all seven dependency/build files.
Executed server SHA-256 was
3a912fa302cdd9aad9f2f9fdad285a1fe95e9388a0073ae9d5d02d412d63c74e;
paired Profiler artifacts/platform-013-adoption-20261031/ owns those receipts.
They bind that immutable provider, not a later merge commit. A new consumer pin
retains its normal native/installed and macOS qualification requirements.

The original PR native-provider-source and browser jobs passed. Its old root
release-routing fixture failed; current main's replacement passed both releases.
Installed-runtime and advisory preparation failed at the separately tracked
Timers/executor conflict (#116), before their intended qualification. Do not
weaken guards, claim those jobs passed or infer a new installed-runtime proof.
Hosted checks on updated source remain asynchronous unless GitHub enforces them.

## Retained constraints and evidence

The maintained installed-runtime lock still selects Timers 0.32.3 and its older
Platform/executor graph. Preserve every runtime/release gate and source guard.
Profiler owns recorder/viewer and PNG/range product mechanics; Platform owns
native integration/state adapters. Preserve the IC source pin and copyrights.

#50 remains unresolved: retained evidence at target/issue50-20261030/evidence.json
found crypto-bigint 0.5.5 without a complete compatible correction; yanked checks
were explicitly excluded. Protected crypto bodies, DER exception, fingerprints
and vectors remain unchanged. External Candid's fixed 16-level admission and
legacy checkpoint compatibility remain required.

Registry Candid remains selected: the rejected lean reader saved 7,631 probe
bytes but added about 3k maintained source lines. Earlier Vec/management evidence
remains at target/vec-runtime-20261030 and target/management-adoption; it is not
fresh acceptance. The migration contract owns detailed provenance and contracts.
Earlier installed/consumer context remains in the
[0.14 release handoff](https://github.com/dragginzgame/ic-platform/blob/7b4ed265804a60b796c4680880cb7a4e66e53d1e/docs/status/current.md).
