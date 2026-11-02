# Current handoff

## Platform delivery — 2026-11-02

Shared 0.27.27 at 651f5f7acb2e3055abd51ba5f835a2112801666a is adopted and
pushed. Optional group-plan mapping remains off; complete existing gates apply.
Source commits are 3920d12 (snapshot) and 543c22c (reviewed Rust batch).

| Delivered group | Version | Release commit |
| --- | --- | --- |
| Types | 0.17.0 | 42beb561942f33e5859afef95b8b35a2131cb530 |
| PocketIC | 0.18.0 | 42a6b5c7d987ee4c7e9c38a437df0950f9f210c2 |

Both passed full CI, independent owner and Timers installed suites, standard
release/tag/atomic push and Cargo publication. Official registry readback,
archive checksums and clean VCS identities match. Durable plans/logs/archives
are in `.git/release-state/platform-018-deployment/` and the normal group plans.
PocketIC now independently ships management DTOs 0.11 and Rust 1.99.1. Its new
implementation is available directly; published facade 0.17 keeps the old client.
Browser, simulator, other sibling releases and live IC acceptance are outside scope.

## Runtime and facade delivery blocker

Runtime 0.18 passed complete CI and both installed suites, then its prepared-graph
check correctly refused Timers' old Runtime-0.17-only source. No Runtime tag,
push or upload happened. Its nine generated metadata files were preserved under
`target/shared-02727-release/runtime-prepared-blocked/`, then restored to the
committed source to permit independent PocketIC delivery. The failed plan and
logs remain. Do not weaken the singleton or relabel this as a completed release.

The user authorized an isolated Scheduling source change. Commit
`1c57a6d9abfd432fc683c5016d4625763682483a` on
`adopt/runtime-018-platform-qualification` in
`target/shared-02727-release/scheduling` admits `>=0.17.1, <0.19`. Only the root
catalog and pending Timers notes change; timer bodies and its lock are identical.
All 175 tests pass on both lines, strict native/Wasm Clippy passes, and all four
installed Runtime-0.18/Timers profiles pass in the disposable candidate clone.
The exact guard and one resolved owner are retained.

**Awaiting explicit permission to push that Scheduling topic branch** so fresh
Platform qualification can fetch its immutable commit. Local commit permission
was granted; no Scheduling merge, release or crate publication is authorized.
After the branch push, adopt that Git pin and version in Platform's test-only
Timers catalog, refresh the affected actual composition lock, and qualify it.
Then commit the source fix and start a fresh Runtime minor release, followed by
publication/archive readback and the facade minor release/publication. Do not
resume the old attempt against changed source or repeat Types/PocketIC uploads.
The remaining controller must select only Runtime and Platform.

Blob Client and Testkit direct Types/Runtime changes are integrated in their
normal sibling trees; their own delivery remains owner-scoped. The mistaken
Tooling deployment prompt was withdrawn. Its small local guide/lint fixes remain.
Earlier focused evidence is in the old handoff; it never replaces complete gates.

## Pending issue batch

#125 limits docs-only CI while preserving native jobs. #132 bounds new Profiler
compiler scratch under assembly custody; existing artifacts and legacy caches
stay untouched. #133 selects Rust 1.99.1 across all six members and retains each
distinct native/Wasm/consumer/installed proof. #123 removes test-only assertions
with original-source reconstruction. Earlier focused results are retained in
`target/issue-125-132-133/` and `target/issue-repairs-20261102/`; they do not
replace this delivery's complete gates. No crypto advisory suppression was made.

## Pending owned Principal provider update — 2026-11-01

The maintainer requested dependency modernization for owned Principal, keeping
Candid's Principal external. Local Types changes select upstream SHA-2 0.11.1
without defaults; the original derivation, vectors and source fingerprint remain.
Certification/creation were tested but retained at SHA-2 0.10 because splitting
them from BLS added roughly 9 KB to verifier Wasm. Principal's own probe changes
by -386 bytes with size optimization and +784 bytes at opt-level 3.
See [the measured tradeoff](../benchmarks/upstream-2026-10-31/README.md#owned-principal-sha-2-provider-qualification-2026-11-01).
Evidence and the broader experimental patch are in `target/sha2-011/`.
No new release, publication or sibling edits are authorized by this batch.

The authorized BLS dependency trial is isolated in `target/bls-dependency-trial/`.
Manifest resolution passed; full Clippy failed on Digest, curve-trait and RNG API
changes. No BLS source/dependency changes were adopted and no size claim is made.
Newer ff/group alone cannot remove Digest 0.10: zkcrypto's newer release retains
it too. The benchmark notes above record the failed trial and upstream findings.
The maintainer chose to retain DFINITY's qualified BLS dependency versions;
do not pursue that migration merely to remove outdated-version notices.

## Published independent Runtime batch — 2026-11-01

The maintainer authorized and completed publication of the #131 and retained
Principal/#116 batch through all three Rust release groups. Do not repeat uploads.

| Group | Published packages | Release commit and tag |
| --- | --- | --- |
| Types | ic-platform-types 0.16.0 | e20aa7b94e708ad8319426d854f019709fed3353 / types-v0.16.0 |
| Runtime | ic-platform-runtime, ic-platform-macros, ic-platform-runtime-guard 0.17.1 | f41cb50360542e4dc1bd504c802ffd30cd30b260 / runtime-v0.17.1 |
| Platform | ic-platform, ic-platform-pocket-ic 0.17.0 | 35cffb6f0ab768e00b0f11d6429eff4d7d7e252c / platform-v0.17.0 |

All six archives passed full release gates and published checksum/VCS checks.
Browser 0.1.1 and sibling publications were unchanged. Keep the three original
release plans and `.git/release-state/platform-017-publication/` recovery receipts.
The first Platform attempt hit ENOSPC before preparation/upload. No approved
incremental deletion was needed; external cleanup later removed `target/` and
`/tmp/platform-131` during validation. The live log was recovered and full gates
completed. Earlier temporary consumer/archive evidence was lost; registry receipts
were downloaded and verified again. Do not claim deleted receipts remain available.
See the [published handoff](https://github.com/dragginzgame/ic-platform/blob/35cffb6f0ab768e00b0f11d6429eff4d7d7e252c/docs/status/current.md)
for detailed history. Linux qualification does not establish macOS acceptance.

## Ownership and downstream adoption

The [runtime contract](../migration.md#independent-runtime-owner-131) retains one
IC0/executor/CDK/prepared-call owner, empty defaults and exact singleton refusal.
Types 0.16 separates self-authentication from registry conversions. Crypto/source
guards and fixed 16-level external Candid admission remain unchanged.
#116 keeps separate owner and Timers installed suites/locks. The test-only Timers
catalog still pins 036fd50c08c2c1d05ab1cb2381abc7b8f1a36103; return it to a
qualified registry selection after adapter publication.


User-authorized drafts are published as
[Scheduling #86](https://github.com/dragginzgame/ic-scheduling/pull/86) and
[Wire #41](https://github.com/dragginzgame/ic-wire/pull/41). Their isolated clones
were removed by the external cleanup; their branches are safe on GitHub.
Scheduling head is 690db36208daf68d03b3063700ed6c6bfcd3bdc1; Wire head is
ca0b10fc6e7e5fe94fa6cb6e2503a51407576080. Both incorporate then-current main.
Sibling working checkouts were untouched. Do not merge with stale supplier locks.

Timers/Endpoint archive and installed proofs passed against published Runtime
0.17.1/Types 0.16/Core 0.13.5 before cleanup. Their temporary receipts were lost;
requalify changed inputs rather than claiming retained evidence.

Wire needs canonical Types 0.16 adoption in Core/identity fixtures and native-client
qualification, then its registry lock and affected gates. Scheduling's Jobs fixture
still selects Endpoint 0.3; refresh its complete registry lock after matching
Endpoint publication. The drafts record these obligations. No sibling release,
full adoption completion or macOS evidence is implied by Platform publication.

Shared snapshot payloads stay immutable; refresh only through the exporter.
