# System API and stable adapter implementation record

On 2026-11-10 a separate implementer context authored the adapters from the
[reviewed contract](system-adapters-contract.md). The existing adapter bodies,
tests, supplier implementations, provenance and Git history were not inputs.

## Inputs and separation

The only repository file read for this component was
`docs/design/system-adapters-contract.md`, relative to
`/home/adam/projects/ic-platform`. Previously read `AGENTS.md`,
`TOKO-TOOLING.md`, and `docs/principles/rust-code-hygiene.md` supplied policy and
were not reread. No other module or previous adapter/test source was inspected.

The reviewer clarified that a zero-length byte or Principal read still calls
its supplier copy exactly once with an empty slice and offset zero. Only an
absent certificate skips size/copy. The reviewer also confirmed the
PerformanceCounterType representation tags follow declaration order 0, 1, 2;
the Unrecognized payload's numeric projection is independent of its tag.
A supplemental public observation transcript was permitted but was not read or
used by this implementer. No work was delegated.

After qualification, the reviewer supplied one omitted optimization requirement:
`performance_counter`, `instruction_counter`, and
`call_context_instruction_counter` retain cross-crate inlining eligibility.
The implementer added `#[inline]` to those three functions without changing their
bodies, using only this supplemental fact and its own authored declarations.
No prior source was inspected and no size or speed improvement is claimed here.

A process-name check before editing found no Cargo/rustc process. The reviewer
later ran a separate existing-surface probe while the newly authored modules
were not wired; no implementation inputs were shared with that build.

## Authored files

Only these five files were written:

- `crates/ic-platform-runtime/src/cdk/api/mod.rs`
- `crates/ic-platform-runtime/src/cdk/api/tests.rs`
- `crates/ic-platform-runtime/src/cdk/stable/mod.rs`
- `crates/ic-platform-runtime/src/cdk/stable/tests.rs`
- `docs/design/system-adapters-implementation.md`

Exact-signature IC0 operations use explicit named exports/aliases. Other
functions adapt text, identity or error/result shape while delegating IC effects
to `crate::ic0::upstream`. Principal reads use a 29-byte stack buffer and reject
oversized lengths before copy. Byte readers copy into spare capacity and publish
length only after successful initialization. The single unsafe `set_len` block
has a local lint expectation and the supplier's exact-slice, all-or-trap safety
argument; tests add no unsafe allocator instrumentation.

UTF-8 conversion is strict and panics with the specified reason without including
rejected bytes. Certificate absence skips size/copy, while present-empty returns
Some(empty). Boolean and counter conversions preserve specified refusal/projection
semantics. Signing costs retain full u128 values and u32 errors. Stable adapters
preserve u64 offsets and growth values, map only the growth failure sentinel,
and leave bounds/traps upstream. No storage owner, scheduler, budget, dependency,
feature, cryptographic logic or additional public API was introduced.

Eight API helper tests and three stable tests were independently authored from
the contract. They cover initialized/empty/failed copies, strict text, bounded
identity reads, certificate presence, boolean errors, counter variants and
numeric projections, cost/error preservation, growth mapping, error formatting
and stable function signatures. These narrow helper tests do not simulate the
full IC0 layer or qualify installed IC effects.

## Validation and limitations

Rustfmt 1.99.1 completed successfully with no output on all four authored Rust
files, using `--edition 2024 --config-path /dev/null --config skip_children=true`.
Formatting was performed in separate implementation and test batches.
The API module was formatted again after the three inline attributes were added.
No Cargo, Clippy, compilation, tests or installed IC qualification was run by the
implementer. Parent integration owns wiring, old-owner removal, source/provenance
updates and actual native/Wasm/installed qualification.

This record describes permitted inputs and actions; it does not provide legal
clearance, claim absence of model training exposure, or mathematically prove
independent authorship. Passing compatibility checks alone would not establish
those conclusions.
