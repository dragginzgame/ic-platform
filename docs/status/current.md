# Current handoff

## Pending 0.8.3: focused complexity cleanup

Source release `b0bf82f` is 0.8.2; its merge is complete. Package versions remain
0.8.2. Pending notes use 0.8.3 for compatible implementation/tooling cleanup;
release, publication, staging and commits remain maintainer-owned. Preserve the
pre-existing dirty Cargo.lock selections, including Host Artifacts 0.70.2.
Siblings stay read-only. Shared Tooling payloads and protected crypto stay intact.

Owned `ic_platform::Account` remains under `principal`, without Candid or a
registry ledger dependency. The separate selected external Candid Account DTO
is also Platform-maintained. Its Display now delegates to the owned formatter;
external parsing/errors, Candid/Serde, effective-zero equality/order/hash and
absent versus explicit zero fields retain their contracts. Required external
Principal/Nat/function identities remain upstream. Value-hash bodies are unchanged.

Manual exhaustive feature auditing now requires 1–12 explicit features, e.g.
`make check-features-exhaustive FEATURES=runtime,blake3`; the production power set
would exceed two billion selections. Routine 505-selection coverage is unchanged.
Consumer profiles now carry graph families, with one target iteration and the
same admission rules. Unknown families fail before compilation. ic-call/runtime
proofs reuse Cargo targets; runtime proofs retain their own executable/Wasm copies.

Native REST/MCP profiling requests share parsing, including diagnostic order and
existing defaults/clamps. Conditional-clear fields are named without changing
absent/null cursor semantics. Pricing comparison borrows JSON rows. Snapshot
ranges remain numeric from recorder to renderer; only response construction
formats counters. Read/write masks share the sweep without fake read records or
counter-key renaming. PNG bytes, response schemas and limits remain unchanged.

[PR #99](https://github.com/dragginzgame/ic-platform/pull/99) is open and no longer
draft at this inspection. It separately supplies native operation completion and
backend descriptions with Profiler 0.11.0's paired product interface. Do not
replace it with these edits or claim it is merged. Integrate the paired changes
before consumer pin adoption and renewed complete native-server qualification.
Extra native strict-Clippy debt remains [#98](https://github.com/dragginzgame/ic-platform/issues/98).
[#50](https://github.com/dragginzgame/ic-platform/issues/50) remains deferred with
its advisory check unchanged; no crypto override, ignore or Agent fork is admitted.

## Cleanup evidence and limits

Evidence is retained under `target/complexity-cleanup-20261030`. Full owner
all-target/all-feature `make clippy` passes before longer tests. Principal/ICRC
standalone native and Wasm proofs pass on current Rust and MSRV 1.91, including
frozen Account text, external wire and ownership isolation. Feature/consumer
admission fixtures cover scope/family refusal, failure propagation, concurrent
script edits and repeated runtime target reuse with retained artifacts. A real
scoped runtime/BLAKE3 audit passes for native/Wasm. Source/license guards pass.

An isolated copy of the prepared PR #99 native dependency context compiles the
changed modules with Rust 1.99.0 and warnings denied. Focused memory/recorder/
viewer/snapshot and pricing tests pass. The separate before/after image control
compares complete JSON and exact PNG bytes for data/read/write views at seven
counter magnitudes, including u64::MAX. Compiler inputs and logs are retained.
This uses prepared compiled dependencies; it is not a fresh Cargo assembly,
server build, installed-canister run or qualification of #99's whole integration.
No native macOS, full Bazel/NNS or live-provider acceptance is added. No new
optimized-Wasm-size or runtime-speed saving is measured.

## Released supply and native source pins

0.8.2 adopted ordinary registry Host Artifacts/Testkit 0.70.0 and the managed
release editor with `cli`. Runtime drivers import raw PocketIC through Platform
and borrow Testkit instances with `instance()`. Testkit's registry Platform
requirement stays independent of local release versions; no global Cargo patch.
Workspace Wire minima remain 0.7.22; the main lock selects 0.7.23 and the separate
runtime qualification lock retains 0.7.22. Shared Tooling 0.27.7 stays at
`a8baa51099ca48cdb12e2171f031d488db89d67d`, including graph observations.
The maintainer's one-time 0.8.2 exception for the unused gzip error identity does
not alter future compatibility policy. Ordinary gzip and native source graphs
remain separate.

The main native recipe retains IC `ee8086a9773725fd1058832a9400297cf0990112`,
Rust 1.99.0, Host `c20dbd11785c17d369651f51fa0967fbec69c57a`, Wire 0.7.21 and
Platform scaffold `ba98bd33ef1cbe463d6daf34f0511f17d7770c20`. #99 updates its
Wire/scaffold separately. Profiler owns query/cache/tool/rendering and PNG/range
mechanics; Platform retains native integration/state, HTTP admission and pricing.
Keep 8 MiB/query, 32 MiB aggregate, 128-entry FIFO and cross-run checks. Rebuild
private ICPROF03 peers together; saved formats and retry authority stay unchanged.

Historical #97 qualification used provider `f5051bf` + Profiler `e9f68b6`, with
74 recorder, 26 product, three catalog and five installed tests. Its Linux server
SHA-256 is `b863005e0272c82e2f94a6a3684bbdeb0652fdb8162b6aa823d1da02b2b290e1`.
Original scratch executable/receipts were externally removed; the PR retains
observations. Those results do not qualify the pending changed native modules.
Earlier adoption/merge observations remain in the handoff at `b0bf82f` and PRs
#95/#97; retain artifacts that remain available.

## Retained safety and protocol contracts

Owned endpoints use Wire; required external Candid and legacy checkpoint bytes
retain their contracts. Bounded Candid rejects more than 16 constructed wire
levels, including unused definitions/skipped values, within existing work quotas.
This does not guarantee arbitrary trusted Rust schemas/custom visitors are safe.
The [migration contract](../migration.md#external-candid-nesting-qualification-73)
retains the 16/17-level, original 8,192-opt trap and measured admission evidence.

Management =0.11.0, executor =2.0.0, ic0 =1.2.0 and stable-structures =0.7.2
identities remain fixed. Memory/Storage own allocation, reopening and leases.
Protected crypto bodies, encodings, constants, vectors and licenses stay intact.
#72 permits only its reviewed short-form DER admission check; long-form and
encoder changes remain outside that exception. #74's Host/Origin admission stays.
`signature-verification` explicitly selects external Principal Serde for certified
ranges; other Principal tiers stay narrow. The [packaging contract](../migration.md#minimal-principal-and-selected-icrc-packaging)
retains the 28 independently frozen selected-ledger schemas and migration rules.
