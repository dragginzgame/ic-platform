# Current handoff

## Pending 0.16: Rust SDK maintenance ownership

The maintainer authorized owning the Rust SDK while retaining the IC network,
replica and simulator as external infrastructure. Imported code retains licenses,
attribution and protected cryptographic bodies. Sibling checkouts remain read-only.
The maintainer requested deployment of Platform 0.16 on 2026-11-01, authorizing
source integration, release validation, version preparation, push and publication.
Platform/macros/guard remain 0.15.3 until the release transaction; Types 0.15.1
and browser 0.1.1 are outside this release.
Pending Platform notes select 0.16.0 for the public type changes below.

The latest batch upgrades ed25519-dalek 2.2 -> 3.0, curve25519-dalek 4.1.3 -> 5.0,
Syn 2 -> 3.0.9, and the development-only representation-independent-hash reference
3.2 -> 4.0.1. Defaults stay disabled for Dalek; public signature traits now belong
to generation 3. Existing byte-based call sites compile unchanged; consumers
exchanging older concrete Dalek/signature types need deliberate adoption. Native
Agent retains its separate older generation. No crypto implementation is replaced
with locally authored arithmetic. The macro generator needed no source changes.

The maintainer authorized the exact upstream identity-key rejection and vector in
[#127](https://github.com/dragginzgame/ic-platform/issues/127). They are implemented
locally. The source guard checks the precise entry check/vector before removing
only those additions and checking the unchanged original source fingerprints.
Raw BLS and forged identity-root certificates are refused. Native Agent still uses
registry BLS 0.6.0 and does not inherit this fix. See
[BLS provenance](../../crates/ic-platform/src/bls/upstream/NOTICE.md).

Full Clippy, 198 signature/curve/certificate library tests and 14 macro tests pass
on Rust 1.99.1 and 1.91.0. Five standalone consumers pass current/MSRV native/Wasm
qualification. The nine-selection graph matrix, source/mutation guards, docs,
manifest inventory, boundaries, formatting and runtime-lock custody checks pass.
The runtime fixture lock was refreshed for Syn 3 and its native/Wasm graphs pass;
this is not renewed 0.16 Timers composition acceptance.
Installed Rust-1.91/PocketIC-16 proofs pass BLS, Ed25519 RFC/strict refusals, existing
certificate/signature vectors and forged identity-root refusal. An initial combined
query exceeded the normal instruction limit; splitting the independent suites
into three queries passed without raising limits. Failed and successful artifacts
are retained. This is Linux simulator evidence, not live IC or sibling adoption.
Evidence: /tmp/platform-updates-20261101; graphs: target/feature-graphs/attempt.k2VHLO.
Full CI remains release-owned.

The [remaining external repository inventory](../ic-inventory.md#remaining-external-dfinity-repositories-pending-016)
records seven Rust DFINITY repositories. Only candid/stable-structures appear in
selected Wasm normal/build graphs; candid_derive runs on the compiler host.
Browser SDK Core and its Auth development fixture are separate external sources.
Standalone BLS/signature graphs have zero DFINITY registry dependencies.
PocketIC pins thiserror =2.0.18; newer crypto-common/Pest require other dependency
downgrades. The coupled BLS hash/field/RNG interfaces and Candid bigint identities
remain at their qualified generations. This is not whole-stack dependency removal.

## Earlier local ownership and crypto qualification

Certification 3.2.0 certificate/tree/RbTree types now belong to Platform, with
unchanged hashing/Serde/tests and original/admitted source guards. The unused
nested_rb_tree/rstest layer is omitted. Native Agent has explicit byte-preserving
ExternalCertificate/ExternalHashTree conversions; PocketIC retains its registry
identity. Host/Query crossings require coordinated adoption. Upstream review
through 4.0.1 found only a docs.rs attribute change in the selected source.
Evidence: /tmp/platform-certification-adoption-20261031;
feature graphs: target/feature-graphs/attempt.iwZFda.

Canister-signature keys/range decoding use canonical owned Principal. DER/CBOR,
length-first ordering, inclusive bounds and original verifier bodies remain.
The constructor/field identity is a 0.16 API cut; Candid/client profiles retain
external Principal. Current/MSRV differential tests and installed signed
legacy/sharded range fixtures passed. Evidence/Wasm:
/tmp/platform-signature-principal-20261031; graphs: target/feature-graphs/attempt.kUU5Af.

The private bls::curve module imports complete ic_bls12_381 0.10.1 source, original
Rust tests and four binary vectors, with both licenses/COPYRIGHT. Only documented
path/edition/cfg/lint scaffolding changes are admitted. No curve API or crate is
added. Upstream review found only later CI/lock changes. Generic digest/ff/group/
pairing/subtle/rand_core dependencies remain external; rand_xorshift/SHA-3 are
test-only. Evidence: /tmp/platform-curve-adoption-20261031;
graphs: target/feature-graphs/attempt.AoYXdC. See
[curve provenance](../../crates/ic-platform/src/bls/curve/NOTICE.md).

The maintained BLS wrapper retains optimized allocated verification and std
LazyLock with the original initializer; Rand is test-only. The earlier no-alloc
experiment saved 1,989 combined bytes but cost about 51% more warm instructions.
ECDSA uses K256 0.14/signature 3 and crypto-bigint ^0.7.10. Native Agent/ic-secp256k1
retain the older bigint generation; the full advisory graph is not cleared.
Root .cargo/config.toml selects upstream cpubits="32" only for Wasm: the z/fat-LTO
ECDSA probe fell from 107,421 to 57,575 bytes and from 8,532,810 to 4,381,468 IC
instructions. Consumers must carry the cfg; dependency Cargo config is not inherited.
Fixed old bytes and isolated Auth qualification passed. See
[measurements/evidence](../benchmarks/upstream-2026-10-31/README.md),
/tmp/platform-ecdsa-footprint-20261031 and /tmp/platform-bls-adoption-20261031.

Published Timers 0.34.0 qualified the current Platform 0.15 runtime fixture:
Wire/external-Candid callbacks, traps, cancellation, upgrades and rollback passed
on both compilers with Testkit 0.72.1/PocketIC 16.0.0. Platform 0.16 still requires
coordinated Timers adoption/qualification. No later sibling adoption is claimed.

## Release custody and native profile

Shared 143a9abb9ae81feae9400ff3844b03b1ddfdd75a is adopted (0.27.22 release commit;
supplier VERSION still 0.27.21 as authorized). Snapshot payloads stay untouched.
One workspace/lockfile and independent Platform/Types groups remain. Every release
and publication requires explicit RELEASE_GROUP. Runtime-lock and exact guard-pin
custody stay transactional; saved attempts cannot be reinterpreted under new policy.
Historical 0.15.3 receipts: /tmp/platform-deploy-platform-0153.log,
/tmp/platform-deploy-platform-0153-publication.log, .git/release-state/platform-0.15.3.plan
and target/publication/. Preserve old push journals and failed/successful evidence.

Native Profiler #124 uses provider b9f4f0fa8f57f1448c5cd024924339ab546909bb,
Host Process/FS/Artifacts 0.73.1, Metrics 0.24 and Wire 0.13. Its graph excludes
registry Platform/types/endpoint and Motoko/lifeline. Paired Profiler evidence is
artifacts/fleet-adoption-20261031/: source/patch/server hashes, 83 recorder and
114 server tests, six installed scenarios and Bazel repin. No macOS or later
consumer acceptance follows from this native profile qualification.

External Candid retains the fixed 16-level admission and legacy checkpoint bytes.
The earlier 7,631-byte/3k-line codec experiment was rejected as an optimization;
a future ownership migration must establish its own maintenance case. Earlier
installed context remains in the [0.14 release handoff](https://github.com/dragginzgame/ic-platform/blob/7b4ed265804a60b796c4680880cb7a4e66e53d1e/docs/status/current.md).
