# IC0 boundary for the first Runtime call

Status: scoped policy approved by the maintainer on 2026-11-15, explicitly
limited to IC code. Cost observation and consuming one-way submission are
implemented. Experimental awaited calls and integer callback registration are
also implemented, with experimental method-attached executor integration;
entrypoint and installed qualification remain pending.

The first backend needs IC0 imports for call construction, argument copying,
cycle attachment, bounded-wait selection, cleanup registration, submission,
reply/rejection reads and callback-time refunds. Rust requires unsafe declarations
and calls for these foreign functions. The former root `unsafe_code = "forbid"`
blocked this boundary. The approved change below permits only the narrow IC
boundary anticipated by DESIGN.md.

## Approved policy change

- Change the root lint from `forbid` to `deny`; retain
  `unsafe_op_in_unsafe_fn = "deny"` and inherited lints for every package.
- Add crate-level `#![forbid(unsafe_code)]` to Types and Protocol so their existing
  prohibition remains enforced. Qualification keeps its independent prohibition.
- Permit narrowly justified `#[expect(unsafe_code, reason = ...)]` only on the
  necessary Wasm import declarations and wrapper functions in Runtime's private
  `call/ic0` module. No crate-wide allowance, transmute, unchecked UTF-8 or raw
  allocation ownership conversion is included.
- Record this scope in AGENTS.md. The exception admits implementation and tests,
  not deployment, publication or claims of installed qualification.

## Callback identity and memory custody

Use a safe registry of owned completion registrations, indexed by a monotonically
issued nonzero u32 handle. Pass that integer as the callback environment, never a
Rust pointer. Do not reuse handles during the instance lifetime; exhaustion
refuses before submission. This makes stale handles unable to address a later
registration, without raw Arc/Rc reconstruction. The registry owns exactly one
physical-call obligation; the future separately owns local interest.

Reserve/register before perform. A synchronous refusal removes and releases the
registration outside internal borrows. Successful perform retains it through
the enclosing execution's commit. Reply/reject removes the entry, captures exact
response and refund, then releases borrows before completion/wake. A callback
trap rolls back this removal, allowing cleanup to find the original registration.
Cleanup removes it and delegates local task termination to the single executor;
it does not fabricate a remote rejection. A cleanup trap cannot claim successful
termination or a released physical-call obligation.

The backend must qualify that rollback premise in installed canisters. Native
unwinding tests cannot substitute for it. Duplicate/stale handles must fail closed
without waking or terminating another task. Registrations never retain borrowed
request pointers; request bytes only cross imports that synchronously copy them.

## Narrow unsafe obligations

Each wrapper checks target width, lengths, valid source/destination memory and
the IC execution context required by its import. Output memory is fully
initialized before use; no uninitialized vector-length manipulation is needed.
Callback function-table indices must refer to the exact required Wasm signature.
Refusal and cleanup must not invoke arbitrary destructors under registry borrows.
No public callback handle, registration constructor or arbitrary backend injection
is exposed to applications.

## Required implementation evidence

Preserve the deterministic first-poll model, and add a single-executor installed
probe covering reply/reject, unknown codes, bounded-wait uncertainty, dropped
interest, refusal, callback trap, cleanup trap, waker/destructor reentrancy and
subsequent valid calls. Verify callback-time refund capture before waking.
Inspect the final Wasm imports and exact local Types/Runtime dependency identity.
Until these pass, the backend remains unpublished and incomplete; no successful
`execute` or `notify` behavior is inferred from request/observation tests.

## Implemented cost import

`call-ic0` implies `call`, but defaults stay empty. On wasm32 it adds
`Call::required_cycles`, which reads `ic0.cost_call` and checked-adds the requested
attachment. Overflow returns `CostOverflow` rather than a saturated value that
could pass later balance admission incorrectly. This method does not reserve or
submit anything. Native builds expose no fake implementation of the observation.
The system still owns execution-context restrictions and valid request limits.

The foreign declaration and individual IC wrappers carry unsafe expectations.
The wrapper passes byte lengths as u64 and a wasm32 pointer to sixteen initialized
bytes. The synchronous result is interpreted as little-endian u128. Types and
Protocol retain crate-level unsafe prohibitions; qualification remains forbidden.
The later [awaited transport](runtime-awaited.md) adds a safe integer callback
registry; no raw ownership transfer is used.

`make qualify-runtime-ic0` runs strict Wasm Clippy/Rustdoc and builds an independent
consumer artifact. The original cost-only Node host required just `ic0.cost_call`,
checks Unicode/NUL method byte length and argument length, verifies the output
buffer is initialized/in-bounds, and writes independent 128-bit values. Five cases
cover zero, a high-word value, maximum and two overflowing attachment sums.

The ABI harness passed locally on Rust 1.100.0 and Node 24.21.0, artifact SHA-256
`5966eabd9c87f4696cbe79129c17392f62e883e2de6b662400d0c87da81d95ae`.
Selected graph admission also passed. This verifies the Wasm import boundary
against a deterministic host; it does not prove live replica costs or installed
IC execution-context handling. The source contract is the
[IC system API cost section](https://docs.internetcomputer.org/references/ic-interface-spec/canister-interface/#cycle-cost-calculation),
reviewed on 2026-11-15. Hosted/macOS results remain pending.

## Consuming one-way submission

`Call::notify(self)` is now available on wasm32 with `call-ic0`. It first checks
cost-plus-attachment overflow and liquid balance, then assembles the exact target,
method, arguments and attachment and invokes `call_perform` once. Insufficient
balance preserves both observed amounts; synchronous refusal preserves the full
nonzero u32 result. These are submission errors, not remote rejections. A private
`Submitted` receipt proves only local queue admission, conditional on the current
execution committing without a later trap.

One-way calls follow the published upstream
[ic0 1.2.0 convention](https://github.com/dfinity/cdk-rs/blob/624606f7fd1ecd6668e9608594946486aec29167/ic0/src/lib.rs):
both function indices and environments are u32::MAX. No Rust callback function,
callback state, cleanup registration, future or executor is installed. This is
not a claim that the replica never processes a response. Wait selection is not
forwarded; notify has no response-wait policy. The wrapper was independently
written after reviewing this upstream ABI convention; no upstream dependency,
implementation module or test corpus is copied. Unsafe code stays in `call/ic0`.

The expanded host admits exactly six IC0 imports and validates all 28 import
invocations in order. In addition to the original cost cases, it covers exact
balance with a high-word cycle attachment, zero cost/empty payload, insufficient
balance, overflow, known refusal and an unknown raw refusal. Early failures must
not reach `call_new`; both failures after perform must not retry. No wait or
cleanup import is admitted. Public receipt/error matching executes in the
independent consumer. Its final artifact SHA-256 is
`5c9b43337cbfe2071d198e33e12dd28260982f3bc4b04c70ab06ee39da263fdf`.
Strict Wasm Clippy/Rustdoc, the ABI execution and singleton graph checks pass
locally on Rust 1.100.0 / Node 24.21.0. Installed delivery, rollback and one-way
replica behavior remain required evidence, not inferred from this host model.
