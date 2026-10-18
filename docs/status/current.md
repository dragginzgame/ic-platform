# Current handoff

The maintainer released 0.1.8 at `57fcf8d4f55914dedf0a25e70cec2e8affb3fb82`
and committed the raw-call/graph repairs in `b1c64f2` (message `0.1.9`). This task
starts from that clean commit; package metadata remains 0.1.8 and the 0.1.9 notes
are undated. The new Candid-path, DTO and qualification work is uncommitted.
No version change, commit, push, publication or remote creation was performed;
siblings remain read-only.

Maintained qualification now covers native and Wasm on Rust 1.88: all eight
standalone consumer selections pass native tests on 1.99/1.88, and all five
supported Wasm selections pass on both. `make msrv` passes native all-feature
tests and the new all-feature Wasm check. CI explicitly prepares the 1.88 Wasm
target. The injected Wasm-only `File::lock` fixture passes every old check but
fails the added MSRV check; evidence and incoming before-images are under
`target/msrv-wasm/`. This follow-on preserves the pending Candid proposal and
changes no Rust API, production dependencies or wire policy. Native macOS CI
qualification of the new check remains outstanding. See
[Wasm MSRV enforcement](https://github.com/dragginzgame/ic-platform/issues/15).

The management feature additionally exports upstream `DeleteCanisterSnapshotArgs`
for Canic's existing snapshot-delete request. Read-only dirty Canic source at
base `5583496b` is bound under `target/delete-snapshot/`. Its original DTO and
upstream encoder agree on empty/nonempty ID bytes and cross-decoding; the
Platform-only consumer passes native/MSRV/Wasm with wire/refusal checks, and
Rust 1.88 native/Wasm identity checks preserve the exact 0.11.0 type. The released
0.1.8 import reproduces E0432. No call, storage, authority, wait or retry policy
changed; Canic migration and native macOS qualification of this addition await
delivery. See [the management contract](../migration.md#management-canister-dtos).

The Candid contract now directly exposes upstream `DecoderConfig` and
`decode_args_with_config` for generated-code paths, preserving existing policy.
`patches/cdk-rs-candid-path.patch` proposes the canonical generator correction
against upstream `070233b9`: parsed entrypoint Candid paths plus matching
`export_candid!` resolution. It is not selected by any production build and has
not been submitted upstream. Published CDK 0.20.3 still needs direct Candid.
See [the correction and adoption contract](../migration.md#cdk-candid-path-correction).

Under `target/candid-path/`, focused macro tests pass on Rust 1.99/1.88. A
standalone Platform/CDK consumer with no direct Candid passes native/MSRV tests
and actual Rust 1.88 release Wasm compilation. Its DID matches the original-CDK
comparison byte for byte. Linux/PocketIC 16.1.0 preserves typed/tuple/unit bytes,
composite/manual/custom paths, guards, awaited calls and callback-trap cleanup,
malformed/quota traps and subsequent success, and init/post-upgrade arguments.
Testkit owns the simulator; the refused output-file reuse and fresh attempt are
retained. A separate Platform-only Candid consumer passes native MSRV tests.
It also compiles for Wasm at 1.88; the patched default CDK export path preserves
the original DID. Clippy, strict Rustdoc, formatting, snapshot/pin/boundary and
documentation checks pass. The patch applies to exact main preimages and the
registry 0.20.3 sources; it reproduces the tested candidate files.
This qualifies the candidate path correction, not full upstream CI, native macOS,
combined Timers, live IC, publication or a Platform CDK frontend. No production
dependencies, lock selection, executor implementation or wire policy changed.

The existing independent `ic0` feature now directly exports IC 1.2.0
`in_replicated_execution() -> u32`, `is_controller(&[u8]) -> u32`,
`msg_reply_data_append(&[u8])` and `msg_reject(&[u8])`. These cover IcyDB's current
metrics/entropy and codec-query requirements and IC Wire's raw controller check,
under [the runtime contract](https://github.com/dragginzgame/ic-platform/issues/1).
No dependency/feature selection, executor, byte/authorization policy, storage or
allocation implementation changes. Principal conversion, raw argument handling,
append/finalization order and consumer error/retry policy stay with callers.

Read-only review used dirty IcyDB/IC Wire working trees at bases `c93a7440` and
`ce56d5eb`; `target/raw-boundary-before/consumers.sha256` binds reviewed inputs.
The standalone Platform-only raw consumer passes native Rust 1.99/1.88 and Wasm
1.88 compilation without running system calls on native hosts. Root Cargo.lock
and reviewed sibling input hashes are unchanged.
Strict Clippy/Rustdoc, all 256 native/Wasm feature combinations plus defaults,
the actual all-feature Wasm build, formatting, pin/direct-boundary/documentation
and unchanged-snapshot checks pass; logs are `target/raw-boundary-*.log`.

The follow-on feature-graph repair rejects changes to the reviewed feature
inventory, CDK frontends even with executor selected, and additional executor/
system versions or alternate sources. Six adversarial metadata/graph replays
passed the old checker and fail the repaired checker; unchanged replay passes.
All current Cargo-produced native/Wasm graphs pass, as do native default and
Wasm executor compilation, script syntax and snapshot/boundary/pin/doc checks.
These simulations test admission, not runtime behavior. Before-images and
focused evidence are under `target/graph-audit-before/` and `target/graph-audit/`;
the contract and delivery are tracked in
[graph admission](https://github.com/dragginzgame/ic-platform/issues/14).
This follow-on changes no Rust API, dependency selection or wire policy.

A disposable copy of IcyDB's actual 64-byte codec-query adapter changes only
raw-call imports and manifest declarations. Its synchronous test canister uses
Platform and that adapter, with no Candid, CDK frontend, parser, Agent or executor
in the production Wasm graph and exactly ic0 1.2.0. On Linux/PocketIC 16.1.0 it
preserves query/update replicated flags, controller/non-controller identities,
ordered append/finalize replies, byte-exact rejects including NUL/UTF-8, lengths
0/1/63/64, refusal of 65-byte input and oversized/error replies, a double-reply
trap and subsequent successful query. The fixture is raw ABI qualification,
not a supported Platform entrypoint frontend or a new executor.

Testkit 0.53 owns server admission/startup/cleanup. Its existing reviewed bundle
was checked offline without changing it. Initial fixture failures for parent
traversal and duplicate query/update names are retained; corrected fixtures pass.
Sandbox loopback refusal required a permitted local run. Source, lock, Wasm,
graph, server output and passing runtime evidence are under `target/raw-runtime/`;
standalone proof is under `target/raw-standalone/`. Native macOS and live-IC or
full database/codec/browser acceptance are not established by these Linux tests.

Published 0.1.8 is independently verified against its current non-yanked registry
record and archive SHA-256
`4a3adb6d9785133587b854b66b37dfdd8f7945555bdc59b6d0be26995187484d`;
archive VCS identifies `57fcf8d`. All eight separate consumer workspaces pass
against registry-only `=0.1.8` on Rust 1.99/1.88, with applicable Wasm checks.
They retain external DTO bytes/signatures, configured/default/staged decoding,
native dynamic fixtures, Agent identities/responses and all 24 Backup snapshot/
status types. Evidence is under `target/registry-proof/`; this does not publish
or qualify the 0.1.9 draft.

The [0.1.8 native CI run](https://github.com/dragginzgame/ic-platform/actions/runs/39118523505)
completed successfully on Linux x86_64, macOS 15 Intel and Apple Silicon at
`57fcf8d`. The delivered producer qualification is complete; older cancelled
0.1.6/0.1.7 runs remain historical evidence. This result does not qualify the
current uncommitted DTO, Candid-path or MSRV-gate changes, which have no hosted
CI result. Full local CI was not run.

Remaining runtime work needs a supported generated-path solution plus exact CDK
call/entrypoint/protected-spawn integration and combined Timers callback/trap/
upgrade qualification. CDK macros 0.20.3 accept a runtime `crate` override but
hardcode typed Candid paths; they have no Candid path override. The documented
zero-argument/manual/hidden subset cannot replace consumers' typed service
contracts, and raw `spawn_protected` is not CDK `futures::spawn`. Do not introduce
aliases, blanket re-exports, duplicate runtime policy or an alternative executor.
Storage/allocation APIs remain outside this work.

Fresh public registry inspection still ends at Testkit 0.53.0. Its distinct
limits/error identities remain [the producer blocker](https://github.com/dragginzgame/ic-platform/issues/2).
The pending 0.54 source proof is not publication. Consumers must retain the narrow
native old-decoder exception until a published Testkit and their real drivers
qualify Platform's identities; no conversion/re-export package is appropriate.

Shared Tooling stays pinned to 0.24.0 at `cab12cd`; snapshot payloads are unchanged.
See [the migration contract](../migration.md) for supported imports, manifests,
generators, limits, feature/executor ownership and previous source provenance.
Direct dependency isolation is distinct from transitive removal or measured
Wasm-size reduction. Deployment consumers still own admission of their final
feature/target/lock selection and binding deployed bytes to source/artifacts.
