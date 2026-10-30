# Current handoff

## Pending 0.9.1: runtime-only singleton packaging

Platform 0.9.0 is released at `47dae924`; workspace versions remain 0.9.0.
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
Feature mutations/closures, published identity admission, consumer runners,
source/license guards, release lock/pin transitions, formatting, inventory,
dependency boundary/pins and documentation checks pass. All four packages pass
a coordinated 0.9.1 dry run in `/tmp/platform-103-package/publish.log`; the main
checkout stays at 0.9.0. No upload occurred.

The combined installed fixture remains blocked at resolution: Timers 0.32.3
requires Platform 0.8 and therefore the old external executor, conflicting with
local 0.9.0. No sibling adoption is made here. The native Profiler profile still
selects gzip-only published Platform 0.8.3 until a supplying release is adopted.
Publication and installed evidence for the released runtime remain distinct.

The user authorized any actionable issues; the open Platform issues are #103
and #50. Fresh registry evidence shows Timers 0.32.4 still requires Platform 0.8.
Existing dirty Wire/derive 0.7.25 lock selections are preserved. No main-checkout
commit, release, upload or sibling dependency adoption is performed.

## Candid size work awaiting adoption decisions

The reader experiment is retained under `scripts/benchmarks/0.9/candid-reader`;
production still uses registry Candid. Evidence in `target/candid-reader.XD3MEw`
compares upstream 0.10.39 with framing-reader and fixed-endian variants.
191,205 differential cases agree per variant; native tests, Rust 1.91 Wasm and
installed comparisons pass. The compact stripped probe is 771,338 bytes versus
794,205 baseline, saving 22,867 bytes (2.9%). This is not a production Canic size.
The experiment removes binrw/array-init/bytemuck/byteorder from that probe's
runtime graph and binrw_derive from its build graph; other consumers can retain
them. No Candid fork is adopted and no protected crypto code changes.

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
