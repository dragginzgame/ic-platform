# Platform release validation (#137)

Shared 0.30.7 supplies [frozen release batches](frozen-releases.md), but Platform
does not set `RELEASE_FROZEN_GROUPS`. Activation requires qualification of the
actual three-group release/publication hooks, interrupted recovery, supplier
ordering and preservation of concurrent normal-checkout lock edits. Snapshot
adoption and synthetic runner tests do not establish that evidence. Releases
continue through the existing workflow described below.

`release-verify` now dispatches a fresh source-bound plan through Shared's
[plan interface](release-group-validation.md). The qualified-host allowlist in
`ci/release-validation-hosts.json` is empty following #139’s client retirement.
All releases use the full roster until the new three-group mapping has fresh
Linux/macOS qualification. Runtime and Types retain selected-plan candidates for
explicit qualification; facade/shared/unclassified changes remain full. The
previous four-group native evidence below remains historical. Recovery rechecks the saved receipt through Shared's
existing release hooks. This does not change package versions or publish anything.

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
| Browser | Types/facade retain it. Pure runtime source candidates may omit it only when all browser/schema/Types inputs are unchanged. |
| Release-command regressions | Pure owner-source candidates may omit them only when recipes, release metadata, locks, scripts and fixtures are unchanged from their baseline. |

Every omitted stage carries its reason in the plan. These are deliberately
conservative candidates: there is no package-only Clippy shortcut or version
projection. The normal Linux and both macOS CI jobs remain unchanged; a qualification
receipt proves only its recorded host, never a substitute native-host result.

## Run and inspect

From clean committed source, with tools/dependencies and the PocketIC server
already prepared:

```sh
bash scripts/release/qualify-validation-plan.sh runtime runtime-v0.19.0 workspace
bash scripts/release/qualify-validation-plan.sh runtime runtime-v0.19.0 auto
```

These are explicit broad qualification commands, not ordinary development checks.
Plans/logs/receipts are retained under `target/release-validation/`. Their exact
source and baseline must match before comparing full and selected outcomes. A
shared-input change correctly makes both plans full; never force narrow selection
by changing the baseline just to shorten a release.

Shared's `validation-evidence.pl verify` interprets events, timings, checkpoints
and logs for the exact original run. Platform retains its complete proof in
the consumer receipt, including relative run/log paths, the expanded input
checkpoint and every evidence-file digest. A PASS records target success; it
does not independently assert that an underlying check executed afresh. Stage
reuse remains disabled. Excluded stages must remain not-applicable. Receipts bind
plan/policy/input hashes,
source/group/baseline, host/compiler and relevant build/server environment.
Failed or interrupted attempts produce no success receipt. Retry uses a fresh
attempt. Recovery admission checks unchanged input evidence for the same saved
release intent. Stage reuse between fresh releases is not enabled.

## Release and recovery admission

The Shared release runner still owns the lock, saved intent, metadata editor,
commit, tag and delivery. The consumer hook records a receipt pointer beside
that intent at `.git/release-state/GROUP-VERSION.validation`. The journal binds
the receipt's path and SHA256, protecting the complete original Shared proof
before candidate projection begins. Its baseline is the
selected group's previous-version tag; a missing tag selects the full roster.
Release mode permits omissions only on an explicitly qualified OS/architecture
(and macOS major). Unknown hosts always select the full roster.

Preparation and subsequent adapter checks re-admit that receipt. Source, group,
previous/candidate versions, remote and branch must match. Only the existing
metadata editor's declared files may differ in the index or working tree; its
semantic checks remain mandatory. Unchanged-source admission invokes Shared's
`check` at the original physical source/run location. After metadata preparation,
the generic checker cannot observe original inputs through prepared live manifests.
Platform's separate custody contract therefore verifies the journal's receipt
digest, reconstructs the immutable original source with `git archive` for Shared's
input checkpoint and checks every file in the retained proof's digest inventory.
It does not reverse live Cargo versions, rewrite runner paths, regenerate the
proof or claim that the generic checker admits a projection. Relocation refuses.
Receipt, stage logs and input hashes must match, as must compiler executables,
versions, host and build/server settings.
A local PocketIC executable is also fingerprinted. Missing or changed evidence
refuses recovery; retain the complete attempt directory and journal pointer.
Version 3 receipts require the digest-bearing journal. Older saved attempts
retain their original admission code and evidence; restore that source to resume
them rather than converting or regenerating their proof under this implementation.

Historical saved intents with the recognized former full-gate recipe retain
their original Shared recovery checks. Their logs are never reinterpreted as new
plan receipts. An unknown historical recipe refuses this fallback.

The `Release validation qualification` workflow runs full and selected plans on
Linux x86_64, macOS 15 Intel and macOS 15 ARM64. It prepares the pinned compiler,
dependencies and Testkit-owned server, then runs
`scripts/release/qualify-native-validation.sh` in a disposable clone. All three
receipts must name the same source, baseline and host, with the expected stage
counts. The workflow retains receipts, logs, timing, installed evidence and a
Git bundle of the synthetic qualification commit over its recorded input source.
Synthetic tags exercise the mappings without changing actual release history.
Host allowlist activation requires successful retained results; changing only
that allowlist does not trigger another broad PR run. Manual dispatch remains
available. Normal CI and installed-runtime workflows remain unchanged.

## Current qualification

`bash scripts/ci/test-platform-validation-plan.sh` runs the real selector,
Shared plan runner and receipt admission against disposable stage bodies. It
covers all group rosters, full expansion for shared locks, required-stage failure,
excluded-stage failure, stale source, new stages, dirty input, failed attempts and
fresh retry. These are dispatch/custody proofs, not product builds or timings.

`bash scripts/ci/test-platform-validation-recovery.sh` exercises the real Shared
release runner and receipt hooks for all three groups, with substitute product
stages and metadata. It covers preparation interruption, commit/tag/pre-push
recovery, local bare-repository delivery and refusal of source, environment,
server, baseline, receipt, journal and stage-log drift. The existing grouped
preparation fixture separately exercises the real metadata editor.

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

### Native activation qualification, 2026-11-03

[Run 41625352854](https://github.com/dragginzgame/ic-platform/actions/runs/41625352854)
passed all three hosts against committed input
`082be2ff19f52ced0653f2a589c092bc0ffc3e90`, using Rust/Cargo 1.99.1 and the
Testkit-owned PocketIC server. Each host ran the complete facade roster, followed
by the three selected rosters, on one synthetic commit with identical product
source. Recovered Git bundles differ from the input only by the qualification
sentence in the handoff. No package version or actual release ref was changed.

| Native host | Full / facade (20 stages) | PocketIC (18) | Runtime (18) | Types (19) |
| --- | --- | --- | --- | --- |
| Linux x86_64 | 1,709 s | 302 s | 296 s | 307 s |
| macOS 15 ARM64 | 3,045 s | 654 s | 658 s | 687 s |
| macOS 15 Intel | 5,962 s | 939 s | 1,091 s | 1,308 s |

These are sums of recorded stage durations. Selected runs had warm compiler
artifacts after the full run; the table is not a controlled speedup benchmark.
All 12 receipts and 24 installed Runtime/Timers profile statuses passed.
Archive digests, retained plan/context/input/log hashes and source identities
were verified after download. Ordinary same-source CI also passed Linux and
both macOS hosts; browser and installed-runtime workflows passed separately.
The dependency advisory tracked by #50 is unchanged.

The run retains `release-validation-Linux-x86_64` (artifact 13229210141),
`release-validation-Darwin-arm64` (13231567401), and
`release-validation-Darwin-x86_64` (13237276477). Local copies and verification
reports are under `target/issue-137-native-41625352854/`. Each artifact includes
its exact synthetic source identity/bundle, four receipts, stage logs, timing
and installed-profile evidence.

That activation enabled those three qualified host identities. Following #139's
client retirement, the allowlist was emptied; this four-group evidence does not
activate the current three-group mapping. The qualified
selector, receipt, recovery and runner code is unchanged by activation. Fresh
releases still execute every required stage; these qualification receipts cannot
be reused as release receipts. The first release containing this shared-script
batch still selects the full roster because its baseline inputs changed.
