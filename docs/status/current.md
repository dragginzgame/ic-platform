# Current handoff

## Source and accepted scope

The checkout is at maintainer release commit `6733f50` (0.7.3). Rust metadata
remains 0.7.3; new compatible repair notes target 0.7.4. No release, publication
or deployment is part of this task. npm browser 0.1.1 is separately published.
Shared Tooling stays 0.26.26 at `5683060a5d8ce79af2fcfedb53b3eb24037fd064`
(109 payloads). Earlier source/delivery records below retain their own scope.

Preserve incoming root lockfile work: Wire/derive 0.7.19, ic-host-artifacts
0.69.5 and unicode-width 0.2.3. Sibling checkouts remain read-only.

## Bash 3.2 call qualification repair

The released `ic-call` implementation is present on main; its PR's two macOS
jobs exposed a runner defect, `features[@]: unbound variable`. Empty feature,
admission and preparation argument arrays now use Bash-3.2-compatible expansion.
The runtime's published-version regex also uses a variable so earlier Bash 3.2
parsers accept the runner. No call runtime, codec, executor or dependency changes.

Full `make clippy` passes. The maintained command-admission regression passes on
Bash 5.2 and a real Linux-built Bash 3.2.0, covering all three consumer selections,
empty/nonempty argument arrays, eight raw/owned runtime graph commands, version
refusal and propagation of a failing final Wasm check. The complete standalone
consumer also passes under Bash 3.2 with strict Clippy and current/Rust-1.91 native
and Wasm checks; real combined runtime graph admission passes under that shell.
ShellCheck, formatting and links pass. This is shell compatibility evidence on
Linux, not a new native macOS run. Hosted macOS acceptance remains with the fix PR.
Evidence: `/tmp/platform-backlog-20261028`; the repair is isolated from later
Candid-depth/adapter work and changes no release metadata or advisory policy.

## IC Wire 0.7.19 adoption

Both catalog declarations now require published Wire 0.7.19: the production
native metadata alias and the independent tooling/runtime dependency. The incoming
root lock already selected Wire/derive 0.7.19 and is preserved byte-for-byte.
Only Wire and its derive change in the maintained runtime fixture lock; other
selections, Platform exports, features, codecs and consumer policies are unchanged.
The compatible changelog note targets 0.7.3 without changing release metadata.

Full `make clippy` passed before longer checks. The standalone Rust 1.91 schema
consumer and strict generated TypeScript/Node tests pass for Principal/Account
frames, lengths, absent/zero/nonzero subaccounts and malformed frame refusals.
Standalone `ic-call` raw/Wire/external-Candid consumers pass strict Clippy and
native tests/Wasm compilation under Rust 1.99.1 and 1.91. Rust 1.91 additionally
compiles Platform's selected `wire-schema` feature for Wasm. Native/Wasm default
and executor graphs remain Wire/JSON/Candid/host-free; Wasm `wire-schema` also
excludes native metadata edges. Combined owned/raw runtime graphs retain registry
executor 2.0.0 and ic0 1.2.0. Inventory, boundaries, pins, docs and formatting pass.

Evidence is retained at `/tmp/platform-wire-adoption.sbnFJ8`. This is Linux native,
TypeScript and Wasm compile/graph qualification; no new installed-canister or native
macOS execution is claimed. Existing callback/upgrade installed records below
predate this Wire selection. No sibling checkout, cryptographic implementation,
Platform release version, Git history or publication is changed by this adoption.

## Prepared one-shot calls (#75)

The maintainer authorized the additional `crates/ic-call` consumer library.
It owns prepared method/argument custody and delegates execution and exact cost
queries to Platform's unchanged raw call implementation. Dynamic method Strings
and argument Vecs can outlive their source locals; borrowed inputs stay supported.
Wait policy is explicit, with unchanged response/error identities and no codec,
retry, funding, instrumentation policy, Relay or Timers dependency. Existing raw
callers remain supported. Canic #700 is the demonstrated ownership gap; Blob #108,
Query #108, IC Crates #19, Jobs #75, Relay #14 and Timers #80 retain their documented
consumer responsibilities. See [the migration](../migration.md#prepared-one-shot-calls-issue-75).

Full `make clippy` passed before longer checks. The working tree passes
`make test docs-check wasm-check msrv`, formatting, dependency/inventory/pin/link
checks, feature-selection regressions, release-command/runtime-lock/source guards.
The isolated default-branch contribution separately passes full Clippy, five
Rust 1.91 unit tests and standalone raw/Wire/external-Candid consumers under
Rust 1.99.1 and 1.91, native and Wasm. Those graphs keep one registry executor
2.0.0 and ic0 1.2.0, with no Candid in raw/Wire selections and no host stack,
Relay, Timers or entrypoint generator in the standalone call dependency.

The installed lifecycle fixture now selects either original raw calls or owned
requests. All eight Linux/PocketIC combinations of raw/owned, Wire/external Candid
and both compilers pass; the exact isolated contribution's owned profile also
passes all four combinations. These cover dynamic method ownership, borrowed and
owned arguments, exact cost plus attachments, reply/reject refund timing,
structured balance/submission/rejection failures, successful transport with bad
codec bytes, callback/encoding traps, cancellation/drop and upgrade controls.
Unknown and SysUnknown codes are additionally checked as typed native fixtures;
no deliberately delayed physical bounded-timeout experiment is claimed.
Registry dependency identities/edges in the installed disposable lock match the
maintained qualification lock; callback/submission source fingerprints are unchanged.

Implementation is proposed in [PR #83](https://github.com/dragginzgame/ic-platform/pull/83);
#75 stays open pending default-branch delivery.
Evidence is retained at `/tmp/platform-icc-20261028`, including the isolated
contribution under `delivery/repo`. The local release metadata and default-branch
metadata differ; the new crate inherits each without a version bump. Published `ic-call` 0.7.2 is the initial supplier; its registry-only ownership
and Rust 1.91 native/Wasm proof is recorded in #75's publication comment. Changelog notes, publication ordering and normal CI
include the additional crate. Native macOS, publication and downstream adoption
remain separate; no application Wasm-size reduction is claimed. Candid Wasm depth
#73 remains unresolved. Sibling sources and incoming security/browser work are
preserved.

## Security corrections and evidence

For [#72](https://github.com/dragginzgame/ic-platform/issues/72), the maintainer
explicitly approved a narrow DER-admission exception. One owned framing check
precedes the unchanged imported parser. The source guard requires that exact
call once at parser entry, removes only it, and verifies the existing original
hashes. No encoder, cryptographic body, constant, vector or license changed.
Supported short-form DER now rejects truncated/noncanonical envelopes; long-form
DER and oversized encoder-length conversions are not newly supported.
Consumers retain byte/depth budgets, issuer/seed, trust and application policy.

Full Clippy passed before longer tests. All 33 signature-module tests and four
consumer tests pass on native and Rust 1.91. Standalone creation/verification
consumers pass native/MSRV/Wasm. Source mutation checks reject changed bodies,
vectors/licenses and missing/duplicate/moved admission calls. Installed Wasm
rejects 48 truncated prefixes, 19 malformed headers and trailing data through
parser/both verifiers; the original valid signature still succeeds.

For [#74](https://github.com/dragginzgame/ic-platform/issues/74), main-port run
APIs, trace/snapshot controls and viewer startup share the viewer's Host/Origin
admission. The actual bound listener determines loopback policy. Nonbrowser
clients may omit Origin; explicit remote binds keep valid hostname support.
Missing listener configuration, malformed/duplicate headers and cross-origin
requests fail closed. Unrelated PocketIC routes keep their existing behavior.

The four exact-source HTTP/Aide tests and strict standalone Clippy pass. The
integrated local provider proof passes 44 assembler fixtures after #77. Real
Chromium accepts local reads, rejects a
rebound hostname and cross-origin form POST (403), and preserves local nonbrowser
POSTs without Origin. The new patch applies to the recorded assembled server
sources, and manifest integrity is refreshed. This is not a new complete native
server build or consumer-pin adoption; those remain separate qualifications.

For [#73](https://github.com/dragginzgame/ic-platform/issues/73), installed local
Wasm confirms a trap: 8,192 recursive opt values in 8,202 bytes, finite
1,000,000 decoding/skipping quotas, one type entry and a nine-byte header.
The response is CanisterTrapped / heap out of bounds with a recursive decoder
backtrace. Candid 0.10.39 explicitly disables its stack check on Wasm and has no
supported depth setter. Lower quotas reject the probe but are not a general
depth guarantee. No automatic ceiling, decoder fork or breaking DecodeLimits
field is introduced. Caller-selected depth control and installed boundary tests
remain unresolved. Module docs and the migration contract state the limitation.

Current raw sources, commands, logs, browser probe and installed fixtures remain
at `/tmp/platform-security-20261028`. No native macOS security execution, new
provider server binary or full release CI is claimed.

## Retained contracts and qualification

[#80](https://github.com/dragginzgame/ic-platform/pull/80) delivers #72's approved
correction; [#81](https://github.com/dragginzgame/ic-platform/pull/81) delivers
#74's shared HTTP guard and maintained isolated source-CI proof. Checks run
asynchronously; neither ticket is closed before default-branch delivery.
The maintainer explicitly approved merging reviewed #77/#80/#81/#82 once each
functional matrix passes with #50 as the only remaining failed check, unchanged.
#77 met that condition and merged normally. The other three current heads still
have pending functional checks; do not merge them early or add an advisory ignore.

[#82](https://github.com/dragginzgame/ic-platform/pull/82) supplies #76's packed
browser proof, which now compiles strict public and Caffeine/Auth
fixtures with consumer-floor TypeScript 6.0.3 and producer/current 7.0.2. Both
emitted plain consumers execute under Node 24.21.0. The floor compiler has its
own private fixture, prepared explicitly without runtime dependencies or peer
overrides. Linux's 16 adapter tests and packed/graph/bundle proofs pass; new
macOS/browser-worker runs remain separate. Application lint peers remain owned
by applications. No browser package version or SDK selection changed.
All seven declaration files match the published 0.1.1 archive. Its initial CI
formatting failure was the unignored compiler cache; the artifact exclusion is
fixed and the full local formatting/hook proof passes. New checks are running.

[The migration contract](../migration.md#non-cryptographic-size-qualification-071)
retains the 0.7.1 optimization measurements and their limits. Principal/Account
and raw-call scaffolding improvements are retained; Candid setup/visitor sharing
prototypes were rejected after native/runtime and single-limit size tradeoffs.
No cryptographic body or wire/persisted format changed. Raw evidence remains in
`/tmp/platform-size-work`, `/tmp/platform-size-followup` and
`/tmp/platform-candid-experiments`; previous detailed handoff is in local Git
`5862651`. Fixture Wasm deltas are not application-size or IC-cost claims.

Provider `9dcef4b` has recorded successful Linux native assembly/build, 69 recorder
tests and five system_profiling regressions against server SHA-256
`8cd8968f8ed9dd7c6902379f18c31cb3aa2b97333c23189ffa3fe975d584c706`.
Profiler default main `4b9ad74` selects that exact provider. Platform's default
[CI](https://github.com/dragginzgame/ic-platform/actions/runs/40557242718)
passed source fixtures and Linux/macOS Intel/ARM64 qualification. This satisfied
#65/#66, now closed. Full Bazel builds, NNS-genesis artifacts and a native macOS
provider-server run remain unqualified; keep immutable inputs and receipts.

Required external Candid remains separate from owned Wire RPC. Published
[external DTO contracts](../external-contracts.md) and browser supply retain
consumer-owned authority/retry/effect qualification. No consumer adoption is
inferred from a facade export or package release.
Completed DTO supplier tickets #61–64 are closed; remaining Canic obligations
are retained in its existing #613 tracker, not declared complete here.
New #75/#78/#79 remain API work: #75's first reuse inventory is posted, while
the remaining consumers and codec extraction require deeper implementation work.
#78 and #79 cover distinct endpoint and outbound/host codec responsibilities;
neither may promise effective Wasm nesting admission before #73 is qualified.

[#50](https://github.com/dragginzgame/ic-platform/issues/50) remains the independent
RUSTSEC-2026-0387 blocker through coupled ECDSA/Agent/derivation dependencies.
No advisory ignore, incompatible override, Agent fork or in-house crypto is
approved. The default advisory workflow remains failed independently of passing
functional CI; no blanket public-data or verification-only exemption is supported.
