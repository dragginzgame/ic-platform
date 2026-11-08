# Current handoff — 2026-11-08

## Deployment preparation — 2026-11-08

The maintainer authorized Runtime 0.20.1 and Platform 0.26.0 delivery to
`dragginzgame/ic-platform:main` and crates.io. Shared 0.31.6 supports one
increment kind per batch, so the two standard batches run Runtime patch first,
then Platform minor; Types/browser are unchanged. No release has run yet.

The deferred lock draft is preserved durably on
`retained/platform026-runtime0201-lock-draft`, commit
`24c865191d14c628b86e2cc7307988827c3f1e58`. The release lock uses the preceding
committed suppliers plus BLAKE3 removal (SHA-256
`3443891a2358040d74a92f1040b8d1150509b2a3662ab917e9db6a5e660f0fc4`).
Exact external draft/reconciliation inputs are retained under
`target/deployment-platform026-runtime0201-20261108/`. Restore only that external
patch after reconciliation, never its older package-version metadata.

Release formatting preparation places the four native driver sources and their
shared support module together in `scripts/fixtures/cdk-runtime/`, preserving
ordinary module discovery in both the source tree and rendered fixtures. The
standard `make fmt-check` passes with all four drivers selected.

## Ten simplifications implemented locally — 2026-11-08

All ten maintainer-selected cleanups are implemented in this checkout. Nothing
was staged, committed, pushed or published. Pending Platform **0.26.0** removes
its public BLAKE3 feature/export; pending Runtime **0.20.1** contains compatible
macro-parser maintenance. Package versions and Types/browser groups are unchanged.
The preceding 0.25.1 deployment evidence below remains historical release evidence.

- Retired the BLAKE3 facade, catalog selection and dedicated fixtures after the
  maintainer confirmed IcyDB migration. A read-only projects-wide source/manifest
  scan found no remaining facade callers. Platform
  [#150](https://github.com/dragginzgame/ic-platform/issues/150) is closed with
  explicit local/unpublished status; IcyDB deployment remains its owner's work.
  Existing minimal native/Wasm graph checks now forbid the removed dependency.
- Ordinary copy qualification retains both implementations and every size in
  ten queries rather than sixty. The former six-round medians are explicit via
  `PLATFORM_BUFFER_COPY_BENCHMARK=1`, with the same byte assertions.
- Four native drivers share ordinary test-only server/runtime setup. Each keeps
  its explicit timeout, topology, client close and managed-server custody.
- Runtime's three qualification manifests render once from explicit source/
  published, owner/Timers and external-adapter selections, with no later edits.
- Five integration consumers share retained-hex decoding; standalone and installed
  fixture copying includes its ordinary support module. Test-only use stays gated.
- Runtime/ICRC provenance guards share their exact existing normalization recipe,
  retaining separate fingerprints, reconstruction, licenses and mutation refusals.
- Two admission runners use Shared's existing fixture lifecycle operations,
  retaining child cleanup, failure status and incomplete-run refusal.
- Exact Candid arity checks have one private implementation; the protocol macros
  share decoder emission. Existing diagnostics, optional/zero/single/multiple
  result behavior, budgets and upstream bytes remain unchanged.
- Both macro adapters share duplicate-option and boolean parsing, retaining their
  distinct accepted options, repeated guard order and explicit codec selection.

Focused Linux x86-64 / Rust 1.99.1 validation passed: affected-package Clippy;
21 Candid tests (one manual benchmark ignored), 18 selected integration cases,
16 macro tests and six generated native-driver tests. Provenance reconstruction
and mutation refusals, both admission runners, 144 synthetic feature-graph cases
plus target/source controls, inventory, dependency boundaries and six real minimal
native/Wasm graphs passed. The retired feature is rejected before traversal.

Actual Runtime owner graphs: `target/cdk-runtime.Mt7hd6/`. Installed Timers
raw/prepared Wire/external-Candid profiles: `target/cdk-runtime.9nyeDC/`, with
callback traps, upgrades, stable rollback, certificate/management byte proofs,
auth-before-decode and 16/17 nesting admission preserved. Actual Agent acceptance:
`target/agent-installed.DySTDY/`, including signed query, update/read-state and
wrong-root refusal for Ed25519, delegated, P-256 and secp256k1 identities. Both
benchmark drivers passed Clippy in `target/cleanup-benchmark-drivers.dJ1qt3/`.
The opt-in buffer-copy measurement also completed all sizes; log:
`target/platform-cleanup-buffer-copy-benchmark.log`. Standalone codec consumer
native tests and Wasm compilation passed in `target/boundary-consumer.uMINbk/`.
Each retained fixture binds its actual captured inputs; these focused results
are not a new release receipt. No macOS/Profiler acceptance or hosted CI was run.

The pre-existing dependency draft remains. Compared with its retained supplier
records, Cargo.lock only prunes BLAKE3 1.8.7, arrayvec 0.7.8 and constant_time_eq
0.4.2; pretty's remaining arrayvec reference loses unnecessary version
qualification. No retained registry version/checksum changed. Current lock SHA-256:
`94f86637af25de54bf34f3fbfe4741bf379565e60f77ab880444122463abc37c`.
Shared 0.31.6 snapshot payloads, prior handoff/recovery custody, native Profiler,
browser and sibling sources are preserved. Platform #50 remains deferred.

## Platform 0.25.1 published and reconciled

The maintainer authorized delivery of #148/#149 and Shared's new hashing policy.
Platform **0.25.1** is published on crates.io and GitHub through completed frozen
batch `batch-pB_048Ja` (schema 2, index 1). Normal `main` and `origin/main` are
reconciled to release **`daf454df49d8de3edbb11cd260e8dacf6524c266`**.
Annotated tag: **`platform-v0.25.1`**, object
`36833ca0a2fcd5582b85a245e5498e172f88fb4e`.
Preparation source: `924828e32c7a5d5e71d656ea373a6825c90314ff`.

The official registry record is non-yanked. Downloaded archive SHA-256:
`76bc9d6ea3bf2d8c274541aa482c0b47f43177b7d3a6bfe2555531080bfb0100`.
Its clean VCS provenance names that exact release commit and `crates/ic-platform`.
Remote main/tag/peeled commit match local delivery. Runtime/macros/guard remain
**0.20.0**, Types **0.18.2**, browser **0.2.0**; none was republished.

Canonical evidence: `target/deployment-platform0251-20261107/`, including registry
archive/index/VCS, `remote-delivery.txt`, `reconcile.log`, draft custody, source
commits and all release-attempt logs. The final handoff update is local: pushing
only status docs now would cancel the running main-branch code qualification.
No implementation change remains unpublished.

## Delivered scope and qualification

[#148](https://github.com/dragginzgame/ic-platform/issues/148): owned native Runtime,
Agent, BLS and ECDSA fixtures use published **Testkit 0.92.0**, one caller-owned
Tokio runtime, explicit positive 60-second request budgets and awaited teardown.
Startup remains separately bounded. Catalog/generators and both maintained locks
agree; each lock loses 47 package entries. Production Wasm gains no Tokio edge.
Crypto/protocol assertions remain. Historical benchmark/provider source stays
outside adoption; native benchmark drivers pass Clippy, without new perf claims.

[#149](https://github.com/dragginzgame/ic-platform/issues/149): only Linux x86_64
release-plan/recovery policy fixtures reuse Shared exact-input stage proofs.
Every group retains all 20 parent stages and fresh real Cargo/native/version,
metadata, archive/publication and source/index custody. Schema-4 receipts seal
fresh admissions and immutable original proof/logs; schema-3 recovery is preserved.
Sealing re-enters the execution's Make owner, preserving its actual PATH prefix.

The corrected activation fixture `target/release-plan-test.rI7xjI` passed five
complete substitute-parent gates: three executed leaves and two reuses. Matching
execution/admission took **135.86s / 18.50s**. Source/tool drift forced execution;
missing/tampered evidence and direct parent-environment sealing refused. Primary
27/143 failures produced no receipt; retry admitted the intact original proof.
Its product stages/metadata are substitutes. Structured details:
`make-context-qualified-summary.json` in deployment evidence. Earlier 133s/16s
observations predate this Make-dispatch correction and are not its qualification.

The actual release passed **20/20** local stages on **Linux x86_64 / Rust 1.99.1**,
including package Clippy/tests/docs, native/Wasm consumers, feature closures,
provenance, real metadata/recovery and installed Runtime owner/Timers raw/prepared
Wire/external-Candid profiles. Receipt:
`target/release-frozen/batch-pB_048Ja/target/release-validation/qualification.hd1kU0/receipt.json`.
It retains workspace scope, no omissions and one executed policy leaf (no reuse
in this single-group release). Metadata recovery admitted that same receipt;
publication separately verified/linted the final 0.25.1 archive.

Focused Agent installed acceptance also passed signed queries, update/read-state
and wrong-root refusal for Ed25519, delegated, P-256 and secp256k1 identities.
The new Shared dependency checker now reaches three owned fixture constructors
through its declared companion closure; consumer/prepared-call admission and real
Runtime independence pass, and failed consumer fixtures retain evidence.

Hosted [CI](https://github.com/dragginzgame/ic-platform/actions/runs/42399099889),
[installed Runtime](https://github.com/dragginzgame/ic-platform/actions/runs/42399099726),
browser and advisory jobs were in progress at the single final observation.
[Wasm size](https://github.com/dragginzgame/ic-platform/actions/runs/42399099687)
passed. `hosted-observation.json` binds these observations to the release SHA.
Leave hosted work asynchronous. No fresh macOS or native Profiler/Bazel acceptance
is claimed; unqualified reuse hosts execute the same policy fixtures freshly.

## Boundaries and remaining owner

Shared **0.31.6**, `84b52d14a9082eca0c43db807f649ddf8a74f02c`, is adopted through
the canonical exporter with **145 verified payloads**. IC Build editor stays
0.73.0. No snapshot payload was hand-edited. Production crypto, certificate,
delegation, freshness policy and external bytes are unchanged.

[#150](https://github.com/dragginzgame/ic-platform/issues/150) remains pending:
Wire removed its unused BLAKE3 activation at `72ab905`; IcyDB Core/SQL/integration
still use the facade at the final read-only observation. Another session owns
IcyDB #631. Keep Platform's export until callers migrate under Shared's approved
direct hashing policy. No sibling checkout or release was changed here.
#50 remains explicitly deferred for native Agent's crypto-bigint 0.5.5 advisory;
no waiver or cryptographic rewrite was added.

## Deferred draft and recovery custody

The independent lock draft retains arc-swap **1.9.4**, Candid/derive **0.10.40**,
cc **1.11.1** and SmallVec **1.16.5**. It was excluded from delivery and reapplied
as the exact external-only patch over released 0.25.1 metadata. Current lock SHA:
`d5e76b0a72c3be0e4afd789b936c45c040244dd2251b9152b1cfd60cf6bb88be`.
Durable branch: `retained/platform0251-lock-draft`, commit
`25a397b1055bf2d20cec22ee03d3185da6cb866f`.
Its original whole lock has older owner metadata; never restore it over 0.25.1.
The before/after patches differ only in Git's base/result blob header.

Preserve `batch-ZNfZ3p9z` (stopped before preparation after finding the Make-prefix
mismatch) and `batch-2ylQaRca` (consumer fixture missing the new hash-policy file).
Neither tagged, pushed or uploaded. The initial dirty capture refused before a
batch existed. Corrective source commits are `68f624f` and `924828e`; no old proof
was rebound. All previous failed/completed batches, refs and interrupted 0.23.1
state remain as recorded in the
[previous handoff](https://github.com/dragginzgame/ic-platform/blob/924828e32c7a5d5e71d656ea373a6825c90314ff/docs/status/current.md).

Historical archives/bundles/receipts stay at
`recovery-evidence/retired-checkouts/20261104-issue139/`, with maintainer review due
**2026-12-04**. Retention refuses excess/expiry without deletion; no cleanup or
mtime renewal is authorized. Previously removed target evidence/0.18.1 binaries
must not be claimed present. Use the existing batch lifecycle and supplying
source for recovery, never reinterpret historical receipts under new tooling.
