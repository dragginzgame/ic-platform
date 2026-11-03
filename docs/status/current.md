# Current handoff — 2026-11-03

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

Runtime/macros/guard remain **0.19.0**, Types remains **0.18.0**. No browser/npm
release or sibling consumer migration is included. The independent native
Profiler provider remains unchanged. No Platform normal/build graph selects
Testkit or the retired PocketIC client.

## Shared adoption and qualification

The requested Shared **0.29.1** snapshot is pinned to committed main
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

The maintainer approved direct main/tag delivery from
`/home/adam/projects/ic-platform-contributions/issue-139/ic-platform`.
The normal checkout's incoming Pest update is excluded from the published lock.
Retain it during synchronization; preserve the original source snapshot/stash.
Existing sibling working checkouts remain untouched.

The successful gate receipt is
`target/release-validation/qualification.5gefDU/receipt.json` in that durable
checkout. The earlier passing gate and failed preparation remain preserved there;
`.git/release-state/platform021-prepared-before-fixture-fix.patch` records the
rejected candidate. Deployment logs are retained under
`.git/release-state/issue-139-delivery/`; publication evidence remains in
`target/publication/`. Shared's superseded 0.28.3 attempt and successful 0.29.0
logs remain in its durable checkout's matching release-state directory.

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
