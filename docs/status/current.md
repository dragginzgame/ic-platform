# Current handoff

## Pending 0.7.5: explicit external Candid nesting admission (#73)

Prepared [PR #87](https://github.com/dragginzgame/ic-platform/pull/87); hosted CI
is pending. #73 stays open until default-branch delivery. Root lock updates
remain unrelated and excluded from the isolated contribution.

The maintainer requested a wire-nesting sanity limit rather than a Candid fork.
`DecodeLimits::with_nesting(NestingLimits { depth, work })` selects an iterative
admission walk, then the existing decoder. `DecodePolicy` offers `admit`,
`decode_one`, `decode_args` and `decode_exactly_one`. Existing helper signatures,
limit structs, arity and closed error variants remain unchanged; `nesting_error()`
recovers the typed cause through the existing Candid error. No implicit limit,
new dependency, feature, unsafe code or byte rewriting is introduced.

The flat walk checks unused type definitions, every simple constructed-type path
and all values including skipped fields/arguments. It accepts recursive type
references, bounds actual recursive values, charges zero-byte vector elements and
caps DAG path expansion with the separate work budget. Future/opaque forms fail
closed. `ArgumentLimits::decode_with_nesting`, `endpoint::read_with_nesting` and
optional endpoint `nesting = "PATH"` preserve guards and pre-copy byte admission.
Consumers must select budgets and adopt these APIs in their generators/reply code.
See the [migration contract](../migration.md#external-candid-nesting-qualification-73).

Full `make clippy` passes before longer tests. The focused Candid suite passes
22 tests on Rust 1.99.1 and 1.91; macro adapter tests pass. Standalone facade-only
nesting, endpoint, management and ICRC consumers pass native tests and applicable
Wasm on both compilers. They preserve normal DTO bytes and registry identities.
Strict Rustdoc, consumer command admission, boundary/inventory and source checks
pass. Runtime graphs preserve executor/ic0 identity and Wire/runtime isolation.
Evidence: `/tmp/platform-nesting-20261029`; fixture `target/cdk-runtime.p3w1eD`.

Installed Linux/PocketIC qualification passes Rust 1.99.1/1.91 × raw/owned calls.
The probe returns a typed refusal for the original 8,202-byte/8,192-opt trap input,
accepts the selected 64-level boundary and refuses 2,048 unused record definitions.
Malformed/truncated inputs are refused; normal guarded arguments and management
raw_rand replies work, and refused updates leave handler state unchanged. Existing
await/trap/timer/cancellation/upgrade subjects pass in the same fixture. Native
macOS CI and live provider acceptance remain separate; no performance/size saving
is claimed. Maintained installed lock selects Candid 0.10.38; root consumers test
0.10.39. No registry selections or production version are changed.

This mitigates **wire nesting**, not all Candid defects or arbitrary trusted
expected Rust schemas/custom visitors. The unchanged helpers, `candid::decoder`
and low-level `candid::contract` APIs retain upstream/caller-owned budgeting.
Candid 0.10.39 still exposes no Wasm depth setter; its stack check is a no-op there.
The rejected custom Serde visitor panicked in upstream option recovery, and
header normalization recurses before visitors. Preserve those findings in #73;
do not claim the upstream decoder is repaired or unadopted consumers protected.

## Shared ICRC-10 supplier and Canic adoption

Default main `7d85df1` includes [PR #86](https://github.com/dragginzgame/ic-platform/pull/86),
the named external `SupportedStandard` under `icrc-ledger-types`. All functional
CI passed Linux/macOS Intel/ARM64/browser/installed; only #50 advisory failed.
Its types-only, codec and independent schema/bytes proofs remain in
`/tmp/platform-icrc10-20261028`. Main is source 0.7.4 with pending **0.7.5** notes;
leave release/publication to the maintainer. Preserve unrelated local lock changes.

[Canic #706](https://github.com/dragginzgame/canic/issues/706) tracks its optional
ICRC adapter. The isolated pending 0.113 patch requires published Platform 0.7.5
and its pending source base upstream. Its local DTO is replaced by the exact
Platform identity; mounting, consent, authorization and budgets remain Canic-owned.
Source-candidate strict lint, schema/type/byte tests, MSRV/Wasm and four installed
profiles pass; registry/managed adoption is still pending. Normal sibling
checkouts remain untouched. Discovery bytes match both prior mounted selections.

## Retained contracts and open advisory

Source release 0.7.4 supplies external endpoint/protocol adapters (#78/#79).
Issues are closed; redundant PR #85 was closed after implementation/test/doc
identity comparison with main. #84's macOS Bash 3.2 runner repair merged with
functional CI green. PRs #80–83 were redundant; #72/#74/#75/#76 shipped. Detailed
historical evidence is in `/tmp/platform-backlog-20261028` and Git history.

[#50](https://github.com/dragginzgame/ic-platform/issues/50) remains open for
RUSTSEC-2026-0387 through coupled ECDSA/Agent/derivation dependencies. The maintainer
approved setting it aside for integration only. No advisory ignore, check
suppression, incompatible override, Agent fork or in-house crypto is authorized.
#72's narrow DER parser-entry admission remains the sole source-guard exception;
crypto bodies, encoding, constants, vectors and licenses stay protected.
Only short-form framing refusal is qualified there; long-form support and
oversized encoder conversions remain outside that change. #74 retains native
browser Origin/Host admission and nonbrowser clients; 44 assembler and Chromium
proofs remain in `/tmp/platform-security-20261028`.

Provider `9dcef4b`, server SHA-256
`8cd8968f8ed9dd7c6902379f18c31cb3aa2b97333c23189ffa3fe975d584c706`, retains
Linux native/69 recorder/five profiling proofs. Full Bazel/NNS genesis and native
macOS provider-server execution remain unqualified. Historical size results are
fixture-specific; preserve immutable inputs and preparation receipts.
