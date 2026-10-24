# Current handoff

## Pending 0.6.13 validation cleanup

The working batch starts from released 0.6.12 (`a332a15`). No production Rust API,
codec, feature selection, cryptographic implementation or Cargo dependency changes.
The pending changelog is 0.6.13; package versions remain 0.6.12. No staging,
commit, push, publication or sibling edits are part of this batch.

`make check-wire-schema` now runs after full Clippy in `make ci`. Explicit setup
uses Node 24.21.0, npm 12.2.0 and integrity-locked TypeScript 7.0.2; `make install-tools`
includes the local installer and `make tools-check` includes offline admission.
The native CI matrix prepares these tools on Linux and both macOS architectures.
Validation never downloads npm dependencies. The standalone proof covers owned
Principal/Account frames, representation preservation and malformed request/response
refusals. Tool declarations live in `scripts/fixtures/wire-schema/package.json`.

Wasm graph caching now admits `wire-schema` only after checking both native-only
optional dependency declarations and the exact feature edges. Principal remains
in the closure; native metadata selections are not collapsed. Regression cases
reject missing, duplicate, portable, required or renamed dependency contracts and
added local feature implications. All feature selections still receive admission.

Full Clippy passed before longer checks. The Rust 1.91 standalone consumer,
TypeScript 7.0.2 and Node 24.21.0 proof passes, including seven malformed Account
cases in both directions. Evidence: `target/principal-wire-schema.ZI4cyz` and
`/tmp/platform-cleanup-*.log`. Dependency declarations were checked with a
throwaway index admitting the new npm files; the real index remains untouched.
All 240 routine feature selections per target pass: 219 native and 127 Wasm
queries (previously 144 Wasm). Independent principal/metadata Wasm Cargo graphs
are identical; metadata compilation passes on Rust 1.91. Graph guard regressions,
formatting, inventory, boundaries, release-command tests, documentation links,
ShellCheck, actionlint and offline tool admission pass. No timing gain is claimed.
New native macOS CI execution and installed-canister qualification are not claimed.
Full release CI remains maintainer-owned.

## Published capabilities and consumer ownership

Principal metadata shipped in 0.6.9 and Account metadata in published 0.6.12.
Both use native-only `wire-schema`; canister consumers retain `principal`.
Rust 1.91 applies to all features. Wire 0.6.7 supplies metadata and the independent
runtime fixture; Shared Tooling 0.26.8 is selected at `0b07c2a` through its snapshot.

The [migration contract](../migration.md), [IC inventory](../ic-inventory.md) and
[individual export audit](../audits/export-usage-2026-10-21.md) define the boundary.
Account bytes and absent/explicit-zero fields remain distinct while effective-zero
Eq/Ord/Hash agree. External Candid DTOs preserve upstream identities. Consumer
adoption is tracked in [Canic #628](https://github.com/dragginzgame/canic/issues/628)
and [IcyDB #70](https://github.com/dragginzgame/icydb/issues/70); IcyDB's existing
Account also has different binary serialization, so it is not a drop-in alias.

Owned endpoints and clients use Wire. Required external IC interfaces retain
Candid, and existing checkpoint bytes remain consumer-owned. The coordinated
migration is [Canic #613](https://github.com/dragginzgame/canic/issues/613) and
[Tooling #225](https://github.com/dragginzgame/ic-tooling/issues/225). Do not replace
active sibling work. Parser retirement and deferred Handlebars packaging stay on
[#38](https://github.com/dragginzgame/ic-platform/issues/38); the crypto-bigint
advisory remains separate on [#50](https://github.com/dragginzgame/ic-platform/issues/50).

Executor =2.0.0 and ic0 =1.2.0 retain single registry identities. Callback, cleanup,
imported cryptographic bodies and licenses remain source-guarded. No crypto rewrite,
Agent fork, alternate executor, storage allocation or consumer policy transfer.
Stable-structures manager exports shipped in 0.6.5; #45's Linux/macOS and installed
runtime evidence is recorded on its closed issue. Persistence owns its adoption.

Historical implementation and qualification detail remains in this handoff at
`a332a15`, including the 0.6.12 Account proof, earlier source guards, fuzz runs,
MSRV/Wasm tests and installed-runtime reports. Those historical results do not
qualify new changes or establish current consumer adoption.
