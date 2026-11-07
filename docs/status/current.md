# Current handoff — 2026-11-07

## Platform 0.25.1 release preparation (#148–#150)

Shared **0.31.6** at `84b52d14a9082eca0c43db807f649ddf8a74f02c` is now
adopted through the canonical exporter: **145** verified payloads, including the
new pure-hashing policy and exact-input stage runner/sealer entrypoints.
The maintainer authorized Platform **0.25.1** delivery to GitHub and crates.io.
Release preparation is in progress; the published identities below remain the
last verified versions until batch/registry reconciliation. Runtime, Types and
browser have no pending increment.

[#148](https://github.com/dragginzgame/ic-platform/issues/148): native Runtime,
Agent, BLS and ECDSA fixtures select published **Testkit 0.92.0**. Each owns one
Tokio runtime, awaits network/lifecycle operations, uses an explicit finite
60-second client request budget and awaits instance teardown. Startup remains
separately bounded. Catalog/template generators and both maintained qualification
locks agree and each loses 47 package entries; source and protocol assertions remain unchanged. Native fixture-only
dependencies do not enter production Wasm graphs.

Focused Linux/Rust 1.99.1 evidence under `target/issues148-150/`:

- Owner raw: `target/cdk-runtime.T4GaI3`; Timers raw:
  `target/cdk-runtime.DEbia1`.
- Owner raw/prepared: `target/cdk-runtime.HcepQ3`; Timers raw/prepared:
  `target/cdk-runtime.MiWx4c`. Each covers Wire and external Candid, graph
  admission, driver Clippy/six tests and installed behaviour; all status files are 0.
- Agent Clippy and installed signed query/update/read-state/wrong-root refusals
  passed for Ed25519, delegated, P-256 and secp256k1 identities
  (`target/agent-installed.tDElpF`). Both benchmark native drivers pass Clippy;
  their historical performance experiments were not rerun.
- Snapshot, links, pins, inventory, boundary, lock identity, standalone formatting
  and changed-script ShellCheck passed. No blanket fleet gate was run.

[#149](https://github.com/dragginzgame/ic-platform/issues/149): only the bounded
release-plan/recovery policy fixtures can reuse Shared exact-input proofs on
Linux x86_64. All 20 required parent stages, real package-version/native/archive
checks and per-group source/index/metadata custody remain. Schema-4 receipts seal
fresh leaf admissions and original proof/log identities; schema-3 recovery retains
its original semantics. The actual-leaf activation harness passed at
`target/release-plan-test.WBLEAT` (`target/issues148-150/stage-reuse-qualified.log`).
Five successful group receipts retain 20/20 parent stages each: three executed
leaves and two reuses. Exact-match execution/admission took 133.19s versus 16.43s
(116.76s saved for this leaf); changed source/tools forced execution. Missing or
altered evidence refused, failures retained primary status 27/143 with no receipt,
and recovery reused the original intact proof. The other product stages and
metadata are substitutes, not current product/native macOS acceptance. Structured
results are in `target/issues148-150/reuse-qualified-summary.json`. Prior real
Runtime/Platform gates took 1,049s/833s, including 481s/500s for the whole
release-command stage; those totals are repeated cost, not reusable savings.

[#150](https://github.com/dragginzgame/ic-platform/issues/150): the new policy is
installed. Wire removed its unused activation at `72ab905`. IcyDB Core, SQL
helpers and the independent integration workspace still import the facade at the
last read-only observation; IcyDB #631 belongs to another session. Preserve the
forwarder until those callers migrate, following policy → consumers → removal.
`target/issues148-150/blake3-callers.txt` records the observed paths. No sibling
source or release was changed here.

The separate root Cargo.lock draft now also includes arc-swap **1.9.4**, alongside
Candid/derive 0.10.40, cc 1.11.1 and SmallVec 1.16.5. It remains excluded from this
release. Deployment-start SHA-256:
`ca4c45c0e5e60b15f07d12b3f7b331a0fe237cf590f7bd16d1d1ce252fec0c50`.
`target/deployment-platform0251-20261107/` retains its bytes and external-only
patch; the earlier task-start lock remains under `target/issues148-150/`.
No new full release gate, publication, native macOS or Profiler evidence is claimed.

## Platform 0.25.1 delivery state

The initial capture refused the independent dirty lock without release effects.
Its exact draft is retained durably at `retained/platform0251-lock-draft`, based
on preparation commit `1b5909acd90f58620e7f5e653ca1a57bf27dd79d`. After clean
capture, that same draft was restored to the normal tree and remains deferred.

Batch `batch-ZNfZ3p9z` pinned that preparation source. Its validator was stopped
before metadata preparation/publication after identifying that real child Make
adds a tool-path prefix absent from the earlier substituted dispatch. Preserve
its failed/interrupted evidence; no package or tag was delivered by it. Receipt
observation now uses the same child Make owner as leaf execution. The regression
fixture retains the actual PATH export/sealing recipe and refuses direct parent-
environment sealing. Qualification is being rerun before a new frozen capture.
See `target/deployment-platform0251-20261107/` for capture, correction and draft
custody evidence. No schema or old proof is rebound to the correction.

The Make-context regression passed all five successful group receipts and its
negative/recovery cases at `target/release-plan-test.rI7xjI`. Correction source is
`68f624f98d022886925de6c26e65685b4f9c73a6`. Batch `batch-2ylQaRca` then passed
through workspace tests/docs but failed consumer-admission fixture setup: the
new Shared dependency checker was copied without its declared hash-policy file.
No metadata, tag or upload was produced. Three owned fixture constructors now
use Shared's declared companion copier; consumer/prepared-call admission and real
Runtime independence pass. Failed consumer fixtures also retain their evidence.
The next source receives a fresh complete gate; no old receipt is reused.

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

The two publications used Shared **0.31.5** at
`b2026a118e38de4af1555c527af327cb604eacdf`, with all 141 payloads verified.
The current local adoption is recorded above. The metadata editor remains ic-build 0.73.0.

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
