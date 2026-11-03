# PocketIC client transfer to Testkit (#139)

The maintained PocketIC 16.1 Rust client now lives in Tooling’s existing
`ic_testkit::pocket_ic` module. Its client, builders, DTOs and typed errors ship in
one Testkit package/version with the lifecycle helpers in `ic_testkit::pic`.
Platform no longer maintains `ic-platform-pocket-ic`, its release group,
`pocket-ic` feature or `ic_platform::pocket_ic` forwarder. There is no replacement
Platform-to-Testkit normal/build dependency.

## Consumer migration

Replace `ic_platform_pocket_ic::...` and `ic_platform::pocket_ic::...` with
`ic_testkit::pocket_ic::...`. Async clients remain under `nonblocking`; REST DTOs
remain under `common::rest`. Remove the old client dependency and Platform feature.
Keep other Platform features only where required for their own APIs.

Testkit 0.84.0 owns this breaking identity change. Platform 0.21 development
fixtures select that registry release directly; the supplying release must be
published before Platform qualification and publication. Production canister
graphs do not select Testkit.

## Source and behavior custody

Tooling imported Platform `41688ced81066b90aee756def7468a8388ee67e9`, including the
qualified macOS blocking-socket test correction after the 0.20.0 publication.
The original Apache-2.0 licenses, upstream notices, original fingerprints,
fallible-client and snapshot patches, and tests moved with the implementation.
The destination’s module-layout patch reconstructs every original donor source;
its guard then runs the preserved upstream reconstruction checks. No client
function bodies, cryptographic behavior, protocol constants or budgets changed.

Tooling’s focused Linux checks passed: native and Wasm Clippy, 111 library tests,
management identity, source/mutation guards, native/Wasm graph profiles, archived
client imports and installed PocketIC 16.1 management/snapshot restoration.
Fresh macOS Intel/ARM transfer qualification remains pending. Platform’s historical
three-host donor evidence does not substitute for destination qualification.

Typed failures, accepted-operation custody, explicit Busy-only retry, uncertain
request no-replay behavior and bounded snapshot transfer remain owned by the
client. Consumers retain budgets, retries, topology and fixture policy. The IC
simulator and separately pinned native Profiler provider remain outside this move.

The retired [client ledger](../releases/pocket-ic/CHANGELOG.md) and published
versions remain intact. The earlier [client contract](https://github.com/dragginzgame/ic-platform/blob/41688ced81066b90aee756def7468a8388ee67e9/docs/pocket-ic-client.md)
records historical behavior and release evidence. No package was yanked or republished.
