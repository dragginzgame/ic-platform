# Current handoff

## Pending maintained PocketIC client

The maintainer authorized owning the released PocketIC 16.1.0 Rust client while
preserving signatures and operation behavior. The native-only published
implementation package is `ic-platform-pocket-ic`; consumers keep selecting the
existing feature and importing `ic_platform::pocket_ic`. The simulator server,
Testkit server pin/lifecycle and separate native Profiler source profile are
unchanged. Client-owned Rust type identities move, so the Platform ledger proposes
0.16.0. No package version, release or publication has been prepared by this task.
See [the maintenance contract](../pocket-ic-client.md) and its source provenance.

The extra implementation package prevents Candid/crypto selection on Wasm and
belongs to the Platform release group. External management DTOs retain their
existing 0.8 identities, separate from the facade's 0.11 surface. No fallible API,
server fallback, retry or provisioning-policy change is part of this transfer.

Shared Tooling [PR #332](https://github.com/dragginzgame/shared-tooling/pull/332)
provides the narrow native implementation-owner exception. Its exact source
`5caa130a8705c43aceaa18993a30c87b959ef823` is adopted through the canonical
exporter, recording supplied version 0.27.23 and all 117 snapshot files. The
supplier change still needs integration; no managed payload was edited locally.

## Focused evidence and limits

Evidence is retained under `/tmp/ic-platform-pocketic-client.JQG7s5`.
Required full all-feature/all-target Clippy, owned-client protocol and facade
consumer tests on current Rust and MSRV 1.91, 444 feature selections, native/Wasm
graph policies, boundary/inventory, source/license fingerprints and runtime-lock
custody checks pass. The imported registry client is absent from the qualified
graph; default, executor-only and Wasm PocketIC selections exclude its replacement.

An isolated IC Tooling source override qualified the unchanged Testkit transport,
startup and installed-server contracts against the prepared Linux PocketIC
16.1.0 binary. All six explicitly selected installed lifecycle cases, including
the asynchronous client, pass. The override is qualification only; production
consumer adoption waits for the supplier release. Native macOS qualification and
hosted CI are separate and are not claimed by these Linux results.

The historical standalone canister runtime preparation still fails at the
Timers/executor links conflict. The same failure is reproduced on unchanged
`f6b3293e2d6adabf65bd5afdf0194463b84330ea`; keep its guard. Closed issue #116
records the separation of Platform proofs from Timers composition, not a pass
for this historical optional fixture.
The retained fixture lock changes only the stock client owner/edge, with all
other package records preserved. Custody success is not fresh composition
qualification, and no installed canister-runtime execution is inferred.

Earlier Platform 0.15.3 delivery receipts, native Profiler qualification, advisory
#50 and recovery constraints remain in the
[previous handoff](https://github.com/dragginzgame/ic-platform/blob/f6b3293e2d6adabf65bd5afdf0194463b84330ea/docs/status/current.md).
Preserve those artifacts and saved release attempts; this implementation does not
reconcile them or authorize delivery, cleanup or publication. Ordinary Platform,
Tooling and Shared working checkouts retain their unrelated work.
