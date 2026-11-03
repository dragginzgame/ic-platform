# Current handoff

## Delivered batch — 2026-11-03

The maintainer authorized all pending changes and their publication. All six Rust
crates are released, pushed and published on crates.io:

| Group | Version | Release commit | Tag |
| --- | --- | --- | --- |
| Types | 0.18.0 | 0ea35e7 | types-v0.18.0 |
| PocketIC | 0.19.0 | c999f1d | pocket-ic-v0.19.0 |
| Runtime, macros, guard | 0.19.0 | 4c011da | runtime-v0.19.0 |
| Platform | 0.19.0 | c93fef4 | platform-v0.19.0 |

Every group passed its standard full release gate and installed Runtime/Timers
composition checks. Every registry archive matches its index checksum and exact
release commit, with clean VCS metadata. The registry-only consumer at
target/full-batch-registry-consumer.pYSBIa passed native and Wasm Clippy, proving
canonical Principal/Account/call/client identities without path patches. Its six
Platform-family packages have the released versions; the selected Wasm graph
excludes PocketIC, Agent, Candid and an external executor. This is Linux/PocketIC
and Wasm evidence, not fresh macOS or live IC acceptance.

Release/publication logs, registry readbacks and final consumer evidence are in
.git/release-state/full-batch-20261102.NPXE3e/. Completed per-group plans remain in
.git/release-state/. Do not repeat successful uploads or increments.
All six crates require Rust 1.99.1. Shared 0.27.27 at
651f5f7acb2e3055abd51ba5f835a2112801666a remains the installed executable snapshot;
revision-bound policy does not refresh it. Browser/npm publication is unchanged.

## Released contract changes

[#138](https://github.com/dragginzgame/ic-platform/issues/138): Account present
subaccounts now use Serde bytes. Postcard Some gains a length prefix; CBOR Some
uses bytes. JSON, raw None/zero fields and logical equality remain qualified.
Old frames can decode to different valid values: consumers require matching
Wire/browser schemas, fresh RPC contract IDs and separate persisted-data handling.
See [the migration contract](../migration.md#account-byte-adapter-cut-138).
Runtime 0.19 retains the canonical Types identity; its implementation is unchanged.

[#134](https://github.com/dragginzgame/ic-platform/issues/134): PocketIC's sync and
async operations are fallible, with bounded HTTP exchange and monotonic polling.
Accepted computation/ingress IDs survive failures; only explicit Busy permits
initial resubmission. Failed explicit deletion retains state and Drop does not
replay it. I/O-performing Default and the unused stringifying response helper
are removed. Exact reverse-patch and mutation checks preserve source fingerprints.
See [the client contract](../pocket-ic-client.md). Strum/macros 0.28 and Base64
0.23.1 are selected. Installed 16.1 status/settings/snapshot lifecycle passed.

## Agent decision and remaining work

The maintainer withdrew the unreleased owned Agent and cryptographic-provider
migration. The native facade selects upstream ic-agent (compatible 0.49.2
requirement, locked 0.49.3), with PEM and without upstream default features.
External certificate identities and explicit conversions remain. Agent is absent
from Wasm; no production canister size saving is claimed. The facade's unused
direct ic_principal/thiserror dependencies are removed; Types owns Principal.

The pre-revert dirty tree and withdrawn source remain recoverable under
/tmp/platform-agent-revert/. This is undo evidence, not a supported implementation.
Five maintained Agent consumer tests and all four installed identity profiles
passed, including wrong-root refusal. The installed gateway fixture owns its PEM
fixtures/license under scripts/fixtures/agent-installed.

[#50](https://github.com/dragginzgame/ic-platform/issues/50) remains open for native
Agent and ic-secp256k1 derivation's crypto-bigint 0.5.5 paths. Native tools can sign
real requests; using upstream is not a test-only advisory exemption. No crypto
rewrite, waiver or incompatible patch was adopted. Standalone ECDSA keeps its
existing patched provider. [#134](https://github.com/dragginzgame/ic-platform/issues/134)
is closed against published 0.19.0; its stale unreleased documentation is corrected.

[#136](https://github.com/dragginzgame/ic-platform/issues/136) is implemented in
the reviewed delivery batch, authorized for release and publication. Snapshot
metadata/data methods and caller-owned incremental transfer preserve partial IDs,
accepted ingress, timer/hook state and upstream DTO identities. HTTP/Candid/chunk
and total byte bounds are explicit; interrupted steps refuse implicit replay.
Pending client 0.20 notes cover the new exhaustive ClientError variant; no package
version or publication changed. Client and facade Clippy, 20 client unit tests,
management identity, source mutation guards and installed 16.1 restoration pass.
Evidence: /tmp/platform-136-*.log and target/pocket-ic-client.DDyVwL.
No fresh macOS result or sibling adoption is claimed.
[#137](https://github.com/dragginzgame/ic-platform/issues/137) now has a local
[validation pilot](../platform-release-validation.md): exhaustive roster, source
classification, Shared runner receipts and failure/recovery fixtures pass. Actual
same-source Linux full (20 stages), PocketIC/Runtime selected (18 each) and Types
selected (19) mappings all passed on isolated source 21e9180 with Rust 1.99.1,
including installed Runtime/Timers proofs. Timings and exact evidence are in the
pilot document. Release-recovery integration and new native-host acceptance remain;
the active full release gate is unchanged. No controlled speedup is claimed.
The unrelated working-tree Cargo.lock Pest update is preserved, not adopted by this task.

## Scheduling and recovery custody

The explicitly authorized isolated branch adopt/runtime-019-platform-qualification
is pushed at a61f35174f512b17d1c7974f4d0be79ef1e2777b. It admits Runtime 0.19
without timer-body changes. Native/Wasm Clippy and 175 Timers tests passed against
the isolated candidate; Platform's installed composition gates subsequently passed.
Composition selects this immutable Timers 0.37.0 source. No Scheduling merge,
release or sibling checkout edit was performed. Return to a qualified registry
source after Scheduling's owner publishes. Evidence and isolated checkout remain
under /tmp/platform-runtime019-scheduling.EBatnW/.

Preserve prior recovery evidence at .git/release-state/runtime-0181-delivery.LgU8r6/,
.git/release-state/platform-018-final-delivery.9UcrQT/ and
.git/release-state/platform-018-deployment/. A prior custom CARGO_TARGET_DIR failed
an admission fixture; standard release gates passed without that override.
#133 and #138 are closed against delivered compiler/Account supplier evidence.
Hosted CI run 41498018089 passed actual native-provider source checks and full
Linux/macOS Intel/macOS ARM64 jobs. #132 is closed after fresh isolated native
qualification on Linux with Rust 1.99.1: server Clippy/build, 83 recorder tests,
114 server tests and all six installed profiling tests passed. Provider 21e9180
has identical native/cache inputs to delivered d9af0f6; the isolated consumer is
50d9548. The retained server's SHA-256 is
d572e50a3f06ff6e0404a0ce6857f920e87cf8cf3d5c17370f67c0dd5c1f6348.
Receipts and artifacts remain under target/issues-132-137.sYRgR0/profiler/.build;
the nonincremental managed compiler cache is separate under target/, measured
at 17 GiB after qualification against its 256 GiB / seven idle days policy.
This is current-provider qualification, not consumer pin adoption or a new
macOS installed-server result. Sibling checkouts and existing caches are untouched.
Sibling stable-structures removals remain maintainer-owned; the mistaken Tooling
deployment request remains withdrawn. Preserve canonical Types, Runtime singleton
and guarded crypto bodies. Earlier evidence is retained in the
[pre-delivery handoff](https://github.com/dragginzgame/ic-platform/blob/b03dbcb/docs/status/current.md).
