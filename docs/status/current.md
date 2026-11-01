# Current handoff

## Pending Platform 0.17 client ownership — 2026-11-01

The maintainer requested merging Platform #126 and #128 and implementing the
separate Principal self-authentication packaging change. No new release or
publication is authorized. Preserve unrelated work and sibling checkouts.

[#126](https://github.com/dragginzgame/ic-platform/pull/126) is merged. Native
Profiler selects Host Process/FS/Artifacts 0.73.2 and Wire 0.13.1; its source
payload remains identical to the already qualified immutable provider
07039d91df10faa7c1a5cdfcf30940db30443354. Metrics remains 0.24.0.
The release adapter now enters temporary workspaces physically. Aliased-TMPDIR
qualification passes both groups' rollback, retry, exact custody, local delivery
and resume; prior failing evidence is retained. Native-provider Clippy and four
HTTP tests pass. Logs: /tmp/platform-pr-merges-20261101/pr126-*.log.

[#128](https://github.com/dragginzgame/ic-platform/pull/128) maintains the released
PocketIC 16.1.0 Rust client in native-only `ic-platform-pocket-ic`, released with
Platform. Its initial source version matches Platform 0.16.0; the breaking client
identity change is pending 0.17.0. No version has been published for that new
package. External HTTP/JSON/Candid bytes and original operation behavior remain;
registry client types must align on the facade. See the
[client contract](../pocket-ic-client.md) and retained Apache/source fingerprints.
The simulator, Testkit provisioning/policy and native Profiler stay separate.

Current-main integration passes full Clippy, the four client tests, source/license
and mutation guards, graph/closure refusals, inventory, pins, boundary and runtime
lock custody. Evidence: /tmp/platform-pr-merges-20261101/pr128-*.log. Prior unchanged
client evidence at /tmp/ic-platform-pocketic-client.JQG7s5 includes current/MSRV
client/archive tests and isolated Tooling Clippy, 28 startup/12 transport tests,
four installed cycle/reply and six lifecycle scenarios on PocketIC 16.1.0.
This is Linux qualification, not macOS or production consumer adoption.

Shared 4c84efa9dd5f4b3ff7bdea89972e747fd394d406 (0.27.24) is adopted through its
exporter. Snapshot payloads remain immutable. Shared #332 is merged and supplies the native client
boundary exception for Platform #128. Its snapshot
fixture closure and bounded startup synchronization repairs pass focused tests.
The supplier merge is authorized and complete; no supplier release is implied.

## Published state and retained recovery evidence

Platform, macros and runtime guard 0.16.0 are published; Types remains 0.15.1 and
browser remains 0.1.1. Release commit b1303ffa38daf2ec1551cbce6e39197ad76681ff and
[`platform-v0.16.0`](https://github.com/dragginzgame/ic-platform/releases/tag/platform-v0.16.0)
were atomically pushed. Registry archives match upload archives, SHA-256 and VCS
identity. Do not repeat publication. Evidence: /tmp/platform-016-registry/,
/tmp/platform-016-publish.log, target/publication/publish.selection.r28kzG/ and
.git/release-state/platform-0.16.0.plan. The full release gate passed; its initial
stale Types-lock fixture failure and successful rerun are retained in
/tmp/platform-016-release.log and /tmp/platform-016-release-fixed.log.

0.16 updates Ed25519 to Dalek 3/curve25519 5/signature 3, Syn 3 and the development
hash reference to 4.0.1. Native Agent retains its separate older generation.
The exact upstream BLS identity-key refusal and vector in #127 shipped in 0.16;
Agent's registry BLS 0.6 does not inherit it. Crypto bodies, constants, original
vectors and licenses remain guarded. No locally designed primitives are allowed.
Current/MSRV and installed Linux/PocketIC proofs passed; detailed evidence is
linked from the [0.16 handoff](https://github.com/dragginzgame/ic-platform/blob/1be6a1728ae8a488db91e337fd701aa8c3a22c18/docs/status/current.md).

Certification/tree types and private BLS12-381 arithmetic are maintained locally
with unchanged protected bodies. Signature keys/range decoding use canonical
owned Principal with unchanged DER/CBOR, ordering and bounds. Native Agent retains
explicit external certification conversions. The repository's 32-bit Wasm
RustCrypto cfg must be carried by consuming workspaces; dependency Cargo config
is not inherited. Candid remains the required external IC boundary, with fixed
16-level admission; stable-storage ownership stays with Persistence/Memory.

One workspace/lockfile has independent Platform and Types release groups. Every
release/publication requires explicit RELEASE_GROUP. Keep exact runtime guard
identity, transactional runtime-lock custody, old journals and failed/successful
artifacts. New Platform minor runtime adoption still requires compatible Timers
and Testkit composition. The earlier installed eight-profile runtime proof
(/tmp/platform-016-installed-runtime.log, target/cdk-runtime.g6Ksnu) precedes the
0.16 version transaction; do not relabel it as fresh composition evidence.

The two-codec experiment was rejected as a maintenance/size tradeoff. Do not
resume that adoption or infer a Candid fork from the client ownership work.
GitHub remains the follow-up tracker; this handoff records contracts and evidence.
