# Current handoff — 2026-11-08

## Deployment in progress — Runtime 0.20.2, Platform 0.26.1

The maintainer authorized delivery of the reviewed simplification/audit batch.
The selected frozen cohort is runtime,platform with patch increments, publishing
to crates.io and origin/main with runtime-v0.20.2 and platform-v0.26.1 tags.
Types and browser are excluded. The standard complete local gate is required;
hosted/native CI remains asynchronous unless an enforced check requires it.

The exact incoming Cargo.lock draft is durably retained at
refs/retained/platform0261-runtime0202-lock-draft-20261108, commit
b91e65b11dae893e510e1d459718903594840768, with SHA-256
d47ee5c95cd219e49a36e45a890596fdc810a39ad5f87f204080d2ee28f87e00.
It is excluded from release source and will be reapplied over delivered owner
metadata after reconciliation. Evidence: target/deploy-runtime0202-platform0261-20261108.

## Qualified implementation and Shared 0.31.11

The canonical exporter adopted **0.31.11** at
`6c529e6940a97226cca826726491e03173b08cd2`: 149 verified payloads, including the
newly selected common-tools action and its cache-key helper. AGENTS names the new
baseline. The work below was qualified locally before deployment; the incoming
lock draft is now retained separately as described above.

The second ten-item simplification batch is implemented. Owned Candid admission
and signing-time errors use the existing thiserror supplier; variants, exact
messages and concrete source identity are tested. No nesting/verification/DER/time
algorithm bodies changed. The facade explicitly selects that dependency for the
two affected features; Types already brings the same supplier. Runtime, Types,
macros' generator source and browser public APIs are unchanged.

Each release-adapter operation reuses one admitted group plan, with fresh later
operations. Generated fixtures share one root catalog observation and policy
fixtures use the admitted compiler floor. Graph classifications are centralized
without merging actual observations. Only full Platform CI uses Shared's common
tool setup action; narrow host jobs retain their own setup.

One Wasm reader serves TSV and Canic JSON reports. Dated 0.4/0.22 orchestration
is retired with immutable reproduction references; current read-only capture and
reporting live under scripts/benchmarks/canic-wasm. Macro-test rendering and
ordered assertions share helpers. A Rust/browser response corpus keeps SDK tagged
and Rust untagged CBOR framing distinct, preserving payloads and encoder bytes.

Focused Linux / Rust 1.99.1 evidence: target/complexity-second-20261108. Affected
facade/macros Clippy passes; 55 facade and 16 macro tests pass (one manual benchmark
ignored). Browser build and seven ingress/corpus tests, five Wasm comparison/reader
tests, external Candid/signature consumer checks on native/Wasm, and generated
raw/prepared Wire/Candid graph checks pass. Synthetic feature/graph admission,
compiler-floor/catalog refusals, concurrent artifact custody, grouped projection
and three-group disposable preparation/recovery are checked separately. Product
release stages in disposable policy tests are substitutes, not release gates.
That second batch did not run a new full release gate, installed simulator, macOS
or native Profiler acceptance. Integrity/docs/format results accompany its evidence.

The follow-up audit's ten fixes are also implemented locally. Canic capture
checks copied hashes, fresh source rosters and Git identity. Runtime captures a
verified input snapshot before deriving/copying fixtures, then checks both the
snapshot and fresh original roster/bytes. Explicit lock refresh exempts only the
selected original lock. Canonical dependency paths remain; source writers must
still be stopped. Wasm reporting uses one byte snapshot. Runtime and benchmarks
share artifact retention; callback Clippy/test failures stop before build/copy.
Feature metadata is per-attempt. Formatting and independence fixtures reuse
completion admission, and Agent qualification uses the catalog/compiler readers.
Native protocol fixtures reject over-budget bytes/text before cloning; encoded
bytes and errors remain. Shared corpus tests require all four operation/producer
pairs. Agent/dynamic consumer tests reuse existing hex support.

Follow-up evidence: target/complexity-third-20261108. Affected facade Clippy,
47 library tests (one manual benchmark ignored), five Agent and two dynamic tests,
seven browser corpus tests, nine benchmark tests and independent Agent/dynamic
consumer proofs pass. Snapshot addition/edit/corruption and narrow lock-refresh
refusals, failed-Clippy propagation, concurrent artifact custody, completion-aware
formatting/hook checks and compiler-reader refusals pass. Concurrent bounded
Principal/executor feature checks retain separate observations. One disposable
Platform metadata preparation/rollback check passes with product stages substituted.
Real owner build: target/cdk-runtime.cMXklk; its six native driver tests and four
raw/prepared Wire/Candid Wasms pass. Installed Agent acceptance at
target/agent-installed.ZuXBBz passes verified queries, update-and-wait, read-state
and wrong-root refusal for Ed25519, delegated, P-256 and secp256k1 identities.
No new installed Runtime/Timers, full release gate, macOS or native Profiler
acceptance is claimed. Cryptographic bodies, identities and verification policy
are unchanged. Source/snapshot/lint/format/doc checks pass; all three lock hashes
match the incoming draft and qualification locks.

The owned error source and facade feature declarations change published Platform
inputs. Macro-test helpers also belong to the macros package's src/** inclusion;
the actual package list confirms src/export/tests/mod.rs is shipped. Under
Shared's package-content rule, later publication planning includes Platform
0.26.1 and the coordinated Runtime 0.20.2 group, despite unchanged generator and
Runtime behavior. Both pending ledgers record the work for the selected release.
Types is untouched and browser tests are outside its npm package file selection.

The new issue-closure policy distinguishes completed local implementation/focused
checks from delivery; genuinely issue-specific external acceptance still applies.
GitHub review found only [#50](https://github.com/dragginzgame/ic-platform/issues/50)
open. Its crypto/Agent correction is unimplemented and explicitly deferred, so it
remains open. No issue/comment was changed and no fresh advisory audit is claimed.

The governance-only adoption evidence remains at
`target/adoption-shared03111-20261108/`; the current combined batch adds the
source/workflow checks above. Sibling checkouts and releases remain untouched.

## Delivered maintenance and published baseline

The previous maintenance batch was pushed to origin/main as
9880ba816461f777266a0aedd54763f412af1522. Shared 0.31.10 and the first tooling/fixture
simplifications were delivered Git-only, with no new tag/upload. Its detailed
scope, installed owner/Timers results and asynchronous hosted observations remain
in the [delivered handoff](https://github.com/dragginzgame/ic-platform/blob/9880ba816461f777266a0aedd54763f412af1522/docs/status/current.md).
Evidence: target/deployment-maintenance-20261108 and target/complexity-20261108.
Those hosted outcomes have not been polled again here.

Published versions remain Platform **0.26.0**, Runtime/macros/guard **0.20.1**,
Types **0.18.2**, browser **0.2.0**. Platform's published source/tag is
41a128751528f8ccd3c926cff550dd8b2ea62926 / platform-v0.26.0; Runtime's is
206cc5990c330cafbe15f18fc7fa5b1b6a31093b / runtime-v0.20.1. Both passed 20/20 Linux /
Rust 1.99.1 stages and final archive checks. Completed schema-2 journals remain
under .git/release-state/frozen: Runtime batch-_leIreCI and Platform batch-C5BoFKA5;
neither batch was retired. Their receipts remain at:

- target/release-frozen/batch-_leIreCI/target/release-validation/qualification.xynaOV/receipt.json
- target/release-frozen/batch-C5BoFKA5/target/release-validation/qualification.aaSiWQ/receipt.json

Registry/archive/VCS/draft reconciliation evidence remains in
target/deployment-platform026-runtime0201-20261108. Current ownership is in the
[migration contract](../migration.md); earlier investigations are
[historical](../history/migrations-through-0.26.md). Retired Candid prototypes keep
[immutable reproduction references](../../scripts/benchmarks/0.9/README.md).

## Deferred draft and recovery custody

Cargo.lock's SHA-256 remains
`d47ee5c95cd219e49a36e45a890596fdc810a39ad5f87f204080d2ee28f87e00`.
All 391 external supplier records match the preserved draft. Ref
`retained/platform026-runtime0201-lock-draft` points at
`24c865191d14c628b86e2cc7307988827c3f1e58`; never restore its whole older lock over
published owner metadata. Both qualification locks are unchanged.

Preserve all historical failed/completed batches, draft refs and interrupted
0.23.1 state. Use their supplying source/journals; never reinterpret old receipts
under new tooling. Archives/bundles/receipts remain at
`recovery-evidence/retired-checkouts/20261104-issue139/`, with maintainer review
by **2026-12-04**. Retention refuses excess/expiry without deletion. No cleanup,
mtime renewal or retirement is authorized. Previously removed target evidence
and Runtime 0.18.1 binaries must not be claimed present. #50's deferral adds no
advisory waiver or cryptographic rewrite; sibling sources/releases stay untouched.
