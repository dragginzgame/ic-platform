# Current handoff

## Pending 0.7.5: fixed 16-level Candid admission (#73)

The maintainer replaced #73's unpublished opt-in design with one fixed rule:
Platform's bounded decoders reject more than **16 constructed wire levels** before
Candid runs. Exactly 16 levels are allowed. No new caller policy object, limit
setting, macro option or feature is needed. Existing helper signatures, limit
structs and closed error variants remain; typed refusals use `nesting_error()`.
[PR #87](https://github.com/dragginzgame/ic-platform/pull/87) is the delivery vehicle.

The same check runs from the shared decoder configuration, covering single,
argument, exact-single and protocol replies plus external endpoint adapters.
`candid::nesting::{MAX_DEPTH, NestingError, check}` supplies the fixed contract;
low-level upstream decoder/contract APIs retain their behavior, with explicit
`check(bytes, limits)` available to their staged callers. The earlier draft's
policy/methods and nesting macro option were removed before publication.

The iterative walk checks unused type definitions and skipped fields/arguments.
It permits recursive declarations while limiting each simple type path and
actual value depth; byte blobs remain valid. Admission work uses the saturating
sum of existing decode/skip quotas, counting bytes, edges and values including
zero-byte vector expansion. Candid subsequently receives its original quotas.
Unsupported future/opaque framing fails closed. No dependency, unsafe code,
executor/crypto change or byte rewriting is introduced. Wire endpoints remain Wire.

Full `make clippy` passes before longer tests. Focused Candid tests pass 21 cases
on Rust 1.99.1/1.91; macro adapter tests pass. Standalone facade-only nesting,
Candid, endpoint, management, ICRC and protocol consumers pass native tests and
applicable Wasm on both compilers. Normal external identities and bytes remain.
Strict Rustdoc, inventory/links and protected source checks pass. Evidence:
`/tmp/platform-fixed-depth-20261029`; fixture `target/cdk-runtime.p3w1eD`.

Installed Linux/PocketIC passes current Rust/1.91 × raw/owned calls: 16 levels
succeed; 17 and the original 8,202-byte/8,192-opt trap return typed refusals.
Deep unused headers and malformed/truncated inputs are refused. Guarded normal
arguments, real management raw_rand and unchanged refused-handler state pass;
existing callback/trap/timer/upgrade subjects pass in the same fixture. Native
macOS CI and live external-provider acceptance remain separate. Installed lock
tests Candid 0.10.38; root/standalone checks select 0.10.39. No production version,
registry lock selection, publication or performance/size reduction is claimed.

This mitigates wire nesting, not arbitrary trusted expected Rust schemas/custom
Serde visitors. The upstream Wasm stack check remains a no-op with no depth setter.
The rejected custom visitor panicked during upstream option recovery; header
normalization recurses before visitors. Do not claim Candid itself is repaired.
See the [contract](../migration.md#external-candid-nesting-qualification-73).

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
