# Current handoff

The maintainer released 0.1.8 at `57fcf8d4f55914dedf0a25e70cec2e8affb3fb82`.
Package metadata remains 0.1.8; the compatible 0.1.9 draft records the new raw
system exports. No commit, version change, push, publication or remote creation
was performed by this task. Sibling checkouts remain read-only.

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
or qualify the uncommitted 0.1.9 additions.

The latest [0.1.8 native CI run](https://github.com/dragginzgame/ic-platform/actions/runs/39118523505)
had Linux/Apple Silicon success and Intel macOS still validating when inspected.
The incomplete required matrix leaves delivered producer acceptance open; older
cancelled 0.1.6/0.1.7 runs are historical evidence, not active gates to rerun.
Current dirty changes have no hosted CI result. Full local CI was not run.

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
