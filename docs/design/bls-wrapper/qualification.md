# Independent BLS wrapper adoption — 2026-11-11

The qualified implementation shipped in Platform 0.31.0 under `ic_platform::bls`. The
separate experimental implementation package is removed; qualification retains
only the published-baseline/current-source probe. No new product crate or release
group was introduced; publication follows the ordinary Platform release workflow.
The rerun preserves the prototype's 13.88% one-shot instruction reduction and
2,643-byte optimized Wasm saving (1.10%) against published Platform 0.30.0.

Signature and deserialization error types now have Platform identity; consumers
passing registry verifier types directly must use the Platform imports or exchange
serialized bytes explicitly. Error variants and external bytes remain unchanged.
Keys already had Platform identity. Certificate/tree identity, domains, delegation,
issuer authorization, authenticated time and caller-owned freshness remain intact.
The new `PreparedPublicKey` API carries mathematical preparation, not trust.

## Source separation and ownership

The implementer received the reviewed [contract](contract.md) in a fresh context
without the conversation history, upstream verifier bodies, Platform's BLS or
signature implementation, or their tests. The integrator had read existing code
and authored separate public-boundary tests; the implementer did not read those
tests. Measurement feedback identified unnecessary key admission/preparation on
failure paths, which the same implementer corrected without reading old code.

Recorded implementation inputs were the contract, AGENTS, TOKO baseline, current
handoff, qualification manifests and pinned toolchain; the pinned Toko Rust
hygiene guide; public Digest 0.10.7, SHA-2 0.11.1, G1Affine and G2Prepared API
documentation; and introductory protocol material from the BLS draft referenced
by the contract. Public curve API pages resolved to `ic_bls12_381` 0.10.1. No
supplier implementation or expressive test/vector suite was provided or read.
This records engineering separation, not legal clearance or a cryptographic audit.

The two implementation files total 193 Rust code lines (`cloc` 2.10, excluding
comments/blanks). They own admission and API composition. Curve arithmetic,
pairings, hash-to-curve, point/scalar codecs and SHA-256 remain supplier APIs.
The small Digest adapter forwards updates, reset and finalization to SHA-2 0.11;
it does not implement hashing or message expansion. No random-key API is added.

## Dependency closure

| Path | Published Platform 0.30.0 | Adopted wrapper |
| --- | --- | --- |
| BLS API | `ic-platform/bls → ic-verify-bls-signature 0.6.0` | `ic_platform::bls` |
| Curve/hash-to-curve | `ic_bls12_381 0.10.1` | same supplier/version/features |
| SHA-256 | `sha2 0.10.9` | `sha2 0.11.1`, matching Platform's owned hash selection |
| Formatting/lazy preparation | `hex`, `lazy_static` | standard formatting and `LazyLock` |
| Curve's hash trait | Digest 0.10.7 | Digest 0.10.7 adapter over SHA-2 0.11 |

The candidate has three direct normal dependencies: the curve supplier, SHA-2,
and Digest 0.10. Default features and RNG selection are disabled. The retained
curve enables alloc/groups/pairings/experimental, as required for verification.
Its field/group APIs still bring `rand_core` 0.6.4 as a trait dependency; this is
not a selected random-key generator or entropy source.

The isolated candidate graph excludes the registry verifier, SHA-2 0.10, `hex`
and `lazy_static`. It still contains both Digest 0.10/0.11, crypto-common 0.1/0.2
and block-buffer 0.10/0.12, plus the curve's ff 0.13, group 0.13 and pairing 0.23.
Changing the wrapper cannot remove those old curve interfaces. Full certificate
verification also uses `ic-certification`, which independently retains SHA-2
0.10; this adoption does not eliminate that whole-application dependency.

Runtime IC0 belongs only to the probe transport. Testkit, its native client graph,
Toko and the registry verifier oracle are development infrastructure, excluded
from the candidate normal/build Wasm graph. The root product workspace remains
five packages with three release groups; the sole qualification member is private.

## Matched Wasm and installed results

Both arms use the same `bls-probe` entrypoint source, qualification lock, Rust
1.99.1 (`58cf8f9c3`, LLVM 23.1.1), Wasm target, opt-level 3, fat LTO, one codegen
unit, panic abort, stripped symbols, path remapping and `ic-wasm` 0.11.1 shrink.
The baseline calls the published 0.30.0 adapter, including identity-key refusal.
The candidate calls the adopted production module. There are no custom Wasm sections.

| Uncompressed bytes | Platform 0.30.0 | Adopted module | Reduction |
| --- | ---: | ---: | ---: |
| Raw total | 248,794 | 246,052 | 2,742 |
| Raw code section | 241,695 | 238,906 | 2,789 |
| Optimized total | 240,000 | 237,357 | 2,643 |
| Optimized code section | 232,898 | 230,205 | 2,693 |

Installed Linux PocketIC 16.1.0 runs use identical signature/key/message bytes,
fresh canisters for each arm/method, one cold call and five warm calls. All five
warm counts were identical. `performance_counter(0)` measures verification after
input copying and before output construction; these are IC instruction counts,
not native wall-clock timings or full certificate verification costs.

| Verification instructions | Platform 0.30.0 | Adopted module | Reduction |
| --- | ---: | ---: | ---: |
| Ordinary, cold | 259,540,141 | 225,383,325 | 13.16% |
| Ordinary, warm | 246,096,451 | 211,939,615 | 13.88% |
| Cached key, cold | 259,541,091 | 225,383,637 | 13.16% |
| Cached key, warm | 177,794,404 | 164,194,395 | 7.65% |

Baseline caching retains a checked Platform public key; candidate caching retains
explicit prepared pairing data. Neither adds trust or freshness policy. The
prepared-key API is caller-owned, not an application cache in Platform. These
figures measure the combined implementation and SHA change; they do not isolate
each change's contribution. Repeated-key gains also carry unmeasured prepared-key
memory costs. No large whole-canister Wasm saving is established.

## Compatibility and focused validation

Four independently written integration tests pass. They compare public-key and
signature bytes against the registry supplier for three keys and twelve message
lengths, including empty input and SHA padding/block boundaries. Both ordinary
and prepared APIs verify supplier signatures, and the supplier verifies candidate
signatures. Tests also reject wrong messages/keys, malformed lengths/encodings,
out-of-range scalars, non-subgroup points and identity keys, including keys derived
from zero. Private-key Debug is redacted. Curve algorithm correctness itself is
the external supplier's responsibility.

The installed probe separately checks key replacement, empty messages, wrong
messages, malformed signatures/keys, identity keys, truncated requests and a
successful call after refusals. Candidate malformed-signature admission costs
11,870,395 instructions versus baseline 11,870,456; identity-key refusal costs
46,015,467 versus 80,175,451. No admission check was removed to obtain the gains.

Production native library/selected-integration Clippy and 39 tests pass (one native
benchmark deliberately ignored). They include certificate/delegation admission,
identity refusal at root and delegated keys, range/witness binding, malformed CBOR,
and signing-time authentication. Both Wasm arms and their native driver pass
Clippy. All five warm BLS samples agree; the adopted probe has the same sizes and
instruction counts as the prototype. Its raw evidence is
`target/evidence/bls-wrapper.fc3YDp/`; initial experiment evidence remains at
`target/evidence/bls-wrapper.fNqWT0/` and `bls-wrapper.QhyZrI/`.
Optimized artifact SHA-256 identities for the adoption rerun:

```text
baseline  f4e64f9aeb605bfdff6c77239ed74f59955df5c5d532abf1d6a23ffa438a3cfd
candidate 10fb10507087d245b047b5f2f79e28e81239ca3b7d3a0252131f1379553c45cb
```

The complete installed signature probe also passes direct, legacy/sharded
certificate delegation, wrong-message and identity-root cases on Linux PocketIC.
Warm adopted instruction counts are 211,621,685 direct, 423,205,602 delegated legacy
and 423,231,119 delegated sharded. These are absolute counts for this fixture,
not a paired before/after certificate-speed claim. Evidence:
`target/evidence/signature-instructions.amN9C1/`.
The existing duplicate certificate key parse was removed because the adopted
BLS boundary now enforces the same checked subgroup/identity admission itself.
The trust and freshness policies remain with callers; no automatic key cache
was inserted into certificate verification.

Focused BLS/signature feature-contract and native/Wasm closure tests pass; they
reject the registry verifier in owned production graphs and preserve the native
Agent exception. The root lock changes only Platform's direct Digest edge, with
no supplier upgrades during implementation. Published Toko 0.39.10 was subsequently
adopted in both locks and the installed snapshot. Snapshot, inventory and the
actual dependency-boundary gate pass with its reviewed Digest-adapter exception.
Snapshot-owned files were not hand-edited and no gate was bypassed.

See [reproduction commands](../../../qualification/README.md). Platform 0.31.0
passed all 19 local release stages and package archive verification before
publication. Its commit is `52e6af66c9183f4f0dc777e7290b445738b7bed5`, tagged
`platform-v0.31.0`; the [handoff](../../status/current.md) records the release receipt.
The measurements above describe the qualified pre-release implementation. No fresh
macOS, live-network or sibling Auth acceptance is claimed; hosted CI is asynchronous.
