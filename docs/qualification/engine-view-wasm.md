# EngineView standalone Wasm comparison

## Current execution and acceptance

`make measure-engine-wasm` builds only the current Protocol probe using the
`candidate` feature. `make qualify-engine-wasm` executes its independent literal
admission corpus. No legacy decoder or published feature remains. Defaults stay
empty. Node validates the module, requires no imports, executes assertions and
reports the module hash and (for measurement) file and code-section sizes.

Size acceptance remains **undecided**. Historical paired measurements below
remain evidence; current runs do not rebuild the old artifact or report a new
paired delta. This standalone probe is not an installed canister, IC cost model,
or proof of production Query migration.

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
  Reproduce with `rg --files crates/ic-platform-protocol qualification/engine-view-wasm | sort | xargs sha256sum | sha256sum`.
  This identifies working manifests and source, not a commit. Artifacts remain
  under `target/engine-view-wasm/{published,candidate}/wasm32-unknown-unknown/release/`.

## Executed admission corpus — 2026-11-15

`make qualify-engine-wasm` builds the separate `engine-view-admission` binary for
each decoder and runs it through `node qualification/engine-view-wasm.mjs admission`.
This preserves the original size-probe source and workload. Admission mode
validates both modules, requires no imports, executes their assertions and emits
artifact hashes without comparing sizes. The same fixed compiler profile applies.

An independently specified literal record carries id, optional int64 deletion
time and one anonymous Principal. Four payloads exercise i64 minimum, -1, zero
and maximum; each asserts all projected fields. Every truncated prefix must
return Malformed, appended bytes must return Trailing, and separate byte/header/
type/work/allocation cap violations must return Limit. A zero-argument frame
returns Arity. An empty vec bool retains published value-based admission.

Both native binaries and both Wasm artifacts passed. Wasm execution used Linux
x86-64, Rust 1.100.0 and Node v24.21.0 / V8 13.6.233.17-node.53. Clippy passed
for both feature selections with warnings denied. These are executed standalone
Wasm checks, not only compilation; they do not establish IC system behavior,
instruction cost, macOS support or real Query adoption. Production source and
the qualification lock remain unchanged.

Admission evidence SHA-256 identities:

- Published artifact: `e1c437569cccc2a0d9a1a4d6730ec78c725438a2183b41b8a5c92e406f95290d`.
- Candidate artifact: `936d18960c121f32edc015233763d1b789ba8160c15c5362ed7ee764abfe4598`.
- Authored admission binary source: `8d4c120b7ddf5f369c0e6fc4f730228b1f31dfd64fb54f802a5ef81d6a7a9031`.
- Node runner source: `bac62cf39624de07d3d1a0a9e5fa779d10d9c162996277186c3cd076a0fabac7`.

Artifact paths use `engine-view-admission.wasm` in the same two target directories
listed above. These hashes describe the current working sources and artifacts;
they do not replace the earlier size-probe evidence.

Toko Tooling 0.44.7 adoption moved the qualification package directly under
`qualification/`. Historical inventory hashes above remain bound to their
original source revision and directory layout; current reproduction paths are updated.
