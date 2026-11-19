# Installed update and callback rollback — Runtime 0.23.7

This probe runs the candidate's update adapters in PocketIC 16.1.0 on
application subnets. It covers immediate update rollback and two resumed-handler
trap/cleanup paths plus a trapping cleanup, not the complete Runtime acceptance gate.

## Native management and Entropy callers

`make qualify-runtime-ic0` runs the public `management_context` consumer on the
native host and type-checks the same seven calls for Wasm. Native task admission
returns `NonReplicated` without polling. Balance, page count, version and both
pricing operations require a canister. The real IC implementations retain the
installed raw, timer and call qualification.

For [#34](https://github.com/dragginzgame/ic-platform/issues/34), disposable
copies of the Canic and Entropy working trees based on `1a7dd271` and `aca4c6be`
selected the local Platform cohort and exposed the affected handlers and
dependencies on both targets. Canic Management checked with all features,
including Backup, and its Store maintenance module. Entropy's buffered adapter
and HTTP probe checked on native Linux and Wasm. Its native tests exercised
refusal before source custody and cancellation. These were supplier checks with
local dependency and module substitutions.

## Supported upgrade contract

Use the IC stop/drain/upgrade/start workflow. Only `Stopped`, not `Stopping`,
admits replacing code. Heap-local callback registrations and suspended futures
are not persisted or reconstructed. The [operator workflow](../development.md#canister-upgrades)
reports progress and enforces this boundary outside Runtime.

## Ownership and setup

`qualification/runtime-rollback` builds one private Wasm canister using the
candidate Runtime, guard, macros and Types. Its graph check requires exactly one
of each and excludes Testkit and the published runtime. No historical canister
source or generated fixture was copied.

The native driver is the `qualification/runtime-installed` member of the
standard qualification workspace. It now selects published `ic-testkit` 0.107.1
and uses its upstream PocketIC client re-export. The [private qualification supervisor](testkit-server.md) owns checksum-verified
server setup, offline admission and managed process teardown. The driver owns
one instance with four canisters on two application subnets and explicitly closes it after the checks. Failure drops the owned
server, stopping its disposable instance. No public network canisters are used.

The product catalog pins `thiserror = "=2.0.18"`, matching PocketIC 16.1.0, at
the maintainer's direction. Product and qualification locks select that same
version. Native adapter identities never cross into the canister graph; the
driver passes raw bytes. There are only the product and qualification workspaces,
with the same exact Toko Tooling version and shared formatting/dependency commands.

Prepare locked sources with
`cargo fetch --locked --manifest-path qualification/Cargo.toml`,
then run `make install-runtime-server` explicitly. `make runtime-server-check`
verifies the installed bundle offline. `make qualify-runtime-installed` performs
strict host/fixture Clippy, builds the Wasm with the existing fixed build settings,
runs the driver and checks its production graph. Tests do not download a server.
CI uses these same targets on each configured native host.

## Observed contract

The canister has one byte of thread-local state and a raw one-byte command.
Its generated export uses explicit bounded decoding and manual replies. The
handler also verifies that a second ordinary reply is refused.

| Update | Expected committed result |
| --- | --- |
| Read initial state | Reply `[0]` |
| Increment and reply | Reply `[1]` |
| Increment, then panic before reply | `CanisterTrapped`, no reply |
| Read after initial trap | Reply `[1]` |
| Increment, reply, then panic | `CanisterTrapped`, no reply |
| Read after post-reply trap | Reply `[1]` |
| Increment after both failures | Reply `[2]` |

Rust's Wasm panic executes `unreachable`; the expected IC error is
`CanisterTrapped`, not `CanisterCalledTrap` (an explicit IC trap). Both failures
must be typed execution rejections. A rejection alone does not establish rollback:
the subsequent reads assert the committed state, and successful updates verify
continued admission and usable reply authority.

## Awaited callback rollback and cleanup

The driver installs the same candidate artifact into a separate peer canister.
The caller's `await_probe` receives an explicit mode and the peer's Principal
bytes, admits them through `entrypoints::update_manual`, increments a started counter,
and awaits an unbounded raw call to the peer's incrementing `probe` method.
The successful callback returns the peer's exact one-byte reply.

The first six awaited calls exercise success, trap immediately after await, trap after
the resumed handler issues its reply, success after both failures, a trap during
cleanup destruction, and another successful call. After
each call, independent updates read caller counters and the peer's committed
state. Expected observations are:

| Mode | Caller started/completed/dropped | Peer state |
| --- | --- | --- |
| Success | `[1, 1, 1]` | `[1]` |
| Trap after await | `[2, 1, 2]` | `[2]` |
| Trap after callback reply | `[3, 1, 3]` | `[3]` |
| Subsequent success | `[4, 2, 4]` | `[4]` |
| Trap during cleanup | `[5, 2, 4]` | `[5]` |
| Success after failed cleanup | `[6, 3, 5]` | `[6]` |
| Drop local call interest, then await another | `[7, 4, 6]` | `[8]` |
| Subsequent ordinary success | `[8, 5, 7]` | `[9]` |

The completed counter is written before either callback trap and must roll back.
The started counter commits before suspension and remains. A destructor held
across suspension increments the dropped counter and attempts an ordinary reply:
normal completion must refuse with `AlreadyReplied`, while cleanup must refuse
with `NoActiveTask`. The destructor also calls the real `update_manual` adapter
through the existing entry-refusal helper and requires `NestedEntry`. Its guard,
decoder and handler would panic if invoked. Normal completion and successful
cleanup therefore exercise user destructor re-entry without an executor borrow
trap or accidental task admission. Only after this assertion does the destructor
increment its observed counter. The deliberately trapping cleanup case cannot
independently prove which assertion it reached, so it is not counted as successful
re-entry evidence. The restored future is therefore destroyed exactly once
in these cleanup paths. Both trapping callbacks return `CanisterTrapped`; the
post-reply trap must not deliver its rolled-back reply. The peer's increment
survives each caller trap, demonstrating that caller rollback does not undo
already committed remote effects.

For the cleanup-failure case, mode 3 holds a destructor across suspension that
increments `DROPPED` and then panics. The resumed handler first panics after
await, causing cleanup to destroy the restored future. Both callback writes and
the cleanup's increment must roll back: the observed counters stay `[5, 2, 4]`.
The peer still reports `[5]`, and a subsequent independent call succeeds. This
checks rollback of failed cleanup, not reclamation of the restored task or
callback registration. A later successful task does not settle that retained
local state or authorize retrying the failed operation.

### Handler ownership after cleanup failure (0.23.7)

Each of the eight awaited handlers owns one strong `Rc<()>` marker across its
suspension. The fixture retains only a weak reference in a bounded observation
list; neither that list nor the public task handle owns the marker. After every
ingress, `observe_custody` reads all marker strong counts. Completion and
successful cleanup leave zero; the deliberately trapping cleanup leaves one,
which remains one across subsequent successful handlers. This directly observes
retained handler ownership, beyond the rolled-back destructor counter. Weak
records retain their control blocks, so this is not an allocator-byte measurement
or proof about the private callback registry.

The [IC cleanup contract](https://docs.internetcomputer.org/references/ic-interface-spec/canister-interface/)
rolls back a trapping cleanup and takes no further action for that call. Runtime
cannot commit retirement inside that failed execution. Keep user destructors
small and nontrapping; application reconciliation must not infer remote failure
or retry permission from a local trap. No automatic retry, deferred destructor
queue or callback-recovery subsystem is introduced. The later [installed accounting run](#callback-reclamation-timer-dispatch-and-late-completion) measures the retained entries.

### Committed task observations

The awaited entry uses a fixed-ABI IC export calling the same public
`update_manual` adapter used by the generated wrapper. This small test wrapper
retains its returned `TaskHandle` instead of dropping it. Immediate `probe` and
`observe` exports still use the macro. The export attribute has an item-scoped
IC-only safety expectation; the wrapper body is safe Rust.

Each initial submission observes `Running` before returning. The fixture retains
at most eight handles, bounded by this corpus, and the driver reads all prior
handles after every completed ingress:

| Call | Committed handle state |
| --- | --- |
| Initial success | `Completed` |
| Trap after await | `TerminatedByCleanup` |
| Trap after callback reply | `TerminatedByCleanup` |
| Subsequent success | `Completed` |
| Trap during cleanup | `Running` |
| Success after failed cleanup | `Completed` |
| Drop local call interest, then await another | `Completed` |
| Subsequent ordinary success | `Completed` |

The failed-cleanup handle remains `Running` after the last successful call too.
This is the restored local state, not a promise that the task will resume, nor
permission to retry. The probe verifies no false committed termination claim;
it does not demonstrate reclamation of the task or callback registration, or
integration with a persistent timer dispatch record.

### Dropped local call interest

A seventh handler constructs and drops an unpolled call, then first-polls a
second call using borrowed local method/argument storage. It asserts `Pending`,
drops that submitted future, exits the scope containing its inputs, and awaits
an ordinary third call to the same peer. The unpolled call contributes no peer
effect; the submitted-but-dropped call and the awaited call each increment the
peer. The reply and independent peer read both report `[8]`, rather than `[7]`
(local drop cancelled submission) or `[9]` (unpolled construction submitted).
The handler completes and destroys its observer once. An eighth ordinary call
then returns `[9]`; every prior task handle retains its expected state.

This checks installed submission/drop behavior and continued execution with
abandoned call interest. The later [installed accounting run](#callback-reclamation-timer-dispatch-and-late-completion) counts the entries after the abandoned response drains.

### Rejected completion and captured refunds

A separate generated `refund_probe` handler runs after the eight lifecycle cases.
It reads the peer through an awaited call carrying 17 cycles; the peer replies
`[9]` and accepts none of the attachment. The returned completion records
`Some(17)`. While retaining that completion, the handler makes another call with
29 cycles to the peer's increment-then-panic path. It receives `Outcome::Rejected`
with raw code 5, classification `CanisterError`, nonempty rejection text and
`Some(29)` refunded cycles. The first completion still owns `[9]` and `Some(17)`
after this different callback context.

The driver independently checks both refunds encoded as little-endian u128s and
the rejection code. It then reads peer state `[9]`, proving the rejected peer
update's increment rolled back, and rechecks all retained task states unchanged.
A second invocation exercises delayed consumption. It first-polls the 17-cycle
call with a dedicated observation waker, then awaits the 29-cycle rejecting
call without polling the first future again. Once the second await completes,
the test requires that the first waker has fired. It polls that first future
exactly once and requires immediate `Ready`, with reply `[9]` and refund
`Some(17)`, while the second completion records `Some(29)`. This proves the
first completion was published before delayed polling and retained its own
callback refund rather than reading the later callback's refund context.
The test fails if this ordering is not observed; it does not silently accept
a pending future or wait for another callback.

A third invocation repeats delayed consumption with attachments `2^64 + 17`
and `2^65 + 29`, exercising distinct nonzero high words and low words. The
native driver funds the disposable caller with `2^80` cycles and independently
compares all 16 bytes of each returned refund against decimal literal values
18,446,744,073,709,551,633 and 36,893,488,147,419,103,261. Both full refunds are
preserved through actual submission, callbacks and delayed consumption.

This covers owned refund observations across callbacks, delayed consumption,
large attachments and ordinary rejection, not partial cycle acceptance or exact
replica diagnostic text. These observations are distinct from a full cost or
settlement ledger; no retry policy is inferred.

### Resource refusal before submission

The generated `resource_probe` handler tries attachments of `u128::MAX` and
`2^100`, each through both `notify` and first-poll awaited execution. The former
must return `SubmitError::CostOverflow`; the latter must return
`InsufficientBalance` with an exact required cost matching `required_cycles`,
a positive cost above the attachment, and an available balance below that cost.
The caller has only `2^80` initial cycles. Each awaited future is explicitly
polled once and must immediately return `Ready(Err(...))`; a pending future
fails the probe instead of waiting for a callback.

All four refused requests would increment the peer if submitted. After them,
the same handler makes a valid read call and replies `[9]`. The driver also
reads `[9]` independently from the peer. Thus these refusals preserve the peer's
state and leave the current handler able to submit and complete another call.
This covers preflight resource errors, not a nonzero `call_perform` refusal or
exhaustion of callback identifiers. It makes no claim about cancellation after
successful queue admission.

### One-way admission, commit and rollback

Four `notify_probe` invocations exercise successful notification, an initial
caller trap after local admission, a notification whose peer traps, and a final
successful notification. Each nontrapping handler independently awaits a peer
read after obtaining its notification receipt. The native driver also reads
the peer after every invocation. Starting at `[9]`, both observations require:

| Case | Peer counter |
| --- | --- |
| Committed increment notification | `[10]` |
| Caller traps after receipt, before commit | `[10]` |
| Receipt returned, peer increment then traps | `[10]` |
| Subsequent committed increment notification | `[11]` |

The caller trap is `CanisterTrapped`. In the peer-trap case the caller's receipt
still succeeds, but the independently read peer state shows no committed
increment. The receipt therefore establishes local admission only. The final
success and unchanged prior task observations check continued operation. This
is one installed same-subnet corpus, not a delivery guarantee, retry policy,
or direct measurement of private callback allocations.

### Non-replicated query and nested-entry refusal

A dedicated fixed-ABI query export calls `update_manual` and requires
`UpdateError::Entry(NonReplicated)`. Its guard, decoder and handler each panic
if reached. After the assertion it returns without replying, so the driver
requires exactly `CanisterDidNotReply`; a wrong error or an invoked admission
stage would instead trap. This deliberately avoids adding a raw reply API for
queries merely to carry the test result. It is not a supported query adapter.

A generated update handler invokes the same refusal helper while its outer task
is live, requiring `NestedEntry` before any admission stage runs. The outer
handler then successfully replies `nested-refused`. A subsequent ordinary update
increments the caller's original committed counter from `[2]` to `[3]` and
replies successfully. The query export uses only the existing item-scoped IC
export attribute expectation; its body and assertion helper are safe Rust.

These cases qualify non-replicated query rejection and nested root-entry
exclusion. They do not qualify replicated/composite queries, migration between
method contexts, child tasks or general context isolation.

### Stale completed-task wakers

The generated `stale_probe` captures its executor waker during one poll, replies,
and completes. A later invocation starts a normal awaited peer read. After its
first poll returns `Pending`, it invokes both `wake_by_ref` and consuming `wake`
on the completed task's saved waker. The live call must be polled exactly twice:
once for submission and once for its real response. Extra scheduling fails the
probe. Its peer reply must remain `[11]`, and the capture poll count must not
increase. The driver repeats the capture/wake sequence with a new pair of tasks,
checking counts 1 then 2 and two live-call polls in both runs.

This checks installed stale-waker isolation after normal task completion, not
after failed cleanup or upgrade. Saved wakers are consumed by the probe; no
persistent production registry or scheduler was added.

### Bounded-wait uncertainty with committed remote effects

The caller and original peer are explicitly placed on the same application
subnet. A third canister, using the same artifact, is placed on a different
application subnet for this case. The driver submits `bounded_probe` without
waiting, ticks once to start its one-second bounded call, advances simulated
time by 60 seconds, and then awaits the ingress result. No wall-clock sleep,
background timer or product retry mechanism is introduced.

The caller requires `Outcome::OutcomeUnknown` with raw code 6 and classification
`SysUnknown`, and reports `[6]`. An independent read of the remote canister
requires `[1]`: the uncertain operation has committed its increment. This is
explicit evidence that timeout does not establish nonexecution. The fixture
then makes a separate 30-second bounded call with no artificial delay; its
reply and independent peer read both require `[2]`.

The scenario fails unless both the uncertain callback and the first committed
remote effect are observed. A zero-timeout same-subnet call was unsuitable for
forcing expiry because it completed successfully in the simulator; the committed
probe uses the cross-subnet/time-controlled setup above. This qualifies one
installed uncertain-outcome path, not every deadline boundary, late-response
custody or settlement condition. No automatic retry is performed.

### Heap reset and fresh execution after upgrade

After the preceding calls have returned, the driver re-observes all eight
retained task handles, including the failed-cleanup handle still reporting
`Running`. It stops the caller, confirms IC status `Stopped`, upgrades it with
the same Wasm artifact and starts it again, without reinstalling either peer. This fixture has no
stable-memory persistence or pre/post-upgrade hooks.

The caller's committed counter must reset from `[3]` to `[0]`; its task/counter
observation must become `[0, 0, 0]` with no retained handles. Independent reads
still return `[11]` from the original peer and `[2]` from the cross-subnet peer.
A fresh ordinary awaited call then increments the cross-subnet peer to `[3]`,
returns that reply, and reports `[1, 1, 1, 1]` for one started, completed,
destroyed task with a `Completed` handle. A final peer read confirms `[3]`.

This proves fresh executor operation after a same-artifact heap-reset upgrade
and survival of independent remote effects. Erasing a heap is not evidence that
failed cleanup reclaimed ownership correctly before upgrade. The test does not
carry a live network callback across upgrade, restore task handles, migrate
stable state, qualify a version transition or prescribe production recovery.
Application state migration and snapshot procedures belong to the consuming
application. Preserving live Rust futures across upgrade is not required.

### System queue refusal and later submission

After the upgrade checks, `submission_probe` queues read-only notifications to
the original peer within one update execution. The loop is bounded to 1,024
attempts and fails if it cannot reach a system refusal; it does not retry a
failed request. With the pinned PocketIC 16.1.0 setup, exactly 500 notifications
are admitted before `notify` returns `SubmitError::Refused` with raw code 2.

While the queue remains full, the handler constructs an awaited read call and
polls it once. It must return `Ready(Err(SubmitError::Refused(code)))` with the
same raw code, rather than `Pending`, a preflight balance error or a remote
rejection. The handler replies with its accepted count and code; the native
driver independently requires `(500, 2)`. This is a pinned qualification
observation, not a portable queue-capacity promise or a `RejectCode` mapping.

The driver advances ten simulator rounds, then invokes the existing notification
probe. Its notification and awaited read succeed and return `[12]`; an
independent peer read also returns `[12]`. The saturating calls are read-only,
so only this final successful notification increments the peer. This qualifies
refusal propagation and subsequent admission, without direct accounting of
private callback-registry entries or adding product backoff/retry policy.

### Cleanup with a second outstanding response

After queue recovery, mode 5 of the existing awaited handler first-polls two
calls. The first increments the peer directly. The second calls `delayed_reply`,
which makes an additional awaited hop to increment that same peer before replying.
Both are submitted before the handler awaits the first completion. Before its
intentional trap, the handler checks that the second future still polls pending;
if not, it replies with an ordering-failure sentinel instead of trapping, so the
driver cannot mistake that failure for the required callback trap.

Cleanup destroys the suspended handler and drops local interest in both calls.
The delayed response then arrives without a live owning task. In this pinned
corpus the final ingress result is `CanisterDidNotReply`, rather than the
`CanisterTrapped` result of the single-outstanding-call cases. The driver requires
that precise outcome, advances ten additional rounds, and reads peer `[14]`,
proving both increments committed after the preceding `[12]` state.

The caller observation must be `[2, 1, 2, 1, 2]`: one earlier completed task,
one new cleanup termination, no committed completion for the trapped handler,
and exactly one new destructor count. A subsequent ordinary awaited call replies
`[15]` and observes `[3, 2, 3, 1, 2, 1]`. This provides installed evidence for
late response handling after successful cleanup without reviving or destroying
the terminated handler twice. It does not directly measure private registry
allocations or establish recovery after cleanup itself fails.

### Partial acceptance and rollback of acceptance

A fourth, independent peer is compiled from the small, hand-authored
`qualification/runtime-installed/accept-cycles.wat` fixture. It has no Rust
executor, identity types, storage machinery or copied historical source. Its
`accept` export invokes `ic0.msg_cycles_accept128` for five cycles into the first
16 bytes of its initialized linear memory, replies with those exact bytes, and
returns. Its `accept_trap` export makes the same acceptance request then traps.

The candidate calls `accept` with 17 cycles and requires a reply encoding five
accepted cycles plus refund `Some(12)`. While retaining that completion, it calls
`accept_trap` with 29 cycles and requires ordinary rejection code 5 plus refund
`Some(29)`: the trapping peer's acceptance did not commit. The first refund must
still be 12. The native driver independently checks all three returned u128s:
accepted amount 5, partial refund 12 and rollback refund 29.

`wat = 1.263.0` is selected only in the native qualification driver. This is
a local exact pin, not a PocketIC constraint: PocketIC 16.1.0 lists WAT only as
its own development dependency, which does not constrain this consumer. Cargo's
qualification lock binds WAT and its parser dependencies. The caller's production graph still
excludes these tools; no cycle-acceptance API or new Rust unsafe boundary was
added. The fixture's IC0 signature was checked against published `ic0` 1.2.0's
foreign declaration. That crate is not added as a dependency.

WAT source SHA-256:
`70e55d3b6ba87ebfd3e316bbaa8feaf6779098f350d9a7de439a27e00ac57c16`.

### Stop, drain, upgrade and restart

`upgrade.rs` supplies a bounded peer handshake through existing Runtime APIs.
The peer records started/finished flags and makes at most 128 cross-subnet read
calls until an independent release update arrives. The driver requires peer
status `[1, 0]` and no final ingress result before requesting stop. It submits
the standard management `stop_canister` call without blocking its ability to
release the peer, using the existing native Candid and management types suppliers.
These direct qualification dependencies add no product dependency.

The driver observes `Stopping` while both the original ingress and stop request
remain unfinished. It releases the peer, lets callbacks finish, and waits for
the stop response. Before installation it requires `Stopped` and the original
handler's successful `released` reply. After upgrade it still requires `Stopped`,
then starts and requires `Running`. Heap observations reset; a fresh awaited call
returns `[16]` and reports exactly one completed/destroyed task. The driver logs
each stage so an operator can distinguish draining from installation/restart.
The earlier heap-reset check also now explicitly stops before upgrading.

A previous negative experiment bypassed stopping and demonstrated callback-ID
aliasing after heap reset. That explains the operational prerequisite; it is
not a requirement to preserve live Rust futures through upgrade. Its deliberately
failing acceptance test and second overlapping peer were removed. No callback
persistence, automatic cancellation/retry or Runtime pre-upgrade guard is added.
This matches the [IC specification's upgrade guidance](https://docs.internetcomputer.org/references/ic-interface-spec/management-canister/).

These are real replica-simulator callbacks and cleanup invocations, not direct
host calls into function-table entries. The drop case includes abandoned and awaited calls in the same handler; no
cleanup-failure recovery is inferred.

Local Linux x86-64 validation passed seven immediate updates, eight awaited handlers
and sixteen observation updates plus eight weak-ownership observations, plus three invocations of the two-call refund handler and two
post-refund observations, and a resource handler checking four refusals plus
a successful read and independent peer observation. Four notification handlers,
four peer reads and a final task-state observation also passed, followed by
a non-replicated query refusal, nested-entry refusal and successful ordinary
update, plus two stale-waker capture/wake pairs and two cross-subnet bounded
calls with independent peer reads, followed by a same-artifact upgrade and
seven state/call observations, then bounded queue saturation, two system
refusals and successful later submission, followed by two-outstanding-call
cleanup and a fresh successful awaited call, and the independent partial-cycle
acceptance/rollback peer and stopped-state draining before upgrade, with Rust 1.100.0,
Testkit 0.106.0 and its admitted PocketIC 16.1.0 server. Strict host/fixture Clippy
and the Wasm graph check passed. Hosted/macOS execution remains pending.

Installed artifact SHA-256:
`248f58dc52781ed261067e12a17a140d6df55469daeb426fc93530dcbf025cc8`.

## Manual reply omission (0.23.7)

After the final drained upgrade and successful fresh call, mode 6 of the existing
awaited handler calls the peer, consumes its response and returns without replying.
The destructor skips its reply-attempt assertion for this case so it cannot supply
an accidental reply; ownership and nested-entry checks remain enabled. PocketIC
reports `CanisterDidNotReply`, while counters become `[2, 2, 2]`, both retained
handles report `Completed`, and both weak marker strong counts are zero. The
peer read returns 17, confirming its effect committed. Missing an application
reply does not turn normal local completion into cleanup termination or undo the
remote operation. Runtime supplies no automatic response or retry.

The expanded installed corpus, strict host/Wasm Clippy and dependency graph checks
passed locally. This closes the missing-reply lifecycle case, not the remaining
resource-accounting or production-adoption gates below.

## Consumer dispatch observation (0.23.7)

The same bounded awaited fixture now keeps a consumer-owned record per handler:
`Dispatched` (wire byte 0) before the first await, then `ReplyObserved` (byte 1)
after receiving the peer response. The native driver reads every prior record
after each ingress, alongside the public task handles and weak ownership probes.
No record is updated by cleanup or inferred from a task handle.

| Path | Committed consumer record | Task handle |
| --- | --- | --- |
| Successful reply | ReplyObserved | Completed |
| Callback trap with successful cleanup | Dispatched | TerminatedByCleanup |
| Callback and cleanup both trap | Dispatched | Running |
| Handler omits its own reply after observing the peer | ReplyObserved | Completed |

The peer effect commits in every row. A consumer therefore cannot equate task
termination with remote rollback, or local completion with delivery of its own
reply. Repeated reads after later successful calls preserve those distinctions.
This minimal probe tests transaction boundaries of an in-heap dispatch record;
it introduces no Scheduling dependency, timer, retry decision, storage allocation
or persistence across upgrade. Actual timer-dispatch integration remains separate.
The expanded installed corpus, host/Wasm Clippy and dependency checks passed.

## Callback reclamation, timer dispatch and late completion

The Linux PocketIC 16.1.0 run of `make qualify-runtime-installed` on 18 November
2026 exercised the local Runtime 0.23.17 implementation. The ordinary fixture
uses that implementation directly. For private accounting the same Make target
builds a second Wasm with `--cfg ic_platform_qualification_accounting` from
the same working sources. Its query reads the lengths of the callback and
executor maps. The [installed driver](../../qualification/runtime-installed/src/native/accounting.rs)
checks both counts. The ordinary fixture also checks its export table for
absence of the accounting query.
Only the instrumented artifact exposes the two counts. The units below are
occupied entries. Heap capacity, allocator overhead and bytes captured by an
arbitrary user future have no implied bound.

| Installed observation | Callback entries | Executor tasks |
| --- | ---: | ---: |
| Fresh instance, ordinary reply, successful cleanup after callback trap, or submitted call dropped before peer reply and then drained | 0 | 0 |
| First trapping destructor in cleanup, followed by an ordinary successful call | 1 | 1 |
| Second trapping destructor, followed by a successful cleanup and another ordinary call | 2 | 2 |
| Stop, observe Stopped, upgrade, start | 0 | 0 |

The earlier weak-reference observations identify application data retained in a
suspended handler. These counters locate the corresponding retained owners.
Failed cleanup rolls back their removal. Repetition can accumulate live entries
and captured application memory until the heap is replaced. A successful
subsequent call releases its own entries. Consumers must design destructors to
complete without trapping and retain uncertainty about the failed attempt.

For the late-call case two independent ingress callers issue calls to distinct
held peers on a different application subnet. Both peers report started and
unfinished. The caller owns two tasks and two registrations before expiry.
After a five-second advance within the ten-second window both remain. After
expiry the bounded caller receives raw SYS_UNKNOWN (6) and the counts fall to
one each. Releasing that peer permits its remote handler to finish while the
other peer still reports unfinished and its caller still owns one slot and
task. Only the second peer's release delivers its expected reply and returns
both counts to zero. The driver observes these transitions without resubmission.

The uninstrumented timer case commits a versioned three-byte record in
fixture-owned stable memory before arming a real global timer. On successful
reply the observed stage is complete. A resumed callback trap restores the
submitted stage, with the public handle distinguishing committed cleanup from
failed cleanup. An initial poll trap leaves the pre-dispatch stage and no
handle, as does explicit disarming. A stopped upgrade migrates record schema
1 to 2, preserving mode and stage while clearing the heap-only observations.
The fixture supplies the schema and migration policy.

### Consumer destructor review

The installed test verifies reclamation after a callback trap when cleanup
completes. A trap during cleanup rolls back the entire cleanup, including
registry removal. Repeated retention in that case is a Runtime limitation.

In the pinned Auth host fixture (`b78991e9833320692921bf766e51ad361c9c8c4d`),
the receiver's `control` handler awaits `enrollment::refresh`. The mutable
state borrow ends before the bounded call, the request bytes move into the call,
and no application-defined destructor or lock guard spans that suspension.
The injected metadata and entry traps execute in the resumed handler.
The [Auth fixture preparation](../../qualification/auth-runtime/prepare.mjs)
identifies the source revision and adapter.

In Runtime, dropping a [call future](../../crates/ic-platform-runtime/src/call/future.rs)
retires its shared state. The [state implementation](../../crates/ic-platform-runtime/src/call/state/mod.rs)
releases the mutex before dropping the previous waker. During cleanup the
[executor](../../crates/ic-platform-runtime/src/executor/mod.rs) releases its
mutable borrow before dropping the handler. The installed callback and timer
probes deliberately put assertions in destructors to exercise re-entry and
cleanup failure.

For each new consumer, inspect values and guards held across an awaited call,
including their nested destructors. Keep cleanup work bounded. Check for
panicking assertions or unwraps, conflicting state borrows and work that grows
with application state. The [IC security guidance](https://docs.internetcomputer.org/guides/security/inter-canister-calls/)
illustrates a small Rust guard that releases local bookkeeping in `Drop`.

## Remaining gates

The installed cases above qualify the selected method-attached update and
timer contracts. Repeated failed cleanup remains a resource-retention limitation.
Application-specific memory bounds, Scheduling's production watchdog and
dispatch migration, and consumer application schema migrations require their
respective consumer evidence. Child and migratory tasks are outside the
supported executor scope. The recorded deadline and context cases cover the
specified interleaving on PocketIC. They establish neither arbitrary ordering
coverage nor mainnet or native macOS execution.
