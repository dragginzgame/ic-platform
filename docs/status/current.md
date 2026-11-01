# Current handoff

## Published independent Runtime batch — 2026-11-01

The maintainer authorized and completed publication of the #131 and retained
Principal/#116 batch through all three Rust release groups. Do not repeat uploads.

| Group | Published packages | Release commit and tag |
| --- | --- | --- |
| Types | ic-platform-types 0.16.0 | e20aa7b94e708ad8319426d854f019709fed3353 / types-v0.16.0 |
| Runtime | ic-platform-runtime, ic-platform-macros, ic-platform-runtime-guard 0.17.1 | f41cb50360542e4dc1bd504c802ffd30cd30b260 / runtime-v0.17.1 |
| Platform | ic-platform, ic-platform-pocket-ic 0.17.0 | 35cffb6f0ab768e00b0f11d6429eff4d7d7e252c / platform-v0.17.0 |

Every group passed its complete release gate and was atomically pushed. All six
registry archives match their published checksums and clean release VCS identities.
The final Platform/PocketIC archives also match Cargo's actual multi-package upload
files under `target/package/tmp-crate/` and `tmp-registry/` byte for byte.
Browser 0.1.1 is unchanged; no npm or sibling package publication occurred.

Current durable local receipts are under
`.git/release-state/platform-017-publication/`: recovered full final release log,
publication log/attempt, registry archives/index/VCS/check receipts and upload
SHA-256 values. Keep all three group release plans. Final Platform full CI,
32 consumer profiles, feature/Wasm/MSRV checks and both installed suites passed.
Installed owner/composition evidence remains in `target/cdk-runtime.htCxVr` and
`target/cdk-runtime.xp41SS`; prepared final graph is `target/cdk-runtime.27MHW2`.
This is local Linux qualification, not macOS acceptance.

## Failure recovery and evidence custody

The first Types attempts found stale admission mocks and a feature-policy gap.
Commits 1571fa3/eae1122 repair fixture Runtime ownership/#116 source receipts,
refuse unconditional facade links, and mutate the actual Runtime guard dependency.
Fresh complete gates passed before publication. The initial source batch is 3a72f6f.

The first Platform attempt failed with ENOSPC before version preparation or
upload. The user approved clearing only two incremental caches, but space became
available before that action was needed. This agent performed no cache deletion.
Separate cleanup subsequently removed `/tmp/platform-131` and the entire `target/`
while a fresh gate was running. The live log was recovered from its open descriptor
into `.git/release-state/`; validation rebuilt missing outputs and completed.
Some earlier copied failure logs were truncated by ENOSPC. Original temporary
consumer/archive receipts and the earlier local upload archives are no longer
available; do not describe those paths as retained or independently reinspectable.
Registry archives/checksum/VCS receipts were downloaded and verified again.

## Ownership and published contracts

[Platform #131](https://github.com/dragginzgame/ic-platform/issues/131) and the
[runtime contract](../migration.md#independent-runtime-owner-131) govern ownership.
Runtime owns IC0, the sole executor, CDK adapters and prepared calls. Facade paths
delegate to it. Runtime/macros/exact guard form one independent group; Types and
Platform remain separate. Empty defaults, Candid-free runtime profiles, protected
bodies/licenses and exact singleton refusal remain. Legacy embedded Platform
executors cannot coexist with the new Runtime owner. Runtime initially selected
unpublished 0.17.0 and first shipped at 0.17.1 through the ordinary patch workflow.

Types 0.16 removes registry Principal selection from self-authentication. Select
`principal-external` for conversions. The unchanged upstream SHA-224 derivation/tag
and source/license/vector guards remain. Platform 0.17 also ships the owned native
PocketIC client identity; Testkit retains lifecycle and policy. See
[the client contract](../pocket-ic-client.md). Crypto bodies/constants/vectors remain
guarded; Candid is the explicit external IC boundary with fixed 16-level admission.

#116 retains separate owner and Timers-composition installed suites and locks.
Each suite covers eight current Rust 1.99.1/MSRV 1.91 × Wire/external Candid ×
raw/prepared-call profiles, including traps, cancellation, rearm and upgrade.
The test-only Timers catalog still pins immutable adoption commit
036fd50c08c2c1d05ab1cb2381abc7b8f1a36103. Normal qualification stays locked/offline;
return it to a qualified registry release after adapter publication.

## Downstream adoption remains separate

User-authorized drafts are published as
[Scheduling #86](https://github.com/dragginzgame/ic-scheduling/pull/86) and
[Wire #41](https://github.com/dragginzgame/ic-wire/pull/41). Their isolated clones
were removed by the external cleanup; their branches are safe on GitHub.
Scheduling head is 690db36208daf68d03b3063700ed6c6bfcd3bdc1; Wire head is
ca0b10fc6e7e5fe94fa6cb6e2503a51407576080. Both incorporate then-current main.
Sibling working checkouts were untouched. Do not merge with stale supplier locks.

Reviewed Timers/Endpoint archives passed Clippy, three schema tests, Rust 1.91
optimized Wasm and an installed transaction test against registry Runtime 0.17.1,
Types 0.16.0 and Core/derive 0.13.5, without local Platform patches. That Wasm graph
had one Runtime owner and no facade/Candid. These results were observed before
cleanup; their temporary receipts were lost. Requalify changed adoption inputs.

Wire needs canonical Types 0.16 adoption in Core/identity fixtures and native-client
qualification, then its registry lock and affected gates. Scheduling's Jobs fixture
still selects Endpoint 0.3; refresh its complete registry lock after matching
Endpoint publication. The drafts record these obligations. No sibling release,
full adoption completion or macOS evidence is implied by Platform publication.

Shared snapshot 4c84efa9dd5f4b3ff7bdea89972e747fd394d406 (0.27.24) stays immutable.
Earlier contract/history pointers are in
[the prior handoff](https://github.com/dragginzgame/ic-platform/blob/35cffb6f0ab768e00b0f11d6429eff4d7d7e252c/docs/status/current.md),
but temporary evidence paths there may have been removed by the cleanup above.
