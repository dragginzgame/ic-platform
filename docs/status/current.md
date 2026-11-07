# Current handoff — 2026-11-07

## Runtime 0.20.0 and Platform 0.25.0 deployed

Both groups are published on crates.io and GitHub, delivered through Shared's
standard frozen batch `batch-x2PJM4D1`. Its schema-2 intent is complete at index 2;
normal-tree reconciliation completed through the guarded shared command.

| Group | Published packages | Release commit | Annotated tag |
| --- | --- | --- | --- |
| Runtime | Runtime, macros, exact guard **0.20.0** | `8e8ed30e8090f3c06ed8b6dc72e6825e2a35e653` | `runtime-v0.20.0` |
| Platform | Facade **0.25.0** | `0a32e8a2f7977a71062d4edec542bea96d63c508` | `platform-v0.25.0` |

Source preparation is `0c4c41f7e830982b55d6db4a01ea93cc72fa2909`; the second group
qualifies the Runtime release as its source. All four registry versions are
non-yanked. Downloaded archives match official registry checksums and record
clean VCS provenance at their supplying release commit. Remote annotated tag
objects and peeled commits match local refs. Types stays **0.18.2**; browser
stays **0.2.0**, with no Types or npm publication.

Evidence: `target/deployment-platform0250-runtime0200-20261107/`, including
`registry/{runtime,platform}-verified.tsv`, remote-tag observations, batch logs,
`reconcile.log` and preserved copies of the two qualification receipts.
Runtime archive SHA-256:
`edd7cdf5798dc7830346a971d33a9790073d769d676d8902af3f5928a3a434f7`.
Platform archive SHA-256:
`f446080bf2af9617bb49a18b6b1ec872b59867a51e82911029a2f9ad6780c3e1`.

## Contract and consumer migration

Runtime owns the single conventional `cdk::{query, update, init, ...}` macro family.
The facade's `cdk` feature/ordinary macro exports and implementation `runtime_*`
family are removed. Facade raw `runtime` adapters and external-Candid endpoints
remain. Consumers/scaffolds select Runtime `cdk` directly; explicit crate overrides,
codecs, ABI, guard ordering and executor identity are preserved. See
[the migration](../migration.md#canonical-ordinary-entrypoints-runtime-020--platform-025).
No new cryptographic implementation or measured Wasm/performance win is claimed.

The inactive selected-stage pilot, host allowlist and qualification workflow are
removed. Every release retains all 20 stages and source/metadata/evidence recovery
admission. Preparation/rollback/custody covers every group; Runtime supplies one
full fixture delivery/resume cycle. Admission tests retain real graph rejection,
final Cargo failure and concurrent-edit propagation using the current profiles.

Metadata preparation now resolves qualification locks offline from their maintained
selections inside the transaction. Native Testkit's older facade retains a registry
Runtime 0.19.5 record with no enabled executor after the local owner moves to 0.20;
prepared canister graphs still require the sole current executor. Foreign pins
remain unchanged. Preparation fixtures use real post-version graphs and the shared
changelog validator to select the pending increment or a compatible default patch.

Scheduling qualification source is `e50ca4f91e7cc213e089d7ef79f9c224d36d61ad`,
branch `adopt/runtime-020-platform-qualification`. Only the previous supplier's
Runtime upper bound changes to <0.21; Timers 0.37 implementation stays unchanged.
No Scheduling working files, product release or other sibling adoption was taken over.

## Qualification and baseline

Both groups passed **20/20** required local stages on Linux x86_64 / Rust **1.99.1**:
full lint/tests/docs, standalone native/Wasm contracts, feature closures, provenance,
real metadata/recovery and installed Runtime owner/Timers raw/prepared Wire and
external-Candid profiles. Installed proofs include awaits, traps, cancellation,
upgrades, stable rollback, management bytes, certificates and endpoint admission.
Receipts admit workspace scope and no omissions; no historical stage reuse occurred.

Canonical receipts under `target/release-frozen/batch-x2PJM4D1/target/release-validation/`:
`qualification.VZ8wwE/receipt.json` (Runtime, logs `validation.kCAdoy`) and
`qualification.pIOdf9/receipt.json` (Platform). The complete unmodified Shared proofs
remain bound to their source, compiler, server, environment and metadata projection.

Hosted [CI](https://github.com/dragginzgame/ic-platform/actions/runs/42382021050),
[installed Runtime](https://github.com/dragginzgame/ic-platform/actions/runs/42382021051),
browser, Wasm and advisory jobs were in progress at the single final observation;
see `hosted-observation.json` in deployment evidence. Do not wait solely for that
matrix. No fresh macOS or native Profiler/Bazel acceptance is claimed.

Shared **0.31.5** at `b2026a118e38de4af1555c527af327cb604eacdf` is the adopted
baseline and latest remote main observed during this task. The canonical exporter
verified all 141 payloads. The metadata editor remains ic-build 0.73.0.

## Deferred draft and recovery custody

Candid/derive **0.10.40**, cc **1.11.1**, SmallVec **1.16.5** remain an independent,
uncommitted Cargo.lock draft and were excluded from both publications. Their external-only
patch was reapplied over delivered owner versions after reconciliation. Current
lock SHA-256: `fc78e45b197bd46d6050031f30094eea6b4f4e089540463a618ce6caa511530a`.
Original custody: `refs/retained/platform0250-runtime0200-20261107-lock-draft`;
deployment evidence retains the exact old lock and original/restored patches.
Never restore that old whole lock over delivered owner versions.

Preserve failed `batch-d_fzxuMA` (old admission fixture) and `batch-TC9AK4RR`
(post-version lock failure, saved phase prepare), plus the interrupted first
validation of the completed current batch. All are retained without source,
receipt or journal reinterpretation. Recovery removed only the interrupted batch's
empty ownerless mutex after checking live processes; its stat receipt is retained.
Prior completed batches `batch-KU2DvnyL`, `batch-U1NhGrpl`, `batch-hEX13lnF`, retained
refs and interrupted unpublished 0.23.1 state remain untouched. No cleanup is authorized.

Historical archives/bundles/receipts remain at
`recovery-evidence/retired-checkouts/20261104-issue139/`; maintainer review is due
**2026-12-04**. Retention refuses expiry/excess without deletion. Older target evidence
was previously removed externally; do not claim those receipts or Runtime 0.18.1
binaries still exist. Follow [the batch lifecycle](../platform-release-validation.md#commands-and-effects).

#50 remains open/deferred for native Agent's crypto-bigint 0.5.5 advisory; no waiver
was added. Existing .24 caller migrations stay with Canic #739/#740, Blob #119 and
IC Crates #25. Earlier detail remains in
[the previous handoff](https://github.com/dragginzgame/ic-platform/blob/0c4c41f/docs/status/current.md).
