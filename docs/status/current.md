# Current handoff — 2026-11-07

## Local complexity cleanup and Shared adoption (unpublished)

Pending **Runtime 0.20.0 / Platform 0.25.0** removes the paired ordinary macro
families: Runtime's conventional `cdk::{query, update, init, ...}` exports are
canonical. The facade's `cdk` feature and ordinary macro exports are removed;
its raw `runtime` adapters and explicit external-Candid bridge remain. Consumers
must select Runtime `cdk` directly, including generated/scaffold dependencies.
Explicit crate overrides, codecs, ABI, guard ordering and executor identity are
preserved. See [the migration](../migration.md#canonical-ordinary-entrypoints-runtime-020--platform-025).
No sibling source or release was changed.

The inactive selected-stage pilot, host allowlist and qualification workflow are
removed. All **20** local release stages remain mandatory for every group;
source/metadata/evidence receipts and recovery refusals remain. Real preparation,
rollback and custody tests still cover all groups; Runtime supplies one full
local delivery/resume cycle instead of repeating it for every group.

Focused Linux/Rust 1.99.1 checks pass: affected-package all-target/all-feature
Clippy; Runtime/macros and moved CDK consumer tests; dedicated external-Candid
consumer tests and standalone native/Wasm contracts; feature-policy regressions
and actual Linux/Wasm closures; singleton/type-identity refusals; full-only planner,
three-group receipt recovery and real group preparation. Installed owner and
Timers raw/prepared Wire/external-Candid checks pass, including awaits, callback
traps, cancellation, upgrade, stable rollback, certificates and admission.
Evidence: `target/complexity-cleanup/`; installed fixtures
`target/cdk-runtime.y9J8xH/` and `target/cdk-runtime.6ReMRn/`.
Planner/recovery fixtures substitute product stage bodies; installed checks use
the prepared real Testkit server. No full release gate, fresh macOS/Profiler
acceptance, publication or matched size/performance improvement is claimed.
Runtime 0.20 release qualification will need a compatible immutable Timers supplier;
current tests qualify the local 0.19.5 owner with the macro changes, not a published
0.20 cohort. Types/browser and the independent lock draft remain untouched.

Adopted committed Shared `b2026a118e38de4af1555c527af327cb604eacdf` through the
canonical exporter after impact review; all 141 payloads match. Local overlay
and pending Platform 0.25.0 notes follow that baseline. No package version changed.
Fresh override-scratch LOC reporting and selected shell/Perl syntax/lint pass;
Platform group routing/publication-subset checks pass. Evidence:
`target/shared0315-adoption/`.
[Advisory run 42365731394](https://github.com/dragginzgame/ic-platform/actions/runs/42365731394)
failed on crypto-bigint 0.5.5 / RUSTSEC-2026-0387 only. Runtime and Timers
composition audit statuses are zero with complete index observations. #50 stays
open and deferred at the maintainer’s instruction; no waiver or retry was added.

## Platform 0.24.0 deployed

Platform **0.24.0** is published on crates.io and GitHub under annotated tag
`platform-v0.24.0`, release commit `fc6d419474a94a606e3c49729747e61978b4c9c7`.
Source commit `467279b2134e5812048786c6c2b0616eff606ac8` was delivered through
Shared schema-2 batch `batch-KU2DvnyL`; delivery and normal-tree reconciliation
completed. The registry archive is non-yanked, matches its registry checksum
and records that exact clean release commit. Remote tag object and peeled commit
match. Evidence: `target/deployment-platform0240-20261107/`, including
`registry/verified.tsv`, `remote-tag.txt`, `release.log` and `reconcile.log`.

Only Platform advanced. Runtime/macros/guard remain **0.19.5**, Types **0.18.2**,
and browser **0.2.0**. No sibling source, release or npm publication was taken over.
No batch or recovery evidence was retired or deleted.

## Hard cuts and consumer handoff

The native `secp256k1` feature/module and `ic-secp256k1` supplier are removed,
along with their tests and unused derivation experiments. The source lock loses
five packages (ic-secp256k1, base64 0.13, pem 1, simple_asn1, time-macros) and adds
none. This is a native dependency cut; no production Wasm saving is claimed.
The separate `ecdsa` and Agent capabilities remain.

Unconfigured `candid::decoder::{decode_one, decode_args}` exports are removed.
Explicit configuration, staged decoding, canonical identities, numeric ranges,
external bytes and every cryptographic check remain. Owned fixtures retain their
prior decoder choices through explicit configuration. The four remaining unused
crypto packaging patches and their instructions are retired; no replacement
cryptographic implementation or dependency is introduced. The release-command
roster also stops replaying Shared's generic changelog-parser test matrix while
retaining Platform preparation, custody and recovery coverage.

Required caller migrations remain at their owners:

- [Canic #739](https://github.com/dragginzgame/canic/issues/739): authenticated
  chain-key lookup/session caching and safe bootstrap before adopting the cut.
- [Canic #740](https://github.com/dragginzgame/canic/issues/740): production
  history/host decoder calls and associated tests.
- [Blob #119](https://github.com/dragginzgame/ic-blob-storage/issues/119): the
  gateway test-canister decoder call; existing configured production paths remain.
- [IC Crates #25](https://github.com/dragginzgame/ic-crates/issues/25): managed
  HTTP test-helper decoding.

Agent still brings the older K256/bigint family. #50 is unresolved and excluded;
no advisory waiver or cryptographic change was added.

## Qualification

All **20** required local stages passed on Linux x86_64 / Rust **1.99.1**, including
full lint/tests/docs, feature graphs, standalone native/Wasm consumers and installed
Runtime owner/Timers raw/prepared Wire/external-Candid profiles. Installed proofs
include await/trap cleanup, upgrade, stable rollback, certificates, management
bytes and endpoint admission. No full-gate omission or historical reuse was admitted.
Canonical receipt: `target/release-frozen/batch-KU2DvnyL/target/release-validation/qualification.mXhnnO/receipt.json`;
logs `validation.Oc5mXN`, copied under the deployment evidence's `qualification/`.

Release gates used committed Candid/derive 0.10.39, cc 1.11.0 and SmallVec 1.16.4;
focused pre-release checks separately passed with the deferred newer versions.
The release commit's [CI](https://github.com/dragginzgame/ic-platform/actions/runs/42365548736),
[installed Runtime](https://github.com/dragginzgame/ic-platform/actions/runs/42365548756),
browser, Wasm and advisory jobs were in progress at observation. They remain
asynchronous; no fresh macOS or native Profiler acceptance is claimed. The source
commit's hosted browser and Wasm comparisons passed. Recorded observations are in
`{source,release}-hosted-observation.json` beside deployment evidence.

## Deferred dependency draft

The independent Candid/derive **0.10.40**, cc **1.11.1** and SmallVec **1.16.5**
updates are deferred from publication; the committed graph is selected for deployment.
The draft reappeared during frozen qualification with its exact retained checksum;
frozen inputs stayed unchanged. After reconciliation its normalized external patch
was restored exactly over the delivered owner versions. Current lock SHA-256:
`4ce766eee04d278fe4885ece92342bf4fa27f21f572259e5021b0422f435fe3f`.

Current custody: `refs/retained/platform0250-runtime0200-20261107-lock-draft`
and `target/deployment-platform0250-runtime0200-20261107/`. Restore its external
dependency patch over delivered owner versions after reconciliation. Earlier
custody: `refs/retained/platform0240-20261107-reviewed-draft`, plus
`deferred.Cargo.lock`, `deferred-version.patch` and `restored-version.patch` in
deployment evidence. Earlier `refs/retained/audit-deploy-20261107-lock-draft` is
untouched. Never restore an old whole lock over delivered owner versions.

## Baseline and retained custody

The published 0.24.0 release used Shared **0.31.1** at
`5efbda2ab81ad84e30f490f3137d8ef083c303d2`. Local Shared **0.31.5** adoption is
recorded above and is not yet delivered. The metadata editor remains ic-build 0.73.0.
The selected-stage pilot is now retired, so releases retain full gates. Historical Shared
macOS setup/native reuse is separate from Platform acceptance; the supplier's broader
portable suite had failed at `test-release-editor-setup.sh`. No new result is claimed.

Prior Runtime 0.19.5 / Platform 0.23.6 publication and measured audit improvements
remain in the [previous handoff](https://github.com/dragginzgame/ic-platform/blob/10a04d9/docs/status/current.md),
owning ledgers and `target/deployment-audit-20261107/`. External cleanup previously
removed older target evidence; current focused and release evidence was regenerated
after the maintainer confirmed cleanup stopped. Do not claim the removed receipts
or Runtime 0.18.1 driver binaries are still available.

Completed batches `batch-KU2DvnyL`, `batch-U1NhGrpl` and `batch-hEX13lnF` retain
durable Git/journals under `.git/release-state/frozen/`. Earlier retained refs and
the interrupted unpublished 0.23.1 journal remain untouched. Historical archives,
bundles and receipts remain at `recovery-evidence/retired-checkouts/20261104-issue139/`;
maintainer review is due **2026-12-04**. Retention refuses expiry/excess without
automatic cleanup. Follow the [batch lifecycle](../platform-release-validation.md#commands-and-effects)
for future delivery. Root receipts do not qualify the separate native Profiler
provider; its [paired Linux evidence](../audits/native-profiler-140-2026-11-05.md)
remains historical, with macOS/Bazel unqualified.
