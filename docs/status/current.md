# Current handoff

## Released Platform 0.11 / Wire 0.10; pending tooling-only 0.11.1

The maintainer released Platform 0.11.0 at `b8ecf49` and Wire 0.10.0 at
`c9d4e7b`. Cargo registry metadata confirms Wire 0.10.0 supplies `platform-types`
and selects Platform 0.11 for that feature and endpoints. Platform no longer owns
Wire schema integration. Consumers can now remove Platform `wire-schema`, retain
`principal`, and enable Wire `platform-types` on the matched published pair.
Wire owns transferred Rust/TS byte/malformed tests and endpoint qualification;
see `ic-wire/docs/evidence/platform-types-transfer/published/` at its release.

This follow-up raises only Platform’s test/tooling Wire catalog entry to 0.10.0
and updates the handoff documentation. No published member inherits that entry;
the facade’s all-feature production graph and root lock remain Wire-free. Root
Cargo.lock is unchanged. No feature or API changes require another minor:
pending notes select 0.11.1; workspace package versions remain 0.11.0. No commit
or release is performed. Full Clippy, inventory, pins, boundaries, docs and format
pass for this follow-up. Raw/Wire/external-Candid standalone consumer checks
pass on current Rust and 1.91, native/Wasm
(`target/prepared-call-consumer.RMsFvd`).

Prior removal evidence remains `target/schema-boundary.IDd2ew`: Principal/Account
bytes/Serde stay unchanged; six root lock entries were removed. Browser SDK
Node/npm admission uses its compiler-floor fixture independently of retired
schema tooling. Wire’s own supplier proof covers standalone derives, Rust 1.91,
host/Wasm feature isolation, exact frames and installed endpoint rollback.

Next runtime adoption is Timers/Scheduling: latest published Timers remains
0.32.4. The inspected Scheduling workspace prepares 0.33.0 against Platform 0.9.1,
which still does not align with Platform 0.11’s runtime identity. The combined
Platform runtime fixture cannot refresh until a compatible Timers supplier is
available. Canic’s inspected catalog still selects Platform 0.8.3/Wire 0.7.25;
Blob selects Platform 0.10/Wire 0.8. Existing sibling checkouts are unchanged.
Their schema/runtime adoption and installed qualification remain consumer work.
The separate native Profiler source recipe also retains its qualified old pins.

## Released 0.10.0: prepared calls and Tooling 0.71

Prepared calls now live at `ic_platform::call`, selected by `runtime` (also
included by `cdk`). The separate `ic-call` package and publication entry are
retired. Consumers remove that dependency and change `ic_call::` imports.
Raw transport/executor and response/error identities are unchanged.

Platform 0.10 adopted Host Artifacts/Testkit/IC Build 0.71 and Wire 0.8. The managed
release editor is installed and admitted. Direct GzipError consumers align Host
0.71. Original adoption logs: `target/upstream-adoption.NKKWnS`; native driver
Clippy, six tests and MSRV compilation: `target/upstream-driver.wY2Z0P`.
Those receipts predate this Wire 0.9 update and are not new installed evidence.

## Released 0.9.1: runtime-only singleton packaging

The maintainer released 0.9.1 at `8e21a5e`.
The [#103 fix](https://github.com/dragginzgame/ic-platform/issues/103) moves only
Cargo's inherited executor links guard into the optional, dependency-free
`ic-platform-runtime-guard` package. `executor` selects its exact coordinated
version. The executor/IC0 implementations, licenses and callback sources stay
unchanged. No runtime implementation or public API moves into the helper.

Native default/gzip-only and IC0-only consumers coexist with registry executor
2.0.0; all six tested executor/runtime/CDK combinations, with and without gzip,
refuse mixed suppliers. Cargo resolves all features of workspace roots for its
lock: a native root declaring a Platform-runtime feature still needs compatible
suppliers. The helper is included in coordinated publication, release metadata,
pin-exception updates, the maintained runtime lock and feature/source checks.

Full owner Clippy passed before tests. Real Cargo coexistence/refusal evidence:
`target/runtime-singleton.ah3TQl` also covers two facade patch versions with zero/one/two runtime owners.
Current/MSRV executor native/Wasm consumer:
`target/boundary-consumer.qCbvBR`. All 33 selections of executor/ic0/gzip/runtime/CDK
pass native/Wasm graph admission in `target/feature-graphs/attempt.vdfqz7`.
Feature/source/consumer/release/governance checks passed before release. All four
packages passed the pre-release 0.9.1 dry run in `/tmp/platform-103-package/publish.log`.

The combined installed fixture remains blocked at resolution: Timers 0.32.3
requires Platform 0.8 and therefore the old external executor, conflicting with
the 0.9 runtime. No sibling adoption is made here. The native Profiler profile still
selects gzip-only published Platform 0.8.3; registry adoption remains separate.
Publication and installed evidence for the released runtime remain distinct.

The 2026-10-30 registry check also found Timers 0.32.4 still requires Platform 0.8.

## Candid codec packaging investigation

The user selected investigation of shipping the lean reader without consumer
patches. The original full-package experiment saved 22,867 stripped probe bytes;
that result does not carry over when registry Candid keeps its type identities.
The new independent consumer accepts registry DTOs without conversion, but the
single-codec saving is 7,631 bytes (0.96%). Mixing owned and registry codecs adds
328,182 bytes (40.85%). Cargo retains all five targeted dependencies. The candidate
copies about 3,040 source lines including comments and private helpers.
Concrete configuration and argument-trait identities differ, as do parser error
variants. The maintainer rejected adoption: retain registry production Candid. The prototype
remains evidence only; maintaining about 3k lines for this saving is not accepted.

Repro: `scripts/benchmarks/0.9/candid-packaging/run.sh`; summary `evidence.json` in
that directory; receipts `target/candid-packaging.n6WZul`. Strict owner/candidate/
consumer Clippy, native DTO/Error identity tests, 191,205 differential cases,
three API identity refusals and Rust 1.91 Wasm checks pass. No new installed,
fuzz, production or sibling qualification is claimed. See the
[packaging findings](../migration.md#candid-codec-packaging-investigation).

[Canic PR #717](https://github.com/dragginzgame/canic/pull/717), commit `b173944`,
gates service printing to host/export builds and propagates its explicit feature
through declaration generation. Its isolated source is
`/tmp/canic-candid-printer-20261030`; the normal sibling checkout stays untouched.
Macro/protocol/build-command tests, strict Clippy and formatting passed. The PR
is open; Platform DTO adoption remains a separate prerequisite for removing
Canic's other printer edge.

## Released runtime and management contracts

0.9.0 embeds IC0 1.2.0 and the maintained executor 2.0.0 implementation.
Preserve source/license fingerprints, SlotMap generations, protected wake routing,
wake coalescing, cached wakers, completed-task reclamation and destructor ordering.
The selected Vec task lists save 2,762 stripped Wasm bytes in the Wire fixture;
small methods trade one allocation and some IC instructions for that size saving.
Eight installed raw/ic-call × Wire/Candid × Rust 1.99.1/1.91 cases passed before
release: `target/vec-runtime-20261030/evidence.json`, `target/cdk-runtime.gTkMdr`.

Wasm management exposes 43 names backed by 51 complete 0.11.0 declarations;
native retains all 57 upstream exports. Preserve external schemas, field/variant
coverage and Principal/Nat identities. Removing the registry DTO/printer edge
saved 55,386 stripped bytes (6.6%) in the status probe, not measured Canic output.
Source/schema/current-MSRV/installed evidence: `target/management-adoption/evidence.json`
and `target/cdk-runtime.Sh5GVo`. Neither change qualifies live IC or macOS.

## Native profile and retained boundaries

The source recipe retains IC `ee8086a9773725fd1058832a9400297cf0990112`, Rust
1.99.0, registry Host Process/FS/Artifacts 0.70.3, Wire 0.7.23 and Platform 0.8.3.
Profiler owns recorder/viewer, retention/report policy and PNG/range mechanics;
Platform owns native state/HTTP admission and pricing. Preserve private ICPROF03
peer coordination, saved schemas, existing bounds and one registry Host family.
[#99](https://github.com/dragginzgame/ic-platform/pull/99) and
[#100](https://github.com/dragginzgame/ic-platform/pull/100) retain prior source,
registry, native lint and installed receipts. The viewer integer-projection cut
has separate evidence at `/tmp/platform-json-cut-20261030/qualification.json`:
113 product tests and six Linux installed scenarios passed. No macOS, fresh-host,
full replica Bazel/NNS or live IC acceptance is inferred from those records.

Owned Principal/Account remain Candid-free; required external Candid and legacy
checkpoints retain bytes and identities. Memory/Storage own allocation and leases.
Keep the fixed 16-level Candid admission, #72 DER envelope exception, #74 Host/Origin
admission and protected crypto bodies/encodings/vectors/licenses. #50 remains unresolved. Fresh registry records and RustSec revision `2295466`
still provide no compatible complete correction. The captured workspace audit
finds crypto-bigint 0.5.5; yanked checks were explicitly excluded. Exact inputs
and findings: `target/issue50-20261030/evidence.json`. No ignore, override or Agent fork.
See [the migration contract](../migration.md) for provenance and qualification.
