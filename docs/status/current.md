# Current handoff

## Pending 0.8.0: Host and Profiler ownership

Remote main includes 0.7.6 and merged #88/#89 at `26ad783`. This isolated batch
keeps their ordinary API layout, adapter cleanup and published IC Metrics inputs.
Own package versions remain 0.7.6; release/publication belong to the maintainer.
The pending notes move to 0.8.0 because the gzip feature and native source-input
contract change. No other public hard cut is included.

[#90 — Return Profiler CLI and rendering source ownership to IC Profiler](https://github.com/dragginzgame/ic-platform/issues/90)
moves those two complete modules to the consumer, through existing disjoint source
injection. Their assembled paths, bodies and output remain unchanged. Platform
retains IC capture/state adapters, patches, pins and native build inputs.
[#91 — Retire the generic gzip facade after direct Host adoption](https://github.com/dragginzgame/ic-platform/issues/91)
removes the public facade, optional feature and consumer fixture. Profiler is its
only audited sibling consumer and now uses Host Artifacts directly. Match its
source update and provider pin before adopting this batch; see the migration contract.

The native server now uses Host's complete-member gzip decoder and returns the
core's existing level-1 export directly. API input/expanded limits remain 128 MiB;
core input/expanded limits remain 128/256 MiB. Raw JSON remains supported. Corrupt,
truncated, concatenated and trailing gzip input is refused before importing.
This completes the source work tracked by
[Profiler #1](https://github.com/dragginzgame/ic-profiler/issues/1), pending paired
qualification/integration. PNG zlib is independent and unchanged.
[Profiler #4](https://github.com/dragginzgame/ic-profiler/issues/4) separately tracks
private rendering-helper convergence; no generic Host chart API is added.

Evidence is retained in `/tmp/platform-ownership-20261029`. Full Platform and
independent Profiler Clippy pass before longer tests. The standalone recorder's
73 tests and seven script tests pass. The existing assembler's 44 tests pass.
Feature policy/closure/graph fixtures, inventory, pin/source checks and Rust 1.91
all-feature Wasm compilation pass. Native Cargo/Bazel locks are refreshed with
normal resolvers, retaining Host Artifacts 0.69.1 and the selected backend while
removing the old Git Platform gzip package. Actual paired native qualification
and final consumer pin are still pending; do not infer them from fixture checks.
Native macOS, full replica Bazel/NNS genesis and live provider acceptance remain
separate. No performance or Wasm-size saving is measured for this batch.

## Retained safety and external contracts

Owned endpoints remain Wire. External DTO identities, Candid bytes, persisted
formats, executor/IC0 pins, cryptographic bodies and source guards are unchanged.
The fixed 16-level Candid admission remains as released in 0.7.5; its measured
costs and limitations remain in the migration contract. #73 is closed as delivered;
#87 was closed after exact comparison with 0.7.5. Canic #706 owns optional ICRC
adapter adoption; no sibling adoption or new endpoint mounting is asserted here.

[#50](https://github.com/dragginzgame/ic-platform/issues/50) remains deferred until
a supplying solution exists. Its dependency choices and failed advisory check
are unchanged. No advisory ignore, incompatible override, Agent fork or new
cryptography is authorized. #72's narrow DER admission remains the sole source
exception; protected encoding, constants, vectors and licenses stay exact.
