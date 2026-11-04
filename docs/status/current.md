# Current handoff — 2026-11-04

## Current local Shared adoption and upstream crypto cut

The maintainer reversed embedded cryptographic ownership. BLS curve/verifier,
certification certificate/hash/tree and canister-signature creation/verification
bodies, algorithm suites/vectors and four reconstruction guards are removed.
Registry suppliers now own those implementations: verifier 0.6.0, curve 0.10.1,
certification 3.2.0, signature creation 1.3.1 and verification 0.3.0. The unused
direct development-only representation-independent-hash 4.x selection is gone.

Platform retains delegating BLS keys, CanisterSigPublicKey with canonical owned
Principal/seed fields, #72 DER framing, #127 identity-key admission and signing-time
extraction. Certificate/tree, BLS Signature and BLS errors now have upstream
identity. Native Agent shares certificate types directly; recursive conversion
adapters are removed. Runtime and Types identities/versions do not change.

Registry BLS still lacks #127. Both root and delegated subnet identity keys are
therefore refused around upstream canister verification. Regressions demonstrate
that raw upstream verification accepts the forged identity-key envelopes while
both Platform APIs reject them. No algorithm is implemented in those guards.
The adapter repeats CBOR decoding after upstream verification for delegated-key
admission and optional signing-time extraction; upstream exposes no evidence API.
Time retains unique top-level/u64/nonminimal-encoding behavior and never reads
delegation time.

Upstream signature packaging restores transitive Candid/printer, raw IC0, CBOR,
representation-independent hashing and a lazy root cache. No store/cache API is
exported. Raw Runtime/executor/IC0, standalone certification and BLS remain codec
isolated. Signature+Runtime admits only the qualified registry raw IC0 supplier;
external CDK/executors and alternate/local IC0 suppliers still refuse.
This is maintenance reduction, without a linked-Wasm-size or performance claim.

Focused facade Clippy, eight library tests and twelve integration tests pass.
Four separate native/Wasm consumers (certification, creation, verification, Agent),
the 17-selection native/Wasm feature matrix, combined signature+Runtime Wasm
compilation/graph, feature-contract/closure/graph and runtime-refusal fixtures
pass. Remaining Runtime/Principal/management source guards pass. Evidence:
`target/shared0293-upstream-signature-*`; prior BLS-only evidence is retained in
`target/shared0293-upstream-bls-*`. Full CI/release qualification was not run.

Shared 0.29.3 at `afccb9947d26bbae13d4c4a143d2fd5f0c2c07a4` remains adopted
locally through the committed-revision exporter. The earlier owned unit/fixture
test separations remain in macros, CDK call policy/System API text, prepared-call,
installed-runtime, stable-identity and packaging fixtures; generator/admission
copies follow their canonical tests. Certificate conversion tests and the
conversion implementation are now obsolete and removed by the cut above.
Malformed-Candid integration/fuzz fixtures use ordinary module discovery; group
ledgers own pending notes and the root changelog is navigation only. Existing
focused evidence for those changes is retained under `target/shared0293-*`.

LOC now: 6,876 runtime-classified and 6,093 separate test Rust code lines. Before
this final cut: 9,314 and 6,892; before BLS removal: 20,614 and 7,188. Counts exclude
comments/blanks, and a few imported tests remain in runtime-classified files.
Evidence: `target/shared0293-loc-after-signature-removal.json`.

Pending notes are Platform 0.22.0 and Runtime 0.19.1; package versions are unchanged.
All changes remain local, uncommitted and unreleased. Preserve the incoming Pest
update and prior #139 documentation corrections. Siblings remain untouched.

## Delivered Platform 0.21.0 — #139

Platform **0.21.0** is pushed and published from
`9d5b6c6d0e9a88010a22d0267aa8395e7a7f6b93`, tagged `platform-v0.21.0`.
The full Linux release gate passed, including Clippy, protected-source guards,
all consumer/feature graphs, release preparation/recovery, installed Runtime
and Timers composition. The actual prepared-version graph passed before delivery.
Publication's separate lint and package verification passed before upload.
Hosted push CI remains separate from these local release results.

The maintained PocketIC client now belongs to `ic_testkit::pocket_ic` in published
Testkit **0.84.0** (archive source `5f91f4cdbaa72e04cc0d7ba92e4e1c3953b0ae8e`).
Tooling PRs #326–#328 integrated the transfer and qualification fixes; its other
session owned publication. Platform's client crate, facade feature/module and
release group are retired. Historical ledgers and published packages remain.
See [consumer migration](../pocket-ic-client.md).

#139's acceptance review verified destination native CI/publication qualification
on Linux and macOS Intel/ARM in [Tooling run 41681210184](https://github.com/dragginzgame/ic-tooling/actions/runs/41681210184),
against Testkit's exact release source above. Its separate focused release-contract
stage remained running on Intel; no overall workflow success is claimed.

Runtime/macros/guard remain **0.19.0**, Types remains **0.18.0**. No browser/npm
release or sibling consumer migration is included. The independent native
Profiler provider remains unchanged. No Platform normal/build graph selects
Testkit or the retired PocketIC client.

## Shared adoption and qualification

The published Platform 0.21.0 release used Shared **0.29.1**, pinned to committed main
`55960755351c127b8ef2a03a1f08fdeba068f5e4`, exported without payload edits.
It is a governance refresh: local release groups lead the root dependency catalog,
and the root changelog directly links #139 and Tooling #324. Rust remains pinned
at 1.99.1. The earlier Shared 0.29.0 delivery is tagged and pushed at `5767ac6`.
No Shared 0.29.1 release is implied by snapshot adoption.

The first Platform gate passed but its prepared-version check caught a fixture
that globally patched Testkit's Platform 0.20 dependency to local Platform 0.21.
Qualification now keeps that published native DTO adapter separate while retaining
the canonical Runtime patches. The exact candidate graph and a fresh full gate
passed. This is test wiring; it adds no production facade or executor duplication.

The three-group release mapping retains the full gate. Its host allowlist stays
empty until fresh full/selected native mapping qualification. Historical #137
receipts remain intact: run 41625352854 qualified the prior four-group mapping
on Linux and macOS 15 Intel/ARM at Rust 1.99.1. Those receipts do not activate the
new mapping. See [release validation](../platform-release-validation.md).

## Delivery and evidence custody

The obsolete Platform contribution container and its three clean checkouts were
retired on 2026-11-04. Complete contents, all Git refs/bundles and release evidence
are retained in the normal checkout at
`.git/retired-checkouts/20261104-issue139/ic-platform-contributions.tar.zst`.
Archive integrity and full filesystem comparison passed before removal. Three
missing worktree registrations were pruned with their metadata preserved there.
Use this normal checkout for subsequent work; its incoming Pest update and
uncommitted implementation were preserved.

The successful gate receipt is
`target/release-validation/qualification.5gefDU/receipt.json` inside the archived
Platform checkout. The earlier passing gate and failed preparation remain there;
`.git/release-state/platform021-prepared-before-fixture-fix.patch` records the
rejected candidate. Deployment logs are retained under
`.git/release-state/issue-139-delivery/`; publication evidence is in that checkout's
`target/publication/`. Shared's superseded 0.28.3 and successful 0.29.0 logs remain
in the archived Shared checkout's matching release-state directory.

The donor is Platform `41688ce`, including the macOS mock-socket correction.
Tooling retains its licenses, bodies/tests and source reconstruction hashes.
Original source and full Git bundles remain in the normal Platform checkout's
`.git/issue-139-suppliers/` and recorded donor artifacts.

#138's Account Serde-byte change requires matching Wire/browser schemas, new RPC
contract IDs and consumer-owned persisted-data handling. #134's fallible/no-replay
client and #136's bounded snapshot transfer shipped in 0.19/0.20; preserve their
accepted IDs, bytes, limits, crypto bodies and canonical identities.

Native Agent remains upstream ic-agent 0.49.3; advisory #50 is unchanged.
Scheduling qualification stays pinned to the authorized Runtime 0.19 adoption
`a61f3517`, without a Scheduling merge or publication. Earlier native Profiler,
withdrawn Agent and release evidence retain custody in the
[previous handoff](https://github.com/dragginzgame/ic-platform/blob/41688ced81066b90aee756def7468a8388ee67e9/docs/status/current.md).
