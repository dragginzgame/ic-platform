# Development

The initial product workspace contains one unpublished Protocol package at 0.1.0.
The private consumer fixture also starts at 0.1.0. Neither package name appeared
in the crates.io sparse index when checked on 2026-11-15. Publication and release
adapters are not enabled; name availability does not reserve a registry name.

Prefer existing names for retained concepts. Check crates.io when introducing
each package: select its next SemVer version if it already exists, otherwise
0.1.0. Breaking pre-1.0 APIs require the next minor, while compatible changes use
the next patch. A published dependency such as Types retains its supplier's
version until a local replacement is actually introduced.

## Setup and dependencies

Use Rust 1.100.0 with rustfmt, Clippy and `wasm32-unknown-unknown`, as selected in
`rust-toolchain.toml`. Toko 0.44.3 is selected exactly by Cargo and installed via
its authenticated exporter. Root company files and `vendor/toko-tooling/` are
managed; consumer commands live in the root Makefile.

```sh
make install-tools
make tools-check
make toko-tooling-check
make deps-list
```

Tool setup can download executables and populate Toko's per-user store. Checks
are offline and do not install tools. Linux x86-64 and macOS ARM64/x86-64 are the
intended development hosts; native macOS execution has not yet been qualified in
this checkout.

`deps-list` lists workspace packages, versions and release groups, not transitive
dependencies. A dash means no release group has been configured. Use Cargo's
`tree --locked` with the relevant target/features to inspect a dependency graph.

```sh
# Networked, deliberate lockfile update; optional package/version selection:
make deps-update DEPENDENCY_PACKAGE=toko-tooling DEPENDENCY_PRECISE=0.44.3
```

Without selectors, `deps-update` updates all dependencies within the declared
requirements. It covers the root and a qualification workspace when one exists.
For a deliberate Toko upgrade, review the new release, edit the exact root pin,
update the lock, then run `make toko-tooling-preview` and `make toko-tooling-update`.
Do not edit installed bytes or their receipt.

## Focused implementation checks

```sh
make test-engine-request
make lint-engine-request
make doc-engine-request
make check-engine-request-wasm
make fmt-check
```

These select the Cloud Engine request encoder, not a full future Platform gate.
`make fmt` uses the installed formatting tools for Rust and Cargo manifests.
No commit hook has been activated. Literal-frame unit tests stay with the product;
the separate public consumer proof lives under `qualification/`.

## Initial contract and implementation boundary

`cloud_engine::encode_get_engine_views_args(&[String], max_bytes)` encodes exactly
one Candid text-vector argument. It preserves empty/Unicode/NUL/whitespace text
and order, checks the complete frame size before output allocation, and reports
typed output-limit/allocation failures. Request-count limits belong to the caller.
The request-only production dependency graph is empty; the source-only Toko dev marker is not
compiled into the product. The default feature set is empty.

The wire contract comes from the selected external method documented in
[the requirements](requirements.md) and the Candid binary format. The old
Platform implementation was inspected at the design's pinned source; no source
or tests were copied. This is not a clean-room claim. The new bounded encoder
and literal tests are authored here. Request tests do not establish response
decoding, identity crossing, a general codec, or complete Query migration.
The response-specific implementation and evidence are described below.

## Selected response decoding

Enable `cloud-engine-response` for `EngineView`, `EngineSubnet` and
`decode_get_engine_views_response`. This feature selects published Types 0.20.4
and Serde 1.0.229. A private declarative macro emits Rust records, schema and
reader dispatch once; no response Encode implementation is required. Required
and strict-optional policies are explicit. A public/general declaration API and
Candid-compatible optional fallback are not implemented by this increment.

```sh
make test-engine-response
make lint-engine-response
make doc-engine-response
make qualify-engine-response
make measure-engine-response
```

The qualification command checks both tooling selections, runs the independent public
consumer and published-Platform comparisons natively, applies Clippy, and
compiles the portable consumer tests for Wasm. It does not
execute Wasm or prove installed-canister/macOS behavior. `qualification/` owns
its manifest, lockfile and target directory, has no release group and is excluded
from product membership. Product code has no reverse qualification dependency.
`make deps-list`, `make deps-update` and formatting cover both workspaces.

The selected decoder requires exactly one vector result. Unknown fields are
validated and skipped without retaining their values. It accepts the standard
constructed types for skipping, including transparent service/function references;
opaque references and future constructors refuse. Strict option fields require
wire `opt` when present and never use fallback to hide a malformed payload. An
absent field or an encoded None remains absent; no nested value is manufactured.

Wire grammar was reviewed against the [Candid binary format](https://github.com/dfinity/candid/blob/master/spec/Candid.md#binary-format)
on 2026-11-15 (document version 0.1.8; fetched Git blob
`6a95576e230252ce6b5bd0e2280455c799588257`). This is a specification source,
not a copied implementation or a differential oracle.

Limits are mandatory: input bytes, header end offset, type count, cumulative
work, logical allocation, per-vector count and cumulative skipped work. Every
byte read and value traversal costs work; skipped values consume both budgets.
Header references/type traversal and selected vector admission also cost work.
Allocation accounts for the retained type-table containers and projected vectors
and strings before reservation; borrowed service names and discarded text are
not copied. Units are implementation-specific, not a process heap bound or a
promise of numerical equivalence with old decoder work counters. Both simple
constructed type paths and runtime nesting retain the 16-level ceiling.

Acceptance uses independent literal frames, including malformed skipped UTF-8,
strict optional failure, exact/one-below budgets and large zero-sized vectors.
The qualification consumer additionally proves a direct public Principal type
crossing and the JSON Serde shape. Native-only comparison tests use published
Platform 0.33.7's codec with independently authored record declarations and its
existing selected response decoder. They compare 30 combinations of absent/
present bindings, signed timestamps, unsigned sizes, Unicode/NUL text and node
ordering, plus request bytes, truncated/malformed frames and 189 byte mutations.
Refusal classes are compared for the fixed malformed corpus; mutation tests
compare admission and decoded values. Work counters are implementation-specific
and are not equated by these tests.

The oracle is a native-only development dependency of the private consumer;
it does not enter the product or Wasm consumer graph. Both implementations
resolve Principal through registry Types 0.20.4. No direct registry Candid
dependency or copied source/test corpus is introduced. These checks are bounded
differential evidence, not full Query adoption. Broader optional-coercion,
protocol-family and real-consumer qualification remains required before cutover.

`measure-engine-response` reports the smallest admitted logical work, allocation
and skip allowances for four fixed workloads. See the [criteria and measured
results](qualification/engine-view.md). These are deterministic resource-counter
observations, not timing, instruction or allocator measurements. The static
projection schema is now validated by a compile-time assertion; incoming type
table validation remains metered on every decode.

`make profile-engine-response` runs the isolated native heap comparison in the
release profile, then Clippy for that qualification package. It uses the
qualification lock's dhat development dependency; prepare new locked sources
with `cargo fetch --locked --manifest-path qualification/Cargo.toml` when needed.
The five fixed workloads satisfy zero regression in total/peak requested bytes
and zero allocation growth for discarded text. See the [heap criteria and
results](qualification/engine-view-heap.md) for scope, exact inputs and limits.
Profiler dependencies remain outside the product and portable consumer graphs.
The same serial heap test also requires complete allocation cleanup on six
partially constructed response failures: truncation, trailing data, exhausted
work and exhausted logical allocation. It checks the expected typed refusal as
well as zero live blocks/bytes; it does not inject host allocator failure.

`make qualify-engine-response` also exercises the [literal JSON compatibility
corpus](qualification/engine-view-serde.md): numeric boundaries, nested presence,
canonical Principal text, malformed selected fields and duplicate-field refusal.
The literal tests compile for Wasm; the published comparison executes on native
only. Wasm compilation is not evidence of Wasm execution.

`make measure-engine-wasm` builds and executes separate old/new decoder probes
using Node and reports complete module and code-section bytes with hashes. It
reports a 1,693-byte code-size increase, with size acceptance **undecided**; see
the [recipe and evidence](qualification/engine-view-wasm.md). The initial
agent-selected zero-growth threshold had no established launch-budget basis.
Contribution analysis is deferred; this measurement is not release admission.
Its private package requires exactly one of `candidate` or `published`.
