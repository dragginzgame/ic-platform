# Current handoff — 2026-11-04

## Local passive-contract audit — pending Types 0.18.1

The [boundary audit](../audits/passive-contract-boundaries-2026-11-04.md) separates
completed Platform work from required Canic/Blob/IC Crates migrations. Three
release groups and their ownership remain unchanged. No sibling source or release
was changed; hashes of their observed Rust/manifests/locks remained identical.

One real gap is corrected locally: Types `principal-external` now selects upstream
Principal Serde explicitly. Without this, passive ExternalPrincipal DTO fields
borrowed Serde activation from Candid. The new regression failed with E0277 before
the fix and passes afterward across all principal lengths, Postcard/CBOR/JSON and
conversion bytes. No Candid, runtime, hashing or new packages enter Types. The
facade's existing forwarding is sufficient. Package versions remain unchanged;
the pending correction must be published before registry-only external-Serde
consumer adoption. Existing facade consumers already selecting Candid are unchanged.

Focused Types/facade Clippy, 16 Types tests, three identity-consumer and six Candid
tests pass. Native/Wasm identity package sets remain unchanged and exclude
codecs/execution/crypto. Feature-contract mutations and Principal source guards
pass. Read-only consumer checks pass: Canic u128 overflow and bounded-magnitude
canonicality, Blob's independent Caffeine request vector, and IC Crates' gateway
Candid records. Evidence: `target/passive-contract-audit/`. No full CI, fleet
matrix, sibling migration or release was run for this audit.

## Delivered Platform 0.22.0 and Runtime 0.19.1

Platform **0.22.0** is pushed and published from
`b95c9a2e6f4ecfbd25fc2f8466b3ca5826df550f`, tagged `platform-v0.22.0`.
Runtime, macros and the exact guard **0.19.1** are pushed and published from
`74a5071a7b0fdb6470ceb356d1891510ca8d16b1`, tagged `runtime-v0.19.1`.
Types remains **0.18.0**. The maintainer's “publish” means both GitHub main/tags
and crates.io. No sibling source/adoption, browser/npm or native-provider release
is included. Hosted push CI is separate from the local Linux release results.

The crypto reversal removes embedded BLS curve/verifier, certification trees and
canister-signature creation/verification bodies, algorithm suites/vectors and
four reconstruction guards. Registry suppliers own these implementations.
Platform keeps owned issuer fields, delegating key APIs, DER framing, root and
delegated identity-key admission, and authenticated signing time. Certificate/tree,
BLS Signature and BLS error types have registry identity; native Agent shares
certificates directly. Upstream signature packaging restores transitive Candid/
printer, raw IC0, CBOR, hashing and a lazy root cache. Key admission/time extraction
repeat CBOR decoding. No Wasm-size or performance gain is claimed.

Shared **0.29.3** at `afccb9947d26bbae13d4c4a143d2fd5f0c2c07a4`, owned test-layout
changes, the incoming Pest update and #139 documentation corrections are committed.
Source batch: `1264e76`; qualification-lock refresh: `a4683d6`. Runtime's patch
contains its own test-layout work; the facade's crypto cut does not change Types.
Owner/Timers locks select compatible Testkit **0.84.1**, retaining its registry
Platform 0.21 DTO adapter separately from the local facade.

Both full Linux release gates passed all twenty required stages: source/dependency
checks, formatting, Clippy, browser, recovery, tests/docs, consumers, native/Wasm
features and installed owner/Timers raw/prepared-call × Wire/Candid profiles.
Prepared-version composition graphs and publication's separate lint/archive
verification passed. Runtime's receipt is
`target/release-validation/qualification.vfRxyF/receipt.json`; Platform's is
`target/release-validation/qualification.rWavht/receipt.json`.
Publication selections: `target/publication/publish.selection.1jdY3S` (Runtime)
and `target/publication/publish.selection.H2GEEB` (Platform). Cargo confirmed
registry visibility; its refreshed index checksums match all four qualified
archives. Verification: `target/crypto-reversal-publication/registry-archive-verification.txt`.
GitHub refs were reconciled against the local tags.

Retained failures: the initial gate `qualification.fKwyRr` reached installed
Runtime without a configured server. Explicit lock refresh fixed subsequent
stale fixture selections. All eight installed profiles then passed. GitHub's
first Platform atomic push returned HTTP 500; remote reconciliation showed no
partial delivery, and the saved exact-version resume succeeded without retagging.
Logs: `target/runtime0191-{release-qualified,publish}.log` and
`target/platform0220-{release,release-resume,publish}.log`.

For future installed qualification, select
`POCKET_IC_BIN=$PWD/.tools/ic-testkit-server/16.1.0-x86_64-linux/pocket-ic`.
Testkit admitted this prepared PocketIC **16.1** bundle; executable SHA-256:
`b44e1eccd66e02328146b3e209e3403e3a3428a25d25555aca2f67500e5ae9db`.
Admission evidence: `target/pocket-ic1610-admission.log`.

LOC: **6,876** runtime-classified and **6,093** separate test Rust code lines,
down from 20,614/7,188 before the reversal. Counts exclude comments/blanks and
still classify a few imported tests with runtime files. Evidence:
`target/shared0293-loc-after-signature-removal.json`. Focused development evidence
remains under `target/shared0293-*`; no implementation is left only in those files.

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
