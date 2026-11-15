# Runtime raw argument admission

Runtime 0.23.2 adds `system::read_args(maximum)` on wasm32 with `call-ic0`.
This is a system operation used to prepare explicit-codec entrypoints, not an
entrypoint generator or authorization policy. Identity-only guards should run
before it; consumers choose their own byte allowance and decode after capture.

The implementation reads the current argument size, compares it with the
inclusive limit, then allocates initialized storage and copies exact bytes.
Oversize input returns `ArgumentTooLarge { actual, maximum }` without allocation
or copying. Empty input returns an empty vector without a copy. The bound covers
the raw argument buffer, not a decoder's work or the application's total heap.
Allocation failure and invalid IC execution contexts can trap.

## Guard and decoder ordering

`entrypoints::prepare(maximum, guard, decode)` runs the guard before any Runtime
argument-size query, allocation or copy. A successful guard proceeds to bounded
capture, followed by the selected decoder. Each stage runs at most once; errors
retain their original values under `AdmissionError::Guard`, `Arguments` or
`Decode`. No stage turns an error into a reply, remote rejection or retry.
Guards and decoders remain trusted caller code with their own side effects.

The decoded result cannot borrow the temporary argument buffer. Consumers capture
owned inputs before entering an async task. Checks that require decoded fields
run afterward rather than silently changing the pre-copy guard contract. This
helper introduces no Candid dependency, task context or automatic reply owner.

The [IC argument contract](https://docs.internetcomputer.org/references/ic-interface-spec/canister-interface/#method-arguments)
is execution-local: a reply callback sees its reply, not the original method
arguments. Capture method arguments before awaiting when they must survive.
The owned output does not retain an IC pointer. No response-size limit is added
to awaited calls by this caller-selected ingress operation.

Unsafe code remains confined to the approved private `call/ic0` import/wrapper
module, reusing the existing size/copy imports. The public system module and
consumer fixture contain no unsafe code. No dependency or feature default changes.

`make qualify-runtime-ic0` builds a separate `runtime-arguments` binary in the
existing private consumer package. Its host admits exactly `msg_arg_data_size`
and `msg_arg_data_copy`, verifies initialized output memory, and checks the full
import order. Six cases cover empty input, exact-limit binary bytes, one-over
limit, zero allowance with nonempty input, u32::MAX refusal under a small limit,
and a later smaller message while preserving earlier owned bytes. Only two
copies occur. The u32::MAX case is an ABI/admission probe, not a claim that such
a message is accepted by a replica.

Four further cases use the public guarded helper: denied guard with a decoder
that must never run; accepted guard with oversized bytes; decoder refusal with
the exact error retained; and successful owned output. Stage counters check
single execution and guard-before-decode ordering. The host's complete trace
requires no extra size/copy import for the denied guard and no copy for the
oversize refusal. The full corpus has ten cases and four copies.

Local Rust 1.100.0 / Linux / Node 24.21.0: strict Clippy/Rustdoc, the new corpus,
existing notify and awaited executor corpora, and dependency graphs pass.
Argument fixture SHA-256:
`126ae780bc561397b08b04b3624bd93f03b3f8ddc1e73ba64790e0b52c89ebbf`.

Generator integration of this ordering, exports, single reply ownership and
installed canister checks remain pending. This host proves neither replica context
admission nor rollback.
