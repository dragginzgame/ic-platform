# Current handoff

## Pending 0.6.14 issue fixes

The working batch starts from released 0.6.13 (`19ca27a`). It adds
`signature_verification::verify_canister_sig_with_time` and `CanisterSigTimeError`
under the existing signature feature for [#51](https://github.com/dragginzgame/ic-platform/issues/51).
The original verifier and every imported crypto body/vector remain exact. One
child-module declaration permits reuse of the private original helpers; removing
only that line recovers the original verifier source hash. The new owned module
supplies evidence extraction and time-format admission, not crypto algorithms.

The adapter decodes the envelope and signing certificate once each and returns
signing-certificate nanoseconds only after the same ordered checks succeed.
Delegation time is never substituted. Errors distinguish upstream verification
from absent/ambiguous/malformed time. Consumers keep protected trust, issuer/seed,
DER admission, clock and inclusive freshness/skew policy. No dependency, feature,
external DTO, wire/storage byte or original verifier behavior changes.

Full Clippy passed before tests. Sixteen verifier/evidence tests and the standalone
consumer pass at Rust 1.91; standalone native/development/MSRV and Wasm checks pass.
An isolated copy of Auth 0.15.2 changes only its call/error mapping: Clippy, 40
canister-signature/certified-proof tests and all-feature MSRV Wasm compilation pass.
Evidence: `/tmp/platform51-*.log`, `target/boundary-consumer.bfXJi4`, and
`/tmp/platform51-auth.uWZkFS`. Sibling checkouts remain read-only. No speed/size claim.

[Installed-runtime CI #52](https://github.com/dragginzgame/ic-platform/issues/52)
fixes missing Node/npm preparation before the full tool installer. Native CI at
0.6.13 passed on all configured hosts, but installed-runtime setup failed before
qualification. The workflow now uses the already qualified native CI setup.
New native macOS/installed-canister execution remains separate from local proof.

0.6.13's Wire proof automation and graph cleanup are released; historical details
remain in this handoff at `19ca27a`. Packages remain 0.6.13; 0.6.14 is notes only.
Publication and release remain maintainer-owned.

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
