# Independent identity implementation qualification — 2026-11-10

Subsequent tooling adoption resolves the earlier IC0 admission and root-file
metadata blockers. This report retains its original qualification scope; see
[the current handoff](../status/current.md) for remaining work.


The [Principal](principal-implementation.md) and
[Account](account-implementation.md) records document the separated implementer's
inputs and limits. This document records integrating-reviewer evidence, not a
claim of legal clearance or a cryptographic implementation rewrite.

## Self-authentication dependency cut (pending Types 0.19.2)

The maintainer subsequently authorized replacing the registry Principal adapter
with the specified composition: SHA-224 of the exact supplied bytes, followed
by `0x02`. SHA-224 remains RustCrypto's implementation, using the existing
workspace `sha2` 0.11.1 provider. No public-key parsing, key validation, domain
change, new abstraction or local hash algorithm is introduced. Earlier evidence
below describes the released 0.19.1 adapter, which this follow-up supersedes.

Focused qualification on Rust 1.99.1:

- Types native all-target/all-feature and Wasm library Clippy pass with warnings
  denied. Initial registry comparisons passed for 261 input lengths. At the
  maintainer's request that comparison test is removed; the fixed-byte framing
  regression below remains. Types now selects only `convert` on its development
  Principal oracle for the existing base text/byte comparisons.
- An independent OpenSSL SHA-224 observation for `abc` matches the full 29-byte
  Principal including the suffix. Empty and arbitrary non-key inputs continue
  to be accepted; hashing adds no key-admission policy.
- Source consumers for `principal` and `principal-self-authenticating` pass
  native tests and Wasm checks. Exhaustive combinations of those two features
  pass on native/Wasm, preserving the base graph's exclusion of hashing and
  removing registry Principal from both production selections. Evidence is
  `target/feature-graphs/attempt.blQ0p4`.
- Manifest mutations, all 143 graph-refusal cases and standalone admission
  regressions pass, including registry Principal refusal for self-authentication.
- Cargo changed only the local Types dependency list in Cargo.lock; no registry
  version changed. `ic_principal` remains a development oracle and may still be
  selected by native clients. BLS independently retains SHA-2 0.10.

No release, installed-runtime rerun or new native macOS evidence is claimed for
this focused Types change. Existing identity serialization is unmodified.

## Original integration (Types 0.19.1)

The reviewer removed the earlier unit-test modules and integrated the newly
authored `tests.rs` files. One Principal test needed an explicitly typed empty
comparison (`is_empty`) because Serde JSON adds another PartialEq implementation.
The first compatibility review found that binary CBOR text was historically
accepted; that observation was added to the contract and the implementer restored
the behavior before qualification. Optional self-authentication is a byte adapter
to registry `ic_principal`, not an independently implemented derivation.

## Focused checks

Using the pinned Rust 1.99.1 compiler and locked workspace graph:

- Types native all-target/all-feature and Wasm library Clippy pass with warnings
  denied. All 26 Types tests pass: 14 base Principal, one self-authentication
  adapter and 11 Account tests.
- Platform's `candid-codec,icrc-ledger-types` native library/test Clippy passes.
  74 unit tests and three public codec-consumer tests pass; three explicit
  measurement tests remain ignored. Account Candid bytes and ledger value hashes
  retain their independent oracle checks.
- Runtime `cdk` all-target Clippy, 25 unit tests, three public consumer tests and
  two doctests pass; one explicit benchmark remains ignored.
- Native/Wasm exhaustive closures for `principal,principal-self-authenticating`
  pass (four unique graph queries/five selections per target). Base Principal
  excludes registry Principal and crypto. Optional self-authentication enables
  the upstream package and its SHA-2 0.10 dependency. No Candid/Runtime/Agent
  implication is introduced.
- Feature-contract selections, 143 graph mutation/refusal cases, Candid/CBOR
  source admission, manifest inventory, local documentation links and formatting
  of the independently authored files pass.
- Installed raw Wire and external-Candid Runtime profiles pass callback/await,
  trap cleanup, upgrade, cycles, stable rollback, certificates, management bytes
  and external endpoint admission. Evidence is `target/cdk-runtime.nhOywW`;
  no fresh macOS or Timers composition evidence is claimed.
- Installed Agent checks with the regenerated public PEM fixtures pass verified
  queries, update-and-wait, read-state and wrong-root refusal for Ed25519,
  delegated, P-256 and secp256k1 identities. Evidence is
  `target/agent-installed.FyGDbv` and `/tmp/platform-owned-agent-installed.log`.

Logs are `/tmp/platform-owned-types-{clippy,tests,wasm,inventory}.log`,
`/tmp/platform-identity-consumer-{clippy,tests}.log`,
`/tmp/platform-owned-runtime-{clippy,tests,installed}.log` and
`/tmp/platform-principal-feature-closures.log`. Build evidence is reclaimable;
the ordinary source tests and this qualification record remain maintained.

## Public comparison with published Types

A disposable, offline Cargo probe selected registry `ic-platform-types =0.19.0`
and local Types as distinct sources. It used only public APIs; no old test body
or implementation was supplied to the separate implementer.

For Principal, 30 byte lengths times 256 deterministic byte seeds gave 7,680
values. Their canonical/uppercase/dash-removed/trailing-space texts, Debug and
JSON/CBOR/Postcard bytes matched. Another 65 text lengths times 128 repeated
ASCII bytes gave 8,320 malformed/edge cases with matching admission results and
error diagnostics: **16,000 cases in total**.

For Account, every owner length 0–29, significant hex width 0–64, and absent or
present subaccount gave **3,900 cases**. Text, case/whitespace/extra-component
admission, JSON/CBOR/Postcard bytes and decoded field presence matched. Probe
output is `/tmp/platform-identity-public-comparison.log`. This supplements the
maintained tests; it is neither exhaustive fuzzing nor a performance benchmark.

## Limits and remaining work

After qualification, the maintainer requested removal of first-party MIT files
pending a new license. The earlier source replacement does not complete that
license transition: release metadata/text remain pending. External dependencies
retain their licenses. Other packages still
contain attributed imports, including Runtime/CDK/executor/macros and ledger/
management components. Their notices and source guards are not retired by this
qualification. The earlier ICP derivation draft still fails its existing source
guard and needs a separate resolution; this work did not approve its rewrite.

The installed Toko 0.38.2 dependency gate still refuses Runtime's registry IC0
edge. Its narrow supplier correction is committed in the Toko checkout's 0.39.0
batch, but is not adopted here. Direct supplier-checker evidence does not replace
the installed gate. No full release qualification, commit, tag or publication is
claimed. No Wasm-size or speed improvement is inferred from source replacement.
