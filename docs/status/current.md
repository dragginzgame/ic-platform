# Current handoff

## Publication in progress — 2026-11-01

The maintainer authorized publishing the completed #131 and retained Principal/
#116 batch through all three existing Rust groups. Types and Runtime are complete;
Platform 0.17.0 is blocked by disk space. Do not repeat successful uploads.

| Group | Published packages | Release commit and tag |
| --- | --- | --- |
| Types | ic-platform-types 0.16.0 | e20aa7b94e708ad8319426d854f019709fed3353 / types-v0.16.0 |
| Runtime | ic-platform-runtime, ic-platform-macros, ic-platform-runtime-guard 0.17.1 | f41cb50360542e4dc1bd504c802ffd30cd30b260 / runtime-v0.17.1 |

Both groups passed their complete release gates and were atomically pushed.
Registry checksums, downloaded archives, uploaded bytes and clean VCS identities
match. Evidence: /tmp/platform-131/publish-types.log, publish-runtime.log,
publication-registry/, release-types-final.log and release-runtime.log.
Keep .git/release-state/types-0.16.0.plan and runtime-0.17.1.plan.

Platform/facade and the owned PocketIC client remain at source 0.16.0, pending
0.17.0. No platform-v0.17.0 tag or upload was dispatched. The final release gate
passed recovery, workspace tests and docs, then consumer compilation failed with
ENOSPC. The filesystem had about 30 MB free. Original logs remain in
/tmp/platform-131/release-platform.log, /tmp/validation.gZuHsf/15.log and
/tmp/validation.Xki6iE/0.log; disk-full writes truncated some copied/combined logs.
Do not claim this attempt passed or erase its evidence.

A permission question is pending to remove only inactive incremental caches
`target/debug/incremental` (9.8 GB) and `target/consumer-msrv/debug/incremental`
(3.6 GB). No cleanup is authorized until the maintainer answers. Recheck active
builds before any approved cleanup. Preserve package archives, Wasms and receipts.
Then start a fresh `make release-minor RELEASE_GROUP=platform`, followed by
`make publish RELEASE_GROUP=platform` and registry verification. The existing
full gates remain mandatory. The prepared server is
/home/adam/.cache/canic/pocket-ic-server-16.0.0-pocket-ic-x86_64-linux/pocket-ic.

## Independent Runtime and Principal contracts

[Platform #131](https://github.com/dragginzgame/ic-platform/issues/131) and the
[runtime contract](../migration.md#independent-runtime-owner-131) govern ownership.
Runtime owns IC0, the sole executor, CDK adapters and prepared calls. Facade paths
delegate to it. Runtime/macros/exact guard form one independent group; Types and
Platform have separate groups. Empty defaults, protected bodies and licenses,
Candid-free runtime profiles and exact singleton refusal remain. Runtime starts
at published 0.17.1 through the ordinary patch workflow from unpublished 0.17.0.
Legacy embedded Platform executors cannot coexist with the new Runtime owner.

Types 0.16 removes registry Principal selection from self-authentication. Select
`principal-external` for conversions. The exact upstream ic_principal 0.1.5
SHA-224 derivation/tag and source/license/vector guards remain unchanged. Earlier
Auth qualification: /tmp/platform-pr-merges-20261101/principal-*.log and auth-principal.*.
The initial source batch and two release-fixture repairs are committed in
3a72f6f, 1571fa3 and eae1122. Admission mocks now model Runtime and #116 receipts;
feature policy refuses facade links and mutates the actual Runtime guard edge.
Failed initial gates remain in release-types.log and release-types-fixed.log.
Corrected consumers, features/Wasm/MSRV and installed preflight evidence remain in
consumer-release-fixtures-final.log, remaining-release-gates.log and
installed-release-preflight.log beneath /tmp/platform-131.

#116 retains independent owner and Timers-composition installed suites and locks.
Full source qualification passes all eight profiles in each suite, current Rust
1.99.1/MSRV 1.91, Wire/external Candid and raw/prepared calls. Export sets exactly
match the pre-extraction baseline; no executor behavior or performance claim.
The successful Types/Runtime release logs contain fresh complete suite results.
The test-only Timers catalog still pins immutable adoption commit
036fd50c08c2c1d05ab1cb2381abc7b8f1a36103; normal validation is locked/offline.
Return it to a qualified registry release after downstream adapter publication.

## Downstream drafts and registry-supplier proof

User-authorized isolated branches are published as
[Scheduling #86](https://github.com/dragginzgame/ic-scheduling/pull/86) and
[Wire #41](https://github.com/dragginzgame/ic-wire/pull/41). They select Runtime
directly and remain drafts. Current mains were merged without changing the
reviewed adoption diffs: Scheduling head 690db36208daf68d03b3063700ed6c6bfcd3bdc1,
Wire head ca0b10fc6e7e5fe94fa6cb6e2503a51407576080. Sibling working checkouts were
untouched; clones are /tmp/platform-131/scheduling and /tmp/platform-131/wire.

Reviewed normalized Timers/Endpoint archives now qualify against published
Runtime/macros/guard 0.17.1 and Types 0.16.0 without local Platform patches.
Clippy, three Endpoint schema tests, Rust 1.91 optimized Wasm and the installed
transaction test pass. The Wasm graph excludes facade/Candid and has one Runtime
owner. Registry Core/derive 0.13.5 are selected. Evidence:
/tmp/platform-131/registry-adapters/, including metadata, graph and installed.log.
This is Linux archived-adapter/published-supplier evidence, not sibling publication.

Wire still needs canonical Types 0.16 adoption in Core/identity fixtures and the
native client boundary qualified, then its registry lock and affected gates.
Scheduling's Jobs fixture selects old Endpoint 0.3, so its complete registry lock
waits for matching Endpoint publication. Draft bodies record these obligations.
Earlier identity simulations used unpublished Types 0.15.1 source; do not relabel
them as current registry acceptance. No sibling release or macOS acceptance is claimed.

## Retained boundaries and earlier evidence

Shared snapshot 4c84efa9dd5f4b3ff7bdea89972e747fd394d406 (0.27.24) is immutable.
Platform #126/#128 and Shared #332 are merged. The owned PocketIC client has a
breaking identity change pending this Platform release; Testkit retains lifecycle
and policy. See [the client contract](../pocket-ic-client.md). Crypto bodies,
constants, vectors and provenance remain guarded. Candid remains the required
external IC boundary with fixed 16-level admission; storage remains consumer-owned.
Earlier release/native-provider history and evidence locations are retained in
[the prior handoff](https://github.com/dragginzgame/ic-platform/blob/f41cb50360542e4dc1bd504c802ffd30cd30b260/docs/status/current.md).
