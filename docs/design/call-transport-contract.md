# Raw call transport replacement contract

Specification-only input for a separately authorized implementation context.
Do not inspect the existing CDK call implementation/tests, source fingerprints,
supplier call implementations, Git history or diffs. Read this specification,
the executor contract, and policy already admitted to your context. Existing
implementation layout and internal data structures are not requirements.

Write a replacement in `crates/ic-platform-runtime/src/cdk/call/`, leaving the
current implementation untouched for coordinator observations and qualification.
The coordinator owns wiring and retirement. Use ordinary private modules and
fresh `tests.rs` or `tests/` modules, not another crate or transport abstraction.
Write an implementation-input record under `docs/design`.

## Public contract

Keep the following names and signatures exported from `cdk::call`. Do not expose
generic System API, codec, retry, scheduling, metrics or test-injection APIs.

- `WaitPolicy`: Clone, Copy, Debug, Eq, PartialEq; variants
  `Bounded { timeout_seconds: u32 }` and `Unbounded`.
- `Call<'m, 'a>`: Clone, Debug; constructors
  `bounded_wait(Principal, &'m str)` and `unbounded_wait(Principal, &'m str)`.
  Defaults: zero cycles, empty transport bytes; bounded timeout is 300 seconds.
  Methods consume self and return Self: `with_raw_args(&'a [u8])`,
  `take_raw_args(Vec<u8>)`, `with_cycles(u128)`, `with_wait_policy(WaitPolicy)`.
  Last selection wins, with no addition/clamping or implicit argument encoding.
  Borrowed payload remains borrowed; owned payload is consumed. Method remains
  borrowed. `get_cost(&self) -> u128` saturating-adds attached cycles to supplier
  request cost, using UTF-8 method byte length and payload byte length as u64.
  `oneway(&self) -> Result<(), OnewayError>` does not consume the builder.
- `Call` implements IntoFuture with output `Result<Response, CallFailed>`.
  Its concrete public associated future type remains unnamed through the facade
  export list (a private implementation module can supply it). Both Call and
  its future retain Send, Sync, Unpin auto-traits; multithreaded execution is
  not supported. Creating/dropping an unpolled future submits nothing.
- `Response`: Debug; opaque owned bytes; `into_bytes(self) -> Vec<u8>`, Deref
  Target=[u8], AsRef<[u8]>, Borrow<[u8]>, PartialEq<Self>, PartialEq<Vec<u8>>,
  PartialEq<&[u8]>. No public constructor or additional codec methods.
- `CallFailed`: Error, Debug, Clone, three transparent variants wrapping and
  convertible From the same-named error: InsufficientLiquidCycleBalance,
  CallPerformFailed, CallRejected.
- `OnewayError`: Error, Debug, Clone, two transparent variants wrapping and
  convertible From InsufficientLiquidCycleBalance and CallPerformFailed.
- `InsufficientLiquidCycleBalance`: Error, Debug, Clone; public u128 fields
  `available`, `required`. Display:
  `insufficient liquid cycles balance, available: {available}, required: {required}`.
- `CallPerformFailed`: unit Error, Debug, Clone; Display `call perform failed`.
- `CallRejected`: Error, Debug, Clone; private data; constructor
  `with_rejection(u32, String) -> Self`; getters `raw_reject_code() -> u32`,
  `reject_message() -> &str`, `reject_code() -> Result<RejectCode, UnrecognizedRejectCode>`.
  Display `call rejected: {code} - {message}`. Preserve all raw u32 codes and text.
- `UnrecognizedRejectCode`: Error, Debug, Clone, PartialEq, Eq; private numeric
  payload; Display `unrecognized reject code: {code}`.
- Explicitly re-export the existing `crate::cdk::reject_code::RejectCode`.
  Its TryFrom<u64> supplies known 1–6 mappings and unknown refusal. No catalogue
  dependency or duplicate identity is introduced.

Ordinary derived Debug field layout is not a protocol representation. Error
messages above, trait behavior and public byte/error identities are preserved.

## Submission and lifecycle

Submission first compares liquid cycles to saturated get_cost. Equality succeeds;
insufficiency returns the typed amounts without invoking any call construction.
Oneway performs no callback/context retention and returns success only for
supplier status zero; every other u32 status becomes CallPerformFailed.

The first future poll attempts submission once. On insufficient cycles or
nonzero submission result, return the corresponding Ready error and become
terminal. On successful enqueue, retain the current method and optional current
task through the existing executor APIs, remember the supplied Waker, and return
Pending. Subsequent pending polls update the Waker, without resubmission.
Completed results are returned exactly once. Polling after returning Ready traps
with `CallFuture polled after completing`; polling cancelled/trapped state traps
with `Call already trapped`.

Ordinary future destruction does not revoke an enqueued call. Its callback
ownership and originating method survive until delivery/cleanup. Destruction
during explicit executor recovery marks the outstanding completion cancelled;
later reply delivery must trap `Call already trapped`, and cleanup of already
cancelled state must release callback ownership without recancelling tasks.

Reply/reject delivery reads reject code. Zero gives copied argument bytes;
nonzero gives exact raw u32 code and strict UTF-8 rejection text from the API.
Store the result before waking. Resume the retained method through
`in_callback_executor_context_for`, waking the most recently registered Waker
there. Drop locks/borrows before invoking the executor or arbitrary Waker/Drop
code; wakeup may immediately cause polling, further calls and destruction.

If callback execution traps, replica rollback restores pre-callback memory and
then invokes cleanup with the same environment. Cleanup must not read message
reply/reject APIs. Install a synthetic CallRejected(4, "cleanup") completion,
extract the method/task, enter `in_trap_recovery_context_for`, cancel all tasks
attached to that method, then cancel the saved optional task (it may be migratory).
Recovery is owned by the executor; do not construct another scheduler/context.
Externally polled futures may observe the synthetic error. Unexpected callback
state must fail closed. Native unwinding is not a model of IC rollback.

## System API and safety boundary

Use existing `crate::cdk::api` functions: `cost_call(u64,u64)->u128`,
`canister_liquid_cycle_balance()->u128`, `msg_reject_code()->u32`,
`msg_arg_data()->Vec<u8>`, `msg_reject_msg()->String`, `trap(impl AsRef<str>)->!`.
Use existing `crate::Principal::as_slice()` and the executor public contract.
TaskHandle::current returns Option<TaskHandle>; neither task nor method handles
are Clone. Preserve their ownership when replacing pending Wakers.

The explicit `crate::ic0::upstream` internal alias has these supplier operations:

```text
unsafe call_new(callee: &[u8], method: &str,
    reply: unsafe extern "C" fn(usize), reply_env: usize,
    reject: unsafe extern "C" fn(usize), reject_env: usize)
unsafe call_on_cleanup(cleanup: unsafe extern "C" fn(usize), env: usize)
call_new_oneway(callee: &[u8], method: &str)
call_data_append(bytes: &[u8])
call_cycles_add128(cycles: u128)
call_with_best_effort_response(timeout_seconds: u32)
call_perform() -> u32
```

Construction uses the same reply/reject completion path plus cleanup for awaited
calls; oneway has no cleanup. Append nonempty payload only; add nonzero cycles
only; apply any bounded timeout, including zero, unchanged; unbounded omits it.
Then perform. A nonzero result guarantees no callback will run: reclaim any
registered environment immediately. Zero transfers its ownership to exactly one
committed callback or cleanup. They cannot run before perform returns. The
callback-trap rollback means cleanup acquires the restored ownership, not a
second live copy. Document each raw-pointer/callback safety argument locally.

Prefer a callback-owned completion structure containing only owned/static data,
separate from borrowed request data, to avoid passing borrowed request lifetimes
through the ABI. Preserve Send/Sync auto-traits with standard primitives, no
unsafe impls. No new dependency. Keep unsafe scope minimal with local reasoned
lint expectations; the workspace denies unsafe outside authorized ABI bridges.
Drop arbitrary Wakers, method handles and user-owned data outside held locks.
Do not alter cryptography, network checks, Candid, timers or storage behavior.

## Tests and coordination

Author fresh tests from this contract. A private cfg(test) System API seam may
record submissions and inject replies, rejections, cost/balance/status and
callbacks. Never expose injection in production/public API. Verify lazy/unpolled
drop, exactly-once submission, borrowed/owned payloads, timeout/cycle boundaries,
saturating cost and u128 values, status refusal/environment reclamation, reply
and unknown rejection bytes, latest Waker, ordinary pending drop, cleanup
without message reads, recovery cancellation, and Send/Sync/Unpin traits.
Include destructor/reentrant Waker checks. Test callback logic through ordinary
Rust helpers when a native panic would otherwise unwind across extern C.

Use the real executor for context tests. Do not implement a replica, second
executor, cryptographic routine or reused upstream algorithm test suite.
The coordinator owns Cargo/Clippy and installed raw/prepared × Wire/external
Candid tests. Do not run Cargo while the coordinator observes the old owner.
Formatting only your authored files with skip_children is allowed. Record input
separation honestly, including its limits; no legal clearance assertion.
