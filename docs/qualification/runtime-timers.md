# Timer execution boundary

Runtime 0.23.12 is the next compatible candidate after the published 0.23.11
baseline, checked in the crates.io sparse index on 16 November 2026. Macros and
guard advance with Runtime. This work is local and unpublished.

## Consumer and ownership

Read-only Scheduling source `170599fbf3e4fac51b09519645d8dc11d9633e4f`,
`crates/ic-timers/src/transport.rs`, selects Runtime `=0.22.5` for Timers 0.45.11.
Resolving it with Runtime 0.23.11 fails at the executor guard's shared Cargo
`links` identity. Keep that refusal: admitting two executors is not a migration.
The consumer uses IC time, canister identity/version, timer arming, self-dispatch,
awaited calls and protected task destruction. Scheduling owns the global timer
export, cadence, dispatch records, bounded flights, backoff and replay policy.

Runtime now provides `system::global_timer_set`, `system::canister_version` and
`executor::run_global_timer` under wasm32 `call-ic0`. The adapter uses the existing
task/callback owner. Timer roots have no ordinary reply authority before or after
an await. Nested entry refuses. It adds neither child/migratory tasks nor a second
executor. Its export-kind precondition is caller-owned, as for `run_update`;
the replicated-execution flag alone cannot distinguish every replicated context.

The [IC system API](https://docs.internetcomputer.org/references/ic-interface-spec/canister-interface/)
owns execution-context restrictions. Zero disables the timer; the system
deactivates it before invocation. Rearming is explicit consumer policy. Canister
version advances with ordinary state-changing execution and can also advance at
the system's discretion. Capture an initialization value when constructing an
incarnation marker; do not assume repeated version reads stay equal until upgrade.

## Focused evidence

`make qualify-runtime-timer` runs the maintained installed fixture with the
verified PocketIC 16.1.0 Linux server. The same cases belong to the full installed
Runtime qualification; the focused command does not run the unrelated suite.

- A real global timer submits and awaits a self-call, completes and reports
  `Completed`; neither initial nor resumed polling can reply as an update.
- A resumed callback trap runs cleanup and commits `TerminatedByCleanup` only
  after successful future destruction.
- A trapping cleanup rolls back destruction and leaves `Running`; the test
  deliberately does not infer liveness or permission to retry.
- An initial poll trap rolls back the invocation before a handle can escape.
- Disarming prevents invocation; query entry refuses without polling; destruction
  re-entry refuses. Independent updates retain their reply authority.
- Stop/observe-Stopped/upgrade/start clears heap-local observations and advances
  the canister version. No callback or future reconstruction is introduced.

All five installed modes pass. Runtime's 21 native tests also pass, including
timer/update reply isolation and existing reentrancy/custody cases. Native and
Wasm Clippy pass for the affected implementation and fixtures. These observations
do not establish macOS execution or production Scheduling adoption.

## Remaining Scheduling migration

This is a supplier prerequisite, not a fix to the published Timers dependency.
Scheduling must select this one exact Runtime cohort and migrate its transport.
The old tracking-context/protected-spawn/recovery-flag API is not restored.
Use timer roots for delivery calls and update roots for authenticated self-work;
observe their opaque task handles after committed execution. Preserve dispatch
records for initial traps where no handle returns. A failed cleanup must retain
uncertainty rather than release an accepted attempt for replay. Successful cleanup
observation is local task termination, not evidence about remote effects.

Qualify the actual Scheduling installed recovery and watchdog cases after that
migration, including multiple flights, initial work traps, callback/cleanup traps,
stale incarnation envelopes and stop/drain upgrades. Sibling migration requires
separate authorization; no Scheduling checkout was edited by this slice.
