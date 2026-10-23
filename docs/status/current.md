# Current handoff

## Pending 0.6.9 Principal metadata

Release validation exposed two stale critical-group expectations in the feature
selection regression after `wire-schema` was added. They now include the native
metadata feature and exclude it from the portable-only group. Full Clippy and
the complete `make check-features` target pass, including 2,097,153 exhaustive
selections, closure/admission regressions and both target graphs. Evidence:
`/tmp/platform-feature-fix-{clippy,check}.log`. No release command was rerun.

[#48](https://github.com/dragginzgame/ic-platform/issues/48) is implemented locally
with optional native-only `wire-schema`. Owned Principal implements Wire 0.6's
metadata trait by delegating to its bounded byte-buffer schema (maximum 29).
The existing serializer, parser, crypto and external Candid identities are
unchanged. The maintainer selected Rust 1.91 as the minimum for all features;
workspace metadata, CI, consumer checks and active documentation now agree.
The modern host metadata dependency uses the `ic-wire-metadata` alias; the
independent runtime fixture retains its qualified Wire 0.4 dependency.

Full Clippy passed before tests. `make msrv` passes all-feature tests and Wasm
compilation on Rust 1.91. A separate consumer derives metadata and generates
strict TypeScript 7.0.2, executed with Node 24.21.0: empty, anonymous and 29-byte
principal request/response frames agree with Rust; 30/31/255-byte values reject.
Evidence: `target/principal-wire-schema.ZzjZOz`, `/tmp/platform48-wire.log`,
`/tmp/platform48-{clippy,msrv,graphs,checks,source}.log`. The routine graph matrix
passes 240 selections per target; focused native/Wasm graphs prove Wire/JSON
absent from plain Principal/executor and from Wasm with metadata selected.
Imported-source guards, inventory, dependency boundaries/pins, doc links,
formatting and ShellCheck pass. Native macOS and installed-canister qualification
were not rerun. No sibling files, commits, publication or release versions changed.
The incoming Agent lock update and migration notes are preserved. Supplying
version is planned as 0.6.9; Canic adoption and issue closure await delivery.

## Owned-endpoint migration

The maintainer reconfirmed Wire for all controlled application endpoints and
clients, including canister-to-canister calls. Required external IC protocols
remain Candid. The migration contract and local instructions now record this
specific exception to the general hard-cut deferral; no feature is removed.
Canic #613 has active uncommitted schema/endpoint/Blob work. Its newer evidence
reports published Wire 0.6 metadata adoption, but complete endpoint registration,
custom scalar schemas, artifact identities and generated-client qualification
remain with that owner. Avoid a competing implementation over its pending batch.
Tooling #225 still needs the concrete producer contract; Platform #38 retirement
follows actual consumer adoption. Existing sibling checkouts remain untouched.

## Released 0.6.8

Work for [#46](https://github.com/dragginzgame/ic-platform/issues/46) and
[#47](https://github.com/dragginzgame/ic-platform/issues/47) is in local release
commit `fd9ae1f` (0.6.8), including the maintainer's Tokio 1.54.1 update.
This source observation does not establish registry publication or new CI results.
No sibling edits, production API changes, crypto edits or dependency cuts.

The advisory adapter audits both the workspace and maintained runtime fixture
locks against an isolated, recorded RustSec checkout. It preserves input copies,
tool version, logs and statuses; both locks are inspected even after a finding.
Preparation failure prevents auditing, and audit never fetches again. A separate
PR/main/manual workflow installs cargo-audit 0.22.2 and exposes evidence in job logs.
No schedule is activated. Maintenance warnings remain visible, not exemptions.
The canonical RustSec preparation helper was added through the exporter at the
existing Shared Tooling 0.26.4 revision, without modifying snapshot payloads.

The existing parser fuzzer now exercises all three Candid decoder policies with
finite test-owned byte, work, skipping, table and header budgets. Consumer tests
exercise recursive types, overlong headers, extra arguments, zero-width vectors,
truncated prefixes and single-byte mutations. A native regression characterizes
the known short-DER panic outside protected source; catching this panic is not a
production Wasm safety mechanism. Consumers still own input admission and policy.

Full `make clippy` passed before tests. Eight focused consumer tests pass on Rust
1.88, including malformed corpus and short-DER characterization. Advisory adapter
regressions pass, including preparation failure and continuing to the second lock.
Both locks have zero vulnerability findings against RustSec revision
`640bf5a9f06e66d8721e8772766531fe77c8630d`; this local database was freshly observed
in the preceding audit, but local preparation itself does not establish freshness.
`paste` and `serde_cbor` unmaintained warnings remain. Evidence is retained in
`/tmp/platform4647-advisories`, with validation logs `/tmp/platform4647-*.log`.
Formatting, inventory, boundary/pin, documentation, snapshot and imported-source
checks pass. Standalone consumer native tests and Wasm compilation pass on development Rust
and MSRV 1.88 for both changed profiles; see `/tmp/platform4647-consumer.log`.
ShellCheck and workflow actionlint pass. The seeded ASan campaign completed
454,665 executions in 61 seconds with no parser failure. Leak detection was
disabled for the rerun because this environment uses ptrace: the first campaign
completed 476,073 executions but failed during LeakSanitizer shutdown. Both logs
and corpus/artifacts remain at `/tmp/platform4647-fuzz*.log` and
`target/parser-fuzz.2TKFdE`; this is native testing, not installed Wasm proof. Native macOS and installed-canister
checks have not been rerun.

## Current contracts and supplying releases

The [migration contract](../migration.md), [IC inventory](../ic-inventory.md)
and [individual export audit](../audits/export-usage-2026-10-21.md) are canonical.
The inventory/export audit shipped in 0.6.7 at `9d9a440`. Observed sibling usage
is dated evidence, not exhaustive reachability or permission to remove exports.
Retain compatibility until Canic is live. All application endpoints use Wire;
Candid is explicit for required external IC interfaces and legacy checkpoint bytes.
Storage, allocation, retry and consumer trust policies stay with their owners.

Manager exports for [#45](https://github.com/dragginzgame/ic-platform/issues/45)
shipped in 0.6.5 (`14a670c`), retaining exact stable-structures 0.7.2 identities.
On 2026-10-22, #45 closed after unchanged manager source/tests passed the full
[native matrix](https://github.com/dragginzgame/ic-platform/actions/runs/39638269525)
on Linux, macOS arm64 and x86_64 at default-branch `9d9a440` (0.6.7).
The [installed runtime run](https://github.com/dragginzgame/ic-platform/actions/runs/39638269695)
also passed; it does not qualify Persistence's allocation or upgrade policies.
Persistence #130 retains its own Memory adoption requirements.

Executor =2.0.0 and ic0 =1.2.0 remain single registry identities. Imported callback,
cleanup and signature bodies/licenses are guarded; cryptographic replacements
remain out of scope. Agent/PocketIC remain optional native clients, not forks.
No manifest-only change establishes smaller Wasm or transitive independence.

Historical release and qualification detail is retained in the prior handoff at
Git revision `9d9a440`. The incoming uncommitted #45 closure evidence is retained
above. No historical build artifacts were removed. Full release CI, publication,
new native macOS qualification and live IC acceptance remain separate steps.
