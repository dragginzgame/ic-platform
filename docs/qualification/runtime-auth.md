# Auth execution capabilities — Runtime 0.23.10 candidate

The next Runtime candidate restores the retained execution capabilities requested
by Auth. This is additive to 0.23.9; macros and singleton guard advance together.
No application checkout was modified and no production migration is claimed.

## Contract

`system` provides canonical Types caller/canister identities, IC time, controller
observation, argument length, explicit trap, exact 32-byte certified-data publication
and bounded certificate retrieval. Certificate absence is distinct from oversized
input; presence alone never authenticates bytes. Principal sizes are checked before
copying into initialized 29-byte storage; certificate limits precede allocation
and copying. These imports remain in the approved private IC0 boundary.

One generator supplies `update`, `query`, `init`, `pre_upgrade` and `post_upgrade`.
Every handler takes one decoded argument (unit when appropriate). Guards run before
Runtime observes/copies arguments. Codecs and the non-returning error callback are
explicit. Synchronous queries encode before issuing their single reply and exclude
nested Runtime entries. Lifecycle handlers have no reply/task authority; pre-upgrade
decodes empty input without invoking argument imports. Automatic synchronous/async
updates use the existing executor. Async queries/lifecycle handlers are refused.
Manual replies remain explicit async updates; mixed manual/encoder options refuse.

This adds no storage, callback persistence, timers, child tasks, application
permissions or codec dependency. Upgrades remain stop → Stopped → upgrade → start.

## Local evidence — 2026-11-16

- `make qualify-runtime-exports`: macro unit tests, Wasm lint/docs, actual manual
  exports and compile-fail cases. The expanded UI suite passes one accepted and
  ten rejected cases, including async query/lifecycle and invalid encoder types.
- `make qualify-runtime-installed`: real identity/time/controller observations;
  query certificate presence and replicated absence; independent certificate
  authentication against the PocketIC root and exact published certified data;
  exact certificate cap and cap-minus-one refusal; init/post-upgrade effects;
  encoder trap rollback; synchronous exclusion; automatic sync/async replies.
  Existing cross-subnet callback, cleanup and drained-upgrade cases remain active.
- `node qualification/runtime-system.mjs`: ten independent Wasm ABI cases check
  initialized copy destinations, 0/29/30-byte identities, absence without size/copy,
  oversized certificate refusal before copy, exact reply bytes and lifecycle
  imports. Pre-upgrade performs no argument observation and no lifecycle replies.
- `make qualify-auth-runtime`: archive Auth commit
  `b78991e9833320692921bf766e51ad361c9c8c4d` into ignored `target/auth-runtime/`;
  adapt its actual issuer/browser/receiver/enrollment endpoints to explicit Runtime
  codecs and errors; build owned Wasm through Auth's existing Build harness;
  execute its five existing installed tests without weakening assertions.
  All five pass: issuer/root rollback, authenticated completed credentials,
  actual-time revocation and upgrade loss, receiver/application rollback,
  callback-bound enrollment/revocation, and refusal to reconstruct lost authority.

The consumer adaptation preserves its existing pre-copy guards via
`system::argument_size`. Its awaited enrollment uses this Runtime's bounded call
and outcome API. Auth and Runtime public identity boundaries use the same local
Types 0.21.1 source. Legacy Platform 0.33.7 remains a crypto supplier/oracle with
its own internal Types 0.20.4; explicit byte comparisons/construction bridge that
legacy verification boundary. It contributes no second Runtime executor.

## Limits and reproduction

Run from this repository with the pinned Auth commit available in `../ic-auth`
(or set `AUTH_SOURCE`). `make qualify-auth-runtime` checks the installed simulator
and uses the pinned local binary. Dependencies must already be cached for offline
Cargo. The isolated adaptation and source pin are reviewable in
`qualification/auth-runtime/prepare.mjs`; the sibling is read-only.

This proves Auth's Runtime execution requirements with its retained crypto
supplier. It does not prove replacement of that supplier, full Platform migration,
production adoption, macOS execution, composite/async queries or broader executor
contracts. Existing failed-cleanup and experimental transport limits remain.
