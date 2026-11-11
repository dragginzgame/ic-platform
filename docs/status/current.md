# Current handoff — 2026-11-11

## Published state

Installed Toko is 0.39.10; compiler is 1.99.1. The previous deployment is
complete on crates.io and GitHub, and the normal checkout is reconciled:

| Group | Version | Commit | Tag | Frozen batch / local receipt |
|---|---|---|---|---|
| Types | 0.19.2 | `b404696337ef2b1e5a1eb4f6d6404517c4e48ff7` | `types-v0.19.2` | `batch-Y0aGfmfa` / `qualification.YHc22v` |
| Platform | 0.30.0 | `80e600137bb3ab9b1d231ff39473a3621587abed` | `platform-v0.30.0` | `batch-VMBGwxm7` / `qualification.6v1hn9` |

Both registry versions are available and not yanked. Each group passed all 19
local gate stages, including Clippy, workspace tests, consumer/feature graphs,
Wasm builds and installed Runtime qualification; publication verified each
package archive. Different increments required separate sequential frozen batches.
Runtime/macros/guard remain 0.21.1; browser remains 0.2.0. No npm or sibling
publication was performed. Hosted Linux/macOS/browser checks are asynchronous;
this local Linux evidence does not establish macOS or real-browser acceptance.

## Delivered changes and boundaries

Deployment now covers this repository only: Types 0.19.3 followed by Platform
0.31.0, with Runtime/browser unchanged. The maintainer excluded Testkit delivery.
Published Toko 0.39.10 is adopted in both workspaces and the installed snapshot;
its Digest-policy correction passes the actual boundary gate. The maintained
runtime fixture lock was refreshed offline without supplier upgrades. Platform's
native fixtures still use Testkit 0.92; their evidence does not establish adoption
of Tooling's newer client. No sibling source or release effects are included.

#156 is implemented locally and closed on GitHub: `candid_record!`/`candid_variant!` accept concrete
struct/enum declarations, eliminating duplicated field types while retaining
existing mapping forms. New forms preserve Rust attributes/visibility and check
numeric label order/uniqueness at compile time. Native/Wasm standalone Clippy and
consumer checks plus 23 focused tests/doctests pass, covering oracle/frozen bytes,
optional fields, large Nat, recursion, callbacks and bounded refusals. The consumer
leaf is wired into `check-consumer`. Query migration/publication are not performed;
#50 remains deferred at the maintainer's explicit request.

The qualified BLS wrapper is adopted locally into the facade, with its experimental
implementation package removed. Signatures/errors now have Platform identity;
bytes and error variants are unchanged. External curve/hash primitives remain.
Certificate verification uses the same checked BLS boundary without duplicate key
parsing. Native Clippy and 39 tests pass, as do matched Wasm/installed BLS and full
signature probes. The adopted BLS probe retains the 13.88% one-shot / 7.65% cached
instruction reductions and 2,643-byte optimized saving against published 0.30.0.
See [BLS qualification](../design/bls-wrapper/qualification.md).
The previously blocking direct-Digest policy is corrected by the adopted
published Toko 0.39.10 payload. No snapshot files were hand-edited or gates waived.

The follow-up Wasm harness changes are local, not committed or deployed. Explicit
deployment mode retains each source lock and reports dependency/feature changes;
strict mode remains available. Both enforce the existing size allowance and
identical probe/compiler/settings. Twelve focused harness tests pass. Comparing
the whole deployment to `a8c3fff` found four unchanged probes, but self-authenticating
Principal grew 954 optimized total bytes / 698 code-section bytes and exceeded
the 128-byte allowance. No waiver or crypto change was made. The default probe
selection now includes that capability. See the
[measurement report](../benchmarks/platform-030-deployment.md) for absolute sizes,
raw sizes, input identities and dependency attribution limits. Follow-up controlled
builds attribute 928 total / 672 code bytes to the SHA-2 stack upgrade. The local
Types framing fix constructs the completed identity directly, saving 197 total/code
bytes with no growth in the combined ledger probe; identical-graph comparison
with zero allowance, native/Wasm Clippy and the fixed-byte argument-form regression
pass. Helpers with forced sharing were rejected because they grew ledger Wasm.
SHA-2 stays at 0.11.1; +757 total / +501 code bytes remain against the old deployment,
with no threshold waiver. No new versions have been published.

Types' published identity contract and fixed regression are recorded in
[identity qualification](../design/identity-qualification.md). The maintainer
declined an upstream self-authentication comparison suite.

Platform 0.30.0 removes textual DID export, inspection, parsing, fixtures and
`candid_parser`, including the service module and validation target. #154/#155
were closed as **not planned**, superseded by this explicit retirement. There
is no declaration-Wasm or native DID inspector. Canic adoption remains its owner’s
responsibility; no sibling implementation was edited or qualified here.

Independent binary schemas replace the retired text fixtures: all 110 captured
schemas passed structural comparison before retirement. Binary Candid bytes and
external protocol contracts remain unchanged; 34 focused comparison/consumer
tests also passed before the full release gates. Cargo pruned 71 packages without
unrelated supplier upgrades. `binrw` is an already-selected test-only oracle.

The all-feature Wasm normal/build graph excludes upstream Candid, candid_derive,
candid_parser and pretty. Upstream binary Candid remains in independent dev
oracles and native Agent/IC transport. Browser `/candid` remains SDK Core's binary
IDL adapter for required external interfaces. These active capabilities were
explicitly disclosed before deployment; they are not textual DID leftovers.
Generic didc entries remain only in immutable Toko installer/catalog payloads;
no Platform caller selects them. Do not hand-edit those payloads.

Toko 0.39.9 updates the dependency-policy references and direct IC checker. Its
96 installed paths pass snapshot verification. Shared admits native-only PocketIC
dev-dependencies; Platform retains its stricter local prohibition. The adoption
added no simulator dependency. Preserve empty feature defaults, canonical Types,
owned binary codec budgets, external cryptographic primitives and external bytes.

## Prior work and evidence

The independent source replacements and earlier deployment are detailed in the
[pre-deployment handoff](https://github.com/dragginzgame/ic-platform/blob/4c46f64dd393ba8d74424f07086f3098ac67dce7/docs/status/current.md)
and [prior source-retirement handoff](../history/source-retirement-handoff-2026-11-10.md).
Independent authors received reviewed specifications and black-box observations,
not original bodies/tests; parent integration read source. This is engineering
evidence, not legal clearance. Cryptographic primitives remain upstream-owned.

Current compatibility contracts live under `docs/design`: identity, executor,
System API/call adapters, generator, management DTOs, ICRC adapters/values and
ledger composition. Preserve frozen comparisons and installed checks. Ledger
composition matched all 782 frozen observations. Empty-account and Int Serde
regressions are fixed. The separate signature and codec qualification records
remain authoritative; do not conflate infrastructure owners with tested code.

Completed prior batch `batch-nUsSSjzc` published Types 0.19.1, Runtime 0.21.1 and
Platform 0.29.3. The later CI-only archive-tool lookup and exact non-secret
Gitleaks exceptions are retained. No downstream Timers qualification belongs in
Platform; Scheduling #91 owns composition. Wire/Tooling canonical identity
adoption remains consumer-owned. #50 remains deferred by the maintainer.

## Recovery custody

Keep both new completed frozen batches, their Git journals, candidate archives
and qualification receipts. Durable records live in `.git/release-state/frozen/`;
candidates/evidence live under `target/release-frozen/`. Do not resume old batches
over published versions. Retirement/deletion is not part of deployment authority.

Preserve failed `batch-JKeeLzH0` (pre-publication consumer admission refusal) and
all historical batches, including `batch-G3rEbkX7`, `batch-QJcFkCdU`,
`batch-_leIreCI`, `batch-C5BoFKA5`, `batch-AaHi2pgO`, `batch-O3V4GL4T`,
`batch-PIIqwepD` and `batch-mG_yG4uo`. The later successful batches supersede
their failures without rewriting their evidence.

Historical lock refs remain `refs/retained/platform036-lock-draft-20261108`
(`0263ab6e679ae3b8e9f89bddd796ca951b3f1df4`) and
`refs/retained/platform0270-lock-draft-20261108`
(`588b11a38c60b15ab5af19f30aef1c310a8b0975`). Never restore an old complete lock.
Bundles/receipts at `recovery-evidence/retired-checkouts/20261104-issue139/` require
maintainer review by 2026-12-04; no deletion, mtime renewal or budget increase is
authorized. Previously removed artifacts must not be claimed present.

The unreferenced signature draft remains verbatim at
`recovery-evidence/deferred-signature-comparison-20261110.rs`, SHA-256
`9fdee2e8b41c9b13e0d2a3127ddf95559a1094fa48d14dd17b7577d4ec86ed4a`.
It is outside the release; maintained comparisons live in the facade tests.
