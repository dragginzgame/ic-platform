# Current handoff

## Independent Runtime (#131) — 2026-11-01

The maintainer requested publication of the completed #131 and retained
Principal/#116 batch. Delivery is authorized for the existing Rust groups:
Types 0.16.0, Runtime 0.17.1, then Platform 0.17.0. Runtime starts from its
unpublished 0.17.0 source selection and uses the normal patch workflow. Preserve
failed attempts and reconcile registry state before retrying any upload. The [runtime contract](../migration.md#independent-runtime-owner-131)
is the current ownership decision; historical facade-owned descriptions are superseded.

`ic-platform-runtime` now owns the actual IC0, sole executor, CDK adapters and
prepared calls. Facade paths delegate to it. Runtime, macros and exact guard form
an independent Runtime group at explicit unpublished source version 0.17.0;
Platform/facade and native client remain 0.16.0, Types remains 0.15.1. Empty defaults,
Candid-free raw/runtime profiles, protected bodies and licenses remain unchanged.
Macros use one generator with selected Runtime/facade defaults and explicit paths.
The exact guard rejects incompatible Runtime and legacy embedded Platform owners.

Full Clippy, current/MSRV Runtime tests, macro tests/native symbol linking, source
and mutation guards, feature graphs, prepared-call consumers, inventory, pins,
boundary and formatting pass. Both installed source suites pass eight profiles:
current Rust 1.99.1 and MSRV 1.91, Wire/external Candid, raw/prepared calls.
The owner suite excludes Timers; composition retains dispatch, traps, cancellation,
rearm and upgrade. All 16 Wasm export sets exactly match the #116 baseline.
Linux evidence: /tmp/platform-131/installed-owner.log (target/cdk-runtime.8oJ31B),
installed-timers.log (target/cdk-runtime.y5rWye), wasm-exports.log, singleton-final.log,
features.log, feature-mutations-final.log, prepared-consumer.log, runtime-msrv.log.

Extracted normalized `.crate` archives qualify Runtime + Timers and Wire Endpoint
on MSRV Wasm, native consumers and installed PocketIC tests. They use explicit
qualification-root patches for unpublished suppliers; no path patches ship in
normalized manifests. Evidence and archive hashes: /tmp/platform-131/archive/.
A disposable crypto facade 99.0.0 shares task/call types with the current facade;
Timers/Endpoint still compile native/Wasm with Runtime 0.17 and no facade in their
normal graph. Evidence: /tmp/platform-131/crypto-break/ and singleton-final.log.
This is Linux/source/archive evidence, not registry publication or macOS acceptance.

The user explicitly approved isolated topic pushes/draft PRs after automatic
review initially rejected the pushes. [Scheduling #86](https://github.com/dragginzgame/ic-scheduling/pull/86)
and [Wire #41](https://github.com/dragginzgame/ic-wire/pull/41) select Runtime directly.
Their sibling working checkouts remain untouched. Source qualification passes
Timers' 175 tests, probe Clippy and MSRV, and Endpoint's schema/Clippy/MSRV/installed
transaction test. Drafts retain original registry locks pending supplier publication;
local qualification locks are not disguised as registry evidence. Scheduling main
advanced during work; integrate its latest changes before merging its draft.

Platform's test-only Timers catalog pins adoption commit
036fd50c08c2c1d05ab1cb2381abc7b8f1a36103. Explicit preparation fetches it; normal
checks use maintained locks/offline caches. For the pending Principal cut, publish Types first; then Runtime, compatible
Timers/Endpoint/facade releases and application adoption. Return the test catalog
to a qualified registry release afterward. No production Git dependency is added.

Release routing and lock custody cover all three groups, with both qualification
locks shared and the exact pin private to Runtime. Wrong-base, missing/duplicate
identity and idempotent retry tests pass. Platform transaction/recovery passes in /tmp/platform-group-preparation.e0oNtk;
Runtime and Types pass in /tmp/platform-131/group-recovery-runtime.log and
group-recovery-types.log (fixtures Mg8uF0 and 6aMYwE). The Runtime resume wrapper
now admits the third group. Preserve the initial formatting failure and the
subsequently fixed Runtime resume refusal in group-preparation.log and
group-preparation-final.log. Full CI remains release-owned.

## Retained Principal and #116 work

Pending Platform 0.17 / Types 0.16 removes `principal-external` from
`principal-self-authenticating`. Select registry conversions explicitly. The exact
ic_principal 0.1.5 derivation/tag and upstream SHA-224 remain, with source/license/
vector guards. Native/Wasm current/MSRV consumers and isolated Auth 0.29 pass;
its base graph drops registry ic_principal. Evidence:
/tmp/platform-pr-merges-20261101/principal-*.log and auth-principal.*.
No cryptographic algorithm or Wasm-size claim is implied.
The [audit baseline](../reports/audits/2026/11/01/dfinity-runtime-maintenance/01/report.md)
records earlier dependency selections; it is not post-change evidence.

#116 separated owner and Timers installed gates and advisory outcomes, retaining
both release gates and prepared-composition admission. Its final pre-extraction
baseline is /tmp/platform-116-20261101/installed-final.log; target/cdk-runtime.G46omX
(owner), target/cdk-runtime.4xxyuU (composition). Preserve source/lock/artifact
receipts and failed candidates; incompatible schedulers never waive admission.

## Merged and published state; recovery evidence

Platform #126/#128 and Shared #332 are merged. Shared snapshot
4c84efa9dd5f4b3ff7bdea89972e747fd394d406 (0.27.24) remains immutable. #128 baseline
is 143fa8a9df54a6bd763aa9a6d7df8c97e6f9e4cf. The owned native PocketIC 16.1 client
is still unpublished with breaking adoption pending Platform 0.17; Testkit owns
server lifecycle/policy. See [the client contract](../pocket-ic-client.md).
Native Profiler retains provider 07039d91df10faa7c1a5cdfcf30940db30443354 with Host
Process/FS/Artifacts 0.73.2, Wire 0.13.1 and Metrics 0.24.0. Earlier consumer/client
qualification: /tmp/platform-pr-merges-20261101/pr126-*.log and pr128-*.log;
/tmp/ic-platform-pocketic-client.JQG7s5. Do not relabel it as fresh runtime evidence.

Platform/macros/guard 0.16.0 are published; Types remains 0.15.1 and browser 0.1.1.
Release commit b1303ffa38daf2ec1551cbce6e39197ad76681ff and
[`platform-v0.16.0`](https://github.com/dragginzgame/ic-platform/releases/tag/platform-v0.16.0)
were atomically pushed. Registry archives match upload SHA-256/VCS identity.
Do not repeat publication. Preserve /tmp/platform-016-registry/,
/tmp/platform-016-publish.log, target/publication/publish.selection.r28kzG/ and
.git/release-state/platform-0.16.0.plan, plus the failed/successful release logs
/tmp/platform-016-release.log and /tmp/platform-016-release-fixed.log.
Old journals retain their original group-policy identity; do not reinterpret them.

Crypto bodies/constants/vectors remain guarded. Agent retains its external
identities and older crypto generation. Consumers must carry the reviewed 32-bit
Wasm RustCrypto cfg; dependency Cargo configs are not inherited. Candid remains the
required external IC boundary with fixed 16-level admission; storage ownership
stays with Persistence/Memory. The rejected two-codec experiment remains rejected.
GitHub owns follow-up work. The active publication authorization is recorded above.
