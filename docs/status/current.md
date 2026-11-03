# Current handoff — 2026-11-03

## Client ownership transfer — #139 / Tooling #324

The maintainer authorized delivery of Shared, Testkit and Platform from the
explicit durable checkouts under
`/home/adam/projects/ic-platform-contributions/issue-139/`, including direct
main/tag pushes and the affected Rust publications. Existing sibling checkouts
remain untouched. The ordinary Platform checkout's unrelated Pest update is
excluded from this release source; its incoming bytes remain preserved there.

Tooling PRs #326–#328 integrated the client and qualification fixes. The
maintainer assigned Testkit publication to the other session; Testkit 0.84.0 is
now available from crates.io. Both maintained runtime qualification locks select
that registry supplier, with no dependency on the retired client package.

Shared PRs #351/#352 merged the native owner exception and Bash 3.2 group-order
fix. Shared 0.29.0 is released and pushed at
`5767ac6788fab3a9b2350015ab0e74093ddc2f1d`; that release was initially adopted through the exporter. The maintainer subsequently requested the latest
Shared snapshot: 0.29.1 at `55960755351c127b8ef2a03a1f08fdeba068f5e4` is now
adopted from committed main. This governance-only refresh preserves executable
payloads; local groups lead the dependency catalog and root notes link #139
and Tooling #324. Snapshot, manifest order, inventory and documentation checks
pass. Shared 0.29.0's full gate passed (894 seconds), and remote
main plus annotated tag were verified. The earlier 0.28.3 attempt passed its
gate but was stopped before metadata, tag or push after main advanced. Both
logs and the earlier source branch remain in the durable Shared checkout under
`.git/release-state/issue-139-delivery/`.

Platform 0.21.0 release preparation is active: the final published supplier is
selected, and the complete release gate precedes version preparation and upload.
The initial full gate passed, but prepared-version admission caught a fixture
lock that conflated Testkit's registry facade with the local facade. The global
facade patch is removed; Runtime singleton patches remain. The exact 0.21
candidate lock projection and Timers graph now pass. Failed preparation is
preserved in `.git/release-state/platform021-prepared-before-fixture-fix.patch`;
metadata was restored before the corrected source's fresh release. No tag or
upload occurred in that failed attempt.

Platform's client package, facade feature/module and active release group are
removed. Only Platform, Runtime and Types remain active. The historical client
ledger and published versions remain; native Profiler is unchanged. Pending
Platform notes are 0.21.0; manifests remain 0.20.0 until release. Runtime/macros/
guard remain 0.19.0 and Types remains 0.18.0. Browser/npm delivery is unchanged.
See [the migration](../pocket-ic-client.md).

The donor is Platform `41688ce`, including the macOS mock-socket correction.
Tooling preserves licenses, bodies/tests and reconstruction hashes. Original
source and full Git bundles remain in the ordinary checkout's
`.git/issue-139-suppliers/` and recorded donor artifacts. Transferred native/Wasm
Clippy, library tests, management identity, source guards, package imports and
installed 16.1 snapshot restoration passed. Destination full release and current
native-host qualification must be reported separately from those focused checks.

Platform focused checks cover facade/driver Clippy, feature-policy selection and
refusals, canonical native/Wasm identities, benchmark imports, installed runtime
and Agent profiles, Timers composition graphs, locks, inventory/docs, pins and
formatting. Selector/receipt/recovery and three-group preparation/publication
fixtures pass with substitute product stages. The full release gate for the final registry supplier is the next delivery step. The root lock removes 404 lines
without package upgrades. No Platform normal/build graph selects Testkit or a
PocketIC client package.

## Release qualification and recovery

Platform and PocketIC 0.20.0 were published at `cc8214b` and `5c5338d` after
full gates and installed composition proofs; do not repeat those releases.
Platform's new three-group mapping intentionally retains the full release gate.
Its host allowlist is empty until new full/selected native qualification.
Historical #137 receipts remain intact: run 41625352854 qualified the prior
four-group mapping on Linux and macOS 15 Intel/ARM at Rust 1.99.1. Those receipts
do not activate the new mapping. See [release validation](../platform-release-validation.md).

#138's Account Serde-byte cut requires matching Wire/browser schemas, new RPC
contract IDs and consumer-owned persisted-data handling. #134's fallible/no-replay
client and #136's bounded snapshot transfer shipped in 0.19/0.20. Preserve accepted
operation IDs, bytes, limits, original crypto bodies and Types/Runtime identity.

The owned Agent replacement remains withdrawn; native Agent uses ic-agent 0.49.3.
Crypto-bigint advisory #50 remains open and unchanged. Scheduling qualification
remains pinned to the authorized Runtime 0.19 adoption `a61f3517`, without a
Scheduling merge or publication. Earlier release/recovery, native profiler and
withdrawn Agent evidence retain their custody in the
[previous handoff](https://github.com/dragginzgame/ic-platform/blob/41688ced81066b90aee756def7468a8388ee67e9/docs/status/current.md).
Sibling storage removals and routine downstream adoption remain maintainer-owned.
