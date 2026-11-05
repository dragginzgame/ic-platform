# Native Profiler module layout — 2026-11-05

This is the source-profile qualification for
[#140](https://github.com/dragginzgame/ic-platform/issues/140), separate from
published Platform library qualification and consumer pin adoption.

## Source boundary

The existing assembler applies manifest source-prefix mappings after merging
disjoint provider and consumer inventories. Longest-prefix selection routes
server modules, arguments and rendering into `rs/pocket_ic_server/src/profiling`.
Operations and routes retain their private parent modules under `src/pocket_ic`
and `src/state_api/routes`. Public aliases and logical module identities stay
unchanged. All 21 introduced path attributes are removed; there is no forwarding
`include!`, source symlink, new crate or second assembler.

The independent recorder remains `profiling/system`, preserving its Cargo/Bazel
identity and dependency paths. Viewer assets remain `profiling/viewer`; compiled
data includes and Bazel inputs follow the split. Source catalog labels describe
the new compiled location. Rendering fixtures move with their module. Consumers
do not need product-source edits for this layout change.

Nine patch-only DTO serialization, CLI, storage, embedder and memory-tracker test
modules, plus two standalone CLI tests, now live in ordinary overlay files.
Their extracted bodies match the Rustfmt-normalized originals; only layout and
formatting changed. One new regression checks profile subcommand parsing. The
native gate explicitly runs the moved selections before installed scenarios.
The Bazel integration suite excludes the new child module from standalone test
target discovery and supplies it as an extra source. Existing IC lint configuration
is preserved, including its twelve-argument threshold. Two unnecessary scoped
argument expectations are removed. The native CLI's large argument variant is
boxed, with unchanged Clap selection and dispatch semantics, rather than hidden
by a lint allowance. No cryptographic, charging or protocol body is altered.

## Qualification custody

The IC gitlink remains `ee8086a9773725fd1058832a9400297cf0990112` and pristine.
The provider patch pins Rust **1.99.1** for the generated native workspace.
The paired input is committed Profiler source
`50d954813680044d916e799f2d21e926e3f0b9db`, read through `git archive` into a
disposable fixture under `target/issue140/consumer`. This avoids writing into
the concurrently dirty sibling checkout. The fixture contains no implementation
changes and is not a consumer adoption or release.

Assembly and command receipts bind every merged source, patch, overlay, manifest,
compiler and executable hash. Current logs are under `target/issue140/`; generated
receipts and retained executables are under its consumer fixture's `.build/`.
These paths are reclaimable validation evidence, not a new maintained source owner.

## Validation

The exact manifest integrity and real pinned assembly pass. All 57 assembler
tests pass, including ownership, prefix selection, merged consumer origins,
traversal/symlink refusal and gate ordering. The two focused native gate cases
also pass after extending refusal coverage to every moved test selection.

The complete Linux `native-verify` command exits zero on Rust 1.99.1:

| Selection | Result |
| --- | --- |
| Strict server library and executable Clippy, before builds/tests | Pass |
| Server build and retained executable checksum | Pass |
| Recorder | 83 passed |
| Server product and rendering | 114 passed |
| Storage / CLI / DTO serialization | 3 / 11 / 11 passed |
| Moved embedder and memory-tracker tests, including integration | 27 passed |
| Installed profiling scenarios against the retained executable | 6 passed |

All command receipts bind assembly recipe
`1dbc5489baf070e12d4dd8205ec33647534f2e376db53aa5263959796d1b45d6`.
The installed command records the same server SHA-256 as the build:
`e2240c66dc9fae96f9e78c8a183e61dbf0902547b0a7a376496250413d2c383b`.
It finished at `2026-11-05T17:18:14Z`. Summary:
`target/issue140/qualification-summary.json`; full log:
`target/issue140/native-verify-final.log`; immutable command receipts:
`target/issue140/consumer/.build/receipts/`. Normal sandbox execution initially
refused Unix sockets; qualification passed with required host permissions.
No failed check was waived. Documentation, pin, formatting and test-body
preservation checks pass; the unrelated root Cargo.lock selection is unchanged.

macOS execution and Bazel execution have not been run on this Linux host.
Updated Bazel module/data declarations are source checks, not Bazel acceptance.
NNS genesis needs separately supplied real canister artifacts; the native Cargo
placeholder inputs are not claimed as installed NNS qualification.
