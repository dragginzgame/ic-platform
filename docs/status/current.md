# Current handoff — 2026-11-08

## Root-owned tooling migration — Git delivery

The working tree selects crates.io **toko-tooling 0.34.0** through root
`[workspace.metadata.toko-tooling]`. The separate `toko-tooling.lock` records
archive checksum 5ca8ad09ad9359bc4945a4afdaafce301dd5a4b4955cd5c9f5058d7a54ffcf10.
The new installer in `/home/adam/projects/toko-tooling` authenticated and installed
the registry archive, retaining all **74 entrypoints** and **156 payloads**.
Tool setup/checks completed; no snapshot-owned file was edited manually.

The superseded dependency-only member, dependency/Machete declarations and all
membership/policy references are removed. Cargo.lock was refreshed offline:
only the two obsolete tooling records were removed; all **396 retained records**,
including the independent supplier draft below, are unchanged. There are five
real members and the same three groups; all public versions are unchanged.
Use `make shared-tooling-update` after intentionally changing the exact root
version; `make shared-tooling-install` reinstalls the dedicated tooling lock.
Application Cargo updates no longer select or install tooling.

Custom publication, metadata preparation and frozen setup now admit tooling
before effects. Shared pre-capture cleanup is also ordered behind admission,
including parallel Make. Docs-only CI explicitly prepares and caches host parsers
before offline admission, retaining the native/full qualification selections.
The pending Platform **0.27.1** note records internal tooling changes only.
The authorized delivery is Git-only to origin/main: public crate sources,
manifests and dependency selections are unchanged from their published owners.
Package inventories and publication-impact evidence are under
target/deliver-toko034-20261108. No package bump, tag or upload is selected.
The independent Cargo.lock draft remains excluded from this delivery.

Focused Linux evidence is under target/adoption-toko034-20261108: snapshot/status,
tool admission, pins/inventory/documentation, formatting/hook and CI-selection
callers, custom-effect refusal, release routing and Platform-only preparation/
rollback/retry. The existing full **20-stage** product gate and native-host
obligations remain. No full gate or new macOS acceptance is claimed. Historical
adoption evidence remains under target/adoption-toko033-20261108.

## Delivered Platform 0.27.0

Platform **0.27.0** is on crates.io and origin/main, with annotated tag
`platform-v0.27.0` at a8165b91e0e04f65962bad5324f5db31e312c580. Reviewed source
689cf1495120c685d51af70ea7f4c77b8fb317e6 was delivered through the standard frozen
batch **batch-O3V4GL4T**. Its schema-2 journal is complete at index 1; the normal
branch was fast-forwarded using `release-batch-reconcile`. No batch was retired.

Only Platform was released. Runtime/macros/guard remain **0.20.2**, Types
**0.18.2**, browser **0.2.0**. No sibling source, release or npm package changed.
The prior Runtime release/tag remains bb76a069da4d442e1e830636bace9760fbebd880,
`runtime-v0.20.2`; its earlier qualification and custody remain preserved.

All **20/20** required local stages passed on Linux / Rust 1.99.1, including
full Clippy/tests, consumer/feature/Wasm graphs and installed owner/Timers raw and
prepared Wire/external-Candid profiles. The new raw_rand leaf additionally checks
real management replies, structured rejection, callback-trap cleanup, pending-call
cancellation and subsequent successful calls against prepared PocketIC 16.1.0.
The immutable complete-gate receipt is:

- target/release-frozen/batch-O3V4GL4T/target/release-validation/qualification.onAEri/receipt.json

Official index checksum, downloaded archive, retained publication archives and
exact VCS commit agree; GitHub branch and peeled tag matched. Archive SHA-256:
33e040a299213e54bc4a37142a9d164a7daaf74181fd5520bba57fd7acb3a2cd.
Evidence: target/deploy-platform0270-20261108/registry-verification.json,
github-refs.txt, release.log, reconcile.log and draft-restoration.json.

## Published consumer boundary

Select `raw-rand` for `raw_rand::{METHOD, encode, decode, DecodeError}`.
The allocation-free decoder admits the documented management-producer profile,
with at most 74 bytes, bounded padded LEB128 and exactly one 32-byte blob.
All-zero data remains valid; seed admission, transport, waits and retries stay
consumer-owned. No runtime or cryptographic implementation body changed.

The duplicate `candid::protocols::management::raw_rand` helper is removed without
an alias. This is the 0.27 breaking change. Generic Runtime/CDK/Agent transports
remain method-agnostic; there is one typed entropy codec, not a method-name ban.
[#151 — Candid-free raw_rand codec](https://github.com/dragginzgame/ic-platform/issues/151)
is implemented/closed; [Canic #749 — Adopt the codec](https://github.com/dragginzgame/canic/issues/749)
owns separate consumer adoption. Current APIs and migration are in the
[migration contract](../migration.md).

Small external argument encoders are `candid_args::{empty, units, text}`, under
`candid-args`. Raw_rand and existing empty protocol helpers share `empty()`.
These deliberately cover only empty, unit/null and single-text arguments, with
independent Candid byte checks including Unicode and LEB length boundaries.
The standalone native/Wasm feature graphs contain only Platform. General and
native textual parsing remain upstream; owned application protocols use Wire.

The frozen Candid 0.10.39 entropy comparison reports total/code bytes
282719/255413 for the general codec and 45056/38114 for the narrow codec, with
opt-level 3 / fat LTO / one codegen unit and ic-wasm 0.11.1 shrink. Retained
inputs/artifacts/status: target/release-frozen/batch-O3V4GL4T/target/evidence/raw-rand.n286hl.
The earlier Candid 0.10.40 comparison remains at target/evidence/raw-rand.5fgoQi.
Savings require removing the last reachable general codec; sibling Wasms remain
unmeasured. These experiments have different dependency inputs.

## Baseline and asynchronous hosted evidence

The published release used Shared **0.31.12** at
eb3c8e0d2274d2c7babb2c15b3355bb80a796f0f, with 149 verified payloads.
Adoption ced3903 and LOC evidence remain at target/adoption-shared03112-20261108.
The later, uncommitted registry migration is described above.

One observation at release commit a8165b9 found Browser SDK and Wasm size workflows
successful; CI, installed runtime and dependency advisories were still running.
See target/deploy-platform0270-20261108/hosted-runs.json. Leave expensive hosted
integration/native checks asynchronous. No new macOS, browser/worker or native
Profiler acceptance is claimed by Linux qualification.

Earlier hosted validation failed in grouped preparation and Testkit server setup;
the current complete local gate passed both, but does not establish remote fixes.
Historical secret scanning flags documented public test keys and a prose false
positive; no exclusion or waiver was added. The crypto advisory remains explicitly
deferred under [#50 — Resolve RUSTSEC-2026-0387](https://github.com/dragginzgame/ic-platform/issues/50),
unimplemented; do not take ownership of cryptographic bodies.

The controlled Candid/derive 0.10.39→0.10.40 experiment remains a measured next
investigation: total/code 534627/481743→589215/536162 (+54588/+54419), fixed source,
compiler and optimization. Preserve sequence-completion checks; exact symbol
attribution is unprofiled. Evidence: target/next-wasm-20261108/candid-patch-comparison.json.
It is a dependency experiment, not a same-lock implementation comparison.

## Deferred draft and recovery custody

The independent dependency draft is retained at
refs/retained/platform0270-lock-draft-20261108, commit
588b11a38c60b15ab5af19f30aef1c310a8b0975, original lock SHA-256
99591563adb27ffadd705cf60456098506788cdcbb8eebb07a92a5db7f5c0ce9.
It reappeared in the normal checkout during frozen qualification and matched
that saved draft exactly. Only its supplier patch was reapplied after the standard
reconciliation: all **391 external records** match, while the five local owners
match the released versions. Before the registry migration its restored SHA-256 was
98b7f888a38c062420ef54c310c6a1e8b7df2b66ed5d9c9b0bb9b2a421e5438a.
The supplier draft remains uncommitted alongside the new tooling selection.
Never replace released owner metadata with an entire older lock. Earlier retained
refs, batches and receipts remain intact.

Preserve all historical failed/completed batches, including batch-_leIreCI,
batch-C5BoFKA5 and batch-AaHi2pgO; use each attempt's supplying journals/tooling.
Historical archives/bundles/receipts remain in
recovery-evidence/retired-checkouts/20261104-issue139/, with maintainer review by
**2026-12-04**. No cleanup, mtime renewal or retirement was authorized. Previously
removed target evidence and Runtime 0.18.1 binaries must not be claimed present.
The [preceding handoff](https://github.com/dragginzgame/ic-platform/blob/a8165b91e0e04f65962bad5324f5db31e312c580/docs/status/current.md)
retains older release identities, receipts and focused evidence.
