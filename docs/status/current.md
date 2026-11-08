# Current handoff — 2026-11-08

## Authorized deployment in preparation

The maintainer requested deployment of this repository. Select one frozen batch
in supplier order: Types **0.18.3**, Runtime/macros/guard **0.20.3**, Platform
**0.27.1**, with GitHub and crates.io delivery. Browser and sibling repositories
are excluded. These versions are planned, not yet released. Source is on main at
f27b73b4e95d6c579a8ff4267eaa63cf741cf5e7 with the reviewed local batch below.
Remote main agrees; Cargo's registry index still reports Types 0.18.2,
Runtime/macros/guard 0.20.2 and Platform 0.27.0.

Package-impact inspection confirms the mandatory tooling dev-dependency appears
in all five normalized public manifests. All three groups therefore have changed
package requirements, independently of which production Rust source changed.
Evidence: target/deploy-toko036-20261108/package-impact/ and the normalized
manifests alongside it. No release gate is waived: every group retains all 20
local stages, installed-runtime qualification and ordinary native-host CI.
Hosted native/integration checks remain asynchronous; no new macOS, worker or
native Profiler acceptance is claimed.

## Reviewed source batch

Published **toko-tooling 0.36.0** is pinned once in root workspace dependencies
and inherited as an unconditional dev-dependency by all five real members, with
Machete exemptions. Cargo.lock and .shared-tooling.snapshot select the registry
archive checksum de0dd05b6437061622a76942baa25d40d2181497e7ba66d9fd809132a3082936.
The installer retained all 74 entrypoints and verified 157 payloads. The obsolete
metadata table/separate tooling lock are removed; no selection-only crate or Rust
import was added. Keep intentionally isolated fixture/vendored manifests excluded.
Admission runs before common CI/release and custom deployment effects.

After changing the root pin, update Cargo.lock with Cargo and run
make shared-tooling-update or shared-tooling-install. These Make commands install
the existing Cargo selection; they do not edit manifests or resolve the lock.
Focused migration evidence is under target/adoption-toko036-20261108, including
cold-cache admission, pins, Machete, feature graphs, guard Clippy, entrypoint
routing and retained Runtime preparation/resume/policy-drift proofs.

Small Candid unit/text encoders reserve their complete output once, retaining
bytes, arity and public signatures. Static empty/raw_rand arguments stay
allocation-free. The size fixture adds opt-in candid-args. Focused Clippy,
reference-byte tests and the existing 128-byte size check pass. At matched Rust
1.99.1/3-fat-1/IC-Wasm shrink, argument Wasm changes 19,998→19,949 bytes and code
payload 17,867→17,819. The signature-copy and strict-digest borrowing prototypes
were rejected after measured growth and restored to baseline. Cryptographic,
certificate, delegation and signing-time checks remain unchanged. Evidence:
target/evidence/codec-allocation-20261108, including rejected variants.

Four previous audit fixes are included: shared immutable runtime/raw_rand input
capture, one inert runtime-option parser, checked evidence/process helpers over
Shared's existing reader/digest, and Shared #366's validated journal projection
for resume policy. Existing receipt formats, source/key identity, byte/work
budgets, policy-drift refusal and completed-plan behavior remain unchanged.
Focused evidence: target/audit-simplification-20261108 and the migration directory.
The same raw_rand driver/artifacts passed installed retry against PocketIC 16.1.0
at target/evidence/raw-rand.xBHo2m after the sandbox refused initial localhost
binding; original failed evidence remains. These focused proofs do not substitute
for this batch's full release gate.

## Deferred dependency draft

The independent supplier update remains excluded from deployment. Its exact
pre-deployment Cargo.lock (SHA-256
32273b6a5275d6703fc460dadc6f70cab14fa44a5afdd08753bb56ec5ca102e6)
is retained durably at refs/retained/platform036-lock-draft-20261108, commit
0263ab6e679ae3b8e9f89bddd796ca951b3f1df4, and at
target/deploy-toko036-20261108/deferred.Cargo.lock. The deployment lock keeps all
previous non-tooling selections, adding only toko-tooling and five dev edges;
its current SHA-256 is 5201d7132a8b8189f936c34519c4dc4bd1d6b0711c7c5057a3470c1975b78477.
After complete frozen delivery/reconciliation, reapply only the supplier delta
over delivered owner versions; never replace the new lock with the old full file.
Earlier custody remains refs/retained/platform0270-lock-draft-20261108 at
588b11a38c60b15ab5af19f30aef1c310a8b0975 (predates released facade metadata).

The Candid 0.10.39→0.10.40 experiment measured +54,588 total/+54,419 code bytes
at fixed inputs: target/next-wasm-20261108/candid-patch-comparison.json. Keep
sequence-completion checks intact. [#50](https://github.com/dragginzgame/ic-platform/issues/50)
remains deliberately deferred; registry cryptographic implementations stay upstream.

## Delivered baseline and recovery custody

The previous public Platform **0.27.0** is platform-v0.27.0 at
a8165b91e0e04f65962bad5324f5db31e312c580, delivered by complete batch-O3V4GL4T.
Its full Linux gate and registry/archive evidence remain under that batch and
target/deploy-platform0270-20261108. Runtime/macros/guard are **0.20.2**,
Types **0.18.2**, browser **0.2.0**. Runtime's prior tag is runtime-v0.20.2 at
bb76a069da4d442e1e830636bace9760fbebd880. Platform 0.27 already provides raw-rand
and candid-args; [#151](https://github.com/dragginzgame/ic-platform/issues/151) is closed.
Canic [#749](https://github.com/dragginzgame/canic/issues/749) and
[#750](https://github.com/dragginzgame/canic/issues/750) own separate adoption.

Preserve all failed/completed batches and their original supplying recovery code,
including batch-_leIreCI, batch-C5BoFKA5, batch-AaHi2pgO and batch-O3V4GL4T.
Historical bundles/receipts remain at
recovery-evidence/retired-checkouts/20261104-issue139/; maintainer review is due
**2026-12-04**. No retirement, cleanup, mtime renewal or custody-limit change is
authorized. Previously removed target evidence and Runtime 0.18.1 binaries must
not be claimed present.
