# Current handoff

## Pending 0.8.0: minimal Principal and selected ICRC packaging

Packaging source HEAD is `a4da4ce`, based on source release **0.7.6**, `8871f67`.
The maintainer authorized this breaking packaging cut. Package versions remain
0.7.6; pending notes use 0.8.0. Release, publication and sibling adoption remain
separate.
Preserve the pre-existing IC Host Artifacts lock selection; it is unrelated.

`principal` now supplies owned Principal/Account bytes, bounded text and Serde
without registry Principal or hashing. Runtime/CDK select only that capability.
`principal-external` adds registry identities and byte-copy conversions;
`principal-self-authenticating` separately selects unchanged upstream hashing.
Required external protocol/client and signature profiles select conversions.

`icrc-ledger-types` retains its feature/module paths and selected APIs, while
removing the registry package from normal/development dependencies. Platform owns
selected DTOs/callbacks; Candid/Serde representations, effective-zero accounts,
legacy Nat64 and upstream value hashes remain. Storage/CBOR adapters, metadata-key
helpers and consent formatting are omitted. The complete value/hash source and
fixed vectors reconstruct original fingerprints after admitted packaging changes.
Licenses/provenance and all 28 independent frozen registry schemas are retained.
See the [packaging contract](../migration.md#minimal-principal-and-selected-icrc-packaging).

Evidence is in `/tmp/platform-dependencies-20261029`; graph receipts are in
`target/feature-graphs/attempt.FZeIyu`. Full `make clippy` passes before longer
tests. All 505 routine feature selections pass native/Wasm admission. Eight
standalone profiles pass native/current and Rust 1.91; applicable profiles also
compile Wasm (CDK links a cdylib). Independent schema checks pass all 28 selected
ICRC types, including recursive values and query callbacks. Upstream hash vectors,
ledger bytes/accounts, seven Principal unit tests and two parser properties pass.
Inventory/boundary, source/license guards and their mutation refusals pass.
Strict Rustdoc (all features and owned Principal only), formatting, ShellCheck
and local documentation links pass.

The consumer checks caught a test-only import and missing explicit Candid blob/
bignum prerequisites; both are repaired. The release consumer run then exposed
missing external Principal Serde in standalone signature verification, hidden by
all-feature unification. `signature-verification` now explicitly selects
`ic_principal/serde`; protected verifier source is unchanged. Full `make clippy`
passes before the focused rerun. Verification, owned Principal, external conversion
and signature creation standalone profiles pass native tests and Wasm checks on
Rust 1.99.1 and 1.91. Only verification adds external Serde; all four normal/build
graphs remain Candid/runtime-free. Source guards, inventory, formatting and docs
checks pass. Current repair evidence is in `target/signature-serde-fix`.
No code-size saving, consumer/provider adoption, live IC or native macOS acceptance
is claimed. Registry executor/ic0, management DTO identity and protected crypto/
callback sources remain unchanged. Installed runtime was not rerun for packaging.

## Retained 0.7.6 evidence

The complexity cleanup and native provider qualification below are committed in
the source release; detailed prior handoff/source is preserved at `8871f67`.
Evidence remains in `/tmp/platform-simplify-20261029`. PR #88/#89 integration,
provider acceptance and consumer pins retain their separate recorded scope.

## Native provider qualification

Qualification uses isolated provider/consumer copies, upstream IC
`ee8086a9773725fd1058832a9400297cf0990112` and its Rust **1.99.0** toolchain.
The normal sibling checkout remains read-only. The assembled native library
compiles and passes 37 focused adapter/catalog/admission tests. The newly built
Linux server SHA-256 is
`07bd658a47d50bb0dd59176a1012eff09128d0c026abf23c39009379a34922d5`;
all five `system_profiling` scenarios pass against that exact executable.
Preparation/build/test receipts are under the isolated consumer's
`.build/receipts` inside the evidence directory above, binding the dirty provider
source digests and embedded routing input rather than claiming HEAD alone supplied it.

All 69 recorder tests pass with admitted local socket access; native graph
exclusions pass. Native macOS, Bazel/NNS genesis, live provider acceptance and a
consumer's new provider pin remain separate. Root full CI/release gates were not run.
No runtime, crypto, pricing/debit body, capture format or dependency was replaced.

## Retained 0.7.5 Candid and ICRC contracts

The maintainer accepted one fixed rule for #73: bounded decoders reject more than
**16 constructed wire levels** before upstream Candid; exactly 16 are allowed.
No policy object, limit setting or macro option is added. Unused types and skipped
values are checked with the existing combined decode/skip work budget. Candid
still receives its original quotas. Low-level upstream APIs retain their behavior;
staged callers can explicitly use `candid::nesting::check`.

This mitigates wire nesting, not arbitrary trusted Rust schemas/custom visitors.
Do not claim upstream Candid itself is repaired. Original installed proofs include
16/17 levels, the 8,202-byte/8,192-opt trap, malformed input, refused-handler state,
management raw_rand, callbacks, timers and upgrades. Evidence:
`/tmp/platform-fixed-depth-20261029`; fixture `target/cdk-runtime.p3w1eD`.
Historical source/installed locks select Candid 0.10.39/0.10.38 respectively.

The accepted frozen cost study adds about 10–20% decoder instructions for ordinary
structured DTOs, about 1.13M for 100 ICRC-3 blocks, and about 12.4K for 64 KiB/1 MiB
HTTP bodies. Exact fixture cycle deltas, methodology and server identity are in
the [cost report](../migration.md#admission-cost-2026-10-29), with evidence in
`/tmp/platform-candid-cost-20261029`. Further optimization awaits consumer metrics.
Owned endpoints remain Wire; external identities, bytes and persisted formats remain.

Source 0.7.5 also supplies the named external ICRC-10 `SupportedStandard` from
[PR #86](https://github.com/dragginzgame/ic-platform/pull/86). Its independent
schema/type/byte proofs remain in `/tmp/platform-icrc10-20261028`.
[Canic #706](https://github.com/dragginzgame/canic/issues/706) owns optional adapter
adoption on the pending 0.113 base. Source-candidate strict lint, DTO/schema/bytes,
MSRV/Wasm and four installed profiles pass; registry/managed adoption is not
asserted. Mounting, consent, authorization and budgets remain Canic-owned.

## Retained advisory and safety boundaries

[#50](https://github.com/dragginzgame/ic-platform/issues/50) remains the retained
RUSTSEC-2026-0387 blocker through coupled ECDSA/Agent/derivation dependencies.
The maintainer approved setting it aside for integration only. No advisory ignore,
check suppression, incompatible override, Agent fork or in-house crypto is authorized.
#72's narrow DER admission is the sole source-guard exception; crypto bodies,
encoding, constants, vectors and licenses remain protected. Only short-form
framing refusal is qualified; long-form support and oversized encoder conversions
remain outside that change. #74 retains native Origin/Host admission and local
nonbrowser clients; Chromium/assembler evidence is in
`/tmp/platform-security-20261028`. Historical size figures remain fixture-specific.
