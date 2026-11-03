# Platform release validation pilot (#137)

The full `release-verify` gate remains authoritative. This pilot does not omit
checks from a release, reuse a prior release receipt, change versions or publish.
It prepares the consumer mapping for Shared's
[plan interface](release-group-validation.md). Linux same-source full/selected
qualification passed below. Release-recovery integration and native-host
acceptance remain required before activation.

## Complete roster and inputs

`ci/release-validation.json` lists every stage of `ci` followed by both stages of
`check-runtime-all`, in their actual order. Selection refuses if those Make recipes
or Cargo release-group names change without matching the roster. Each plan binds
an exact clean Git HEAD, the selected group's ancestor release tag, the complete
diff since that tag, the index inventory and every tracked regular source file.
Native IC gitlinks are bound by their object identity, not traversed or claimed as
compiled. The full gate's existing source-only native-provider limitations remain.
Tracked symlinks, unmerged indexes, dirty/untracked source and redirected outputs
refuse. Shared checkpoints the plan, policy and expanded inventory before and after
each executed target. Final receipt admission also checks the source inventory,
including added files, and the baseline tag identity.

Only Rust source/tests in one owner, that owner's release ledger and the current
handoff can enter a narrow candidate. Manifests, locks, feature declarations,
macro/guard generators, shared snapshots, CI/release scripts, fixtures, provenance,
other groups and unknown paths select the full roster. Facade candidates remain
full until its crypto/external-adapter scopes are separately qualified.

| Stages | Candidate obligations |
| --- | --- |
| Snapshot, docs, pins, boundary, inventory, formatting, primitive guards | All groups retain them. |
| Clippy, check, test, docs, consumers, feature graphs, Wasm | All groups retain the complete existing package/feature selections, including reverse consumers and canonical identities. |
| Runtime owner and Timers composition | All groups retain both installed profiles; PocketIC is used by their drivers, and Types/Runtime affect public identities and singleton behavior. |
| Browser | Types/facade retain it. Pure client/runtime source candidates may omit it only when all browser/schema/Types inputs are unchanged. |
| Release-command regressions | Pure owner-source candidates may omit them only when recipes, release metadata, locks, scripts and fixtures are unchanged from their baseline. |

Every omitted stage carries its reason in the plan. These are deliberately
conservative candidates: there is no package-only Clippy shortcut or version
projection. The normal Linux and both macOS CI jobs remain unchanged; a pilot
receipt proves only its recorded host, never a substitute native-host result.

## Run and inspect

From clean committed source, with tools/dependencies and the PocketIC server
already prepared:

```sh
bash scripts/release/qualify-validation-plan.sh pocket-ic pocket-ic-v0.19.0 workspace
bash scripts/release/qualify-validation-plan.sh pocket-ic pocket-ic-v0.19.0 auto
```

These are explicit broad qualification commands, not ordinary development checks.
Plans/logs/receipts are retained under `target/release-validation/`. Their exact
source and baseline must match before comparing full and selected outcomes. A
shared-input change correctly makes both plans full; never force narrow selection
by changing the baseline just to shorten a release.

Receipts require every required stage's PASS event, timing and retained log; an
excluded stage must remain not-applicable. They bind plan/policy/input hashes,
source/group/baseline, host/compiler and relevant build/server environment.
Failed or interrupted attempts produce no success receipt. Retry uses a fresh
attempt. Re-admission checks unchanged input evidence but is not permission to
skip any release work. Exact-input stage reuse is not enabled.

## Current qualification

`bash scripts/ci/test-platform-validation-plan.sh` runs the real selector,
Shared plan runner and receipt admission against disposable stage bodies. It
covers all group rosters, full expansion for shared locks, required-stage failure,
excluded-stage failure, stale source, new stages, dirty input, failed attempts and
fresh retry. These are dispatch/custody proofs, not product builds or timings.

### Actual Linux qualification, 2026-11-03

All runs used clean isolated source
`21e9180f611457303aac83065b09f8fc78085f4c`, Linux x86_64 and Rust/Cargo 1.99.1.
The source includes the pending snapshot-client/pilot batch and the preserved
working-tree Pest lock update. The facade's `auto` stage roster was also compared
with `workspace` and is identical; no second identical full run was needed.

| Plan | Required stages | Sum of stage durations | Receipt attempt |
| --- | --- | --- | --- |
| Full / facade | 20, all passed | 1,879 seconds | qualification.H0qDSk |
| PocketIC selected | 18, all passed | 316 seconds | qualification.OZjEyg |
| Runtime selected | 18, all passed | 342 seconds | qualification.hVomEd |
| Types selected | 19, all passed | 327 seconds | qualification.9qlriJ |

Evidence root: `target/issues-132-137.sYRgR0/platform/target/release-validation/`.
Each attempt retains its plan, context, Shared events/checkpoints/timings, stage
logs and successful `receipt.json`. Each selected run actually executed all its
required stages, including both installed Runtime/Timers profiles. Types also
executed browser checks. There was no stage-receipt reuse. Compiler artifacts
were warm after the full run, and concurrent Cargo activity caused cache-lock
waits; these observations are not a controlled speedup benchmark. The full run's
release-command stage accounted for 1,301 seconds.

Commands were `qualify-validation-plan.sh platform platform-v99.0.0 workspace`,
followed sequentially by `qualify-validation-plan.sh GROUP GROUP-v99.0.0 auto`
for `pocket-ic`, `runtime` and `types`, all under `scripts/release/` with
`POCKET_IC_BIN=/tmp/ic-storage-ic0-release-0652/.tools/ic-testkit-server/16.1.0-x86_64-linux/pocket-ic`.
The isolated `GROUP-v99.0.0` tags all bind baseline `de10702`; source differs only
by a handoff sentence. These synthetic baselines exercise real full/selected
commands on identical product sources. They are not published release baselines
and do not authorize narrowing any actual release. Owner/shared-change selection
and required/excluded failure cases are separately covered by the substitute
fixtures described above.

Before changing `release-verify`, qualify recovery admission through the release
adapter and the required native hosts for the new pilot. Earlier hosted full CI
does not qualify these new selector/receipt scripts on macOS. Do not reinterpret
historical full-workspace release logs as pilot receipts. The active full gate
stays intact and receipts explicitly deny release omission; #137 remains open.
