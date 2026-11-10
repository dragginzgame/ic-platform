# Independent native management DTO qualification

The shared declarations have subsequently also been replaced; see
[shared qualification](management-shared-qualification.md). References below to
retained shared source guards describe this native batch's original boundary.

Subsequent tooling adoption resolves the earlier IC0 admission and root-file
metadata blockers. This report retains its original qualification scope; see
[the current handoff](../status/current.md) for remaining work.


## Scope and authorship

The authorized specification-only implementer replaced the 18 native snapshot,
install, provisional-create and log declarations plus `StoredChunksResult`.
Inputs and exclusions are recorded in the [contract](management-native-contract.md)
and [implementation record](management-native-implementation.md). A separate
context supplied [public tests](management-native-public-tests-implementation.md)
without reading either implementation or the existing oracle tests. The parent
captured old-owner results before releasing the implementation hold. These
records describe the process and its limits; they do not establish legal clearance.

The new client module uses ordinary `native/client/mod.rs` discovery. Root
exports, the native target gate, shared maintained declarations and native codec
mappings are byte-identical to the pre-change files. There are still 62 native
and 44 Wasm public names. No dependency, feature, crypto, transport, storage,
timer implementation or consumer policy changed.

## Focused evidence

Local Linux evidence is retained at `target/management-native.2OLhmx`, with
compiler identity, original native source and lock, before/after production
graphs, boundary hashes and serialization observations. The baseline was the
existing dirty checkout over `a37b15165a4f87a99f2845cb8263ad565f4a4864`, not a
claim to compare two published archives. Unrelated dirty work was preserved.

All Cargo commands used `--locked --offline`, `-p ic-platform`,
`--no-default-features --features management-canister-types`, compiler 1.99.1.

| Selection | Result |
| --- | --- |
| Original native oracle tests | 3 passed, including all 19 native schemas |
| Original new public test target | 5 passed before source replacement |
| Replacement management unit tests | 8 passed, covering native and shared DTOs |
| Replacement `management_consumer` | 6 passed |
| Replacement `management_native_consumer` | 5 passed |
| Native library and both public test targets, Clippy `-D warnings` | Passed |
| Wasm library, Clippy `-D warnings` | Passed |
| Native/Wasm normal/build graphs and Cargo.lock | Byte-identical before/after |
| Root management exports, shared maintained source, native codec | Original SHA-256 hashes match |
| Shared management source/license guard and mutation refusals | Passed |
| Manifest inventory, ordinary/design/history doc links, scoped Rustfmt and shell syntax | Passed |

The new 18-group fixture preserves exact JSON text, CBOR bytes and Candid bytes
across both implementations, including every native variant, optional fields,
integer extrema and Nat above u128. Separate assertions preserve signed zero,
infinities and NaN payload bits for f32/f64. Read-response byte strings remain
distinct from upload-request sequences; fields were not normalized for symmetry.

Permanent fixture: `crates/ic-platform/tests/fixtures/management-native-observations.json`,
23,694 bytes, SHA-256
`9cb6d5cb1ffab8c9b5b720ac66f9812700d417756eefac9c0d6a4f70baa6f57f`.
The parent added only the frozen-fixture equality assertion after baseline
capture. Existing dev-only registry 0.11.0 schema/value tests remain intact.
The explicit test target is included by the existing all-features release test
entrypoint; no new fleet or installed-runtime matrix was added.

Logs: `/tmp/platform-management-native-{before,after}-*.log`. No speed or Wasm
size improvement is claimed for passive native declarations. No new installed
IC, macOS, sibling or full-release qualification was performed.

## Retired and retained source machinery

The native `provenance.json`, `client-provenance.json` and native source checker
are removed with the old implementations. The shared management checker no
longer invokes that checker; its mutation test retains all shared source,
license and lint refusals. The shared Apache license, source fingerprints and
historical selections remain, with an additional change note. Historical
benchmark input manifests still name the old native files as historical inputs.

First-party replacement license text/metadata, installed Toko's Runtime-to-IC0
admission and the unrelated ICP derivation draft remain release blockers.
No commit, push, release or publication was performed.

The current handoff was reduced to 96 lines under the installed handoff policy;
its uncommitted historical narratives are preserved in
[the prior handoff](../history/source-retirement-handoff-2026-11-10.md), including
earlier qualification and recovery locations. No evidence artifacts were deleted.
