# Current handoff — 2026-11-06

## Local Shared 0.31.0 adoption

Shared **0.31.0** at `74313941af2fce4399bec0e6909ca6229673c3f8` is
installed through the canonical exporter, with all 141 payloads verified against
that committed remote-main revision. The adoption preserves the existing frozen
deployment work and the independent Cargo.lock draft. No package versions,
commits, tags or real publications were created. Storage stays with its session.

Platform setup, recovery admission, routing fixtures, the optional integration
qualifier and current commands now use `RELEASE_GROUPS`, `RELEASE_PUBLISH_TARGET`,
`RELEASE_SETUP_TARGET`, `RELEASE_SETUP_MODE` and `RELEASE_BATCH_ACTIVE`. Shared
owns automatic single-group capture and the renamed `run-release-batch.pl`/
`docs/release-batches.md` interface. Old settings refuse rather than alias. No
active pre-0.31 batch was present; such a batch must finish with its original
tooling because 0.31 requires schema-2 journals and a saved publication commit.

Use `make release-patch RELEASE_GROUP=platform RELEASE_PUBLISH_TARGET=publish`
for a separately authorized Platform delivery. An affected cohort uses one
`RELEASE_GROUPS=types,runtime,platform` roster, omitting unaffected groups. Shared
owns status, resume, clean fast-forward reconciliation and explicit retirement;
normal-checkout edits stay deferred. Candidate-local setup still uses existing
tool owners, with offline recovery checks and no normal-tree lock/tool copy.
Browser caches outside admitted build roots still require review before retirement.
See [commands and qualification](../platform-release-validation.md#commands-and-effects).

Focused checks passed under `target/shared0310/`: annotated snapshot/source
integrity, documentation links, both npm lock-schema/pinning selections, setup
prepare/recovery policy, all three groups' routing and same-group publication
admission, plus ShellCheck and shell syntax. Retired settings (including empty
values) and a cleared selected-group roster refuse before runner effects. These
routing checks use substitutes and do not perform registry delivery.
The prior 0.30.9
three-group rehearsal, verified archive bytes/receipts and installed Wire owner
profile remain at `target/platform-frozen-qualification.lSXxHi/`; they are
historical evidence for the unchanged product/setup behavior, not proof of a
0.31 full release gate or registry delivery. Real releases still require every
applicable standard gate and native/installed obligation. No new macOS/native
Profiler acceptance is claimed.

## Runtime 0.19.4 published: safe uninitialized stable reads

The maintainer now explicitly supersedes the earlier deployment exclusion of
`stable64_read_uninit` and requests Runtime publication before Storage adoption
and deployment. Runtime, macros and the exact singleton guard **0.19.4** are
published on crates.io and GitHub at `8ec8a395a54998bca3e995ea1a78c07e9977eff0`,
annotated tag `runtime-v0.19.4`. All three downloaded, non-yanked archives match
registry checksums and that clean embedded source commit; the remote annotated
tag object and peeled commit match. The restored export and IC0-only fixture come from
retained commit `998764c3a4d2a80db2b64a8521f3a0580b91a64d`; the imported
implementation body remains unchanged.

All **20** local release stages passed on Rust 1.99.1, including the full
native/Wasm feature matrix, source guards, consumers and installed owner/Timers
profiles. Receipt: `target/release-validation/qualification.KC2SnU/receipt.json`.
The exact registry Runtime additionally passes native/Wasm Clippy and compilation
of the restored API with only `ic0`; its normal/build graph contains just Runtime
and the proof consumer. This is published-API compilation evidence; Storage's
installed uninitialized-read behavior remains its consumer qualification.
Verification, publication and registry-consumer logs are retained under
`target/deployment-runtime-0194/`. Hosted native CI remains asynchronous; no new
macOS or native Profiler qualification is claimed.

Platform **0.23.4**, Types **0.18.2** and browser **0.2.0** remain unchanged.
The separate Candid/derive 0.10.40 and cc 1.11.1 lock draft is restored, retaining
all current owner versions. Its original is protected at
`refs/retained/runtime-0194-deploy-lock-draft`; original/restored patches under
the evidence directory match after excluding their base-blob index lines.
Never restore an old whole lock over the published release identities.

The maintainer assigned Storage adoption/deployment back to its active session;
this session changed no Storage source, manifest or lockfile. Both maintained
Storage workspaces must select published Runtime >=0.19.4 before delivery.
That release used the ordinary workflow before the current frozen wiring.

## Platform 0.23.4 publication and Shared 0.30.7 adoption

Shared **0.30.7** at `22a853e26c05ec323f06c57fa316c08b808f252b`
is installed through the canonical exporter; all 141 payloads match that commit.
Platform **0.23.4** is published on crates.io and GitHub at release commit
`31649c18cea503aba279b7f15996d80575075216`, annotated tag `platform-v0.23.4`.
The downloaded non-yanked crate's checksum and clean embedded VCS commit match;
the remote annotated tag object and peeled commit match the local release.
Verification, archive and publication evidence is under
`target/deployment-platform-0234/` (`verified.json`, `publish.log`). At that
publication Runtime **0.19.3**, Types **0.18.2** and browser **0.2.0** were unchanged.

All **20** local release stages passed on Rust 1.99.1, including the complete
native/Wasm feature matrix, standalone consumers and all installed owner/Timers
profiles. The full, fresh qualification is
`target/release-validation/qualification.KkXPfO/`, bound to implementation source
`3ac90c4cd1a0a81230f381d07c074d2df32e450c`; metadata preparation and recovery
re-admitted its receipt before release delivery. Hosted integration/native CI
remains asynchronous; no new macOS or native Profiler qualification is claimed.

The separate Candid/derive 0.10.40 and cc 1.11.1 Cargo.lock draft is restored,
preserving all published owner versions, including Platform 0.23.4. Its complete
original is retained at `refs/retained/platform-0307-deploy-lock-draft`; the
original/restored patches under `target/deployment-platform-0234/` match after
excluding their base-blob index lines. Only that draft remains dirty after the
handoff commit. Never restore an old whole lock over current release identities.

The boundary gate now includes Shared's lexical Rust CBOR check. Exact
file/crate allowances cover Platform's typed codec implementations/integration
tests, Types' serializer tests and the protected imported PocketIC ingress
fixture. No generic codec export or broad native-directory exemption is added.
The installed-runtime driver uses `certification::cbor::decode_certificate`;
the fuzz harness uses typed certificate/tree APIs. Their standalone manifests
no longer select serde_cbor directly. Independent consumer, runtime, benchmark,
stable-identity and native HTTP fixture workspaces now run Shared's Cargo
boundary admission against their own materialized manifests.

Focused evidence is under `target/shared0307/`: snapshot/root boundaries,
consumer/prepared-call admission (including direct CBOR refusal before builds),
owner/Timers locked graphs, incompatible-Timers refusal, three-group release
routing with substitute runners, facade and driver Clippy, 15 codec tests,
malformed CBOR smoke tests, six driver unit tests and packed browser consumers.
The coverage-guided fuzz campaign was not rerun for this adoption.
These earlier focused checks are separate from the full gate above and do not
qualify macOS/native Profiler. See
[development qualification](../development-and-qualification.md#candid-and-cbor-source-admission-and-experiment-retention).
The changed certificate decoder also passed one installed Wire owner profile
(raw awaits, callback cleanup, upgrade, stable rollback and certificate lookup);
`installed-wire.log` and `runtime-artifacts.sha256` retain that separate evidence.

Shared's frozen batch runner is supplied but **not activated**: Platform must
qualify its actual release/publication hooks, recovery and normal-lock edit
isolation before setting `RELEASE_FROZEN_GROUPS`. The current standard release
workflow and empty native-host allowlist remain in force.
The subsequent [frozen readiness check](../platform-release-validation.md#before-the-0309-wiring-frozen-readiness-check-2026-11-06)
passed Shared's 18 focused runner checks, but actual Platform startup refused
missing candidate-local tools before preflight. Platform's outer Make guard also
rejected a batch-only roster. The current 0.30.9 wiring replaces those startup
limitations. These historical checks used disposable sources/local remotes;
no release or upload occurred, and the normal lock draft remained unchanged.

## Previous verified three-group publication

The preceding three-group deployment completed on crates.io and GitHub:

| Group | Published members | Version | Annotated tag | Release commit |
| --- | --- | --- | --- | --- |
| Types | ic-platform-types | 0.18.2 | types-v0.18.2 | 5eeb016469b15de9bdf9f4d2092a8dbff8ea4d21 |
| Runtime | ic-platform-runtime, ic-platform-macros, ic-platform-runtime-guard | 0.19.3 | runtime-v0.19.3 | 731fc9bc47a38f1c2bd66eaff9ec68b4e422efba |
| Platform | ic-platform | 0.23.3 | platform-v0.23.3 | e71e8a63eebc1135f06fc5a8507404a88ddf5c94 |

Every downloaded registry archive is non-yanked, checksum verified and records
its exact clean release commit. GitHub annotated tags were independently checked.
Summary: `target/deployment-20261106/complete.json`; individual API/archive/VCS
proofs and publication logs are alongside it. No sibling release/adoption occurred.
Browser **0.2.0** remains unchanged. These releases used Shared **0.30.5** at
`be3771031ebfb340583b238dee5446be65929c62`.

All twenty local Linux release stages passed separately for each group on Rust
1.99.1, including all-feature lint/tests/docs, consumer/feature graphs and installed
Runtime/Timers profiles. Receipts under `target/release-validation/`:
Types `qualification.4YbUdJ`, Runtime `qualification.1MJU9W`,
Platform `qualification.dZncxE`. Publication ran once per group in dependency
order, with normal hooks, exact-tree checks and atomic commit/tag pushes.
Hosted CI for the new release commits remains asynchronous and is separate from
these local receipts. #50 remains open awaiting upstream, with no fix or waiver.

## Supplied behavior and consumer ownership

#147 is closed: Platform 0.23.3 supplies
`icp_ledger_types::account_identifier(owner, &subaccount) -> [u8; 32]` under
`icp-ledger-types`. The selected Apache-2.0 ic-ledger-types 0.16.0 constructor
body is unchanged and source/license guarded. Canic/boundary/nonzero-subaccount
native vectors, native/Wasm Clippy and altered-body refusal pass. No Agent,
Runtime, CDK, parser, ledger-client package or Candid printer enters that selection.
Its specific Wasm proof is compilation, not installed account-identifier execution.
Canic retains adoption, CRC dependency removal and mint/receipt requalification;
see `docs/migration.md`. No sibling checkout was changed.

Types 0.18.2 and Platform 0.23.3 select allocation-only data-encoding/serde_bytes.
Probe savings are only 0–6 bytes; no material Wasm gain is claimed. The unused
facade serde_test edge is removed; Types/Runtime retain real uses. See the
[utility comparison](../benchmarks/utility-features-2026-11-06.md).

Runtime 0.19.3 supplies stable-order executor tail removal. Native reverse
cancellation of 8,192 tasks improved 7.55 ms to 0.14 ms, costing 58 optimized
module/code bytes; this is not application IC timing. Platform also supplies
CBOR fuzzing, shared Rust/browser ingress vectors and Wasm regression tooling.
See the [audit evidence](../audits/platform-improvements-2026-11-06.md).
Two allocation prototypes were reverted for about 9 KB growth each; Candid
traversal remains unchanged. Final-source fuzzing passed 420,463 executions.

## Exclusions and restored draft

For the earlier 0.19.3 deployment, the maintainer excluded the concurrent
stable64_read_uninit export. That exclusion is superseded by the restoration
authority above. Its complete
commit remains at `refs/retained/runtime-stable-read-uninit-20261106`; forward
revert `3ad71e6` removed it from release source. The published 0.19.3 Runtime archive's
public IC0 module was independently checked to exclude the export.

The exact unrelated Candid/derive 0.10.40 and cc 1.11.1 draft is restored in
Cargo.lock, preserving all five published owner versions. At deployment completion
only that draft remained dirty. Original custody is `refs/retained/platform-20261106-deploy-lock-draft`,
plus original/restored patches under `target/deployment-20261106/`. Never copy
its original whole lock over current release metadata. Earlier Types attempts
refused lock drift, before tags/uploads; failed proofs `qualification.vNb6k8`
and `qualification.MBdND6` remain preserved and were not reused.

## Host qualification and retained evidence

Three-host mapping [run 42184881886](https://github.com/dragginzgame/ic-platform/actions/runs/42184881886)
passed Linux and both macOS hosts at source `244e085`. It is separate from the
new release receipts. The host allowlist remains empty pending evidence admission;
all actual releases above used the full roster. No stage-receipt reuse was enabled.
Native Profiler #140's paired Linux evidence remains in its
[qualification](../audits/native-profiler-140-2026-11-05.md). Native Profiler
macOS/Bazel remain unqualified; root CI is not that evidence.

Approved archives/bundles remain under
`recovery-evidence/retired-checkouts/20261104-issue139/`, with receipts/checksums.
Maintainer review is due **2026-12-04**. Reproducible Runtime 0.18.1 binaries
remain under `target/recovery-evidence/runtime-0181-delivery.LgU8r6/`.
Retention refuses excess/expiry without cleanup or extension; no evidence cleanup
was performed. Cargo clean can reclaim target evidence: retain active release
receipts and stage logs while recovery may be needed. Earlier retained refs and
the interrupted, unpublished 0.23.1 commit/journal remain intact.
