# Experimental BLS wrapper: frozen implementer contract

Historical input retained for source-separation evidence. The maintainer later
authorized adoption into the facade; see [qualification](qualification.md).

This is the reviewed input for an independently authored, unpublished experiment,
not production adoption. The maintainer requested it on 2026-11-11. The integrator
has read the old verifier; the implementer must not receive or open its source,
Platform's existing BLS/signature implementations, or their tests/vectors. Do not
copy code from standards or dependency implementations. Use public specifications,
API signatures and this behavior contract; external algorithms stay external.

## Public protocol facts

The [IC certificate specification](https://docs.internetcomputer.org/references/ic-interface-spec/certification/)
selects the BLS minimal-signature-size basic ciphersuite
`BLS_SIG_BLS12381G1_XMD:SHA-256_SSWU_RO_NUL_`. Signatures are compressed G1
(48 bytes), public keys compressed G2 (96 bytes). Messages are arbitrary byte
strings. The caller supplies any certificate domain prefix; this wrapper must not
add one. The BLS hash-to-curve DST is exactly the suite string above.

The [BLS draft](https://www.ietf.org/archive/id/draft-irtf-cfrg-bls-signature-06.html)
defines the verification relation between a signature, the message's hash-to-G1,
the public key and the G2 generator. Treat it as a protocol reference, not copied
pseudocode. [RFC 9380](https://www.rfc-editor.org/rfc/rfc9380.html) identifies
hash-to-curve semantics; delegate those entirely to the selected curve supplier.
All point decoding must use its checked, subgroup-validating API. Public-key
identity MUST make verification fail, including keys derived from a zero scalar.
Do not use unchecked decoding or substitute equality of serialized encodings for
point admission. Preserve failure for wrong messages, keys and signatures.

## Selected external APIs and dependency boundary

Use `ic_bls12_381` 0.10.1 with alloc/groups/pairings/experimental only, SHA-2
0.11.1 without defaults, and Digest 0.10.7 without defaults. The latter is required
by the curve supplier's public hash-to-curve trait. Implement only the Digest 0.10
trait adapter around upstream `sha2::Sha256`: stream updates/finalisation/reset
into that provider; fixed output/block sizes are 32/64 bytes. No local hash,
expand-message, curve, scalar arithmetic, point encoding or pairing algorithm.
Supplier public API documentation is allowed; do not read verifier implementations.
Use standard library facilities instead of new hex/lazy-static/RNG dependencies.
No allocation, byte, message-size, certificate or freshness policy is added.

## Public behavioral surface

Provide `PublicKey`, `Signature`, `PrivateKey`; their storage is private and may
use external supplier types. PublicKey: Clone/Eq/PartialEq/Debug; Signature and
PrivateKey: Copy/Clone/Eq/PartialEq/Debug. BYTES constants are 96, 48, 32.
Each has `deserialize(&[u8]) -> Result<Self, matching error>` and
`serialize(&self) -> [u8; Self::BYTES]`. Errors are public Copy/Clone/Debug/Eq enums:
`InvalidPublicKey::{WrongLength, InvalidPoint}`;
`InvalidSignature::{WrongLength, InvalidPoint}`;
`InvalidPrivateKey::{WrongLength, OutOfRange}`.
Check length before encoding admission. Invalid encodings return errors, not panic.

Private keys use 32-byte big-endian canonical scalars; zero remains parseable for
compatibility, values at/above the group order are rejected. Supplier scalar
codecs/arithmetic own admission, derivation and multiplication. PrivateKey exposes
`public_key(&self) -> PublicKey` and `sign(&self, &[u8]) -> Signature`. No random
key generator or RNG dependency. Its Debug must never expose key material.
PublicKey and Signature Debug use their type name around lowercase encoded hex.

PublicKey exposes `verify(&self, message: &[u8], signature: &Signature) -> Result<(), ()>`.
Identity public keys may deserialize/serialize but always fail verify. Add the
bytes convenience function `verify_bls_signature(signature: &[u8], message: &[u8],
public_key: &[u8]) -> Result<(), ()>`. Malformed inputs and mismatches fail closed.

Provide explicit caller-owned `PublicKey::prepare(&self) -> PreparedPublicKey`
and `PreparedPublicKey::verify` with identical validity decisions. Cache only
immutable public mathematical preparation. No application key cache, trust root,
entropy, global resource policy or certificate ownership. A lazily prepared fixed
public generator may use the standard library. Ordinary verification must remain
usable without the caller maintaining a prepared key.

## Output and separation

Write implementation only under `qualification/bls-wrapper/src/`, using ordinary
Rust modules. Keep it small and documented, no copied notices/bodies or algorithm
test vectors. The integrator writes independent public-boundary tests separately;
do not read `tests/`. Cargo.toml and this contract are allowed. Compile/clippy the
library only in an isolated target directory if needed. Report exact source inputs
read, design choices, unresolved constraints and validation; do not assert legal
clearance or production readiness. Production type identities and all certificate,
delegation/freshness policy remain unchanged by this isolated experiment.
