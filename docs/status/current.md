# Current handoff

## Pending 0.7.7: ordinary native API module layout

Source release **0.7.6** is on main at `8871f67`. Its API repair and complete
complexity cleanup remain intact. [PR #88](https://github.com/dragginzgame/ic-platform/pull/88)
now contributes only the ordinary API module directory and matching source links;
the shipped path override is removed because normal module resolution suffices.
No package version, release or publication is performed. Pending notes use
**0.7.7** because public behavior and dependency identities remain compatible.

[#73](https://github.com/dragginzgame/ic-platform/issues/73) is closed as delivered
in 0.7.5. [PR #87](https://github.com/dragginzgame/ic-platform/pull/87) is closed as
already implemented after exact implementation/test comparison with that release.
The fixed nesting source remains unchanged in main; costs and limitations remain
in the migration contract. #50 is set aside until a supplying solution exists;
its advisory check and dependency selection are unchanged.

## Released 0.7.6 cleanup and qualification

External endpoint adapters now pass parsed paths/functions to the existing
generator rather than stringify/reparse them. Single-result and protocol replies
share decoder setup/finalization while retaining their separate arity decisions.
The fixed 16-level Candid guard and protected crypto/callback sources are unchanged.

Native profiler adapters share JSON argument conversion, capture configuration
and image envelopes. Responses carry their encoding class from the producing
adapter; stop settings are read under one scope lock. Configuration key discovery
uses the existing runtime producer instead of parsing Rust macro invocations.
This deliberately narrows the proposed metadata redesign: unit interpretation
stays profiler-owned; no new capture metadata contract or replica patch is added.
The batch also repairs the assembled admission-module path and principal trait
import exposed by compiling the real native sources.

Current-source MSRV runners and CI derive the compiler from the workspace
manifest. Published qualification retains its historical compiler. The runtime
runner shares profile selection across graph admission, artifacts and execution.
Regression coverage exercises the actual Make recipe with a different floor,
failure propagation, exact request/image budgets and equivalent macro emission.

Evidence is retained in `/tmp/platform-simplify-20261029`. Full `make clippy`
passed before longer tests. Focused macro (13) and Candid (21) tests pass on Rust
1.99.1/1.91. Standalone endpoint/external-codec consumers pass native/MSRV/Wasm;
installed Linux/PocketIC passes both compilers with raw and owned calls using
fixture `target/cdk-runtime.OAbzt1`. Runtime graphs, strict Rustdoc, protected
sources, 44 assembler tests, MSRV/Make and Bash 5.2/3.2 runner checks pass.
No performance or Wasm-size saving is measured or claimed for this batch.

## Native provider qualification

Qualification uses isolated provider/consumer copies, upstream IC
`ee8086a9773725fd1058832a9400297cf0990112` and its Rust **1.99.0** toolchain.
The normal sibling checkout remains read-only. The assembled native library
compiles and passes 37 focused adapter/catalog/admission tests. The newly built
Linux server SHA-256 is
`07bd658a47d50bb0dd59176a1012eff09128d0c026abf23c39009379a34922d5`;
all five `system_profiling` scenarios pass against that exact executable.
Preparation/build/test receipts are under the isolated consumer's
`.build/receipts` inside the evidence directory above, binding the dirty provider
source digests and embedded routing input rather than claiming HEAD alone supplied it.

All 69 recorder tests pass with admitted local socket access; native graph
exclusions pass. Native macOS, Bazel/NNS genesis, live provider acceptance and a
consumer's new provider pin remain separate. Root full CI/release gates were not run.
No runtime, crypto, pricing/debit body, capture format or dependency was replaced.

## Retained 0.7.5 Candid and ICRC contracts

The maintainer accepted one fixed rule for #73: bounded decoders reject more than
**16 constructed wire levels** before upstream Candid; exactly 16 are allowed.
No policy object, limit setting or macro option is added. Unused types and skipped
values are checked with the existing combined decode/skip work budget. Candid
still receives its original quotas. Low-level upstream APIs retain their behavior;
staged callers can explicitly use `candid::nesting::check`.

This mitigates wire nesting, not arbitrary trusted Rust schemas/custom visitors.
Do not claim upstream Candid itself is repaired. Original installed proofs include
16/17 levels, the 8,202-byte/8,192-opt trap, malformed input, refused-handler state,
management raw_rand, callbacks, timers and upgrades. Evidence:
`/tmp/platform-fixed-depth-20261029`; fixture `target/cdk-runtime.p3w1eD`.
Historical source/installed locks select Candid 0.10.39/0.10.38 respectively.

The accepted frozen cost study adds about 10–20% decoder instructions for ordinary
structured DTOs, about 1.13M for 100 ICRC-3 blocks, and about 12.4K for 64 KiB/1 MiB
HTTP bodies. Exact fixture cycle deltas, methodology and server identity are in
the [cost report](../migration.md#admission-cost-2026-10-29), with evidence in
`/tmp/platform-candid-cost-20261029`. Further optimization awaits consumer metrics.
Owned endpoints remain Wire; external identities, bytes and persisted formats remain.

Source 0.7.5 also supplies the named external ICRC-10 `SupportedStandard` from
[PR #86](https://github.com/dragginzgame/ic-platform/pull/86). Its independent
schema/type/byte proofs remain in `/tmp/platform-icrc10-20261028`.
[Canic #706](https://github.com/dragginzgame/canic/issues/706) owns optional adapter
adoption on the pending 0.113 base. Source-candidate strict lint, DTO/schema/bytes,
MSRV/Wasm and four installed profiles pass; registry/managed adoption is not
asserted. Mounting, consent, authorization and budgets remain Canic-owned.

## Retained advisory and safety boundaries

[#50](https://github.com/dragginzgame/ic-platform/issues/50) remains the retained
RUSTSEC-2026-0387 blocker through coupled ECDSA/Agent/derivation dependencies.
The maintainer approved setting it aside for integration only. No advisory ignore,
check suppression, incompatible override, Agent fork or in-house crypto is authorized.
#72's narrow DER admission is the sole source-guard exception; crypto bodies,
encoding, constants, vectors and licenses remain protected. Only short-form
framing refusal is qualified; long-form support and oversized encoder conversions
remain outside that change. #74 retains native Origin/Host admission and local
nonbrowser clients; Chromium/assembler evidence is in
`/tmp/platform-security-20261028`. Historical size figures remain fixture-specific.
