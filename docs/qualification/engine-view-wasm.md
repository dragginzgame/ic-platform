# EngineView standalone Wasm comparison

## Comparison scope and acceptance status

Build the private `engine-view-wasm` binary twice, with exactly one of its
`published` and `candidate` features. Defaults are empty. Name lookup returned
no crates.io package on 2026-11-15, so this new private package starts at 0.1.0.
Each artifact selects only one decoder. The shared probe uses an independent
literal response and checks all selected fields. Input and output pass through
`black_box` to retain general decoding and output custody. Native execution
checks both feature selections; native success alone does not prove Wasm execution.

The initial experiment used an agent-selected zero-growth threshold. Review
with the maintainer established that it was not grounded in a launch budget or
an approved requirement. Size acceptance is therefore **undecided**; the relative
increase alone is not a qualification failure. The maintainer deferred analysis
of Wasm contributions. Preserve both measurements and artifact hashes for that
later work; no compressed metric is used.

Both builds use Rust 1.100.0, wasm32-unknown-unknown, the qualification lock,
an explicit release profile (opt-level 3, LTO false, 16 codegen units,
debug false, strip none, panic unwind, incremental/overflow checks/debug assertions
false), and no post-link optimizer. No custom RUSTFLAGS or
compiler wrapper is selected. Separate target directories retain both outputs.
This is an initial library/consumer-glue comparison, not Query's deployment
recipe: IC entrypoints, runtime, application code and deployment optimization
are absent. It establishes no canister admission, instruction count or installed
execution claim. No product or deployment profile is changed.

## Reproduction and measurement

Run `make measure-engine-wasm` with Node on PATH. Both builds are offline and
locked. The local `qualification/engine-view-wasm.mjs` validates each entire
module using Node's WebAssembly engine, requires no imports and invokes exported
`main` to execute the fixture assertions. After validation it scans outer section
lengths, reporting file bytes and section ID 10 payload bytes (including the
function-body count, excluding the section ID and length prefix). SHA-256 binds
the report to the bytes executed. Invalid modules or failed fixture assertions
still fail the command; size differences are reported with acceptance undecided.
Node supplies no IC system APIs or instruction meter.

The package rejects no-feature and combined-feature builds; run its two explicit
selections separately rather than workspace all-features.

## Observed result — 2026-11-15: size acceptance undecided

Linux x86-64, Rust 1.100.0 and Node v24.21.0 / V8 13.6.233.17-node.53 executed
both Wasm fixtures successfully. Both native selections also ran and passed
Clippy with warnings denied. Locked graphs select only the intended decoder,
with registry Types 0.20.4 as the common identity supplier.

| Metric | Published bytes | Candidate bytes | Candidate minus published |
| --- | ---: | ---: | ---: |
| Complete uncompressed module | 1,728,799 | 1,654,168 | -74,631 |
| Code-section payload | 53,279 | 54,972 | +1,693 |

Code increased by 1,693 bytes (about 3.2%). Neither that increase nor the total
module reduction establishes acceptance. These unstripped modules retain standard
library debug/name sections, so total reduction is not demonstrated deployed
size savings. An initial build without explicit strip configuration produced
smaller complete modules; it is not the fixed recipe reported here.

This measurement does not isolate the cause of code growth. Function-contribution
analysis is deferred at the maintainer's request. Explicit
vector bounds and fallible allocation must remain. Instruction-cost, deployed
consumer and macOS qualification are outstanding.

Inputs at observation:

- Published artifact SHA-256: `16c537db5549c4410da7e08ef5f283c726055a14cc8fa1c0088dbdf5a4ca1286`.
- Candidate artifact SHA-256: `2988d4198cc3ec31d157291dcc7fa98de6680e48a16c8432cfb076eb17bf327b`.
- Product lock SHA-256: `bc82b53beb5a552bd71eaec7fb07f2062805485e732b5738da236c9633a0df81`.
- Qualification lock SHA-256: `d3ac9513352fb1abbca5c9b4c00e91cce4fb7994c1a908d1ebceb86c73e5870d`.
- Product/probe source-inventory SHA-256: `8adf716c5bd1b5e92127e0178ddf2dcd156dd24997d0ee56f092e33252ececc8`.
  Reproduce with `rg --files crates/ic-platform-protocol qualification/crates/engine-view-wasm | sort | xargs sha256sum | sha256sum`.
  This identifies working manifests and source, not a commit. Artifacts remain
  under `target/engine-view-wasm/{published,candidate}/wasm32-unknown-unknown/release/`.
