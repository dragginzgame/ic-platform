# Current handoff — 2026-11-06

## Local Shared 0.30.7 adoption

Shared **0.30.7** at `22a853e26c05ec323f06c57fa316c08b808f252b`
is installed through the canonical exporter; all 141 payloads match that commit.
This adoption is local and unpublished, with no package version changes. The
previously restored Cargo.lock draft below is preserved unchanged.

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
These are Linux-focused checks,
not a full release gate or new macOS/native Profiler qualification. See
[development qualification](../development-and-qualification.md#candid-and-cbor-source-admission-and-experiment-retention).
The changed certificate decoder also passed one installed Wire owner profile
(raw awaits, callback cleanup, upgrade, stable rollback and certificate lookup);
`installed-wire.log` and `runtime-artifacts.sha256` retain that separate evidence.

Shared's frozen batch runner is supplied but **not activated**: Platform must
qualify its actual release/publication hooks, recovery and normal-lock edit
isolation before setting `RELEASE_FROZEN_GROUPS`. The current standard release
workflow and empty native-host allowlist remain in force.

## Verified three-group publication

The authorized deployment is complete on crates.io and GitHub:

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

The maintainer excluded the concurrent stable64_read_uninit export. Its complete
commit remains at `refs/retained/runtime-stable-read-uninit-20261106`; forward
revert `3ad71e6` removed it from release source. The published Runtime archive's
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
