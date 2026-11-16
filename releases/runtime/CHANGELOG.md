# Runtime changelog

Release group: `runtime`. Covered packages: `ic-platform-runtime` and, from
0.23.3, `ic-platform-runtime-guard` and `ic-platform-macros`.
Return to the [group index](../../CHANGELOG.md).

## [0.23.13] - Unreleased

### 🔧 Tooling

- Adopt Toko Tooling 0.44.7 in both workspaces and use its flat qualification
  package layout. Retain the approved thiserror 2.0.18 compatibility selection.

## [0.23.12] - 2026-11-16

### ➕ Added

- Supply global-timer roots through the existing executor, without update-reply
  authority, plus IC timer arming and canister-version observation. Preserve
  callback cleanup and uncertain task observations; Scheduling migration remains
  separate. See the [timer boundary](../../docs/qualification/runtime-timers.md).

### 🧪 Qualification

- Adopt Testkit 0.107.1 for installed Runtime and pinned Auth qualification.
  Independently authenticate emitted certificates with the latest published
  Platform as well as the candidate implementation.

## [0.23.11] - 2026-11-16

### 📦 Publication

- Publish Runtime, macros and guard together after full local and archive
  qualification. Auth's five installed tests pass against the registry cohort.
  See the [release evidence](../../docs/qualification/2026-11-16-runtime-protocol-publication.md).

### ✨ Added

- Add dependency-free `system-raw` for Storage's caller-owned synchronous exports
  ([#6 — Synchronous system API for Storage](https://github.com/dragginzgame/ic-platform/issues/6)):
  argument/caller byte slices with offsets, current-controller checks,
  raw append/reply, byte traps and instruction counter zero.
- Qualify raw ABI limits, empty replies, live controller changes, trap rollback
  and lifecycle execution without an executor. Compile the pinned actual Storage
  startup, checkpoint migration-job and benchmark consumers through a retained
  migration patch. The group is published; consumer installed adoption remains separate.
- Advance Runtime support packages after the 0.23.10 source snapshot; preserve
  existing task-managed reply and explicit-entrypoint contracts.

## [0.23.10] - 2026-11-16 (GitHub source snapshot)

### ✨ Added

- Restore caller/canister identity, IC time, controller checks, argument length,
  bounded query certificate retrieval and exact 32-byte certified-data publication.
- Add synchronous queries and init/pre-upgrade/post-upgrade exports with explicit
  guard/decode/error policies. Add synchronous and async automatic updates with
  explicit encoding through the existing executor and single reply authority.
- Qualify system observations, authenticated certified roots, encoder-trap rollback,
  synchronous exclusion and lifecycle effects in the installed Runtime fixture.
  The pinned actual Auth fixture passes all five installed tests; legacy Platform
  crypto remains a supplier. No production migration is claimed.

## [0.23.9] - 2026-11-16

### 📦 Publication

- Publish the implemented scope independently, with explicit repository and
  package documentation. Keep production migration and the broader design gates
  separate; preserve existing experimental limits and all correctness checks.
- Advance package metadata after source delivery; Runtime support packages remain
  coordinated. Types stays at registry 0.21.1.

## [0.23.8] - Unreleased

### 🔧 Dependencies

- Align exact local Types dependency with the 0.21.1 release candidate. Advance
  the group after its delivered source snapshot; registry publication remains
  disabled. Runtime support packages advance together.

## [0.23.7] - 2026-11-16 (GitHub source snapshot)

### 🧪 Tests

- Extend the existing executor destructor fixture to check completion becomes
  observable only after destruction returns, and reply authority is unavailable
  during both completion and cleanup destruction. Later cleanup cannot rewrite
  an already completed handle. These native checks do not simulate IC rollback.
- Check reentrant cleanup of another suspended task restores the enclosing
  task context and preserves its single reply authority.
- Add bounded weak-reference observations to the installed corpus: successful
  completion/cleanup release handler ownership; a trapping cleanup retains it
  across later successful calls. Document the IC rollback boundary without
  adding automatic recovery or claiming private callback reclamation.
- Qualify an awaited manual handler returning without a reply: ingress reports
  `DidNotReply`, the task completes and releases handler ownership, and the
  remote effect remains committed. No automatic reply or retry is added.
- Compare a bounded consumer dispatch record with task lifecycle and ownership
  observations across successful callbacks, callback/cleanup traps and omitted
  replies. Keep actual timer integration and retry decisions outside the probe.
- Update the native qualification WAT compiler from 1.254.0 to 1.263.0 and
  its matching parser/encoder dependencies. PocketIC does not constrain this
  consumer selection; production graphs remain unchanged.
- Advance the coordinated Runtime group after the 0.23.6 GitHub snapshot.

## [0.23.6] - 2026-11-16 (GitHub source snapshot)

### 📚 Documentation

- Clarify that a task handle can remain `Running` after cleanup traps and its
  changes roll back. This local observation guarantees neither another poll nor
  permission to retry; the installed 0.23.5 corpus already demonstrates it.
- Advance Runtime, macros and the singleton guard together after the 0.23.5
  GitHub snapshot. Protocol and Types remain unchanged.

## [0.23.5] - 2026-11-16 (GitHub source snapshot)

Adopt the IC stop/drain/upgrade/start contract. The earlier live-upgrade
experiment bypassed that contract; it is not a supported-workflow blocker.

- Replace the failing live-upgrade acceptance experiment with a real held-call
  test proving Stopping waits for callbacks, Stopped precedes installation, and
  new calls work after restart. Keep callback/future state heap-local.
- Add a preview-first icp operator wrapper with explicit target selection, stage
  reporting, per-command timeout, stopped-state checks and module-hash validation.
  Failures do not force an upgrade or automatically restart a stopped canister.

- Extend the existing installed rollback probe with a destructor that traps
  during cleanup and a subsequent successful call. Check failed-cleanup writes
  roll back without undoing the peer's committed effect. No automatic recovery,
  retry or successful task-termination claim is introduced.
- Observe retained public task handles in the installed probe: completion and
  successful cleanup commit their respective terminal states; failed cleanup
  leaves `Running` observable even after a later independent call succeeds.

- Extend the installed corpus with unpolled drop, submitted future/input drop,
  a second await in the same handler and subsequent ordinary success. Verify
  exact peer effects and retained task states without adding product APIs or
  claiming direct observation of callback-registration reclamation.

- Qualify installed ordinary rejection and full refunds of distinct cycle
  attachments. Retain an earlier completion across a later callback, checking
  its reply/refund remains unchanged and the rejected peer update rolls back.

- Check installed callback-time refund capture against delayed future polling:
  observe publication through a dedicated waker, await a different callback,
  then require the first future to be immediately ready with its original refund.
  Repeat with attachments above u64::MAX and independently compare both u128
  refunds in the native driver to cover nonzero high and low words.

- Qualify installed cost-overflow and insufficient-balance refusals through
  both notify and first-poll awaited execution. Check exact required-cost facts,
  no peer increment and successful subsequent submission in the same handler.

- Qualify installed one-way admission: committed remote effects, rollback of
  queued notification on an initial caller trap, local receipt despite peer
  failure, and a later successful notification with prior task states intact.

- Qualify installed non-replicated query and nested root-entry refusal before
  guard/decoder/handler execution. Check the outer task can still reply and a
  subsequent ordinary update succeeds, without adding a query adapter.

- Qualify stale wakers retained after normal task completion: both wake forms
  leave the old task untouched and do not spuriously poll a later live call.
  Repeat with distinct tasks and verify the real callback completes normally.

- Reuse the entry-refusal helper from user destructors during installed normal
  completion and successful cleanup. Require nested-entry refusal before any
  admission callbacks, then verify destruction commits exactly once.

- Qualify cross-subnet bounded-wait uncertainty with controlled simulator time:
  require SYS_UNKNOWN while an independent read proves the remote increment
  committed, followed by a successful bounded call. Add no retry policy.

- Check fresh executor operation after a same-artifact heap-reset upgrade:
  local observations reset, independent peer effects persist, and a new awaited
  task completes and drops once. No persistence or in-flight migration claim.

- Qualify real system queue refusal after 500 read-only notifications in the
  pinned simulator: both notify and first-poll awaited execution preserve raw
  code 2, and submission succeeds after the queue drains. No retry is added.

- Qualify cleanup with two outstanding calls and a delayed orphan response:
  preserve both peer effects, commit one handler destruction and cleanup state,
  observe the final no-reply ingress result, and admit a later successful call.

- Qualify partial cycle acceptance and its rollback using a small independent
  WAT peer: acceptance of 5 from 17 refunds 12; trapping after acceptance refunds
  all 29 attached cycles. Keep the pinned WAT parser native-qualification-only.

## [0.23.4] - GitHub source snapshot

Source delivery only; not a registry publication or qualified release.

- Remove the duplicate test-only call state machine. Test production completion
  transitions directly, and cover borrowed-request destruction and terminal
  repoll through the real Wasm CallFuture. Merge the compiler-fixture runners
  and give shared IC0 Clippy checks one Make prerequisite.

- Resolve the workspace and hook wiring in [#2](https://github.com/dragginzgame/ic-platform/issues/2):
  consolidate the native driver into the standard qualification workspace, align
  the candidate with PocketIC's `thiserror` 2.0.18, and retain two-workspace
  dependency/formatting coverage. Adopt and activate the canonical Toko formatting
  hook through the authenticated installer, with per-clone setup documentation.

- Add a dedicated installed-canister probe for initial-handler and post-reply
  traps, committed-state rollback and subsequent successful updates. Server
  provisioning and offline admission use Testkit's supported API.
- Extend installed qualification to real inter-canister awaits, traps before and
  after callback replies, exact-once cleanup destruction and subsequent success.
  Verify callback writes roll back while the peer's committed effects survive.

## [0.23.3] - GitHub source snapshot

Delivered in commit `31a6da7`; not a registry publication or qualified release.

- Add `ic-platform-macros` to the coordinated Runtime group and expose its opt-in
  `update` attribute through Runtime. Generate fixed-ABI update exports with
  explicit Runtime path, guard, decoder, byte bound, manual reply and diverging
  failure handler. Qualify actual Wasm exports, conditional exclusion, renamed
  Runtime imports and compiler/codegen refusal of invalid declarations.
- Apply the maintainer-approved exception only to generated IC export attributes.
  Keep macro code and wrapper bodies safe; preserve forbid at every existing
  non-export qualification crate root while the dedicated export fixture uses
  workspace deny plus the generated item-scoped expectation.

- Add Wasm compile-pass/fail admission fixtures under a renamed Runtime import:
  accept owned input, reject invalid decoder signatures, borrowed decoded input
  and handler construction that borrows entry-local state.

- Add the real Cargo singleton guard, exact-pinned and coordinated with Runtime.
  Preserve the published executor `links` identity, reject mismatched compiled
  cohort versions, and qualify old/new and same-version/different-source graph
  conflicts. Default and call-only Runtime graphs still omit the guard.

- Add explicit `system::reply` with task-owned reply authority, exact raw bytes
  and typed refusal outside a live task or after its first reply. Qualify empty
  and binary replies, duplicate refusal, resumed handlers and cleanup refusal.
  No automatic reply or raw-IC0 bypass guarantee is introduced.

- Add `entrypoints::update_manual`, composing context admission, explicit guard,
  bounded capture and decoding with one method-attached task. Preserve stage
  errors and defer handler invocation until the executor owns its context.
- Qualify guard, size, decoder, nested-entry and non-replicated refusals plus
  successful owned input. Installed-canister execution and rollback remain pending.
- Extend the manual adapter proof through awaited replies and cleanup: retain
  decoded input across suspension, run admission stages once, destroy a cleaned
  handler once, and admit a subsequent successful update. The awaited corpus now
  covers 21 submissions and 14 task admissions.

## [0.23.2] - GitHub source snapshot

Delivered in commit `8540d6f`; not a registry publication or qualified release.

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
