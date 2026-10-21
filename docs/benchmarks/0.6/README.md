# Feature-boundary dependency evidence

These are dependency-graph measurements, not runtime or Wasm-size benchmarks.
The baseline is released 0.5.1 (`fe628b3`); the candidate is the uncommitted 0.6
feature cut, still declaring package version 0.5.1. The maintainer's existing
lock update to Candid/derive 0.10.39 was preserved byte-for-byte during this task.
Neither measured profile selects Candid.

Cargo trees use `--locked --offline --edges normal,build --prefix none`.
Counts deduplicate `(package name, version)` and include Platform itself.

| Profile | Target | Before | After | Change |
| --- | --- | ---: | ---: | --- |
| executor + ic0 | wasm32-unknown-unknown | 26 | 6 | Application identity/text/Serde/crypto dependencies absent |
| canister-sig-creation | x86_64-unknown-linux-gnu | 30 | 26 | representation-independent-hash, leb128, serde_cbor and half absent |

Retained trees: [raw before](executor-before.txt), [raw after](executor-after.txt),
[creation before](creation-before.txt), [creation after](creation-after.txt).
The raw graph retains executor 2.0.0, ic0 1.2.0, slotmap, smallvec and version_check.
Creation still receives serde_bytes through certification. Verification still
requires CBOR. Other selected host capabilities can reintroduce packages.

All 199 routine selections pass on native and Wasm (179/111 distinct graph
queries). Admission regression tests cover forbidden minimal-profile packages
and creation-only hashing/CBOR leaks, with positive Principal and verification
cases. Exact source and mutation-refusal guards pass; stripping the eight new
packaging attributes reconstructs the old creation source byte-for-byte.

Rust 1.88 passes 42 focused library/Principal/signature tests. Separate Principal,
ic0, executor+ic0, creation, verification, runtime, Candid and CDK consumers pass
native tests and Wasm checks on development Rust and 1.88. The raw consumer also
has a compile-fail Principal import. Strict focused Clippy and minimal-profile
rustdoc pass. No API-body or cryptographic-function change is part of this cut.

A separate temporary consumer compiles the clean Timers checkout
`41e1e2fb65d6b4f6ea4cf5f6c01112702d723ef2` (0.30.0) on Rust 1.91 native/Wasm,
using an experiment-only root patch to this Platform source. Its production graph
has no Principal/Serde/SHA-2/text dependencies and retains Timers' own Thiserror 2.
This is source integration evidence, not registry publication or a supported
consumer patch instruction. Timers must update its `0.5` Platform requirement to
`0.6` after publication; this experiment uses the unchanged local package version.

Working logs and the temporary Timers fixture are in `/tmp/platform-06-work/`;
feature mappings/trees are in `target/feature-graphs/attempt.mfNepE`.
Native macOS, installed timer/canister execution and live IC were not rerun.
Existing runtime submission/callback/trap and executor sources/identities remain
unchanged; source fingerprints pass. Do not infer deployment qualification from
these compile and graph checks.
