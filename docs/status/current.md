# Current handoff

## Pending Wire 0.7 adoption

This local batch starts at released Platform 0.6.14 (`45b331f`). Published Wire
0.7.2 is selected for native `wire-schema` metadata and the independent runtime
fixture. Upstream release source is `a4da015251beb8fa3346fb3780c0b3dc85c0ef32`.
Package versions remain 0.6.14; 0.6.15 notes are pending. No staging, commit, push,
release, publication or sibling edits are part of this batch.

Only Wire/derive versions change in the root lock. The incoming uncommitted Wire
0.6.13 selection is intentionally superseded by 0.7.2; other selections are
preserved. The maintained runtime lock updates only Wire/derive plus required
edges to its already locked bytemuck/serde_bytes. No unrelated dependency upgrade,
executor/ic0 identity change, cryptographic source edit or feature change.

Host schema producers must update their Wire derives/TypeScript selection to
0.7 together with the next Platform supplier: 0.6/0.7 metadata traits have distinct
identities. Canister/default/executor-only graphs retain their feature isolation.
The [migration contract](../migration.md#owned-principal-and-account-wire-metadata)
describes this coordinated host adoption and the new decode semantics.

[#55](https://github.com/dragginzgame/ic-platform/issues/55) is source-qualified but
awaits maintainer publication of the next Platform release. Published 0.6.14 plus
Wire 0.7.2 reproduces E0277 for both Principal and Account; the pending source passes
the exact two bounds. `check-principal-wire-schema.sh ... --published X.Y.Z` now
admits only the exact registry facade for post-publication qualification.

An isolated copy of Blob Contracts 0.44.3 at source `9534f42` (pending root catalog
preserved) selects local Platform/Wire 0.7.2 for qualification only. All-target and
all-feature Clippy, Rust 1.91 compilation, 126 unit tests and all-feature MSRV Wasm
compilation pass. Its real schema/client/adapter/runtime producer executes; strict
TypeScript with relative-extension rewriting and generated-client import pass.
Evidence: `/tmp/platform55-*.log`, `/tmp/platform55-blob.gcg2ls`, and
`target/platform55-traits.MobRzd`. The temporary workspace patch models the pending
supplier; it is not a supported consumer distribution mechanism. Siblings are
read-only. Blob must publish an aligned contract release before Canic adoption.

Wire 0.7 keeps encoder/frame bytes but no longer reserializes decoded values for
canonical comparison. Consumers retain collection ordering/uniqueness checks and
original frame bytes for exact retries, signatures and hashes. `Noncanonical` is
retired; malformed primitive encodings report `Malformed`. Encode/decode budgets
are independent; existing fixture budgets are unchanged. Fixture decoders now
require only Deserialize. Required external Candid and persisted formats remain
consumer-owned and unchanged by this adoption.

Full root Clippy and all-target/all-feature runtime-fixture Clippy passed before
longer checks. Rust 1.91/strict TypeScript/Node Principal/Account conformance passes;
valid and malformed vectors exactly match the captured pre-upgrade Wire 0.6.13
proof. Generated Principal/Account TypeScript contracts also match exactly.
Evidence: `target/principal-wire-schema.Ydcchk` (before),
`target/principal-wire-schema.Do1csc` (after), `/tmp/platform55-*.log`.
Runtime tests include Deserialize-only response decoding and existing resource
refusals. Source/license, format, inventory, dependency boundary/pin and doc checks
pass. All six runtime tests and four release Wasm builds (Wire/external Candid,
Rust 1.99.1/1.91) pass; runtime evidence is `target/cdk-runtime.imgdwq`.
All-feature MSRV Wasm and all 240 graph selections per target also pass. Every
non-Wire package entry in both locks is unchanged from its captured input.
Native macOS and installed-canister qualification have not been rerun for
this Wire upgrade; the locally admitted Testkit server CLI is unavailable.
No timing or Wasm-size improvement is claimed.

## Current contracts and delivered source

Principal metadata originally shipped in 0.6.9 and Account metadata in 0.6.12.
Owned Serde bytes, length/order/effective-zero semantics and external DTO identities
remain unchanged. Rust 1.91 applies to all features. Shared Tooling 0.26.8 remains
selected at `0b07c2a` through its canonical snapshot.

[Signature time #51](https://github.com/dragginzgame/ic-platform/issues/51) and the
consumer warning/live-script-tail fixes shipped in 0.6.14. Auth adoption is tracked
in [Auth #86](https://github.com/dragginzgame/ic-auth/issues/86). The original
verifier and imported bodies/vectors remain protected by exact reconstruction.
[Runtime CI #52](https://github.com/dragginzgame/ic-platform/issues/52) is closed
after full installed qualification passed on 0.6.14; that result does not qualify
this new Wire selection. Historical detailed evidence remains in this handoff
at `45b331f`, without transferring consumer trust, freshness or retry policy.

The [IC inventory](../ic-inventory.md) and [export audit](../audits/export-usage-2026-10-21.md)
define the dependency boundary. All owned endpoints/clients use Wire; external IC
interfaces retain required Candid. [Canic #613](https://github.com/dragginzgame/canic/issues/613)
owns its active migration; do not duplicate pending sibling work. Parser retirement
and deferred Handlebars packaging stay on [#38](https://github.com/dragginzgame/ic-platform/issues/38).
The crypto-bigint advisory remains separate on [#50](https://github.com/dragginzgame/ic-platform/issues/50).

Executor =2.0.0 and ic0 =1.2.0 preserve their qualified registry identities.
Callback/trap/cleanup, cryptographic implementations and licenses remain guarded.
Storage/allocation, application admission and resource policy stay with consumers.
Full release CI, publication, new native macOS and installed qualification remain
separate maintainer steps.
