# Maintained PocketIC Rust client

`ic-platform-pocket-ic` maintains the PocketIC 16.1 Rust client as an independent,
native-only package. Native consumers can depend on it directly and import
`ic_platform_pocket_ic::{PocketIc, PocketIcBuilder}`. The optional
`ic_platform::pocket_ic` facade re-exports the same owner and types.

The `pocket-ic` release group has its own
[ledger](../releases/pocket-ic/CHANGELOG.md). Client-only releases leave the facade,
Runtime and Types versions unchanged. Use explicit `RELEASE_GROUP=pocket-ic` for
release, resume and publication. When both client and facade requirements change,
publish the client first. Client and facade 0.19.0 are published, including the fallible client cut.
Sibling adoption follows publication and is not performed by this change.

The 0.18 client selects management-canister DTOs =0.11.0, sharing the native
Platform management surface's external identity. Consumers exchanging 0.8 DTOs
must align their dependency. Settings/status expose status visibility and minimum
incoming-call cycles; decoded status requires the new fields. Their Candid schema
changes, so compatibility is qualified against a prepared server rather than
inferred from compilation. Status, settings and snapshot lifecycle pass against
PocketIC 16.1.0. Older server acceptance is not implied.

REST fields, management payloads and server compatibility are retained.
The maintained 0.19 error contract supersedes inherited network panics. Testkit owns
explicit server preparation, pinning, compatibility admission, process lifecycle
and caller retry policy. No simulator or IC execution engine is imported. The
native Profiler retains its independently qualified patched client/server protocol.

The published 0.19 #134 cut changes network operations and builder execution to
`Result<_, ClientError>` in both clients. Match `ClientError::Canister` for a canister
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
record the transfer. The source guard reverses the exact snapshot and fallible-client patches,
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

## Bounded snapshot transfer (#136, pending 0.20)

Both clients expose explicit metadata/data read/upload methods using the existing
management 0.11 DTO identities under `snapshot`. `CallLimits` selects HTTP body,
Candid byte and decoder work bounds. Request time budgets remain selected on the
client and apply to each submit/await exchange. HTTP bodies are bounded while
streaming; `ClientError::ResponseTooLarge` preserves request context. This added
closed-enum variant requires exhaustive matches to change, hence the minor cut.
Snapshot calls wrap typed client errors in `SnapshotError::Call`, retaining accepted
ingress even for a malformed Candid reply. No management wire schema changes.

Read source metadata, then construct a caller-owned `SnapshotTransfer` with explicit
canister, sender, source ID and `TransferLimits`. Metadata must contain timer and
low-memory-hook state. Sparse globals are projected to the management upload's
exported mutable globals in original order. The first `advance`/`advance_async`
creates an upload with no replacement and returns before copying any data. Record
`uploaded_snapshot()` in the caller's pending set immediately. Subsequent advances
copy at most one chunk (1 MiB maximum), checking exact region length, chunk-store
SHA-256 hashes and the caller's total byte budget. Module, heap, stable memory,
globals, certified data, timer and hook are preserved; creation timestamp/source
and canister version retain management upload semantics.

Callers can cancel between advances or drop an async advance. A dropped/failed
advance leaves the attempted status and any accepted ingress in caller-owned
progress; it refuses further automatic advances. Metadata submission may have
succeeded without returning an ID: reconcile the snapshot set/operation explicitly.
Known upload IDs remain available after every later failure. Errors retain HTTP
operation evidence when returned; cancellation before an ingress receipt may leave
only the attempted action. This is not a persisted recovery journal. Use the same
client instance throughout a transfer. Callers own absolute deadlines, cancellation,
funding, pending-ID custody, load/delete decisions, rollback order and cleanup
errors. Neither advance nor Drop retries or deletes a partial snapshot.

The installed fixture copies more than one MiB of both heap and stable memory,
mutates live state, then restores the uploaded snapshot and executes its timer.
Synthetic peers cover exact DTO bytes/sender routing, incomplete and oversized
regions, bad hashes, missing system metadata, codec quotas, lost replies and
cancellation. Sibling Testkit adoption remains separate and requires publication.
