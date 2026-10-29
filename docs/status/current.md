# Current handoff

## Pending 0.8.0: Host and Profiler ownership

This isolated follow-on starts from #92 (`c7ff4d5`), over remote main `26ad783`.
Own package versions remain 0.7.6. The normal checkout and sibling work remain
untouched; release/publication belong to the maintainer. Pending notes remain
0.8.0 for the already authorized public gzip/native-input cut.

[#94 — Supply Host process dependencies for Profiler ownership adoption](https://github.com/dragginzgame/ic-platform/issues/94)
supplies matching native Cargo/Bazel inputs for Host Process observations and
workload mechanics. The implementation owner is
[Tooling #264](https://github.com/dragginzgame/ic-tooling/issues/264), supplied by
[Tooling #265](https://github.com/dragginzgame/ic-tooling/pull/265) at immutable
`c20dbd11785c17d369651f51fa0967fbec69c57a`. Artifacts/FS/Process use one Git source
family until a supplying registry release exists. No local Cargo patch is
required. Normal resolvers add the required packages and update libc from
0.2.186 to 0.2.191 to meet the supplier minimum; no blanket upgrade is performed.
IC source `ee8086a` and its Rust 1.99.0 toolchain remain unchanged.

Profiler projects Host facts into existing capture DTOs and keeps root/creation
identity admission, cadence, aggregation, missingness, retention and reports.
Final sampling remains before reaping. Host owns raw procfs/libproc/Mach facts,
clock/thread/affinity observations, child-list completeness and explicit launch
and custody operations. The CLI retains its launch gate/argv/stdin, 500 ms-or-exit
grace, synchronous reap, keep-command-running choice and exit-status JSON text.
Platform retains IC capture/state adapters and native build inputs; no new Rust
facade feature or Host export is introduced.

The consumer follow-on to #3 is tracked by
[Profiler #6](https://github.com/dragginzgame/ic-profiler/issues/6). It does not copy
the normal Profiler checkout's concurrent rendering/Shared Tooling work.

## Qualification

Evidence: `/tmp/profiler-host-ownership-20261029`. Full Platform Clippy and
independent Profiler Clippy pass before longer checks. The changed recorder's
72 tests pass; its former pure parser case moved to Host. Host's 24 focused child
and four raw/native observation tests pass at Rust 1.91; strict Rustdoc, format
and links pass. Extra recorder Rust 1.91 probing fails on pre-existing unstable
Atomic try_update calls; Profiler already selects Rust 1.99.0 and that selected
compiler remains its supported qualification here. No atomic/report policy change
is included. Platform's published Rust 1.91/default/Wasm contracts are unchanged.

Cargo/Bazel locks are resolver-generated; an initial workspace-only Bazel repin
refused the old locked libc, then targeted libc repinning succeeded. Failures and
artifacts remain. Paired Platform `b4de0d4` and clean Profiler `a238580` pass actual assembly, native
graph admission and the Rust 1.99.0 server build. The corrected native recorder
Bazel target passes all 72 tests. Native Cargo passes 72 recorder, three CLI and
12 API/gzip tests. Three real CLI scenarios preserve natural-exit text, final
sampling before reaping, default cleanup and keep-running behavior. All five
installed profiling scenarios pass against frozen server SHA-256:
`72ae33520917c0a65f0f897aa241c3f2ea3c9d23bacf9b3dde5d8556df13360d`.
Receipts bind those exact inputs and executable; the later handoff-only commit
does not change qualified source. Deliveries: [Platform #95](https://github.com/dragginzgame/ic-platform/pull/95),
[Profiler #7](https://github.com/dragginzgame/ic-profiler/pull/7) and Tooling #265.
The first Bazel test exposed an upstream annotation still naming libc 0.2.186;
its generator input and generated lock are corrected, with the failed log kept.
Native macOS, full replica Bazel/NNS and live provider acceptance remain separate.
No performance or Wasm-size saving is measured for this native ownership move.

## Retained prior work and safety boundaries

#92/#3 independently carry the qualified gzip/product-source move. Their prior
server SHA-256 is `cc1fe0e7078c79b4c255ade7e06bb887e8412c05539d9296c48ad976ab16caf0`,
with evidence in `/tmp/platform-ownership-20261029`; it does not qualify this new
Host/libc graph. Integrate supplier parents before refreshing consumer pins to
merged identities. Issues close only after their default-branch resolution.
Profiler #4 separately owns private PNG/range helper convergence.

Owned endpoints remain Wire. External Candid bytes, persisted formats, executor
and IC0 identities, cryptographic bodies/vectors/licenses and source guards are
unchanged. The fixed 16-level Candid admission remains as released in 0.7.5;
its measured costs and limitations stay in the migration contract. Canic #706
owns optional ICRC adapter adoption.

[#50](https://github.com/dragginzgame/ic-platform/issues/50) remains deferred until
a supplying solution exists. Dependencies and the failed advisory check remain
unchanged; no advisory ignore, incompatible override, Agent fork or new crypto
is authorized. #72's narrow DER admission remains the sole protected-source
exception. Historical qualification artifacts remain preserved.
