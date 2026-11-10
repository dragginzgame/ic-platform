# Owned canister-signature protocol contract

Maintainer decision: 2026-11-10. This is the specification for an independent
implementation of Platform's existing signature surface. It is not an upstream
source translation. Protocol composition belongs here; cryptographic primitive
implementations remain external. No signature store or scheduling API is added.

## Inputs and context boundary

Public protocol sources inspected on 2026-11-10:

- <https://docs.internetcomputer.org/references/ic-interface-spec/#canister-signatures>
- <https://docs.internetcomputer.org/references/ic-interface-spec/#canister-ranges>
- <https://docs.internetcomputer.org/references/ic-interface-spec/certification/>
- <https://docs.internetcomputer.org/references/ic-interface-spec/https-interface/#status>

Public constant values and Rust API signatures are factual compatibility inputs.
The specification/review agent has inspected upstream implementations; the
implementation agent must start without that conversation, upstream bodies,
old adapter bodies, or upstream/old expressive tests. It may read this contract,
repository governance, manifests, Principal APIs and public dependency API docs.
Implement in new files under the existing public module names. The integration
agent owns oracle comparisons and execution of pre-existing regressions. Record
any additional behavioral observations as inputs, never source patches.

The baseline is Platform's existing 1.3.1/0.3.0 supplier behavior plus its stricter
DER and identity-key admission. This is not automatic adoption of every future
IC specification change. In particular, the current online specification's
subnet-type/cloud_engine requirement is outside this compatibility migration;
the selected supplier and existing Platform contract do not enforce it.

## Creation surface

Module `canister_sig_creation`, selected by `canister-sig-creation`:

- `CanisterSigPublicKey { pub canister_id: crate::principal::Principal,
  pub seed: Vec<u8> }`, Clone/Eq/PartialEq/Debug.
- `new(Principal, Vec<u8>) -> Self` preserves both fields.
- `try_from_raw(&[u8]) -> Result<Self, String>`, `to_raw(&self) -> Vec<u8>`,
  `to_der(&self) -> Vec<u8>` and `TryFrom<&[u8], Error=String>` for DER.
- `hash_bytes(impl AsRef<[u8]>) -> [u8;32]` is external SHA-256.
- `hash_with_domain(&[u8], &[u8]) -> [u8;32]` hashes the one-byte domain
  length followed by domain and payload through external SHA-256. Preserve
  existing one-byte length truncation for inputs over 255 bytes; no new policy.
- `extract_raw_root_pk_from_der(&[u8]) -> Result<Vec<u8>, String>` requires
  exactly the public 37-byte prefix and 96-byte BLS public key.
- `IC_ROOT_PK_DER_PREFIX: &[u8;37]`, `IC_ROOT_PK_DER: &[u8;133]` retain
  their existing public types and network values below.

Raw canister keys encode one byte for issuer length, those issuer bytes, then
all remaining bytes as seed. Issuer length is 0..29. Seed is unbounded by this
API. Use owned Principal admission and preserve bytes, including empty seed.
The DER envelope uses OID 1.3.6.1.4.1.56387.1.2, no parameters, and a byte-aligned
BIT STRING carrying the raw key. The selected public encoder historically emits
one-byte length fields even beyond DER short-form limits; retain its bytes for
all input sizes because `to_der` is infallible. Admission intentionally accepts
only correctly sized short-form envelopes: minimum 20 bytes, SEQUENCE 0x30,
matching total length, the exact OID sequence, BIT STRING 0x03, matching remaining
length, and zero unused bits. Both lengths must be <128. This preserves the
existing distinction between encoding compatibility and accepted DER.

Protocol/public constant bytes (hex):

- Canister algorithm identifier: `300c060a2b0601040183b8430102`.
- Root DER prefix: `308182301d060d2b0601040182dc7c0503010201060c2b0601040182dc7c05030201036100`.
- Mainnet raw root key:
  `814c0e6ec71fab583b08bd81373c255c3c371b2e84863c98a4f1e08b74235d14fb5d9c0cd546d9685f913a0c0b2cc5341583bf4b4392e467db96d65b9bb4cb717112f8472e0d5a4d14505ff d7484b01291091c5f87b98883463f98091a0baaae`
  (remove whitespace). The exported DER root is prefix plus raw key. It is
  data, not an automatically selected trust root for verification.

Compatibility error strings are observable API data: empty raw key is
`empty raw canister sig pk`; insufficient issuer bytes is
`canister sig pk too short`; oversized issuer is
`invalid canister id in canister sig pk: Bytes is longer than 29 bytes.`.
Invalid root length is `invalid root pk length`; invalid root prefix is
`invalid OID`. DER diagnostics retain `canister sig pk shorter than
DER prefix`, `invalid OID of canister sig pk`, `canister sig pk too short or
invalid DER length`, and `invalid canister sig pk DER envelope` where applicable.
No panic on arbitrary decoder bytes.

Qualification observations retain these additional diagnostic values: rejected
BLS keys (including identities) and failed BLS verification report `invalid BLS
signature`; missing envelope tags report `signature CBOR doesn't have a
self-describing tag` or `certificate CBOR doesn't have a self-describing tag`
according to the envelope. These are public API observations, not source inputs.

## Verification surface

Module `signature_verification`, selected by `signature-verification`:

- `verify_canister_sig(message: &[u8], signature_cbor: &[u8],
  public_key_der: &[u8], ic_root_public_key_raw: &[u8]) -> Result<(), String>`.
- `verify_canister_sig_with_time` with the same arguments returns
  `Result<u64, CanisterSigTimeError>`.
- `CanisterSigTimeError::{Verification(String), InvalidCertificateTime}` remains
  Clone/Debug/PartialEq/Eq/Error. Diagnostics inside Verification are opaque;
  the time error displays `invalid signing certificate time`.

Use external `ic_certification` Certificate/HashTree and digest/lookup APIs,
`serde_cbor`, `serde_bytes`, external `ic_verify_bls_signature` verification and
`ic_bls12_381` checked parsing/identity queries. Do not implement tree hashing,
BLS or point encoding. Use existing workspace `sha2_11` for SHA-256. No Candid,
registry Principal, IC0, lazy network-root cache, signature store or executor.

The supplied signature envelope contains a byte-string certificate and hash tree.
Require the self-described CBOR prefix d9d9f7 on signature and every certificate.
Retain map and serde-compatible sequence/packed-field representations, reject
duplicate required fields and malformed/trailing data. The nested certificate
contains tree, signature bytes, and optional subnet delegation. Decode ranges
as pairs of canonical owned Principals without external conversions.

Successful verification establishes all of these facts:

1. Public-key framing and issuer are admitted. The supplied witness has an empty
   leaf at `sig / SHA256(seed) / SHA256(message)`.
2. The signing certificate's `canister / issuer / certified_data` value equals
   the external digest of that witness.
3. Without delegation, the signing certificate is authenticated by the supplied
   raw root. With delegation, authenticate the delegation certificate using that
   root; reject a delegation within it. Obtain the subnet's DER BLS key from
   `subnet / subnet_id / public_key`, admit/extract it and authenticate the
   signing certificate using that key. Certificate signed bytes are the length
   prefixed domain `ic-state-root` followed by its external tree digest.
4. Both root and delegated BLS keys are parsed/checked using upstream APIs and
   identity points are refused even when upstream BLS alone would accept them.
5. Delegation authorizes the issuer in the certified canister ranges. Support
   legacy `subnet / subnet_id / canister_ranges` and sharded
   `canister_ranges / subnet_id / shard_start` paths. When a sharded subtree is
   present use the greatest visible shard-start <= issuer bytes; do not fall
   back to legacy ranges merely because the selected shard is missing, invalid
   or excludes the issuer. Absent/unknown sharded subtree retains legacy fallback.
   Shard labels use byte lexicographic order. For compatibility, interval
   membership retains canonical Principal's length-first order, as in the
   selected supplier; do not silently change existing mixed-length admission.

Do not add arbitrary input caps, freshness rules, trust roots or application
authorization. Tree lookups and existing byte compatibility remain external;
this is not a universal stack-safety guarantee for arbitrary inputs.

Time is extracted only after every verification requirement succeeds, only from
the signing certificate. Require one visible top-level `time` label and a leaf
value consisting entirely of a u64 unsigned LEB128 encoding, at most ten bytes.
Accept nonminimal encodings within that range. Reject absent/pruned/non-leaf,
duplicate visible labels, overflow, truncation and trailing bytes. Never use
the delegation certificate's time. Avoid a second certificate decode when
authenticated evidence is already available.

## Qualification and measurement

Write independent unit tests from this contract in `tests.rs`/`tests/` modules.
The integration agent separately runs the existing owned regression cases and
DFINITY public API oracles for byte equivalence and acceptance comparisons.
Intentional Platform DER/identity restrictions must stay stricter. Benchmark
equivalent valid creation and direct/delegated verification workloads with
black-box inputs/results, warmup, repeated alternating runs and release builds.
Report workload sizes and dispersion, including slower cases. Benchmarks are
native latency evidence, not installed IC instruction counts. Prove native/Wasm
production graphs exclude both registry signature packages and Candid. Preserve
unrelated pending changes, existing package paths and public features.
