# Current handoff

## Source and accepted scope

Local base: maintainer release `6733f50` (0.7.3); remote main now has runner
repair `ce92a9f`. Compatible pending notes target 0.7.4;
Rust metadata remains 0.7.3. No release/publication/deployment is authorized here.
Shared Tooling remains 0.26.26 at `5683060a5d8ce79af2fcfedb53b3eb24037fd064`.
Browser npm 0.1.1 is already published separately. Siblings remain read-only.

Preserve concurrent, unrelated root Cargo.lock changes: Wire/derive now resolve
0.7.20 and ic-host-artifacts 0.69.7 (initial task input was 0.7.19/0.69.5).
They are excluded from this contribution. Existing runtime qualification retains
its own reviewed registry lock, including Wire 0.7.19 and Candid 0.10.38.

## External codec supply (#78, #79)

Pending 0.7.4 supplies explicit `candid-endpoint` adapters and pure
`candid::protocols` operations. See the [migration and concrete Canic deletion
map](../migration.md#external-endpoint-and-protocol-codecs-78-79).
The adapters delegate to the existing generator/executor, run caller guards
before argument admission, and check scalar byte length before copying. Budgets,
empty-envelope policy, endpoint names, auth, startup and lifecycle stay caller-owned.
Native service registration does not select the printer; native DID generation
selects `candid-service` separately. Existing raw/Wire entrypoints are unchanged.

Protocol operations encode independently of submission, preserve typed provider
errors and require caller limits/arity. They add no transport, retry, journal,
certificate verification, spending or topology owner. The independent pinned
Cycles DID exposed a real mismatch: its blocks contain legacy `Value::Nat64`,
which standard `ICRC3Value` omits. The Cycles provider codec retains the complete
schema and upstream callback identity under both Cycles and ICRC DTO features.
Existing standard ICRC identities and all cryptographic implementations remain.

Full `make clippy` passes before longer tests. Standalone facade-only external
endpoint/codecs/schema consumers pass strict Clippy and native Rust 1.99.1/1.91;
applicable profiles compile for Wasm. Seven independent schema tests include the
provider Nat64 regression; endpoint tests cover native DID and explicit re-export
paths. The routine feature matrix passes 444 selections (407 native/251 Wasm
graph queries). Raw/executor/Wire remain Candid-free; endpoint Wasm excludes the
printer/parser/Agent/PocketIC. Protected callback/crypto source guards pass.

The existing installed fixture adds explicit external endpoints and reuses its
raw/owned transports. Linux/PocketIC checks cover valid/malformed arguments,
all five budget kinds, guard-before-decode, scalar/tuple/zero/null replies,
synchronous mutation rollback, composite query and async management raw_rand.
Existing callback/trap/drop/timer/upgrade subjects remain in the same runner.
Local sources include these cases; historical published qualification keeps its
original profile. `--external-adapters --published X.Y.Z` opts into the new cases
once the supplier is published. No new native macOS or live provider/ledger
acceptance is inferred from local installed proofs. Evidence:
`/tmp/platform-backlog-20261028`; no Canic/Host migration is claimed.

## Shipped work and CI runner repair

Merged [#84](https://github.com/dragginzgame/ic-platform/pull/84) repairs macOS Bash 3.2
empty-array expansion and published-version parsing in call/runtime runners.
Its full functional CI passed Linux and macOS Intel/ARM64, installed runtime and
browser jobs; only the independent #50 advisory check failed. Local command
admission and complete consumer proofs also pass real Linux-built Bash 3.2.0.
Codec work extends that regression to the new explicit installed selection.

PRs #80/#81/#82/#83 were closed as redundant after implementation/test files
were compared with released default `6733f50`. Issues #72/#74/#76 were closed
because their fixes shipped. #75 is also closed after its runner repair merged;
`ic-call` 0.7.2 is the verified initial published supplier (0.7.3 also available).
These closures do not claim downstream adoption. Existing #65/#66 and DTO #61–64
are already resolved; Canic's remaining consumer work stays in its own tracker.

## Unresolved safety contracts

[#73](https://github.com/dragginzgame/ic-platform/issues/73) remains open. Candid
0.10.39's Wasm stack check is a no-op and its public config has no depth setter.
Installed 8,192 nested opts in 8,202 bytes trap despite finite work/type/header
limits. The new codecs **do not fix depth admission**. A scratch native depth
visitor also panics in upstream optional-value recovery, and type expansion can
recurse before visitors. No private parser fork, unsafe workaround or automatic
quota is shipped. Findings are recorded in #73; source/installed evidence remains
in `/tmp/platform-security-20261028` and `/tmp/platform-backlog-20261028/depth-probe`.

[#50](https://github.com/dragginzgame/ic-platform/issues/50) independently tracks
RUSTSEC-2026-0387 through coupled ECDSA/Agent/derivation dependencies. No advisory
ignore, incompatible override, Agent fork or in-house crypto is authorized.
Functional passes do not turn that failed advisory check into a pass.

#72's approved DER check is the sole admitted parser-entry call; source guards
reconstruct original fingerprints. Short-form framing refusal is qualified;
long-form support and oversized encoder conversions remain outside that change.
No crypto encoder, algorithm, constant, vector or license changed. #74's shared
native HTTP guard retains browser Origin/Host admission and nonbrowser clients.
Prior exact-source, 44 assembler and real Chromium evidence remains at
`/tmp/platform-security-20261028`; the preceding detailed handoff is in Git.

## Retained evidence

The [0.7.1 size observations](../migration.md#non-cryptographic-size-qualification-071)
remain fixture measurements, not application-size or IC-cost claims. No new
size/performance reduction is claimed for codecs or manifest routing.
Provider `9dcef4b`, server SHA-256
`8cd8968f8ed9dd7c6902379f18c31cb3aa2b97333c23189ffa3fe975d584c706`, retains
Linux native build/69 recorder/five profiling regressions and default CI host
qualification. Full Bazel/NNS genesis and native macOS provider-server execution
remain unqualified. Preserve immutable inputs and preparation receipts.
