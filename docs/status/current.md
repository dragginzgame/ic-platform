# Current handoff — 2026-11-07

## Audit improvements deployed

The reviewed audit batch and Shared 0.31.1 repair are published on crates.io and
GitHub. Shared schema-2 batch `batch-U1NhGrpl` completed both groups in order from
source `2e3a54f2d6531be8800ce031e2fd71a3fdad6cb8`; normal main is reconciled.

| Group | Published version / annotated tag | Release commit |
| --- | --- | --- |
| Runtime/macros/guard | 0.19.5 / runtime-v0.19.5 | `c07b714fad92d7c19206427a2bc81c99d61f2dba` |
| Platform | 0.23.6 / platform-v0.23.6 | `51f5fbb7c53000fbb6f7bc3219ee3c2978c08521` |

All four downloaded registry archives are non-yanked, match registry checksums
and record their exact clean release commit. Remote annotated tag objects and
peeled commits match. Evidence: `target/deployment-audit-20261107/`, including
`registry/verified.tsv`, `batch-complete.json`, `release.log` and `reconcile.log`.
Types **0.18.2** and browser **0.2.0** remain unchanged. No sibling source or
release was taken over; Storage and Tooling remain with their other sessions.

Each group passed all **20** local release stages on Rust 1.99.1, including full
lint/tests/docs, native/Wasm consumers and feature graphs, source/ABI/license
checks and installed owner/Timers raw/prepared Wire/external-Candid profiles.
Canonical candidate receipts are under `target/release-frozen/batch-U1NhGrpl/target/release-validation/`:
Runtime `qualification.1XHqxr`, Platform `qualification.D0ws9t`. Copies remain in
`target/deployment-audit-20261107/{runtime,platform}-qualification/`.

Hosted checks continue asynchronously. At the recorded observation, the delivered
source's [Wasm comparison](https://github.com/dragginzgame/ic-platform/actions/runs/42358426521)
passed; [CI](https://github.com/dragginzgame/ic-platform/actions/runs/42358426580),
installed Runtime, browser and advisory jobs were in progress. No fresh macOS
acceptance is claimed; hosted/native results are separate from the Linux receipts.
The native-host reuse allowlist remains empty, so actual releases retain full gates.
#50 remains upstream-dependent and excluded from this work; no waiver was added.

## Supplied changes and measurements

Lock/source-roster readers fail closed on failed, partial or empty observations.
Release fields share one admitted plan per command, with fresh later admission;
publication carries its package selection to Clippy. Relevant main pushes run
matched Wasm comparisons. Runtime retains stable Vec cancellation order with bulk
tail shifting and removes the unused private null-context entry. Five System API
readers share one reviewed all-or-trap spare-capacity copy. Candid admission uses
one child-reference buffer, preserving traversal, quotas and the 16-level ceiling.
Cryptographic implementations, public signatures and canonical identities remain.

Matched experimental evidence is at `target/evidence/platform-audit-followup/`:
identical probes/compiler/external locks and `3-fat-1`. Those comparisons used the
independent dependency draft below; release gates used the committed release lock.
Optimized total uncompressed/code bytes decreased respectively: Runtime **28/28**,
System readers **85/85**, external Candid **372/402**, Candid admission **200/230**.
A 1,111-byte certificate read saved 1,227 instructions; a 133-byte root-key read
saved 249. Wide 1,024-type admission fell 1,241,774 → 695,195 instructions; simple
recursive refusals cost 161 extra. Exercised acceptance/error outcomes matched.
These are probe measurements, not application fleet gains. Earlier FIFO cancellation
measurements and their 347-byte Runtime saving remain in the owning release ledger.

## Independent dependency draft

The normal lockfile again contains only the original external dependency patch:
Candid/derive **0.10.40**, cc **1.11.1**, SmallVec **1.16.5**. It was updated during
the frozen batch without changing release inputs. The exact patch was restored
over Platform 0.23.6 and Runtime/macros/guard 0.19.5 after reconciliation; normalized
before/after diffs match. Current restored lock SHA-256:
`a683f1fe9cc23a1cd03f5f7c65297ab431338a6b30cfd00c8c50922fc057138f`.
Original custody is `refs/retained/audit-deploy-20261107-lock-draft`; its original
lock hash is `8fa03623313c4866557072e88365b73c6ee05488b1a06417898ddd55881af070`.
Original/restored patches are alongside publication evidence. Never restore an
old whole lock over delivered owner versions. Do not treat this draft as released.

## Baseline and retained custody

Shared **0.31.1** at `5efbda2ab81ad84e30f490f3137d8ef083c303d2` is adopted through
the canonical exporter; both release gates verified all 141 payloads. VERSION and
changelog say 0.31.1; the commit title says 0.31.2. Its Bash 3.2 checksum-parser
repair and the portable Platform qualifier are delivered. The earlier supplier
macOS setup/native reuse passed, but its broader portable suite failed at
`test-release-editor-setup.sh`; [that historical supplier run](https://github.com/dragginzgame/shared-tooling/actions/runs/42274707843)
is not fresh Platform macOS acceptance. The pinned metadata editor is ic-build 0.73.0.

External cleanup removed all target state during implementation. The maintainer
confirmed cleanup stopped, and current focused/release evidence was regenerated.
Earlier target receipts, first comparison artifacts and reproducible Runtime
0.18.1 binaries are gone; older successes remain dated observations. No evidence
was deleted by this session. Preserve active receipts/logs while recovery needs them.

Completed batches `batch-U1NhGrpl` and `batch-hEX13lnF` retain durable Git/journals
under `.git/release-state/frozen/`; neither was retired. Earlier retained refs and
the interrupted unpublished 0.23.1 journal remain untouched. Historical archives,
bundles and receipts remain at `recovery-evidence/retired-checkouts/20261104-issue139/`;
maintainer review is due **2026-12-04**. Retention refuses expiry/excess without
automatic cleanup. Future authorized delivery uses the [batch lifecycle](../platform-release-validation.md#commands-and-effects).
Root evidence does not qualify the separate native Profiler provider; its
[paired Linux evidence](../audits/native-profiler-140-2026-11-05.md) remains historical,
with macOS/Bazel unqualified.
