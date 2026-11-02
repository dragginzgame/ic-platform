# Current handoff

## Full-batch delivery authorized

The maintainer requested all pending changes. Planned releases are Types 0.18.0,
PocketIC 0.19.0, Runtime/macros/guard 0.19.0 and Platform 0.19.0, in dependency
order. Runtime changes because its public Principal must retain the canonical
Types identity. Keep upstream Agent. Use standard full release/publication gates.
Timers' selected qualification source accepts Runtime below 0.19; permission for
an isolated compatible Scheduling branch is pending. No Scheduling release or
sibling checkout edit is part of this delivery. Do not bypass singleton admission.

## Agent decision — 2026-11-02

The maintainer withdrew the unreleased owned Agent and its cryptographic-provider
migration. The explicit native facade again selects upstream ic-agent (compatible
0.49.2 requirement, locked 0.49.3), with PEM and without upstream default features.
External certificate identities and explicit owned/external conversions return.
Agent remains absent from Wasm; no production canister size saving is claimed.
Replacement source, experiments, source guards, crypto pins and Types' unpublished
principal-external-rangemap feature are removed. The unrelated Principal facade
cleanup, Account byte adapter and PocketIC work remain. Nothing was committed,
released, published or changed in siblings by this rollback.

The pre-revert dirty tree and withdrawn source are recoverable under
/tmp/platform-agent-revert/. This is local undo evidence, not a second supported
implementation. The maintained five Agent consumer tests and the installed
four-identity gateway fixture remain; the latter now owns its PEM fixtures and
license under scripts/fixtures/agent-installed. Focused validation logs are in
/tmp/platform-agent-revert/: Clippy, five consumer tests and all four installed
identity profiles pass, including wrong-root refusal. Feature admission/closure
tests, native/Wasm graph isolation, inventory, pins, formatting and links pass.
Native macOS and release qualification remain separate.

[#50](https://github.com/dragginzgame/ic-platform/issues/50) remains open for
native Agent and ic-secp256k1 derivation's crypto-bigint 0.5.5 paths. Native tools
can sign real requests; using upstream is not a test-only advisory exemption.
No cryptographic rewrite, waiver or incompatible Cargo patch is authorized by
this rollback. Standalone ECDSA retains its existing patched provider.

## Pending product changes

[#134](https://github.com/dragginzgame/ic-platform/issues/134): breaking fallible
PocketIC sync/async client, bounded HTTP exchange and monotonic polling are local.
Accepted computation/ingress IDs survive failures; only explicit Busy permits
initial resubmission. Failed explicit deletion retains state and Drop does not
replay it. I/O-performing Default and unused stringifying response helper are
removed. Facade, installed/runtime and benchmark consumers are aligned. Exact
reverse patch and mutation checks preserve original source/license fingerprints.
See [the client contract](../pocket-ic-client.md) and its release ledger.

Evidence /tmp/pocket-134-*.log covers client Clippy, 12 unit/one identity tests,
three facade tests, runtime/benchmark callers, source guards and installed
PocketIC 16.1 status/settings/snapshot lifecycle. These were Linux checks.
A concurrent target/ removal interrupted an earlier compile; the affected build
was repeated in /tmp/platform-134-build. Original source for reconstruction is
retained in /tmp/pocket-134-before. No publication is implied.

[#136](https://github.com/dragginzgame/ic-platform/issues/136) bounded snapshot
transfer and [#137](https://github.com/dragginzgame/ic-platform/issues/137) scoped
release qualification remain unimplemented; retain the existing broad release gate.

[#138](https://github.com/dragginzgame/ic-platform/issues/138): Types' Account
subaccount uses serde_bytes with default/with; Postcard Some gains a length byte
and CBOR Some uses bytes. JSON, raw None/zero fields and logical equality remain
qualified. See [the migration contract](../migration.md#account-byte-adapter-cut-138).
Old frames can decode to different valid values: affected consumers require fresh
contract IDs, matching Wire/browser schemas and separate persisted-data handling.
The next Types minor and genuinely affected canonical facade/Runtime selections
must be released together appropriately; no mechanical unrelated group bump.
Focused native/Wasm, archive and codec evidence is under target/account-138-*.

PocketIC's pending Strum/macros 0.28 adoption preserves imported source and JSON
but changes exposed trait identity, requiring a minor. Base64 0.23.1 remains.
Keep resolver-selected crypto-common 0.1.6/generic-array 0.14.9 rather than pinning
solely to increase a version label. Pest/Schemars experiments were not adopted.

## Delivered state and recovery custody

Published independent groups remain Types 0.17.0, PocketIC 0.18.0, Platform
0.18.0, and Runtime/macros/guard 0.18.1. All six crates require Rust 1.99.1.
Shared 0.27.27 at 651f5f7acb2e3055abd51ba5f835a2112801666a remains the installed
executable snapshot; do not modify its payloads. Revision-bound policy does not
refresh that snapshot. Browser/npm and sibling publications are unchanged.

Runtime #135's IC0 stable64 wrappers shipped at
868c3c788a3b4ee93df3994a06bb44e6746331c8 (runtime-v0.18.1), with full release and
installed owner/Timers gates and registry-only native/Wasm consumer proof.
Receipts and completed plan: .git/release-state/runtime-0181-delivery.LgU8r6/.
A first admission fixture failed with a custom CARGO_TARGET_DIR; the standard
release gate passed after removing that override, without weakening checks.
Other publication/readback receipts and prior failed Runtime intent remain in
.git/release-state/platform-018-final-delivery.9UcrQT/ and
.git/release-state/platform-018-deployment/. Preserve them.

Native-provider setup fix #133 is integrated at 7740fc4. Focused setup/refusal,
52 provider and four HTTP tests passed; fresh hosted acceptance in run
41498018089 was pending when last inspected. Do not claim the overall workflow
green. Logs: target/issue-work-20261102/. No fresh macOS/live IC claim is made.

## Sibling scope

Composition qualification selects immutable Timers 0.37.0 source
1c57a6d9abfd432fc683c5016d4625763682483a on
adopt/runtime-018-platform-qualification. No Scheduling merge/release was done;
return to a qualified registry source after its owner publishes. Tooling/Canic
stable-structures removals are explicitly maintainer-owned. Sibling checkouts
remain untouched, and the mistaken Tooling deployment request stays withdrawn.
Preserve canonical Types, the Runtime singleton and guarded crypto bodies.
Historical publication details remain in the
[delivered handoff](https://github.com/dragginzgame/ic-platform/blob/81490a94fbe35c28f05c6f74d2dc4e50a643822a/docs/status/current.md).
