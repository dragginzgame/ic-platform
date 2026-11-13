# Current handoff — 2026-11-13

## Published state

Frozen batch `batch-q_67SewF` completed GitHub delivery/crates.io publication in
supplier order and reconciled into main. All five exact registry packages and
embedded source commits were verified; annotated tags dereference as follows:

| Group | Version | Commit | Tag | Qualification / runner |
| --- | --- | --- | --- | --- |
| Types | 0.20.0 | `508c60b1ff2e8b5db4a80d9ba9ab50c724c98cc5` | `types-v0.20.0` | `7zMqrl` / `yHItvq` |
| Runtime/macros/guard | 0.22.0 | `8d88fe054816e467e3eaec88e79285737f00500a` | `runtime-v0.22.0` | `qjP0cD` / `oW60AE` |
| Platform | 0.32.0 | `14c954e5ae30ef2100ab311d8b3e6dc3db6948fa` | `platform-v0.32.0` | `624oov` / `XIahtu` |

Source `7d0d865` passed 19 local stages per group (57 parent dispatches), with
fresh metadata/archive admission. Evidence remains under
`target/release-frozen/batch-q_67SewF/target/{release-validation,publication}/`;
log: `target/deploy-0401-rust1100.log`. Durable intent/Git/journals remain in
`.git/release-state/frozen/batch-q_67SewF/`. No collector retry occurred; only the
qualified policy-fixture leaf reused its immutable execution. Browser stays 0.2.0.
No sibling/npm publication occurred. Main's last delivery is `50c7163`, the
Git-only standalone Wasm-probe repair after those tags.

The published batch selected Toko 0.40.1 and Rust 1.100.0 (rustc
`0ef1bc386059596dada8fb65b50425aab2ea637a`, LLVM 23.1.5; Cargo `3acc79c0c`).
All members inherit common lints; Runtime FFI/ownership expectations remain local.
The six directory-module moves and refusal proofs remain qualified. Repaired
Wasm probes require scoped export expectations and omit unused I/O in the raw
Runtime profile. Matched `d524dee` versus 0.32.0 comparisons have zero raw/optimized
total/code growth for all five profiles: `target/evidence/wasm-regression.Do2myE/`.
Retain the failed hosted artifact at `target/evidence/hosted-wasm-43403360773/`.

## Pending Toko/Testkit adoption and warning cleanup

Both workspace pins/locks select published Toko **0.41.0**, checksum
`4aaafc699324611501e051132e971288c16392a3ece605e9909e88e0c7e80a2f`, producer
`b4e6308894c495169278b34ea7803973c125adb6`. Authenticated preview/install retained
98 managed files/entrypoints. No snapshot-owned payload was edited by hand.
`make deps-list` reports all five product versions/groups plus qualification.

Fixture inputs and qualification now select Testkit **0.102.0**, with its required
Host 0.77, registry Platform 0.32 and Types 0.20. Both maintained fixture and
qualification locks retain unrelated selections and have no old registry Platform
0.31 entry. Qualification's explicit BLS baseline is published Platform 0.32.0;
canister backends/IC0 remain Wasm-only, with native Testkit/oracle/Tokio separate.
Native infrastructure retains registry source identity rather than replacing local
application types. No sibling source was edited.

The unused facade dependency alias and its group dependency-roster entry are
removed; all release-group members remain. Root Cargo contains actual member
requirements. Generated fixture inputs now live under
`workspace.metadata.platform-fixtures.dependencies`; their existing reader also
admits archived dependency catalogs and refuses duplicate names. All callers use
that reader. The guard is explicitly used as a link-only crate under `executor`;
its exact version, links identity and no-API boundary remain. Its obsolete Machete
ignore is removed. No warning suppression was added. Identity fixtures select
Wire only for their native schema checks. Crypto/protocol function bodies and
public type identities are unchanged.

Both owning workspaces and generated consumers inherit Cargo's dependency,
workspace-field, lint-inheritance and unknown-lint denies, plus Rustdoc table
admission. Make/publication Clippy uses `CARGO_BUILD_WARNINGS=deny`; genuine fresh
and cached warning refusal is qualified at `target/lint-policy.YlaeRq/`. Plain
JSON collectors retain compiler denial. The singleton fixture uses resolution
for its source-free links proof and still compiles its real cross-facade types.

Fresh 0.41 product native all-target/all-feature and Wasm library Clippy, native
qualification Clippy and both Wasm qualification backends pass **without warnings**.
Strict product Rustdoc passes. Runtime build-only qualification retains driver
tests and all four Wasm profiles at `target/cdk-runtime.fUHUwd/`; identity profiles
pass at `target/stable-identities.xM6mlZ/`. Actual singleton/refusal evidence is
`target/runtime-singleton.s7fSxZ/`, IC0 isolation `target/runtime-ic0.ocnc8y/`, and
publication routing/warning refusal `target/platform-release-routing.jvlSCG/`.
Reader/admission, snapshot, pins, inventory, links, formatting and ShellCheck pass.
Logs: `/tmp/platform-toko041-*.log`.

Earlier 0.40.2 installed/measurement evidence remains retained separately:

| Contract | Evidence under `target/` |
| --- | --- |
| Four installed Runtime profiles | `cdk-runtime.fggxfM/` |
| Agent verified query/update/read-state/wrong-root refusal | `agent-installed.Aejp0p/` |
| Singleton incompatibility/refusal | `runtime-singleton.EK2adI/` |
| Canonical identity/schema profiles | `stable-identities.nbG5N6/` |
| Real facade metadata preparation/rollback without catalog alias | `platform-group-preparation.JR1XOZ/` |
| Matched BLS builds/instructions/refusals | `evidence/bls-wrapper.WoOCjf/` |
| Guard-use Runtime size comparison | `evidence/wasm-regression.n6iYbP/` |

BLS raw total/code match at 246,075/238,906 bytes; optimized 237,381/230,205;
all 28 instruction samples match. Runtime has zero raw/optimized total/code delta.
Runtime Machete also passed in that preceding selection. Logs:
`/tmp/platform-newdeps-*.log`. This adoption did not repeat installed simulator
scenarios, the opt-in nightly sanitizer or fresh native macOS qualification.
#157 retains its native-host acceptance; #50 stays open under its workspace-only
expected advisory disposition. No global ignore or security claim was added.

Deployment retains explicit groups, frozen locks and the full local gate. Native
Testkit/other test suppliers may retain independent qualified generations when
real API/type/protocol/executor boundaries hold. Failed optional upgrades do not
invalidate the unchanged qualified selection; required integration failures still
block. No cooldown or new partial-gate reuse is activated. See
[deployment admission](../development/release-validation.md#deployment-sequencing).

This batch is local/uncommitted, with no push or publication. Pending notes target
Platform 0.32.1, Runtime 0.22.1 and Types 0.20.1; package versions remain published
values. Optional Wire fixture 0.10.0 → 0.21.5 and IC Build editor 0.73.0 → 0.84.0
refreshes are deferred to the next dependency-maintenance batch, preserving this
release's qualified schema/editor contracts; they are not current selections.
Pre-freeze Cargo registry review confirms Toko 0.41.0 and Testkit 0.102.0 are the
latest published selections. Toko's installer policy change requires newly
admitted Cargo tool slots; prior build-policy hashes are not reused implicitly.

## Rust 1.100 cleanup

Runtime's private context/environment guards now use `DropGuard`; context clearing
still precedes task destruction, and callback ownership transfers immediately
after successful perform. Native callback tests own their captured Arc directly
and retain independent Weak lifetime observations. New regressions cover normal
and native-unwind destructor re-entry and callback custody after post-submission
unwinding. Macros use `trim_prefix` without changing generated export names.
Affected native/Wasm Clippy, Runtime/macro tests and doctests pass; four installed
raw/prepared Wire/external-Candid profiles pass at `target/cdk-runtime.wjk3Ao/`.
Matched before/after sources at `target/evidence/rust1100-guards.odTaT8/` use the
same compiler, dependency selection, probe and optimization recipe. Runtime raw
total/code remain 45,384/39,082 bytes, optimized 41,338/35,036: all deltas zero.
Logs: `/tmp/platform-rust1100-guards-*.log`. No fresh macOS qualification or delivery
was performed. `Vec::from_fn` and custom allocators remain unadopted; the former's
isolated audit probe had identical Wasm.

## Recovery custody

The unrelated root/qualification `cc` 1.11.2 lock draft is excluded from this
deployment and preserved as `recovery-evidence/deferred-cc-1.11.2-20261113.patch`,
SHA-256 `485b2bdafb36a21f9335b2a3e91fb1d32a839c622ff629abfe92a5831f5f21e2`.
Restore it after frozen delivery/reconciliation; release selections retain 1.11.1.

The historical verification-size finding against `fc55ae1` (+135 total / +133
code, above 128) remains unresolved; newer zero-delta comparisons use other
baselines. See [the audit](../benchmarks/platform-031-audit.md#post-release-cleanup).
No cryptographic/certificate/delegation/identity-key/witness/freshness check was cut.
For exact batch recovery, prepend `target/deployment-tools/npm-12.2.0/node_modules/.bin`
to PATH; global npm was 12.2.1. Its saved PocketIC selection is
`.tools/ic-testkit-server/16.1.0-x86_64-linux/pocket-ic`; preserve these identities.

Keep completed `batch-CFcV6khZ`, failed pre-upload `batch-VCzP0I1E` and older custody
from the [preceding handoff](https://github.com/dragginzgame/ic-platform/blob/50c71637e1b84306ce178752930c315e50d2122f/docs/status/current.md).
Do not resume obsolete source, retire artifacts, replace tags or replay uploads.
Retired-checkout recovery review is due 2026-12-04. Keep
`recovery-evidence/deferred-signature-comparison-20261110.rs`, SHA-256
`9fdee2e8b41c9b13e0d2a3127ddf95559a1094fa48d14dd17b7577d4ec86ed4a`.
