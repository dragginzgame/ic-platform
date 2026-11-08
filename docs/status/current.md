# Current handoff — 2026-11-08

## Completed deployment

The repository-only deployment is complete on GitHub and crates.io:

| Group | Published version | Tag | Release commit |
| --- | --- | --- | --- |
| Types | 0.18.3 | types-v0.18.3 | 17d5906ca39342c24d993fe05f44cb478c851e2a |
| Runtime/macros/guard | 0.20.3 | runtime-v0.20.3 | 38d6e4832cdf6d5a23dbb39fdefeaeb24e5bbcc0 |
| Platform | 0.27.1 | platform-v0.27.1 | 97fe2863cff29d362d6832e47fd46d16988e8f4d |

Batch **batch-mG_yG4uo** captured b5f90ddd7fa1eb69da629b29938412c799ad62db
and delivered all five packages in supplier order. Registry checksums match the
retained archives; archive VCS source/path and remote annotated tags were verified.
The normal checkout was fast-forwarded through release-batch-reconcile.
Browser remains **0.2.0**; browser/npm and sibling delivery were excluded.

All 20 local Linux stages passed separately for each group. Under the batch's
target/release-validation, receipts are qualification.XmetwX (Types),
qualification.QeR7Uu (Runtime), and qualification.jTpCOg (Platform). Each records
workspace scope and no release omissions. Installed PocketIC runtime/Timers and
raw_rand proofs passed. Only the bounded non-Cargo policy-fixture leaf reused
exact-input evidence; no complete gate was reused. Publication, registry/archive
and reconciliation evidence is under target/deploy-toko036-20261108.

Earlier **batch-PIIqwepD** remains retained at 8b47f38a22dcd5c2a955e0d8468590be55d82116.
It stopped at check-consumer before any release metadata, tag, push or upload:
the compiler-floor fixture omitted PlatformEvidence.pm and ValidationPlan.pm.
The corrected source copies both dependencies; focused compiler-floor/admission
checks and all subsequent full gates passed. Preserve its qualification.TN7IEz
failure evidence; do not resume it over already published versions.

Package-impact inspection confirms the mandatory tooling dev-dependency appears
in all five normalized public manifests. All three groups therefore have changed
package requirements, independently of which production Rust source changed.
Evidence: target/deploy-toko036-20261108/package-impact/ and the normalized
manifests alongside it. Every group completed all 20 local stages and
installed-runtime qualification; ordinary native-host CI remains required.
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
routing and retained Runtime preparation/resume/policy-drift proofs. Deployment
and fixture-fix evidence is under target/deploy-toko036-20261108.

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
target/deploy-toko036-20261108/deferred.Cargo.lock. The deployed lock kept previous
non-tooling selections, adding toko-tooling, five dev edges and released owner
versions. After reconciliation, only the excluded supplier delta is restored
locally with Cargo: arc-swap 1.9.4, Candid/derive 0.10.40, cc 1.11.1 and smallvec
1.16.5. It remains uncommitted and unpublished; delivered owner versions stay
intact. Never replace the delivered lock with the old full file.
Earlier custody remains refs/retained/platform0270-lock-draft-20261108 at
588b11a38c60b15ab5af19f30aef1c310a8b0975 (predates released facade metadata).

The Candid 0.10.39→0.10.40 experiment measured +54,588 total/+54,419 code bytes
at fixed inputs: target/next-wasm-20261108/candid-patch-comparison.json. Keep
sequence-completion checks intact. [#50](https://github.com/dragginzgame/ic-platform/issues/50)
remains deliberately deferred; registry cryptographic implementations stay upstream.

## Recovery custody and consumer adoption

Previous Platform 0.27.0 delivery evidence remains under batch-O3V4GL4T and
target/deploy-platform0270-20261108. Platform 0.27 provides raw-rand and
candid-args; [#151](https://github.com/dragginzgame/ic-platform/issues/151) is closed.
Canic [#749](https://github.com/dragginzgame/canic/issues/749) and
[#750](https://github.com/dragginzgame/canic/issues/750) own separate adoption.

Preserve all failed/completed batches and their original supplying recovery code,
including batch-_leIreCI, batch-C5BoFKA5, batch-AaHi2pgO, batch-O3V4GL4T,
batch-PIIqwepD and batch-mG_yG4uo.
Historical bundles/receipts remain at
recovery-evidence/retired-checkouts/20261104-issue139/; maintainer review is due
**2026-12-04**. No retirement, cleanup, mtime renewal or custody-limit change is
authorized. Previously removed target evidence and Runtime 0.18.1 binaries must
not be claimed present.
