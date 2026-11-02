# Maintained PocketIC Rust client

`ic-platform-pocket-ic` maintains the PocketIC 16.1 Rust client as an independent,
native-only package. Native consumers can depend on it directly and import
`ic_platform_pocket_ic::{PocketIc, PocketIcBuilder}`. The optional
`ic_platform::pocket_ic` facade re-exports the same owner and types.

The `pocket-ic` release group has its own
[ledger](../releases/pocket-ic/CHANGELOG.md). Client-only releases leave the facade,
Runtime and Types versions unchanged. Use explicit `RELEASE_GROUP=pocket-ic` for
release, resume and publication. When both client and facade requirements change,
publish the client first. Client and facade 0.18.0 are published. Facade 0.17
keeps its older client selection; facade 0.18 adopts the independent 0.18 owner.
Sibling adoption follows publication and is not performed by this change.

The 0.18 client selects management-canister DTOs =0.11.0, sharing the native
Platform management surface's external identity. Consumers exchanging 0.8 DTOs
must align their dependency. Settings/status expose status visibility and minimum
incoming-call cycles; decoded status requires the new fields. Their Candid schema
changes, so compatibility is qualified against a prepared server rather than
inferred from compilation. Status, settings and snapshot lifecycle pass against
PocketIC 16.1.0. Older server acceptance is not implied.

Method bodies, REST fields, polling, rejections and existing operation behavior
are retained. This includes inherited panic/convenience behavior. Testkit owns
explicit server preparation, pinning, compatibility admission, process lifecycle
and caller retry policy. No simulator or IC execution engine is imported. The
native Profiler retains its independently qualified patched client/server protocol.

The unreleased #134 cut changes network operations and builder execution to
`Result<_, ClientError>` in both clients. The published 0.18 API described above
still has the inherited behavior. Match `ClientError::Canister` for a canister
rejection; transport, body, JSON, server-status and deadline failures are distinct.
The error retains underlying sources and request evidence, including an accepted
computation's state label and operation ID. That evidence does not authorize a
retry. Lost replies never cause an automatic resubmission; only an explicit Busy
response permits repeating the initial request. Accepted operations are polled by
their original identity under the same monotonic request deadline. Polling yields
the async executor. Creation, blobs, metrics, gateway and deletion use the same
fallible exchange implementation.

`PocketIc::default()` is removed because construction performs I/O. Use
`PocketIc::new()?` or `builder.build()?`. `drop_and_take_state(&mut self)` returns
a Result and transfers state only after successful deletion; a failed explicit
deletion is not repeated by Drop. Destructor cleanup is best effort and reports
failures through tracing. Consumers still own cancellation/retry decisions,
provisioning, recovery and release adoption. No sibling has been migrated here.

All dependencies remain native-target scoped, excluding the client from canister
execution graphs. Direct client consumers need neither the facade nor Runtime;
Candid, certification and HTTP remain native client dependencies. This packaging
change makes no production Wasm-size claim. Rust 1.99.1 is the supported compiler.

[Provenance and retained license](../crates/ic-platform-pocket-ic/upstream/NOTICE.md)
record the transfer. The source guard reverses the exact fallible-client patch,
then the management import rename/ordering, before checking unchanged original
fingerprints. Cryptographic algorithms and management DTO bytes are unchanged.

Focused qualification covers client Clippy/tests, source guard rejection,
management type identity, direct/facade identity across facade versions, native
and Wasm graphs, archive consumption and group preparation/recovery. The installed
fixture is [pocket-ic-client/main.rs](../scripts/fixtures/pocket-ic-client/main.rs);
run `bash scripts/ci/check-pocket-ic-client.sh` with `POCKET_IC_BIN` or
`IC_TESTKIT_POCKET_IC_URL` selecting an explicitly prepared server under Testkit custody. Native macOS and
hosted results remain separate from local Linux evidence. Source inclusion alone
does not prove offline simulator rebuilding or complete-stack recovery.
