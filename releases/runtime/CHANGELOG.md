# Runtime changelog

Release group: `runtime`. Covered package: `ic-platform-runtime`.
Return to the [group index](../../CHANGELOG.md).

## [0.23.2] - Unreleased

Next compatible development increment; publication remains disabled.

- Add explicit `entrypoints::prepare`: run a pre-copy guard, bounded argument
  capture, then a caller-selected decoder, preserving stage-specific errors.
  Qualify rejection ordering and owned output through the public Wasm consumer;
  export generation, reply ownership and installed execution remain pending.

- Add wasm32 `system::read_args(maximum)` under `call-ic0`, checking raw message
  size before allocation/copy and returning an explicit oversize error. Keep
  codec selection and authorization with the caller. Qualify six ABI cases in
  the existing IC0 check, including a u32::MAX size refused before copying.

- Add native refusal-destructor regressions for nested entry and identifier
  exhaustion, checking reentrant admission and preservation of task context.

## [0.23.1] - GitHub source snapshot

Delivered in commit `58e2dae`; not a registry publication or qualified release.

Next compatible development increment; publication remains disabled.

- Fix cross-task polling of submitted call futures: retain the first polling
  context and reject a different task before cloning/replacing its waker or
  consuming a response. Unpolled futures can still move into their initial task.
  Add a Wasm regression reproduced against the previous implementation.

- Qualify synchronous submission refusal within an executor task: preserve the
  raw refusal, complete the local task, and admit a subsequent successful task.
- Correct stale implementation-status prose after reviewing the 0.23.0 delivery.

## [0.23.0] - GitHub source snapshot

Delivered in commit `c7650ca`; not a registry publication or qualified release.

Initial unpublished replacement after registry Runtime 0.22.5. This is an
incomplete runtime, not a qualified executor or submission implementation.

### Added

- Add an experimental manual update adapter with one method-attached root task,
  read-only lifecycle handles and callback-to-task ownership. Cleanup removes
  polling authority and destroys the task outside executor borrows before
  reporting termination. Stale wakes cannot target another task. Entrypoint,
  child/migratory-task and installed rollback qualification remain pending.

- Add experimental wasm32 `Call::execute` / `CallFuture` with first-poll
  submission, non-recycled integer callback handles, owned response/refund
  capture and post-drop callback retirement. Installed rollback qualification
  remains pending.

- Add consuming wasm32 `Call::notify` with cost/balance admission and exact raw
  submission refusals. Preserve one-way callback sentinels, ignore response-wait
  selection, and return only a local queue-admission receipt without retries.

- Add opt-in wasm32 `call-ic0` cost observation with checked attachment addition,
  behind the maintainer-approved IC-only unsafe boundary. No callback or
  submission backend is claimed by this first import.

- Add owned immutable completion observations with exact reply/rejection data,
  explicit SYS_UNKNOWN classification, raw unknown codes and captured optional
  refunds. These data constructors do not claim an installed callback backend.

- Add opt-in inert Call preparation with consuming argument/cycle setters,
  independent method/argument ownership and explicit Wait selection.
- Select local Types 0.21.0 for the new call boundary. Default Runtime has no
  production dependencies; no published Runtime is linked into this cohort.

### Testing

- Extend the Wasm corpus to 15 submissions and seven executor tasks: out-of-order
  replies, another submission from a resumed callback, cleanup with two live
  calls, exact-once orphan callback retirement and isolation of an unrelated task.

- Add six native executor regressions and three Wasm executor tasks covering
  callback-driven completion, cleanup termination, destructor reentry, handle
  drop and non-replicated entry refusal in the existing awaited-call harness.

- Execute actual response/cleanup function-table callbacks in an independent
  Wasm host, including stale-handle and malformed-text traps, plus native registry
  exhaustion tests. Keep notify's no-callback artifact checks separate.

- Extend the IC0 host corpus to validate complete notify import ordering, exact
  bytes, high-word cycle attachments, early refusal and no retry/callback setup.

- Execute an independent Wasm IC0 cost host checking import identity, exact byte
  lengths, initialized memory, 128-bit results and attachment overflow.

- Add an independent local Types/Runtime consumer with native and standalone
  Wasm execution and singleton production-graph checks, without an executor.

- Add a test-only first-poll/callback custody model covering submission refusal,
  post-drop delivery, borrowed request destruction and reentrant waker cleanup.
  This is deterministic ownership evidence, not an installed execution backend.

- Check exact request facts, borrowed storage, owned storage surviving source
  destruction and replacement of an argument borrow by owned storage.
