# Runtime: inert call preparation

The crates.io sparse index checked on 2026-11-15 reports Runtime 0.22.5 as
the latest non-yanked version, checksum
`92a494623fc12ba0673eb18518612e95e9a103d88ba71d4bd8fa902a227fde80`.
The breaking replacement starts at unpublished 0.23.0. No version bump is applied
to already pending Protocol 0.1.0 or Types 0.21.0 merely for GitHub delivery.

The first real slice implements inert Call preparation from the design's proposed
API. It preserves exact bytes, method names, explicit wait values and cycle
attachments with independently borrowed/owned storage. The published prepared
call API was read as contract evidence; no source/tests or backend were copied.
There is one non-cloneable request shape, no lower public builder, and no implicit
codec, endpoint admission, reservation or submission.

The `call` feature uses only local Types 0.21.0. Defaults are empty and select
no production dependencies. The new cohort does not link published Runtime or
an executor. Protocol independently remains on registry Types pending migration.

`make qualify-runtime-call` checks construction/ownership tests, strict Clippy,
Rustdoc and Wasm compilation. It is not evidence of first-poll submission,
callback capture, notify, cancellation, trap recovery or installed IC behavior.
Those remain the next implementation and qualification obligations in DESIGN.md.
No placeholder execute method, simulated production backend or unsupported
success receipt is exposed. Guard/macros packages join the group when real
implementation requires them.

Local Linux validation on Rust 1.100.0 passed three ownership tests, default
compilation, strict Clippy/Rustdoc, Wasm test compilation, formatting and workflow
lint. Selected Wasm normal/build graphs pass: default is dependency-free and
`call` contains exactly one local Types identity, with no executor, Protocol,
published Runtime or hash supplier. Hosted/macOS and installed execution evidence
remain pending.

## Production custody qualification

The exploratory call state machine was retired in Runtime 0.23.4. Its separate
transport/future implementation tested itself and no longer supplied useful
production regression coverage.

The real CallFuture now uses private `call/state.rs` for waker replacement,
completion publication and retirement. Native tests exercise those exact
functions, checking that Wake/Drop can reenter the completion mutex after it is
released. They cover replacement, publication, abandonment and cleanup, plus
retention of the captured completion and delivery after abandonment. Safe Wake
fixtures do not exercise arbitrary RawWaker clone callbacks; cloning remains
outside the lock by construction.

The [awaited ABI fixture](runtime-awaited.md) exercises the real transport:
unpolled drop, first-poll-only submission, repeated pending polls, refusal,
callback survival after borrowed request storage is destroyed, exact captured
responses/refunds, terminal repoll refusal and orphan retirement. No second
transport implementation is maintained for these checks.

The [installed probe](runtime-installed.md) adds immediate/resumed-handler traps,
rollback and successful cleanup. Cleanup failure, bounded-wait uncertainty and
the other listed installed gates remain open. Native unwinding and direct host
callback invocation do not substitute for installed rollback evidence.

## Completion observation contract

The production `call` feature now exposes immutable `Completion` records and
`Outcome`, `Rejection` and `RejectCode`. Public constructors accept observations
as data; they do not authenticate origin or invoke a system API. Reply bytes and
rejection text transfer ownership without normalization, decoding or an arbitrary
size cap. Zero cannot be supplied as a rejection code. Known codes keep protocol
discriminants; unrecognized nonzero codes keep their full u32 value.

The [protocol reject-code catalogue](https://docs.internetcomputer.org/references/ic-interface-spec/https-interface/#reject-codes)
was reviewed on 2026-11-15. Code 6 is classified as `OutcomeUnknown`; other
rejections retain their code and exact message. An unrecognized code has no
invented settlement meaning. Neither rejection nor uncertainty grants retry
permission. The constructors never synthesize a response for local termination.

Refunds remain `Option<u128>`: absence, zero and the maximum value are distinct.
The record is a captured observation, not a settlement ledger. Production-state
tests retain exact captured values; the real awaited ABI fixture reads refunds
at callback time before wake and later polling. Completion constructor tests
cover reply-buffer ownership, empty/large payloads, Unicode/NUL rejection text,
known/unknown codes and absent/zero/full-width refunds. The consolidated native
Runtime selection passes 18 tests; it no longer counts eight model-only tests.

## Independent local-cohort consumer

The existing private `types-consumer` now has an opt-in `runtime-call` selection
with exact local Runtime 0.23.0 and Types 0.21.0 path dependencies. It passes a
direct Principal through Account.owner into Call and assigns Call.target back to
that same directly imported type for all 30 Principal lengths. It also exercises
borrowed/owned request data, exact wait/cycle values, reply ownership, known and
unknown rejection codes, uncertain outcomes and refund presence through public
APIs. No executor is linked or claimed.

`make qualify-runtime-consumer` runs this corpus natively and as standalone Wasm,
plus strict Clippy. `make check-runtime-consumer-graph` admits exactly one Types
and Runtime identity in the selected normal/build closure, with no Protocol,
executor or SHA-2 supplier. CI runs it for Wasm and the native host. Published
Types remains a native-only development oracle for the existing identity tests,
not a production crossing or a conversion hidden in Runtime.

The next IC0 implementation scope is recorded in the
[approved boundary](runtime-ic0-boundary.md). Its first cost import is implemented;
this request/observation consumer slice itself requires no unsafe code.

Local Linux execution passed with Rust 1.100.0 and Node 24.21.0, including
native tests/Clippy and both native/Wasm graph selections. The standalone
Runtime consumer Wasm executed with no imports and artifact SHA-256
`93a242bbe4ddcfe02bb852d51d874861ee02346030bb13c9525617112f3f03aa`.
Hosted/macOS and installed IC execution remain unproved by this result.
