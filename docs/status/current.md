# Current handoff — 2026-11-13

## Published and reconciled

Frozen batch `batch-q_67SewF` completed GitHub delivery and crates.io publication
in supplier order, then reconciled into ordinary main. All five packages were
verified available at their exact new versions; tags dereference to these commits:

| Group | Version | Commit | Tag | Qualification / runner |
| --- | --- | --- | --- | --- |
| Types | 0.20.0 | `508c60b1ff2e8b5db4a80d9ba9ab50c724c98cc5` | `types-v0.20.0` | `7zMqrl` / `yHItvq` |
| Runtime/macros/guard | 0.22.0 | `8d88fe054816e467e3eaec88e79285737f00500a` | `runtime-v0.22.0` | `qjP0cD` / `oW60AE` |
| Platform | 0.32.0 | `14c954e5ae30ef2100ab311d8b3e6dc3db6948fa` | `platform-v0.32.0` | `624oov` / `XIahtu` |

Reviewed source: `7d0d86565092aee1654ef1d3a01dea611b9262c9`. All 19 required local
stages passed per group (57 parent dispatches), including native tests, consumers,
feature/Wasm graphs, strict Clippy, documentation and installed Runtime. Metadata
and exact archives were freshly admitted. Runtime's guard remains exact. Browser
stays 0.2.0; no npm or sibling release occurred.

Deployment log: `target/deploy-0401-rust1100.log`. Complete receipts, runner logs
and archive/publication evidence remain under
`target/release-frozen/batch-q_67SewF/target/{release-validation,publication}/`.
Durable intent/Git/journals remain in `.git/release-state/frozen/batch-q_67SewF/`.
The first release-command stage took 405 seconds; later groups took 277 each,
reusing only the admitted immutable policy-fixture proof, with fresh metadata
and all product obligations. No collector failure or full-gate retry occurred.

## Compiler, tooling and follow-up fixture repair

Both workspaces select published Toko 0.40.1, checksum
`fb884b05e2e5bbca2388dd0b757833fb5ad37e91b3ced09f6a37b454c7fe9f90`, producer
commit `dcc51b5cab3d9993f0668bc0902252364d7dc316`. Authenticated adoption retained
98 managed payloads/entrypoints; 0.40.1 changed only the baseline from 0.40.0.
No snapshot-owned file was edited by hand. The
[release guide](../development/release-validation.md#deployment-sequencing) records
one full-gate owner per group, wiring-first checks and phase-specific recovery.
General partial-gate and automatic collector-only resume remain unsupported;
identify invalidated inputs and disclose rerun scope before a retry.

Rust 1.100.0 is the minimum/development/release compiler. Explicit preparation
admitted rustc `0ef1bc386059596dada8fb65b50425aab2ea637a` (2026-11-09), LLVM
23.1.5 and Cargo `3acc79c0c` (2026-11-05), Linux x86_64, Clippy/Rustfmt and Wasm.
All real members inherit common lints. Safe-only libraries forbid unsafe code;
Runtime/codec ownership and FFI expectations remain scoped and reasoned.
Generated fixtures project the owner's policy; real refusal evidence remains in
`target/lint-policy.LoNnwc/` and the release gate's own primitive-check evidence.
The six directory-module moves preserve public paths and formatting/custody checks.

Hosted Wasm-size runs failed before measurement because the standalone probe's
manual IC export attributes lacked unsafe-code expectations; Runtime-only builds
also retained unused shared I/O helpers. The post-release Git-only fixture repair
adds local ABI expectations, gates those helpers to their actual independent
profiles and documents the active entrypoints. No library or package payload
changed, so it needs no additional crate release. Strict Wasm Clippy passes for
each of the five actual workflow profiles. Comparator tests pass.

Matched pre-release `d524dee` versus 0.32.0 measurements use identical corrected
probes, Rust 1.100.0, opt=3, fat LTO, one codegen unit, abort/strip and ic-wasm
0.11.1. Every raw/optimized total and code delta is zero for principal-binary,
principal-self-auth, Runtime, signature verification and certificate CBOR.
Evidence: `target/evidence/wasm-regression.Do2myE/`;
logs: `/tmp/platform-deploy0401-{wasm-fixed,probe-*,compare-tests}.log`.
Retain the failing hosted artifact at `target/evidence/hosted-wasm-43403360773/`.
Fresh hosted/native macOS qualification remains asynchronous; Linux and installed
IC evidence do not establish native macOS acceptance. #157 retains that remaining
acceptance. #50 remains open and unresolved under its workspace-only expected
advisory disposition; runtime/new findings and incomplete/tool failures still fail.

For this batch, npm 12.2.0 was explicitly prepared under
`target/deployment-tools/npm-12.2.0/node_modules/.bin`; prepend it to PATH for exact
recovery context. The global npm was 12.2.1. PocketIC came from
`.tools/ic-testkit-server/16.1.0-x86_64-linux/pocket-ic`. Do not change their saved
identities while recovering this batch. Qualification's adopted registry baseline
remains Platform 0.31.3; optional Wire fixture 0.10/0.12.1 and editor 0.73 migrations
remain deferred to their separately qualified maintenance batches.

## Preserved findings and recovery custody

The historical verification-size finding against `fc55ae1` (+135 total / +133 code,
above its 128-byte allowance) remains unresolved; the new zero-delta comparison
uses a different baseline. See [the audit](../benchmarks/platform-031-audit.md#post-release-cleanup).
No cryptographic, certificate, delegation, identity-key, witness or freshness check
was removed. Earlier BLS/identity/Agent evidence remains at its existing owner.

Preserve completed `batch-CFcV6khZ` and failed pre-upload `batch-VCzP0I1E`;
`6ef3bf0` fixed the latter's routing input, not its obsolete source. Older custody
is recorded in the [preceding handoff](https://github.com/dragginzgame/ic-platform/blob/7d0d86565092aee1654ef1d3a01dea611b9262c9/docs/status/current.md).
No retirement, tag replacement, upload replay or artifact deletion is authorized.
Retired-checkout recovery review remains due 2026-12-04. Keep
`recovery-evidence/deferred-signature-comparison-20261110.rs`, SHA-256
`9fdee2e8b41c9b13e0d2a3127ddf95559a1094fa48d14dd17b7577d4ec86ed4a`.
