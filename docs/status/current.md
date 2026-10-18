# Current handoff

The maintainer released 0.1.6 at `d6edec2`. Package metadata remains 0.1.6;
the compatible 0.1.7 changelog draft covers additive APIs and contributor tooling.
The registry's non-yanked 0.1.6 archive is now verified: SHA-256
`4c2a6b9c458b47f2913811998a9f6cc49394deaebb9a11e9d0a1bffb6b169cf2`, VCS
`d6edec2661f510297c8c9c77c5b9fa01741092e8`. Its ic0 and management modules match
this checkout byte-for-byte. Download/index evidence is under
`target/issue-boundary-*`; this is publication evidence, not consumer acceptance.

The current API work addresses concrete remaining imports in
[Canic/Blob's Candid contract](https://github.com/dragginzgame/ic-platform/issues/5)
and [the native Agent contract](https://github.com/dragginzgame/ic-platform/issues/9).
`candid::contract::types::subtype::subtype` preserves structural projection checks;
`candid::decoder::{decode_one, decode_args}` preserve upstream defaults and
diagnostics, separately from the parent's finite-limit helpers. Native
`candid_parser` additionally exports `IDLValue`, `IDLField`, `TypeSerialize` and
`ValueSerializer` for refusal fixtures. Native `agent` additionally exports
`AnonymousIdentity`, `DelegatedIdentity`, `Delegation`, `SignedDelegation`,
`CallResponse` and `TransportCallResponse`. These are direct upstream exports;
no default feature, dependency selection, transport policy or executor changes.

Read-only review used Canic `5583496b`, Blob `55d70cb`, Backup `3317b4e6` and
Auth `2da4dc0e` working trees, not clean-source claims. Source hashes and this
task's before-images are under `target/issue-boundary-before/`. An original-owner
Candid 0.10.38 / Agent 0.49.2 generator and lock are retained under
`target/issue-boundary-upstream/`; maintained vectors bind historical-field and
nested/wide-header encodings plus delegation signable bytes and signatures.
The standalone dynamic fixture declares only Platform with Candid/parser;
the Agent/certification fixture adds no Candid or Agent declaration. Consumer
budgets, projection direction, delegation/expiry/target/root-key checks, single
dispatch, pending/lost-response spending and HTTP middleware remain consumer-owned.
Migration and the packaged README specify import, manifest, generator and test
changes. Registry-only adoption and actual consumer installed scenarios still
require delivery/publication and their own acceptance; siblings remain untouched.

Standalone consumers pass native Rust 1.99/1.88 tests and applicable Wasm checks.
The expanded corpus covers narrowed/invalid projections, default decoder arity
and typed failures, historical-field/type-header bytes and refusal, anonymous and
checked/unchecked delegated identities, fixed signatures and signed response type
identity without dispatch. All-feature tests/MSRV, strict Clippy/Rustdoc, all 256
native/Wasm feature combinations plus defaults, actual all-feature Wasm build,
ShellCheck and formatting/pin/boundary/documentation/snapshot checks pass. The
first strict Clippy run rejected a hex-helper idiom; the corrected Rust 1.88
`as_chunks` pattern and final reruns are retained in `target/issue-boundary-*.log`.
No full CI, commit, release version change, push, publication or remote creation
was performed by this task. Native macOS and installed consumer/runtime acceptance
of these dirty additions remain separate.

Historical producer qualification is now verified against the successful
[0.1.5 native CI matrix](https://github.com/dragginzgame/ic-platform/actions/runs/39113043464)
at `4ecc6f86643d57fc4b12a7dfdf1482728dbe5a5d`: Linux x86_64, macOS 15 Intel and
macOS 15 ARM64 each passed `make ci`. That evidence reconciles the older delivered
API/cache issues; it does not qualify the current dirty additions or prove new
installed/live-IC behavior. The latest inspected 0.1.6 run had Linux and Apple
Silicon success with Intel macOS still running; that incomplete gate does not
establish full native acceptance for the delivered marker/history additions.

The root Cargo catalog now has `Production dependencies` and `Tests and tooling
only` sections. All ten current dependencies serve production; the second section
explicitly has no entries. Blank lines after the headings keep cargo-sort from
moving a heading with the first dependency when repairing unsorted input.
Paired local `FORMAT_WRITE_COMMAND` / `FORMAT_CHECK_COMMAND` inputs select
`--grouped` using Shared 0.24.0's supported interface. All release adapter metadata
checks select `--manifest-order-grouped`, including checks against committed source.
That formatting work changes no dependency selection, Rust source, lockfile or
release version.

GitHub main still resolves to Shared 0.24.0 at `cab12cd`; its 0.24.1 sibling draft
is uncommitted and is not adopted. A clean clone of that committed source ran the
canonical exporter preview and refresh: all 96 snapshot payloads and the snapshot
manifest remain unchanged. Focused snapshot/tool admission, formatting, pins,
dependency boundaries, release-command and documentation checks pass. The producer's
formatting-hook checker passes in disposable fixtures, including first-dependency
sorting, heading preservation, unrelated edits, partial-staging refusal and hook
installation. This does not activate hooks in the real checkout. Read-only metadata
checks cover grouped working source, rejection of unsorted source and selection of
an immutable commit independently of dirty working metadata. Logs and before-images
are under `target/shared-adoption-*`; an earlier heading-movement failure is retained.
The formatting task's Rust and lockfile hashes matched its before-images. Full CI
and native macOS checks were not run for that tooling-only change;
installed-canister behavior was not requalified. No commit, version bump,
publication or remote creation ran.

Delivered 0.1.6 adds optional `icrc-ledger-types`, exposing sixteen upstream
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
passed. Published 0.1.6 adoption still needs Query/Canic's actual ledger,
archive, receipt and certificate tests. Native macOS and installed-ledger behavior
remain distinct from native Linux and Wasm compilation.

Delivered 0.1.6 also includes a compatible additive boundary repair for
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
Both Platform issue threads record the local evidence. Version 0.1.6 is delivered
and published; its required native matrix is not yet complete at this inspection.
Registry-only acceptance still needs Testkit/Blob adoption and their existing
installed runtime/recovery checks.
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
adopted. The local paired commands and release adapter explicitly select grouped
ordering on 0.24.0. Role sections do not enforce deployment admission.

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
