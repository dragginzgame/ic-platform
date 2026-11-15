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

## First-poll custody proposal

`call/tests/custody.rs` now contains an executable, test-only ownership model.
It consumes the actual public Call request through a private test transport.
No model implementation is compiled into the product, and no public execute
method claims an available backend. The model is newly written; the published
0.22.5 callback implementation was inspected but not copied.

The proposed transition owns two separate interests: a future's completion
handle and one consuming delivery token retained by successful submission.
The token contains no request borrow. A synchronous refusal drops it; dropping
local interest after submission leaves it alive until delivery. Delivery into
abandoned state discards the observation without waking a task. A completed
future cannot submit again. Observed refund absence is distinct from zero.

The new tests cover unpolled drop, first-poll-only submission, repeated pending
polls, exact request facts, raw synchronous refusal, callback lifetime after
future/request-source destruction, terminal repoll, and reentrant Wake/Drop
inspection during waker replacement, completion and abandonment. Waker cloning
is outside the lock by construction; arbitrary RawWaker clone callbacks are not
exercised by these safe-Wake fixtures. Native mutex behavior is not an IC task
or rollback model.

The [IC system API](https://docs.internetcomputer.org/references/ic-interface-spec/canister-interface/)
was reviewed on 2026-11-15. Submission commits only after the enclosing execution
returns without trapping. A response callback trap can invoke cleanup; a cleanup
trap discards its changes. Bounded-wait SYS_UNKNOWN does not establish whether
the callee executed. Installed proofs must retain these distinctions; native
unwinding or a fake rejection cannot stand in for cleanup/termination evidence.

The [awaited implementation](runtime-awaited.md) now supplies experimental
method-attached task ownership, callback-time response/refund capture and integer
callback registration. Entrypoint admission and installed rollback/cleanup proof
remain pending. Consuming notify is implemented at the
[IC0 boundary](runtime-ic0-boundary.md). Unsafe code remains limited to the
maintainer-approved IC import/wrapper boundary; the custody model is supporting
evidence and does not enable a second executor.

Local validation passes ten tests (three preparation and seven custody-model
tests), strict Clippy/Rustdoc, Wasm test compilation, formatting and both existing
Runtime graph selections. The custody model has executed on Linux only;
Wasm compilation does not establish execution or installed-canister behavior.

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
The record is a captured observation, not a settlement ledger. The test-only
custody model now delivers this production type; its delayed-poll fixture
changes the simulated ambient refund after delivery and verifies the original
snapshot survives alongside exact rejection text and unknown codes. Actual
callback-time system reads now execute in the [awaited ABI fixture](runtime-awaited.md);
installed evidence remains pending.

Fourteen tests now pass locally on Linux, alongside strict Clippy/Rustdoc,
Wasm test compilation and both Runtime dependency graph selections. The added
cases cover reply-buffer ownership, empty/large payloads, Unicode/NUL rejection
text, all known codes, unknown codes and refund snapshot retention. This replaces
the model-only reply record without claiming that the model is a production
execution backend.

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
