# Platform release validation

Every Platform, Runtime and Types release requires the complete local gate:
all stages in `ci/release-validation.json`, checked against the actual `ci` and
`check-runtime-all` recipes in Makefile. Group selection controls package versions
and custody, never which validation stages run. No native-host allowlist or
selected-stage release mode remains.

## Commands and effects

Shared owns [frozen release batches](release-batches.md). With clean committed
source, the pinned compiler and an explicitly prepared Testkit server
(`POCKET_IC_BIN` or `IC_TESTKIT_POCKET_IC_URL`), select only affected groups:

```sh
make release-patch RELEASE_GROUP=platform RELEASE_PUBLISH_TARGET=publish
make release-minor RELEASE_GROUPS=runtime,platform RELEASE_PUBLISH_TARGET=publish
```

These are delivery commands requiring release/publication authority. Use the
increment appropriate to the complete batch. A cohort uses one increment and
supplier order `types,runtime,platform`; unchanged groups are excluded. Omitting
`RELEASE_PUBLISH_TARGET` retains Git-only semantics. Do not append a second
normal-checkout publication command.

The candidate runs under `target/release-frozen/BATCH`; durable Git and journals
remain under `.git/release-state/frozen/BATCH`. Later normal-tree edits are not
release inputs. The candidate prepares and admits its own complete tool roster,
hooks and locked dependency caches, without copying the normal checkout's tool
store. Its ordinary release adapter repeats admission, not installation.

Inspect or recover that exact batch without selecting a different group/version:

```sh
make release-batch-status BATCH=batch-XXXXXXXX
make release-batch-resume BATCH=batch-XXXXXXXX
make release-batch-reconcile BATCH=batch-XXXXXXXX
```

Reconciliation requires completed delivery and clean normal source, and only
fast-forwards. Preserve deferred edits durably, then reapply their exact patch
over delivered package versions. Uncertain publication refuses redispatch;
inspect registry/archive evidence before supplying `PUBLICATION_RESULT=published`
or `not-published` on resume. Never infer publication from current HEAD.

Retirement is a separately authorized deletion, not post-deployment cleanup.
`release-batch-retire` retains the Shared custody and ignored-output restrictions.
Do not broaden its allowlist or remove browser caches to make retirement pass.

## Full-gate evidence and recovery

For standalone full qualification of committed source, without delivery:

```sh
bash scripts/release/qualify-validation-plan.sh runtime runtime-v0.19.5 qualification
```

The final `release` purpose is reserved for the actual release hook, which binds
previous/new versions, group, destination and source. Both purposes require every
stage. The former `auto`/`workspace` selector arguments and native pilot qualifier
are retired. The ordinary Linux/macOS CI and installed-runtime workflows remain.

The planner refuses dirty/stale source, wrong baseline identity, unsafe tracked
inputs and Make/policy roster drift. Shared executes the plan, records complete
input checkpoints, stage events/logs and interruption status, and verifies its
own evidence protocol. Platform retains the full Shared proof and adds source,
group, baseline, compiler/tool/server/environment and release-destination binding.
Receipts cannot admit an omitted stage or a failed/incomplete run.

Prepared metadata is the only admitted source projection. The adapter checks its
exact semantics and resolves qualification locks offline from their maintained
selections before installing metadata. A native supplier may retain a preceding
inactive registry owner after a minor bump; prepared canister graphs still require
the sole current executor. Receipt recovery reconstructs immutable original source
from Git and verifies all retained evidence. Source edits, staged/untracked files,
changed policy, environment/server drift, relocated receipts and altered logs or
checkpoints refuse. Recovery does not rerun product stages or rewrite old runner
paths to manufacture evidence. Historical saved attempts retain their recorded
source/tooling and must not be reinterpreted under this implementation.

## Focused qualification owners

- `test-platform-group-preparation.sh` checks real preparation failure/rollback,
  retry, guard pins, other-group custody, real offline qualification-lock resolution,
  post-version graph admission and publication
  selectors for all three groups. Runtime, with coordinated macros and exact
  guard, supplies the one complete local commit/tag/push/resume integration cycle.
- `test-platform-validation-plan.sh` checks full rosters, failures, interruption,
  input drift, omitted-stage refusal and exact retained Shared proof.
- `test-platform-validation-recovery.sh` checks actual Shared journals and receipt
  recovery for every group, using substituted stage bodies and metadata.
- `qualify-frozen-release.sh` checks actual setup/editor/publication wiring and
  archive identity against disposable remotes; product stages and upload effects
  are substitutes. It is not a fresh full gate or live publication receipt.

These focused checks do not qualify native product execution. Real releases still
run the entire required gate, including installed Runtime owner and Timers
composition profiles. Keep native macOS/Profiler evidence separate from Linux
root-workspace results.

The retired #137 pilot, earlier qualification observations and recovery history
remain in the [0.24.0 validation record](https://github.com/dragginzgame/ic-platform/blob/platform-v0.24.0/docs/platform-release-validation.md).
No retained batch, receipt, ref, package archive or historical recovery evidence
is deleted by retiring the pilot's code and workflow.
