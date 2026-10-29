# Current handoff

## Profiler client and backend description boundary

This topic stacks on #97 / #95 and reuses Profiler #9's paired product interface.
Platform now owns textual-canister request admission and complete IC operation
results, plus compiled scheduler/PocketIC descriptions. The consumer retains
Python commands, export-before-clear and offline rendering. Captures retain the
provider facts; unbound captures have explicit missingness, and saved schema-1
dictionaries remain recorded evidence. No own release/version or crypto change.
Qualification and remaining native host scopes are tracked in [PR #99](https://github.com/dragginzgame/ic-platform/pull/99); its paired consumer retains fresh receipts.


## Pending complexity cleanup (#96)

This paired follow-on retains Platform gzip and includes the #95 native inputs
already selected by released Profiler 0.10.0. Integrate #95 before this cleanup.
Shared Tooling's
single-observation supplier is PR #322 at `1e89c0d`, adopted through the canonical
exporter. Root package versions and crypto/runtime identities are unchanged.

Feature activation comes from Cargo metadata, while upstream identity/source
admission remains explicit. Standalone proofs share preparation only; their
manifests, policy assertions and published checks remain independent. Entrypoint
adapters share syntax facts without changing their own diagnostics or codec rules.

The paired Profiler #8 source supplies pure query/cache/tool/render plumbing.
Platform keeps HTTP protection, native start/stop, metrics/pricing adapters and
scope cleanup. Query admission preserves the former lexical/error order; cached
images retain 8 MiB/query, 32 MiB total and 128-entry FIFO budgets and cross-run
checks. Rendering borrows immutable images; existing Value API paths remain.
One native dependency patch replaces overlapping history; the cadence catalog
reads the existing scheduler constant through a compiled getter, with unchanged
charging behavior. Older Profiler consumers must retain their provider pin.

Full owner and independent-recorder Clippy precede the passing focused checks:
444-selection native/Wasm graphs, 14 macro tests on dev/MSRV, standalone CDK and
external-Candid native/MSRV/Wasm consumers, consumer rejection/failure fixtures,
Shared graph fixtures, 45 assembler tests and protected source/inventory checks.

Provider `f5051bf` and Profiler `e9f68b6` build the actual instrumented server.
All 74 recorder, 26 assembled product, three catalog and five installed profiling
tests pass. The frozen server SHA-256 was
`b863005e0272c82e2f94a6a3684bbdeb0652fdb8162b6aa823d1da02b2b290e1`.
All seven dependency inputs match the ordered recipe's final bytes exactly.
Profiler #9 holds the paired source. Integrate #95, this PR and then the paired
consumer with its final merged supplier pin. Shared's supplying PR is #322.

Original temporary receipts and the executable disappeared in an external
scratch cleanup after validation; session observations and paired PR summaries
retain the results. The extra strict native Clippy probe has existing findings
tracked in #98; it did not pass and changes no owner gate. Native macOS server,
full replica Bazel/NNS and release/live-provider acceptance remain separate.
Hosted macOS recorder jobs pass. No size/speed saving is measured. #50 and its
advisory check remain unchanged.

## Host process and private Wire follow-on (#94 / Profiler #5 and #6)

PR #95 now targets main after merged #93, retaining Platform gzip. Native inputs
wire IC Wire 0.7.21 and Host process from supplying Tooling #265, revision
`c20dbd11785c17d369651f51fa0967fbec69c57a`. Platform gzip uses that same Host
source family; the native library scaffold is `ba98bd33ef1cbe463d6daf34f0511f17d7770c20`.
There are no downstream Cargo patches or mixed registry/Git Host packages.
Profiler owns process selection/accounting and the private protocol cut; Host
owns raw OS facts and workload custody. Rebuild all process peers together.
Supplying publication and native macOS acceptance are separate from local Linux
qualification. The existing IC pin and cryptographic graph remain unchanged.

Profiler adopted this batch directly in released 0.10.0 and closed its superseded
PR #7 and issues #5/#6. PR #95 now also repairs feature admission for the exact
manifest-selected Host source, preserving foreign/path, duplicate-source and
Wasm refusals. This CI-only correction changes no native provider input or
previously qualified executable. Consumer CI separately prepares the complete
locked Cargo cache before its offline release fixtures.


## Pending 0.7.7: paired Profiler product source contract

Main `26ad783` integrates #88 and #89. This paired change returns Profiler's
CLI/query renderer to its consumer source tree (#90) and delegates common PNG
encoding and sparse range mechanics to that same product owner (Profiler #4).
The provider keeps native state/trace access, HTTP admission, IC pin, patched
client and build inputs. Its library API and cryptographic source stay unchanged.
Older consumers must retain their prior provider pin until supplying the product
files listed in `native/ic-profiler/README.md`; no duplicate compatibility owner
is retained. Current native pins remain exact full Git identities.

The maintainer explicitly chose to keep the Platform gzip boundary instead of
#91's proposed direct Host migration. Server imports now use that facade, and
exports return the unchanged core gzip bytes (Profiler #1). API input/decoded
limits remain 128 MiB and the core's decoded limit remains 256 MiB. Strict framing
rejects CRC/length corruption, truncation, concatenated members and trailing data.
PNG remains zlib and keeps its separate snapshot budget and layout policies.

Published Metrics 0.17.2, Platform 0.7.6 and Host Artifacts 0.69.12 are selected
in the paired Cargo/Bazel native inputs. Profiler independently adopts those
registry releases, release editor 0.69.12 and Shared Tooling 0.27.7. It still
selects only Platform gzip; Metrics adds no IC runtime edge or charging authority.

Full owner/consumer Clippy precedes the focused checks. The 44 assembler fixtures
pass. PNG fixtures compare both former encoders before extraction; caller tests
cover independent pixels/CRC, exact source budgets, sparse high addresses and
range-admission differences. Final server/recorder/installed-scenario evidence and
its actual executable hash are recorded by the paired Profiler qualification and
delivery PR; historical receipts do not qualify a changed provider or binary.
Native macOS, full replica Bazel and NNS genesis remain separate.

#73 is delivered in 0.7.5; #87 closed as already integrated. #50 remains set aside
until an available supplying correction exists, with its advisory check intact.

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
