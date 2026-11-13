# Current handoff — 2026-11-13

## Published state and pending delivery

Ordinary main remains `d524dee2fc26d4bb330ffcdbfb5a0be72b461dcb` after completed
frozen batch `batch-CFcV6khZ`. All five packages were verified on crates.io, not
yanked; annotated tags dereference to these release commits:

| Group | Version | Commit | Tag | Qualification / runner |
| --- | --- | --- | --- | --- |
| Types | 0.19.5 | `f0151a0f5762bdb12d1f8f4228bda62badb7a704` | `types-v0.19.5` | `h7EwY8` / `zl8TZ5` |
| Runtime/macros/guard | 0.21.3 | `ef5dbf126aa3776756057474565ce3b5e9074a9f` | `runtime-v0.21.3` | `sQ2IRg` / `m0Nuav` |
| Platform | 0.31.3 | `d524dee2fc26d4bb330ffcdbfb5a0be72b461dcb` | `platform-v0.31.3` | `Xy4j2v` / `EXQK2R` |

All 19 local stages ran for each group. Evidence is under that candidate's
`target/release-validation/`; deployment log:
`target/deploy-0313-tooling-certification-corrected.log`. Browser stays 0.2.0.
The pending compiler/layout/tooling work below remains local, uncommitted and
unpublished. Next notes target Platform 0.32.0, Runtime 0.22.0 and Types 0.20.0;
package versions stay at published values until an authorized release.

## Pending Toko 0.40.1 / deployment-policy adoption

Product and qualification pins/locks select published Toko 0.40.1, checksum
`fb884b05e2e5bbca2388dd0b757833fb5ad37e91b3ced09f6a37b454c7fe9f90`, producer
commit `dcc51b5cab3d9993f0668bc0902252364d7dc316`. Authenticated preview/install
retains all 98 managed files and entrypoints. Only `TOKO-TOOLING.md` differs from
0.40.0; executable payloads are unchanged. No managed file was edited by hand.
Both locks changed only the selected Toko package during this increment;
existing qualification baseline, Host/Types and rustls-webpki changes survive.

The [release guide](../development/release-validation.md#deployment-sequencing)
now records wiring-first checks, retained timing review, one full-gate owner per
group, fresh archive admission and phase-specific retries. Existing adapters
already enforce snapshot admission before effects and avoid a second full gate
in metadata/publication. Only the qualified policy-fixture leaf can reuse prior
execution. General partial-gate and automatic collector-only resume remain
unsupported; diagnose missing admission and disclose rerun scope before retry.
No product tests were repeated solely for the unchanged 0.40.1 executables.
Focused adoption evidence is `/tmp/platform0401-*.log`; no delivery occurred.

## Rust 1.100.0 and common lints (#157)

Both workspace floors and the common development/release pin are 1.100.0.
Explicit preparation admitted rustc `0ef1bc386059596dada8fb65b50425aab2ea637a`
(2026-11-09), LLVM 23.1.5, Cargo `3acc79c0c` (2026-11-05), Linux x86_64,
with Clippy, Rustfmt and Wasm. All real members inherit common Rust/Clippy/
Rustdoc lints; safe-only product libraries forbid unsafe code. Runtime callback,
codec ownership and fixture ABI expectations remain narrow and reasoned.
Cryptographic function bodies, bytes and protocol behavior did not change.

Generated consumers project the owner's compiler/lints through the existing
workspace-dependency helper. `check-primitives` proves actual member inheritance,
idempotency, native/Wasm positives and must-use, unsafe, unsafe-operation,
correctness and Rustdoc refusals (`target/lint-policy.LoNnwc/`). Runtime now
strictly lints every generated Wasm profile before building. Old compiler strings
remain only in frozen reports and reader/admission tests.

On 0.40.0, strict native all-target/all-feature and minimal Clippy, Wasm library
Clippy, all product tests/native macro linkage, strict native/Wasm Rustdoc and
qualification Clippy/Rustdoc passed. Full `check-consumer`, feature graphs,
formatting/refusal, boundary checks and ShellCheck passed. Evidence:

| Contract | Evidence under `target/` |
| --- | --- |
| Four installed Runtime profiles | `cdk-runtime.QAgzTP/` |
| Prepared raw/Wire/external-Candid calls | `prepared-call-consumer.9cicJT/` |
| Installed Agent verification/wrong-root refusal | `agent-installed.CTMiM1/` |
| Canonical identity profiles | `stable-identities.Tr4xtx/` |
| Raw-random bytes, cancellation, callback cleanup | `evidence/raw-rand.lhDAGf/` |
| Matched BLS comparison | `evidence/bls-wrapper.t7uE2v/` |

BLS arms match: raw total/code 246,075/238,906 bytes; optimized 237,381/230,205;
all 28 instruction samples equal. Logs: `/tmp/platform040-*.log`.
Manifest warnings for fixture-only catalog entries and the links-only guard
remain visible; no artificial imports hide them. Native macOS, hosted integration
and opt-in nightly sanitizers have no fresh local evidence. #157 remains open for
its remaining acceptance; these observations do not establish a delivered release.

The preceding six `mod.rs` moves remain preserved, with unchanged public paths,
explicit template formatting and exact boundary allowances. Qualification now
uses registry Platform 0.31.3 alongside the local candidate, preserving Host
0.76.5 and registry Types 0.19.5. Its earlier matched benchmark is retained at
`target/evidence/bls-wrapper.ReL2gW/`. Fixture Wire 0.10/0.12.1 and metadata editor
0.73 upgrades remain deferred to their respective maintenance/adoption batches.

## Unresolved findings and custody

#50 remains open: `RUSTSEC-2026-0387` is expected only in the workspace CI gate,
with raw finding/exit retained. Runtime/new findings and incomplete/tool failures
still fail. Live audit evidence: `target/expected-advisory-live.vi6VFr/report/`.
The historical verification-size finding (+135 total / +133 code bytes against
`fc55ae1`, above its 128-byte allowance) is unresolved; see
[the audit](../benchmarks/platform-031-audit.md#post-release-cleanup).

Preserve completed `batch-CFcV6khZ`, its Git/journals/archives/proofs and the failed
pre-upload `batch-VCzP0I1E` (`qualification.YY6dQL` / `validation.MNl4ew`).
`6ef3bf0` fixed its missing release-routing input; never resume obsolete source.
Preserve completed `batch-CqgSHOoJ`, failed `batch-A6SG2UhN`/`batch-uCPG6Jvb` and
older custody referenced by the [published handoff](https://github.com/dragginzgame/ic-platform/blob/d524dee2fc26d4bb330ffcdbfb5a0be72b461dcb/docs/status/current.md).
Durable state lives in `.git/release-state/frozen/`, candidates/evidence in
`target/release-frozen/`. No tag replacement, upload replay or artifact deletion
is authorized by adoption. Retired-checkout recovery review is due 2026-12-04.
Keep `recovery-evidence/deferred-signature-comparison-20261110.rs`, SHA-256
`9fdee2e8b41c9b13e0d2a3127ddf95559a1094fa48d14dd17b7577d4ec86ed4a`.
