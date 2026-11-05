# Platform 0.21 / 0.22 Canic Wasm comparison — 2026-11-05

Matched Canic release artifacts grow by about 1% with published Platform 0.22.0.
The large increase in Auth's earlier reference-canister measurements does not
reproduce in these Canic artifacts. A small Platform-only correction removes
3,337 code bytes from each role and restores upstream envelope compatibility.
There is still approximately 29 KB of additional code relative to 0.21; this is
not size parity. Preserve the upstream cryptographic ownership boundary and the
required admission/time checks rather than introduce another parser or remove
checks to eliminate this modest residual overhead.

## Matched inputs

- One frozen Canic working-tree capture: HEAD
  `e6a061999dfae207bc20917ffefa2082e7ffe2e7` plus its captured dirty files.
  Sorted source-inventory SHA-256:
  `cb5b302225a6deb9f2fa5e01e67ea9aaf51396e08a96b1c8fa7c46ba1cafbe8b`.
  Every inventoried source file except the diagnostic manifests/lock and native
  tool-selection example remains identical throughout the comparison.
- Issuer: `delegation_issuer_stub`, exported role `issuer`.
  Receiver: `local_authorization_probe`, exported role `app`.
  Their Rust sources, manifests' feature selections, build ID and configuration
  are identical between supplier arms. Both use the same copied published Auth
  0.33.2 Rust sources; only its Platform requirement changes with the supplier.
- Rust **1.99.1**, commit `58cf8f9c35b355199727627d07ae34a4e2846d67`,
  LLVM 23.1.1, `wasm32-unknown-unknown`, Linux x86-64.
  `RUSTFLAGS='--cfg=cpubits="32"'`.
- Canic's actual release recipe: `opt-level="z"`, Full LTO, one codegen unit,
  `strip="symbols"`, no debug information, panic abort, overflow checks off,
  incremental off. Same prebuilt native release builder for every arm.
- Same admitted **ic-wasm 0.11.1** shrink/metadata pipeline, then **Binaryen 133**
  `-Oz` with Canic's validated module-feature flags. Optimizer executable SHA-256:
  `8f25e9fd5db0fc5f210003aaa432922feb2e52d309e430def2f929e34da9466b`.
  ic-wasm executable SHA-256:
  `204dad3ad59b78a46fad559dfefcdcaee3609433e5e4e6d81bda5e051a2bc0bc`.
- Same source directory and dedicated Cargo target, used sequentially. The builder
  warns that the diagnostic target is beneath Platform; no other checkout uses
  this dedicated target. Sibling source and release state were not edited.

Role normal graphs each select exactly one intended facade, Runtime 0.19.1 and
Types 0.18.1. All unrelated normal package versions match. The 0.22 graph adds
the registry creation, certification, representation-independent hash, signature
verifier, BLS verifier/curve and raw IC0 packages in place of embedded source.
Native Agent/parser are absent from these normal Wasm graphs. Candid/printer and
the external DTO capabilities were already selected by these Canic roles.
Complete normal and feature trees are retained with each arm.

## Final uncompressed sizes

All values are **bytes**, measured after the complete Canic release pipeline.
Code means the **section-10 payload**, including its function count and body
lengths, excluding the section ID and payload-length header. The framed code
section is five bytes larger in these modules. Canic's admission diagnostic
additionally omits the two-byte function count; do not mix those measurements.

| Role | Supplier | Total module | Code payload |
| --- | --- | ---: | ---: |
| Issuer | Registry 0.21.0 | 3,381,706 | 3,145,674 |
| Issuer | Registry 0.22.0 | 3,414,881 | 3,177,873 |
| Issuer | Unchanged local 0.22 control | 3,414,075 | 3,177,362 |
| Issuer | Local corrected 0.22 candidate | 3,410,381 | 3,174,025 |
| Receiver | Registry 0.21.0 | 2,987,311 | 2,770,296 |
| Receiver | Registry 0.22.0 | 3,019,343 | 2,801,840 |
| Receiver | Unchanged local 0.22 control | 3,018,782 | 2,801,571 |
| Receiver | Local corrected 0.22 candidate | 3,015,118 | 2,798,234 |

Published 0.21 → 0.22 deltas:

| Role | Total delta | Code delta |
| --- | ---: | ---: |
| Issuer | +33,175 (+0.98%) | +32,199 (+1.02%) |
| Receiver | +32,032 (+1.07%) | +31,544 (+1.14%) |

Matched local control → correction:

| Role | Total delta | Code delta |
| --- | ---: | ---: |
| Issuer | −3,694 | −3,337 |
| Receiver | −3,664 | −3,337 |

Local path identities change code/layout slightly even without a product change.
The local control prevents attributing that difference to the correction. Local
patch arms also align the unchanged canonical Runtime/Types/macros/guard sources
to avoid duplicate executor/type identities. These are diagnostic overrides,
not a downstream distribution mechanism. The candidate is **not published**.

Imports, exports, Candid sidecars, Wire JSON contracts and generated Wire
TypeScript are identical for all four arms of each role. The machine-readable
[result record](platform-022-canic-wasm-2026-11-05.json) includes artifact hashes.

## Added code and retained correction

Separately labelled named-code companions change only `-Cstrip=none` for raw
symbol attribution. After the normal finalization pipeline, their code-section
hashes match the corresponding stripped controls; provenance data bytes differ.
Twiggy 0.8 reports **+31,682 bytes** in raw functions whose symbols contain
`serde_cbor`, of which **22,258 bytes** name the newly introduced envelope
projection. These buckets overlap. This is pre-post-link attribution, not a
claim that all those bytes are independently removable from final Wasm.

The upstream verifier returns only success/failure. Platform must decode the
authenticated envelope/certificate again to reject identity subnet keys and
obtain the signing certificate's time. Extra Serde/CBOR instantiations dominate
the observed growth. The restored transitive package list alone does not tell
which code survives linking.

The retained changes are confined to private integration plumbing:

- Key admission invokes the same upstream checked `G2Affine::from_compressed`
  parser and upstream identity test directly. It retains length, encoding,
  on-curve/subgroup and identity admission while removing the former public-key
  wrapper's parse → serialize → parse round trip. Public BLS adapters and their
  serialization/error contracts remain unchanged.
- The certificate projection describes both upstream envelope fields in their
  original order. `IgnoredAny` skips the already authenticated witness without
  allocating another tree. This restores packed-map and sequence envelope forms
  accepted by upstream and Platform 0.21. A regression compares all three forms
  with the upstream verifier and still refuses a changed message.

The field-set correction alone is not a size win: it adds 74 code bytes versus
the direct-key-admission intermediate. It is retained for byte compatibility.
A second experiment mirrored the complete upstream DTO, including its tree and
Serde name, to encourage decoder sharing. It instead added **1,127 total /
1,126 code bytes** to the receiver and allocated another witness. It was rejected.
No new CBOR library, public decoder API or embedded cryptographic body is added.

All cryptographic implementations, hash/domain constants, key formats, certificate
signature checks, certified-data binding, message/seed paths, delegation limits,
legacy/sharded range checks and trust-root handling remain upstream. Platform's
DER pre-admission and root/delegated identity-key rejection remain. Signing time
is returned only after verification, from a unique visible top-level signing
certificate leaf; delegation time, missing/duplicate/non-leaf time, overflow,
truncation and trailing bytes retain their refusals. Auth's freshness, future
skew, authority, expiry and session policies are unchanged.

## Focused qualification

- Affected Platform Clippy passes with warnings denied, followed by **15 tests**:
  signature-verification unit tests and signature, BLS, certification and creation
  integration contracts. Rust formatting and benchmark-script syntax pass.
- The five registry crypto/certification suppliers' **80 files** compare byte for
  byte with archives matching the locked registry checksums. No imported
  cryptographic implementation was edited.
- Auth's held source was captured read-only at HEAD
  `6af482c6613e20587a8a25caf09903f6eb9f0e0f` plus dirty files; inventory SHA-256
  `515886e5cd93a3b8efa200047b8523e88c37f4ac0a30c7ea2b2dcd28a17daa65`.
  It selects Wire 0.17.1, Testkit 0.86.0 and Storage 0.67.0, with the candidate
  Platform and canonical owner source overrides only in the diagnostic copy.
- Auth engine all-target/all-feature Clippy passes with warnings denied;
  `cargo test -p ic-auth --all-features --lib --tests` passes **197 tests**.
  The qualification canister is rebuilt on Rust 1.99.1. The normal qualification
  runner passes **11 live tests** against admitted PocketIC 16.1, covering real
  certificates/delegation, rejection/freshness, composed stores, upgrades, signed
  ingress, receiver sessions and the actual Node SDK admission/logout adapters.
  The explicitly prepared Chromium case remains ignored; no browser result is
  claimed. This is local qualification, not a fleet or release gate.
- Auth's captured generated `wire-runtime.ts` was stale against its existing
  locked Wire 0.17.1 producer. Initial contract comparison failed. In the
  diagnostic copy only, `client-contracts.sh generate` followed by `check` and
  the client build passed. The diff consolidates error translation without
  changing contract bytes. The original sibling still needs its own
  `make generate-client-contracts`; this is not completed sibling adoption.

Final Rust bodies match the qualified skipped-tree candidate. The subsequent
clarification of its ordinary comment does not change the qualification inputs
that affect behavior. The unsuccessful full-tree trial is not the delivered code.

## Reproduction and evidence

The version-specific helpers live in
[`scripts/benchmarks/0.22`](../../scripts/benchmarks/0.22/prepare.cjs).
They capture read-only sibling sources into disposable validation inputs without
creating a Git checkout. Keep an unchanged local control before modifying
Platform. Never edit a running arm's scripts, manifests or source.

```bash
node scripts/benchmarks/0.22/prepare.cjs /home/adam/projects/canic target/platform022-canic
cargo metadata --manifest-path target/platform022-canic/source/Cargo.toml \
  --offline --format-version 1 >target/platform022-canic/initial-metadata.json
CARGO_TARGET_DIR="$PWD/target/platform022-canic/build" cargo build \
  --manifest-path target/platform022-canic/source/Cargo.toml --locked --offline \
  -p canic-host --example build_artifact
bash scripts/benchmarks/0.22/run-arm.sh target/platform022-canic registry022 0.22.0
bash scripts/benchmarks/0.22/run-arm.sh target/platform022-canic registry021 0.21.0
bash scripts/benchmarks/0.22/run-arm.sh target/platform022-canic path022-control 0.22.0 \
  "$PWD/crates/ic-platform"
# Apply the reviewed Platform correction, then run the same source/target again.
bash scripts/benchmarks/0.22/run-arm.sh target/platform022-canic path022-fixed 0.22.0 \
  "$PWD/crates/ic-platform"
node scripts/benchmarks/0.22/report.cjs target/platform022-canic \
  registry021 registry022 path022-control path022-fixed
```

The source capture requires the prepared Canic tools and published Auth 0.33.2
source in the local registry. The current corrected checkout is not an unchanged
control: use the preserved pre-edit input when reproducing the local patch delta.
The native builder must be compiled once with the 0.22 supplier and reused for
0.21, since current Host Tools uses the 0.22 native DTO identity.

Local evidence: `target/platform022-canic/` retains source inventories, compiler/
tool identities, per-arm locks, graphs, feature closures, logs, Wasms, contract
artifacts, symbol attribution, source-archive comparisons and `measurements.json`.
`target/platform022-auth/` retains its frozen source, diagnostic-only source diff,
tool admission, generated-client comparison and passing native/live logs.
Failed/intermediate captures and experiments are retained separately; they are
not passes. No package version, tag, publication or sibling source was changed.
