# Platform release validation

Every Platform, Runtime and Types release requires the complete local gate:
all stages in `ci/release-validation.json`, checked against the actual `ci` and
`check-runtime-all` recipes in Makefile. Group selection controls package versions
and custody, never which validation stages run. No native-host allowlist or
selected-stage release mode remains.

## Commands and effects

Shared owns [frozen release batches](https://github.com/dragginzgame/toko-tooling/blob/626da7cd9626cc2e152594862a2d82ea82ef51da/docs/release-batches.md). With clean committed
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

### Deployment sequencing

Before an authorized release, inspect the selected group's nested commands and
review retained `timings.tsv` files under the previous batch's
`target/release-validation/qualification.*/logs/validation.*/`. Report the
expensive stages and expected rerun scope before starting; old timings are
observations, not a guarantee for the new compiler or cold caches. Keep optional
tooling/dependency/compiler upgrades outside a frozen batch unless requested or
needed for its blocker.

Native qualification suppliers need not advance with the product release.
Testkit may retain its separately qualified registry Platform/Types generation
when its actual API, protocol and type crossings remain valid. Before capture,
check the locked graphs and affected compilation. An unsuccessful optional
Testkit or other test-dependency upgrade does not invalidate the unchanged
qualified selection. Required test failures, identity/executor conflicts,
security repairs and explicitly selected migrations still block their affected
release. Keep optional experiments outside the frozen attempt; never rewrite its
locks to chase a new supplier. No minimum publication age is selected here.

When release wiring changes, qualify its affected contract before product tests.
For example, `test-platform-frozen-setup.sh` checks candidate prerequisite order;
`test-platform-release-groups.sh` checks cohort/publication routing;
`test-platform-group-projection.sh` checks custody projection. Run the relevant
metadata or recovery fixture when those implementations change. These are
disposable wiring proofs, not permission to omit a required product gate.

The release hook owns one complete 19-stage validation pass per selected group.
Do not prepend `make ci`, `check-runtime-all` or standalone full qualification
merely to repeat that gate. Metadata preparation re-admits the saved receipt;
publication checks the exact selected packages and final archives. The shared
publication adapter invokes only the package-scoped Clippy verifier, not another
complete gate. Only the qualified policy-fixture leaf can reuse earlier execution;
there is no cross-group Rust-test or installed-runtime cache admission.

The Make Clippy leaf uses Cargo 1.100's `CARGO_BUILD_WARNINGS=deny`, qualified
against genuine warnings on fresh and cached artifacts. Publication retains the
same warning refusal while restricting Clippy to its admitted package selection.
Plain JSON diagnostic collectors keep compiler-level denial: Cargo's warning mode
alone does not enforce that output mode. Rustdoc denial also remains explicit.
The shared tool installer now relies on Cargo's default packaged lockfile instead
of its obsolete install-only `--locked` flag; exact version/checksum admission and
all build, test, metadata and publication locks remain required.

### Failure-specific recovery

Identify the failed phase, exact source/group and invalidated inputs before any
retry. Preserve all successful stage evidence, logs and original paths. Never
infer a pass from a log line, edit a journal or manufacture a new origin for an
old proof.

| Failure phase | Current recovery boundary |
| --- | --- |
| Candidate setup or metadata | Repair that phase and use the saved batch's supported resume. Completed validation is reused only if its original receipt and current source/context still admit it. |
| Product stage | Keep earlier stage evidence, but do not claim partial-gate reuse. The current adapter has no general successful-stage resume; explain the required full-gate rerun and changed inputs before starting it. |
| Receipt collection/publication after successful execution | Preserve the original plan, runner evidence and stage admissions. This alone does not invalidate completed runtime checks. There is no automatic collector-only resume/journal repair; identify the missing admission and explain any required rerun before proceeding. |
| Git delivery after metadata preparation | Resume the saved batch, with its source, destination, tags and complete validation receipt re-admitted. Do not re-execute product stages solely because pushing failed. |
| Registry upload or uncertain response | Reconcile each dispatched exact package/version and archive before resume. Use the supported publication disposition; do not retry the upload or full gate merely because the response was uncertain. |

A future collector-only or broader stage-resume implementation must qualify
failure after successful execution as well as missing/tampered evidence, changed
inputs and final delivery admission. Toko 0.40.1's policy does not itself add
that mechanism or waive the current checks. Historical attempts retain their
recorded tooling and semantics.

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

### Exact-input policy-fixture reuse (#149)

On Linux x86_64, the two release-plan/recovery policy fixtures inside
`check-release-commands` use Shared's `run-validation-stage.pl`. Every group still
dispatches all 19 required parent stages, admits its committed source/index and
destination, and freshly runs real group metadata/preparation, Cargo, native,
archive and publication checks. Only this bounded non-Cargo leaf can reuse an
immutable successful execution. It does not reuse a historical whole-gate receipt
or project package versions. Other hosts and checks outside committed group
qualification execute the policy fixtures freshly.

The observer enumerates policy scripts/companions, docs/rules, Make inputs, group
policy, the snapshot and compiler configuration, including newly added files.
It binds selected executable/library/compiler identities, native host and an
explicit isolated environment. The fixtures create their own package manifests,
locks, versions and local Git remotes; real package identities and installed
qualification locks are outside this leaf's input closure. The isolated Git
configuration/template selection prevents ambient signing/hooks from changing
these disposable tests. A closed PATH contains only the enumerated tools selected
from the caller's original PATH; both selections are bound. An unlisted external
command fails, and a changed tool selection changes the proof key. There is no
reusable Rust compilation stage.

Each invocation gets a fresh source/group admission. Schema-4 Platform receipts
retain Shared's complete parent-gate proof plus its sealed leaf admission,
original execution and log digests. Recovery re-observes the leaf's current
closure/tools/environment and checks those immutable bytes; missing or damaged
evidence cannot satisfy a saved receipt. Old schema-3 receipts keep their original
semantics. Observation uses the same child Make dispatch as execution, preserving
its tool-path prefix instead of substituting the parent process's environment.
Normal qualification output distinguishes execution/reuse and reports
admission time. Exact-input mismatch or absent proof executes ordinarily; changed
inputs during execution, primary failure or unsafe custody refuses admission.

`bash scripts/ci/test-platform-stage-reuse.sh --admission` checks real leaf
execution/reuse and missing/tampered receipt admission through substituted product
stages. Omit the selector for the broader source/tool-drift and failure matrix.

`bash scripts/ci/test-platform-stage-reuse.sh` qualifies the actual leaf through
the parent Make/plan/receipt dispatch using disposable product-stage bodies. It
checks different groups and metadata commits, closure/tool drift, missing/log-
tampered evidence and failure recovery. These substituted parent stages do not
establish product or macOS acceptance. Keep this focused activation proof separate
from ordinary release fixtures to avoid recursively testing their own cache.

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
run the entire required gate, including installed standalone Runtime profiles. Keep native macOS/Profiler evidence separate from Linux
root-workspace results.

The retired #137 pilot, earlier qualification observations and recovery history
remain in the [0.24.0 validation record](https://github.com/dragginzgame/ic-platform/blob/platform-v0.24.0/docs/development/release-validation.md).
No retained batch, receipt, ref, package archive or historical recovery evidence
is deleted by retiring the pilot's code and workflow.

## Qualification identities (#152)

Generated runtime, Agent, raw-rand and benchmark fixtures retain Cargo metadata
and `*.identities.json` reports. Admission follows normal dependency edges from
the fixture being exercised, and then Platform's owner edges. It checks the
selected checkout paths (or the published Runtime/macros/guard cohort), rather
than banning every older Platform package in the lockfile.

| Proof | Implementation under test | Infrastructure and actual crossings |
|---|---|---|
| Runtime canister | Selected Runtime, macros and guard; selected facade only for external Candid profiles | Raw/prepared × Wire/external profiles retain native and Wasm graphs. Wire-only canisters exclude the facade. |
| Runtime driver | Selected facade's certificate CBOR and external argument codecs | Testkit installs and calls canisters using bytes. Its own Platform/Types identities are reported separately. |
| Agent installed driver | Selected facade's upstream Agent adapter | Testkit installs an echo WAT canister, which contains no Platform implementation. Testkit's management CanisterId and Agent's Principal must resolve to the same supplier package identity. |
| BLS benchmark | Platform selected by the measured Wasm fixture | The native Testkit driver has no direct Platform dependency; it transports fixtures and reads instruction counts. |
| ECDSA benchmark | Platform selected by the measured Wasm fixture; current facade generates native signing samples | Driver and measured Wasm identities are recorded separately, including when measuring an older source archive. |
| Raw-rand | Selected facade codec and Runtime transport | Testkit remains the native simulator client; bytes cross the canister boundary. |

Testkit's internal older facade is infrastructure, not evidence that the selected
Platform implementation passed. It is deliberately neither patched to the local
facade nor rejected merely for coexisting. The Agent identity check follows the
actual Testkit → management types → Candid → Principal and Agent → Principal
edges. Version equality alone does not establish Rust type identity.

Graph admission identifies code; it does not replace installed tests, source
custody, executor singleton refusal, cryptographic verification or native-host
acceptance. Downstream Timers composition and Testkit adoption remain with
Scheduling and Tooling, respectively; their release cadence is not a Platform
supplier qualification gate.
