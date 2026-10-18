# Current handoff

The maintainer released 0.1.5 at `4ecc6f8`. Package metadata remains 0.1.5.
The cached registry archive has SHA-256
`ad5262d2a30adfd89b55960da7c9faffb5a68b7cab4c219af5958f1ab2ce8fe4`,
matching the archive reported by Testkit and Blob. This verifies cached package
identity, not current registry availability or consumer qualification.

Pending 0.1.6 also adds optional `icrc-ledger-types`, exposing sixteen upstream
0.2.0 identities through the flat `ic_platform::icrc_ledger_types` module. The
surface follows Query/Canic's actual account, metadata, allowance, transfer-error,
block/archive/certificate consumers. It works on native/Wasm, brings transitive
Candid/CBOR dependencies and enables no Platform codec, executor, parser or Agent.
Upstream default/storage features stay disabled; graph admission rejects
`ic-stable-structures` and unselected ledger dependencies. Account semantics,
wire variants, value hashes, archive callbacks and consumer policy remain upstream
or consumer-owned. No storage representation or local ICRC-21 implementation moves.

The standalone ledger/Candid consumer declares only Platform. It checks fixed
original-owner account/block wire bytes and value hashes, all metadata variants,
large amounts, allowance DTOs, transfer errors, typed callbacks, certificate fields,
account text and absent-versus-zero subaccounts. The ledger graph expands admission
to all 256 native/Wasm combinations plus actual defaults. Negative fixtures reject
runtime activation, explicit upstream storage features and an injected real
storage package. The first storage-graph setup needed uncached 0.6.9; its failure
is retained, and graph refusal instead uses the cached 0.7.2 package. Current
logs, original encoder/lock, negative fixtures and
incoming source before-images are under `target/icrc-*`; the consumer source hashes
are in `target/icrc-consumers.sha256`. Standalone consumers pass Rust 1.99/1.88
and Wasm; all 256 native/Wasm feature combinations plus defaults, all-feature
tests/MSRV/Wasm compilation, strict Clippy/Rustdoc, ShellCheck, formatting,
declaration, documentation and unchanged-snapshot checks pass. The complete log
is `target/icrc-qualification.log`; the actual all-feature Wasm build passes in
`target/icrc-wasm-build.log`. No full CI or release was run.
Existing lock selections are preserved; ledger dependencies/features are added.
The registry attempt failed DNS; cached/offline resolution and vector generation
passed. Next adoption needs delivery/publication and Query/Canic's actual ledger,
archive, receipt and certificate tests. Native macOS and installed-ledger behavior
remain distinct from native Linux and Wasm compilation.

Pending 0.1.6 is a compatible additive boundary repair for
[Testkit markers](https://github.com/dragginzgame/ic-platform/issues/11) and
[Blob history DTOs](https://github.com/dragginzgame/ic-platform/issues/12).
The existing `ic0` feature now directly exports upstream `debug_print(&[u8])`;
formatting, labels, byte limits and counter policy remain Testkit-owned.
The existing `management-canister-types` feature directly exports
`CanisterInfoArgs`, `CanisterInfoResult`, `ChangeDetails` and `CodeDeploymentMode`
from exactly 0.11.0, alongside the seven existing HTTP/transform types.
Blob retains bounded calls, change limits, decode budgets, version checks across
await, evidence expiry/completeness, recovery fences and retained obligations.
No storage representation, retry policy or executor implementation changes.

The separate ic0-only consumer compiles the safe byte-slice call without
executing IC system calls on native hosts. The management/Candid consumer uses
fixed original-owner request/result bytes and real upstream change records;
it covers every change variant, all deployment modes, missing details,
truncation, caller byte refusal and exact arity. No management schemas are copied.
A disposable Testkit workspace changes only Platform selection, removal of
its direct ic0 declarations and the debug-print import. Its existing real probe
and benchmark test pass on Linux/PocketIC: parseable paired markers, positive
instruction span, deliberate trap and subsequent successful ping. The copied
probe also compiles at Testkit's Rust 1.91 floor. Its production Wasm graph has
one ic0 1.2.0 identity and no Candid/CDK frontend/parser. The private probe retains
its CDK/Candid exception. The first runtime attempts exposed missing disposable
target setup and forbidden forwarded target environment; after correcting those
fixture prerequisites, sandbox loopback refusal required a permitted local run.
The passing runtime log is `target/fix-11-12/testkit-pocketic.log`; earlier
attempts, build/MSRV logs, admitted server path and graph are retained beside it.
Sibling checkouts remain read-only. Incoming Cargo.lock edits are preserved.

The earlier marker/history batch's focused Linux tests, Rust 1.88, standalone consumers, all 128 native/Wasm feature
combinations plus actual defaults, Wasm compilation, strict Clippy/Rustdoc,
formatting, pin/boundary declarations, documentation and snapshot checks pass.
The all-feature Wasm library build also passes (`wasm-build.log`).
Logs and source before-images are retained under `target/fix-11-12/`, including
`qualification.log`, `shellcheck.log`, `testkit-msrv.log` and `testkit-wasm-graph.txt`.
The original encoder's vector generator and its lock are retained in `upstream/`;
its first online resolution failed DNS, then cached/offline generation passed.
No full CI, commit, version bump, push, publication or remote creation was performed.
Both Platform issue threads record the local evidence and remain open for delivery
and required consumer acceptance; these changes have no hosted CI result yet.
Registry-only acceptance for the new APIs requires the maintainer's release flow,
then Testkit/Blob adoption and their existing installed runtime/recovery checks.
Native macOS and installed-IC qualification remain separate from Linux/PocketIC
and Wasm compilation. Direct dependency isolation is not transitive removal or
measured Wasm-size reduction.

Shared Tooling remains 0.24.0 at
`cab12cd3dfa4fdb34846b25397f0d72881bb9ae0`; the 96-file snapshot is unchanged.
[The exported-link repair](https://github.com/dragginzgame/shared-tooling/issues/284)
is already prepared by its owner and pushed on Shared PR branches, but is not
on main or a qualified adopted snapshot. The guides' local capability-map links
remain affected here until that source is delivered and refreshed through the
canonical exporter. Do not hand-edit those payloads or export Shared's README.
The producer's separate 0.24.1 draft and its grouped formatting defaults are not
adopted. Current role labels/global formatting and release checks remain paired;
role comments do not enforce deployment admission.

The delivered library also provides unconditional Principal/PrincipalError,
bounded and configured/staged Candid decoding, explicit service/derive/encoding
machinery, certification types and hashing, native-only parser/Agent, and exact
executor 2.0.0/raw-system 1.2.0 identities. Defaults remain empty. Executor-only
and ic0-only selections remain Candid-free; management/ledger DTOs necessarily bring
transitive Candid. Runtime features never select host parser or Agent.

See [the migration contract](../migration.md) for source identities, ownership,
imports/manifests/generators/tests and previous evidence. Full CDK entrypoint,
call and protected-spawn integration remains tracked in
[the runtime contract](https://github.com/dragginzgame/ic-platform/issues/1);
changing imports does not fix upstream generated absolute paths. Published
Testkit limits/error identity and coordinated old-decoder retirement remain
subject to [producer coordination](https://github.com/dragginzgame/ic-platform/issues/2).
Deployment consumers must still enforce exclusions against their exact final
feature/target/lock selection and bind admitted source and artifact identity to
deployed bytes. Platform has no deployment entrypoint; publication does not run
the graph gate. Existing qualification evidence remains under `target/` and in
the migration contract; it does not qualify this new dirty tree remotely.
