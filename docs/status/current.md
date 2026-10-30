# Current handoff

## Registry dependency adoption

This isolated branch stacks on PR #99 head `380a1e8`; do not overwrite its active
qualification branch. It replaces the native Git library suppliers with published
Host Process/FS/Artifacts 0.70.3 and gzip-only Platform 0.8.3, retaining Wire 0.7.23.
Cargo/Bazel locks are regenerated from the selected registry sources. Profiler
adopts Shared Tooling 0.27.9 and the paired consumer lint fixes from Profiler #10.
Fresh registry-native qualification is pending; the following parent evidence
retains its original source and does not qualify changed dependency bytes.


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

Installed-runtime CI now selects Testkit's required `cli` feature. Its first
refreshed run passed server setup, then exposed a stale source qualifier in the
maintained runtime lock after the local version update. Remove that redundant
qualifier without changing registry identities, checksums or edges; retain strict
registry comparison. Runtime proofs
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
Three consumer-owned CLI/render conditionals are supplied separately by
[Profiler #10](https://github.com/dragginzgame/ic-profiler/pull/10), commit
`a947b33`. Adopt that source with this profile. #98 remains open until the paired
consumer contribution is integrated; its working checkout remains untouched.

## Qualification

Current evidence is under `/tmp/platform-pr99-evidence-20261030`. Full owner
all-target/all-feature Clippy passes before longer tests. All 47 assembler tests,
locked-build/concurrency/failure admission and the actual native graph pass.
The canonical assembler prepares the combined provider/paired-product source.
A comparison of 7,126 source files confirms every change from the prior native
assembly is within `rs/pocket_ic_server`; dependency source bodies are unchanged.

Strict native primary-package Clippy also passes using the prepared exact
external dependency artifacts and the original IC Clippy configuration. This is
not a new upstream dependency-workspace lint pass. Fresh server library, executable
and test links reuse those exact dependency artifacts. All 110 freshly compiled
server/product tests and six installed-canister scenarios pass on Linux, including
the actual Python export/guarded-clear client. The unchanged installed-test source
uses its qualified cached executable; the server is newly linked. Its SHA-256 is
`71a87f9ec8a0f92f225762c6d335408f191dcf832ad24e228a95d545f3ccf49b`.
The initial product run was denied local sockets by the sandbox; it passed with
socket access. Hosted checks run asynchronously. Native-server Darwin, full
Bazel/NNS, fresh-host and live-provider acceptance remain separate. Do not infer
size or speed savings from this work.

Prior PR #99 qualification at provider `fd845c4` used Profiler 0.11.0's source:
78 recorder, 109 server-product and six installed tests, including actual Python
export/guarded clear. Its server SHA-256 was
`1bff7f3c31ce8dd2a2bd679593c6af64a17f0e06f8497384fdebee0403c57571`.
Those historical receipts remain in Profiler's artifacts/.build; they are not a
fresh execution or the supplying identity for this refresh.

## Supply and retained contracts

The native recipe retains IC `ee8086a9773725fd1058832a9400297cf0990112`, Rust
1.99.0, registry Host Process/FS/Artifacts 0.70.3, Wire 0.7.23 and gzip-only
registry Platform 0.8.3. Testkit/editor selections remain independent. Preserve
one registry Host source family. Private ICPROF03 peers rebuild together; saved formats remain unchanged.

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
