# Current handoff

## Pending 0.8.4: paired Profiler boundary and qualification fixes

Main released 0.8.3 at `f69a759`; its checkout was clean at refresh. PR #99 now
merges that release without rewriting history. Package versions remain 0.8.3.
Keep all released notes unchanged. Release/publication remain maintainer-owned.
Shared Tooling 0.27.7 stays at `a8baa51099ca48cdb12e2171f031d488db89d67d`.
Siblings stay read-only; separate isolated contributions supply consumer fixes.

PR #99 owns textual-canister requests, Busy/Started completion, successful-result
pruning and compiled backend descriptions. Domain errors are not resubmitted;
timeouts retain uncertainty about started operations. Raw PocketIC JSON and
saved schema-1 reports remain. Profiler owns export-before-clear, file publication,
capture/report policy and rendering. The delivered 0.11.0 paired product interface
remains required; older consumers retain their previous provider pin.

Installed-runtime CI now selects Testkit's required `cli` feature. Runtime proofs
retain shared dependency caches, but a kernel file lock covers each build and
artifact copy. The inherited descriptor survives child execution and releases on
exit. A concurrent-run regression pauses after Cargo and before retention, proves
a second run is blocked and checks every retained Wasm's run identity. Failed
builds propagate without copying stale outputs. Protocol/feature graphs stay fixed.

## Native lint boundary

Platform #98's supported check is strict `pocket-ic-server --lib --no-deps` on
Rust 1.99.0, with the IC pin's existing Clippy configuration. `native-verify`
checks it before native builds/tests. The existing provider command also exposes
`clippy -- --no-deps --lib -- -D warnings`. No warning allowance is added.

Provider-owned fixes keep retry/disposition order, use the final event directly,
name the metrics-registry tuple and add meaningful must-use messages before
unchanged async-trait expansion. The native patch changes no cryptographic body.
Three consumer-owned CLI/render conditionals are prepared separately in an
isolated Profiler contribution. #98 remains open until its consumer/main and
native qualification obligations are complete; do not claim closure from a probe.

## Qualification in progress

Current evidence is under `/tmp/platform-pr99-evidence-20261030`. Full owner
all-target/all-feature Clippy passes before longer tests. All 46 assembler tests,
locked-build/concurrency/failure admission and the actual native graph pass.
The canonical assembler prepares the combined provider/paired-product source.
A comparison of 7,126 source files confirms every change from the prior native
assembly is within `rs/pocket_ic_server`; dependency source bodies are unchanged.

Strict native primary-package Clippy also passes using the prepared exact
external dependency artifacts and the original IC Clippy configuration. This is
not a new upstream dependency-workspace lint pass. Fresh linking, native product
and installed scenarios are being qualified before integration; the previous
server's passes alone do not qualify these changed inputs. Hosted checks run
asynchronously. Native-server Darwin, full Bazel/NNS, fresh-host and live-provider
acceptance remain separate. Do not infer size or speed savings from this work.

Prior PR #99 qualification at provider `fd845c4` used Profiler 0.11.0's source:
78 recorder, 109 server-product and six installed tests, including actual Python
export/guarded clear. Its server SHA-256 was
`1bff7f3c31ce8dd2a2bd679593c6af64a17f0e06f8497384fdebee0403c57571`.
Those historical receipts remain in Profiler's artifacts/.build; they are not a
fresh execution or the supplying identity for this refresh.

## Supply and retained contracts

The native recipe retains IC `ee8086a9773725fd1058832a9400297cf0990112`, Rust
1.99.0, Host `c20dbd11785c17d369651f51fa0967fbec69c57a`, Wire 0.7.23 and the
immutable gzip-only Platform 0.8.1 scaffold. Ordinary Platform registry gzip and
Testkit/editor selections remain independent. Preserve one native Host source
family. Private ICPROF03 peers rebuild together; saved formats remain unchanged.

Profiler's independent recorder owns retention/report policy and private
PNG/range mechanics. Platform owns native state/HTTP admission and pricing.
Keep 8 MiB/query, 32 MiB total, 128-entry FIFO, cross-run checks, diagnostic order,
128 MiB API import and separate 256 MiB recorder decoded limits. Preserve exact
read/write PNG/JSON schemas and absent/unknown coverage semantics.

Owned Principal/Account remain Candid-free under `principal`. External DTOs are
separate Platform-maintained types; registry Principal/Nat/function identities
and required external Candid/Serde bytes stay unchanged. Account Display shares
the owned formatter; external parsing/errors and absent/explicit zero fields stay.
The [packaging contract](../migration.md#minimal-principal-and-selected-icrc-packaging)
retains 28 independently frozen selected-ledger schemas. Value hashes are intact.

Management =0.11.0, executor =2.0.0, ic0 =1.2.0 and stable structures =0.7.2 remain
fixed. Memory/Storage own allocation, checked reopening and leases. Owned endpoints
use Wire; required external Candid and legacy checkpoints retain their contracts.
Bounded Candid admits at most 16 constructed wire levels, including unused and
skipped data, within existing quotas; arbitrary trusted visitors are not covered.
The [nesting contract](../migration.md#external-candid-nesting-qualification-73)
retains original trap/16–17-level and cost evidence. #72's narrow DER envelope
exception and #74 Host/Origin admission stay unchanged. Protected crypto bodies,
encodings, vectors and licenses are preserved. #50 remains deferred with its
advisory check intact; no ignore, incompatible crypto override or Agent fork.
