# Auth primitive migration — Platform 0.34.4

This published compatible addition supplies the missing `ed25519` and
`ecdsa` primitives for [Auth crypto migration #7](https://github.com/dragginzgame/ic-platform/issues/7).
The initial 0.34.4 slice alone did not complete certified-issuer construction or
installed migration. The later 0.34.5 registry-cohort evidence at the end of this
record supplies both fixture gates; downstream production adoption is separate.
See the [publication evidence](2026-11-16-timer-crypto-publication.md).

## Reuse and ownership

The implementation reexports original upstream types; it copies no arithmetic,
signature algorithm or RNG implementation. Narrow declaration reuse is consistent
with the retained Platform 0.33.7 primitive boundary. Destination obligations are
native/Wasm support, empty defaults, explicit features and unchanged upstream type
identity. Message framing, enrollment, high-S rejection, strict verification and
budget charging remain Auth's decisions. Signing types support owned fixtures;
these features do not select an RNG, Agent, Runtime or general Candid.

Registry selection on 16 November 2026:

| Supplier | Version | Archive SHA-256 |
| --- | --- | --- |
| ed25519-dalek | 3.0.0 | `6ebaa1a2bf1290ab3bfe5a7b771d050ebffab2711c19a81691c683a5144a25de` |
| curve25519-dalek | 5.0.0 | `b5eed333089e2e1c1ac8c6c0398e5e2497b4c9926ca6d0365ed1e099afa5bc23` |
| k256 | 0.14.0 | `93f50113171a713f4a4231ef82eb26703607139b35dcb56241f0ceab2ae1f7d8` |
| crypto-bigint | 0.7.11 | `914c195c4d7fbc5ed7f88d14cbf85914d5c664967658d43404b263913a477e82` |

Upstream packages retain their original notices and license metadata (Dalek
BSD-3-Clause, K256/crypto-bigint Apache-2.0 OR MIT). No unsafe code is added here.
The optional direct bigint requirement is a resolution floor, deliberately listed
in cargo-machete metadata. K256 admits elliptic-curve 0.14.1, whose bigint minimum
0.7.5 is below the advisory's corrected range. The floor prevents that older
compatible selection without pretending to use arithmetic or overriding an
incompatible package. The selected graph uses elliptic-curve 0.14.3 and bigint
0.7.11. Platform's default normal dependency graph remains empty.

The root and qualification lock audits report zero known vulnerabilities against RustSec
`02e2517f457a359fc6d3440ea4560c8ea188fee8`, independently confirmed as the current
remote revision during this check. Existing unmaintained notices remain.
This correct direct ECDSA graph does not resolve the retained Agent/derivation
obligation in [Platform #27](https://github.com/dragginzgame/ic-platform/issues/27): Agent
0.49.3 still selects the incompatible elliptic-curve 0.13/crypto-bigint 0.5 family.

## Platform #27 upstream availability review — 17 November 2026

Fresh sparse-index inspection still selects non-yanked stable Agent 0.49.3 and
`ic-secp256k1` 0.3.0. Agent has nonoptional elliptic-curve `^0.13`, K256
`^0.13.4` and P256 `^0.13.2`; the derivation supplier requires K256 `^0.13.4`.
The latest compatible K256 0.13.4/P256 0.13.2 paths reach elliptic-curve 0.13.8,
which requires crypto-bigint `^0.5`. Available K256/P256 0.14.0 and
elliptic-curve 0.14.3 are outside those requirements. The
[RustSec advisory](https://rustsec.org/advisories/RUSTSEC-2026-0387.html) still
requires crypto-bigint `>=0.7.6`. There is no compatible supplying correction
for these retained paths; no forced override, fork or risk exemption is selected.

Locked all-feature metadata for both current workspaces contains neither Agent
nor `ic-secp256k1`, and selects K256 0.14.0, elliptic-curve 0.14.3 and
crypto-bigint 0.7.11. Fresh isolated audits of both current locks report zero
known vulnerabilities against RustSec
`233155b4025b854c68a4b199c0e32d7fd53291a2`. This is current Platform graph
evidence, not remediation or migration proof for retained external consumers.
Platform #27 remains open pending compatible upstream releases and qualification.

Evidence: `target/platform27-review/` retains registry observations and both
metadata graphs; `audit-1794949227131/evidence.json` binds lock/report hashes and
the fresh database identity. These observations include pre-existing local lock
changes; no dependency selection or sibling source was changed by this review.
The initial offline metadata attempt lacked the already-locked bytes 1.12.2
archive; fetching it with `--locked` completed graph inspection without updates.

## Actual consumer evidence

`make qualify-auth-crypto` archives Auth
`b78991e9833320692921bf766e51ad361c9c8c4d` and changes only dependency/qualification
manifests inside ignored storage. Auth's executable verification code and tests
are unchanged. The selected primitive graph contains candidate Platform 0.34.4
and no legacy Platform. Results:

- Ten actual Ed25519 verification tests pass, covering canonical signed bytes,
  protected context, scalar/length admission before charging and charged failures.
- The deliberately feature-unified Dalek legacy scalar negative control passes:
  upstream compatibility accepts the altered scalar, while Auth rejects it before
  charging through the explicit canonical-scalar check.
- Eight independently signed root-delegation tests pass, including high-S refusal,
  compressed/uncompressed SEC1 handling, protected key binding and time limits.
- Native consumer Clippy and Wasm compilation pass. The frozen archive hash is
  `6a043179451c4b467dc45ae2d629135e1f643d73d57ff5a7fe40b575a30c0d3b`; the selected
  qualification lock hash is `b917ac6a93f92545967f204b12d4da91b6b7d0a196c4c97178f475a95f104977`.
  Full output is `target/auth-crypto/1794848354345/verified.json`.

`make qualify-platform-crypto` independently executes retained Auth signature
vectors natively and in standalone Wasm. Only named key/digest/signature rows from
the pinned `crates/ic-auth/tests/fixtures/ed25519.tsv` and
`qualification/ic-auth-certified-conformance/tests/fixtures/root-delegation.tsv`
are retained as data. No consumer test implementation is copied. The latter is
Auth's independently OpenSSL-signed root fixture. Checks include changed prehash,
weak Ed25519 keys, small-order R, noncanonical scalars, malformed SEC1/scalars and
high-S distinction. Wasm imports are empty; execution passed under Node 24.21.0.
Native/Wasm Clippy and Rustdoc also pass. These are not installed IC instruction
measurements or macOS execution evidence.

## Primitive-only baseline: remaining acceptance at 0.34.4

Auth can keep the same primitive import names with the new feature selections.
Its certified issuer path must instead map to Platform's existing explicit
canister-signature authentication API, preserving protected canister/seed/root,
the signing/delegation time distinction and exact boundary refusals. Its fixture
key/witness construction needs a separately admitted owner. Do not restore an
unbounded legacy facade or treat the primitive-only fixture as that migration.
Issue #7 stays open until that mapping, no-legacy installed consumer proof and
published replacement cohort are complete. Sibling source is unchanged.

## Hard-cut candidate — Platform 0.34.5

The retained `qualification/auth-crypto/consumer.patch` maps pinned Auth's
issuer-signature component directly to `CanisterSignatureKey`,
`PreparedRootKey::from_raw`, envelope decoding and explicit authentication.
Auth still owns protected signer/seed admission, domain framing, byte caps,
inclusive signing age, zero future grace and charging order. Both certificate
timestamps must authenticate; delegation uses an explicit unrestricted u64 time
window and does not inherit the signing-age window.

Independent Auth fixtures use `certificate::construction` for tagged bounded
certificate/signature encoding and short-form canister keys. Original upstream
`ic-certification` 3.2.0 types and `ic-verify-bls-signature` 0.6.0 native fixture
signing types retain their existing locked provenance/licenses; no arithmetic
or signing implementation is copied. The small CBOR output writer and key framing
are owned adapters, tested against literals, exact limits and the separate strict
decoder. Construction grants no authenticated evidence. Signing is a separate
native-only opt-in surface and does not choose an RNG or key.

The synthetic delegated fixture now includes a valid old timestamp and certified
subnet type; missing authority data is not waived. Actual consumer tests exposed
a refusal-ordering mismatch: direct certificate time was checked before BLS.
Authentication now precedes timestamp diagnostics, including the complete
delegated chain. A product regression and the consumer's malformed-time plus
corrupted-signature test retain this contract.

Native consumer evidence at `target/auth-crypto/1794851767296/verified.json`
passes 10 Ed25519 tests, the scalar negative control, 11 complete certified-proof
tests, eight issuer-signature cases and eight root-delegation cases. Clippy and
Auth Wasm compilation also pass. The actual installed fixture additionally
passes all five cases: issuer/root and trap rollback, receiver charging and
transaction rollback, authority enrollment, timestamp/revocation boundaries and
volatile-state loss across stopped upgrades. It selects one local Platform
0.34.5, Types 0.21.1 and Runtime 0.23.12 plus Testkit 0.107.1, with no legacy
Platform or Agent. `qualification/auth-runtime/run.mjs` now enforces this graph
and retains patch/lock hashes and test output in `verified.json`. Publication
and downstream source adoption remain separate; siblings were not modified.

The final installed run is retained at
`target/auth-runtime/1794853025317/verified.json` (five passed).
Consumer patch SHA-256:
`67b1613bcf2d160c32d0ababde7b5acd70b8170386163a4ad5003d9e62040870`;
qualification lock SHA-256:
`a4cc2e3a0aaa917f185d67afd9f85791073a0b6f7d48f0f96c25dfc689998f71`.
Use `make qualify-auth-runtime`; the server wrapper supplies the explicit
PocketIC endpoint and reaps its owned process. Invoking the inner runner alone
without that endpoint is refused before preparing or running a fixture.

Platform 0.34.5 is now published. The subsequent registry-only product cohort
passed all five actual installed Auth tests, using registry Types 0.21.1 and
Runtime 0.23.12 with no legacy Platform or Agent. Reproduction, metadata and
results are retained at `target/registry-cohort/1794853809864/verified.json`; see
[publication evidence](../releases.md#native-read-and-consumer-publication).
Production source adoption remains with Auth; its checkout was not modified.
