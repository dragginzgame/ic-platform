# Auth primitive migration — Platform 0.34.4

This published compatible addition supplies the missing `ed25519` and
`ecdsa` primitives for [Auth crypto migration #7](https://github.com/dragginzgame/ic-platform/issues/7).
It does not yet complete certified-issuer construction or installed migration.
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
This correct direct ECDSA graph does not resolve Agent/derivation #4: Agent
0.49.3 still selects the incompatible elliptic-curve 0.13/crypto-bigint 0.5 family.

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

## Remaining acceptance

Auth can keep the same primitive import names with the new feature selections.
Its certified issuer path must instead map to Platform's existing explicit
canister-signature authentication API, preserving protected canister/seed/root,
the signing/delegation time distinction and exact boundary refusals. Its fixture
key/witness construction needs a separately admitted owner. Do not restore an
unbounded legacy facade or treat the primitive-only fixture as that migration.
Issue #7 stays open until that mapping, no-legacy installed consumer proof and
published replacement cohort are complete. Sibling source is unchanged.
