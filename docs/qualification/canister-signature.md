# Canister-signature boundary — 0.34.1

## Named consumer and retained obligations

The current read-only inspection found ic-auth HEAD
`b78991e9833320692921bf766e51ad361c9c8c4d`, with the caller now at
`crates/ic-auth/src/issuer_signature.rs`. That file and `certified_message.rs`
are unchanged against HEAD. The isolated current-contract qualification is
recorded below; the earlier inspection remains provenance for the initial slice.

Read-only inspection on 2026-11-16 found ic-auth HEAD
`e52a40be75dd9809d0528189d84b43fe96952ad9`; its tracked
`crates/ic-auth/src/canister_signature.rs` is unchanged against that HEAD.
`verify_canister_signature` calls old Platform's `verify_canister_sig_with_time`.
Auth selects protected signer/seed, message and signature bounds, raw root and
signing-time freshness. It does not apply signing freshness to delegation time.
This is a concrete replacement requirement, not authority to modify Auth.

The [IC signature specification](https://docs.internetcomputer.org/references/ic-interface-spec/)
defines a tagged envelope containing certificate bytes and a witness tree.
The full contract must bind the exact message and canister-signature key to an
empty `sig / SHA256(seed) / SHA256(message)` witness leaf; bind that witness digest
to the issuer's certified data; and authenticate certificate root, issuer authority,
delegation and signing time. Key parsing, hashing and cryptographic primitives
remain upstream-owned. Auth retains application authorization and domain selection.

Certificate authentication and canister-signature authentication have different
inputs and obligations. The latter additionally needs key admission, message/seed
hashing, witness ordering and bounds, empty-leaf admission, and certified-data
binding. Reuse the existing certificate verifier only after those differences
are explicit; witness lookup alone never grants authenticated status. Before Auth
migration, resolve its current delegation-time contract against the replacement's
explicit separate time policy rather than silently applying signing freshness.

## Implemented envelope slice

Platform 0.34.1 adds `canister-signature-decode`, depending only on the existing
`certificate-decode` feature. `decode_canister_signature` checks the caller's
whole-input byte bound before the required self-describing tag or parsing. The
new envelope declaration reuses existing strict byte-string/tree visitors; it
rejects duplicate/unknown fields, malformed node tags/arity/digests and trailing
bytes. Unknown-field refusal deliberately matches the replacement's strict
certificate boundary rather than old Platform's permissive signature envelope.
No historical source/tests, crypto implementation or new dependency is copied.

The immutable result borrows the original complete signature bytes. Its embedded
certificate byte-string payload is retained exactly in owned storage, but remains
opaque and may be invalid. The witness is explicitly unverified: neither sibling
ordering nor message, seed, key, digest, signature, range or time checks occur.
Input bytes are not a heap/work quota; Serde CBOR retains its recursion limit.
This slice is not a substitute for Auth's existing verification call.

## Evidence and remaining work

Three independently authored tests cover original byte custody and deliberately
invalid embedded certificate retention; pre-parse size refusal, missing tag,
every truncation and trailing data; duplicate/unknown fields, wrong payload kinds,
and malformed witness nodes. A compile-fail check rejects mutation of embedded
bytes. The shared certificate visitor tests remain in the same feature closure.
Native/Wasm Clippy and Rustdoc pass with warnings denied. Wasm here is compilation,
not execution of signature-envelope fixtures. The dependency graph excludes BLS,
Agent, Types, Runtime and Protocol for this decode-only feature.

The authentication slice below now supplies key admission and witness/certificate
composition. Observed PocketIC signature replay is recorded below; actual Auth migration remains pending.
The separate certificate mainnet corpus is not canister-signature proof.

## Key and authentication implementation

`canister-signature-authentication` composes the existing decoded envelope with a
caller-admitted `CanisterSignatureKey`, exact message, prepared root, separate
`TimePolicy` windows and `CanisterSignatureLimits`. The returned immutable evidence
retains the original envelope, message and DER key, root, signing time and optional
delegation context. It has no constructor, mutable access or Deserialize.

RustCrypto `spki` 0.8.1 (defaults disabled), `der` 0.8.3 and `const-oid` 0.10.3 own
canonical DER/SPKI parsing. The narrow adapter checks the canister-signature OID,
absent algorithm parameters, byte alignment and canonical local Principal admission.
It retains Auth's supported 20–129 byte key boundary and borrows issuer/seed input;
long-form keys are not admitted. No ASN.1 parser or cryptographic algorithm is
implemented locally. Existing workspace `sha2` 0.11.1 hashes seed/message, while
existing upstream BLS/tree dependencies retain their own SHA-256 implementation.

Witness grammar uses the existing ordered-forest validator before lookup or hashing.
The witness must expose an empty leaf at the exact seed/message hashes. Its digest
must match the issuer's certified-data leaf in an authenticated certificate. One
node budget covers the witness, signing tree and delegation tree; per-tree depth,
embedded certificate/delegation bytes, message length and range pairs are explicit.
Encoded signature size remains a prior decoder obligation. Embedded certificate
and delegation encodings must carry their self-describing tag on this path.

The current IC specification additionally requires the delegation's certified
`/subnet/<id>/type` to be present and not `cloud_engine`. The signature-specific
authority path admits `application`, `system` and `verified_application`; unknown
values fail closed. This is separate from ordinary canister/subnet certificate
verification, whose behavior remains unchanged. Missing type returns the shared
missing-path refusal; forbidden/unknown type returns `NotAuthorized`.

The old Auth consumer does not constrain delegation time. The new API requires a
caller-selected separate window; callers can express any desired historical
interval without reusing the signing window. No live default or migration policy
is chosen here. Auth must still enforce protected issuer, seed and message domain;
cryptographic evidence does not grant application authorization.

## Authentication evidence

Nine independently authored tests use upstream BLS signing for direct/delegated
certificates. They exercise canonical/truncated/wrong-OID/unaligned/parameterized
and boundary-sized keys; exact retained inputs; different signing/delegation times;
wrong message/seed/issuer/root, both corrupted signatures, unavailable/nonempty
witnesses, wrong digest, missing/cloud/unknown subnet type, tag refusals, ambiguous
witnesses and exact shared node budgets across two and three trees. No sibling
source, tests or upstream algorithm was copied. Two compile-fail examples reject
message mutation and fabricated/deserialized authenticated evidence.

The focused target passes native tests, native/Wasm Clippy, Rustdoc and selected
graph checks with no Agent, Runtime, Protocol or additional Principal identity.
Wasm is compile evidence for this signature slice; existing certificate replay is
not signature execution. A cached-database root audit reports no vulnerabilities;
the pre-existing serde_cbor unmaintained warning remains. No advisory is ignored.
The following public consumer adds observed signature execution. Auth migration
remains required before completing this capability.

## Observed public-consumer replay — 2026-11-16

The private `platform-certificates` qualification package now has an opt-in
`canister-signature` selection. Its independent WAT probe installs a witness
digest on root/application subnet canisters and retrieves replica certificates.
The host constructs the witness and outer envelope with upstream APIs; the
replica supplies the BLS signatures and delegation. No candidate signing helper,
copied upstream fixture or product implementation is used to generate them.
See the [frozen observations and provenance](../../qualification/platform-certificates/fixtures/signatures/README.md).
Both fresh direct and delegated observations authenticate through the public API.

`make qualify-canister-signature-wasm` replays those exact envelopes on native
Linux and standalone wasm32 under Node 24.21.0/V8 13.6.233.17. It checks retained
message/key/envelope bytes, explicit root, issuer, seed, subnet and timestamps;
wrong message/seed/root/issuer; corrupted signing and delegation signatures;
an otherwise correct witness with a different digest; stale/future signing time
and independently rejected delegation time. Input/message/certificate/delegation,
tree-depth/node and range refusals are exercised, along with every truncated
envelope and trailing bytes. The replay uses exact inclusive time windows;
the existing synthetic tests separately cover unequal signing/delegation times
and forbidden subnet types.

Native/Wasm Clippy, the selected Wasm graph and executed replay pass. The graph
has one local Types and Platform identity, with no Runtime, Protocol, Agent,
Candid, WAT or Testkit in the Wasm closure. Testkit/WAT remain native-only.
The qualification feature explicitly enables upstream tree Serde for fixture
mutation; it does not rely on native feature unification. The added direct
qualification dependency `serde_cbor` 0.11.2 was confirmed latest stable and
non-yanked against the crates.io sparse index on 2026-11-16. It was already in
the selected graph; its existing unmaintained advisory remains explicit.
No product package contents or product dependency selections changed in this batch.

Executed replay Wasm SHA-256:
`0c929e050a307f9925753ea0423a1ef454b5d216473b6a81cc336fa8ccf258f3`.
CI now selects fresh observation and frozen native/Wasm replay; hosted/macOS
execution has not been established by this local run. Standalone Wasm execution
does not establish installed verifier behavior, instruction costs or size acceptance.
These controlled signature envelopes are not mainnet signer interoperability
or production Auth proof. Auth's separate delegation-time policy still needs
resolution during its separately authorized migration. The full authentication
gate remains open.

## Isolated Auth issuer-policy consumer — 2026-11-16

`make qualify-auth-consumer` executes an independently authored adapter in the
existing private qualification package. It consumes only public candidate APIs;
it neither copies nor links Auth source. The named current caller is
`issuer_signature::verify_issuer_signature`, reached from
`verification/certified.rs` after claims/delegation binding and receiver budget
charging. The fixture models only that issuer-signature component, not complete
credential verification. Canic endpoint wiring is outside this contract.

The inspected inputs are source-bound by SHA-256:

| Auth input at the revision above | SHA-256 |
| --- | --- |
| `crates/ic-auth/src/issuer_signature.rs` | 9d1b600bb900256b3dbd25e709c2bce42af206a5c09249c350b6847365c18c80 |
| `crates/ic-auth/src/certified_message.rs` | 56fb2c5125bcd709baf21a62813cf107893a55431a180d21cf4e2dd3e8f0c49e |
| `Cargo.lock` | c09af18c233561c7e2c2583a58d4b844696dce7774dc5a44d87482d2117dd1bd |

The consumer supplies a protected issuer, SHA-256 seed commitment, raw 96-byte
network root, trusted clock, maximum signing age and signature bound capped at
131072 bytes. The fixture frames a fixed 32-byte claims digest with the exact
length-prefixed `ic-auth-portable-credential` domain. It uses upstream SHA-256
for the seed commitment; no hash or signature primitive is implemented locally.
The 37-byte IC BLS SPKI prefix adapts the protected raw root to `PreparedRootKey`
without discovering a root or trusting one supplied by the proof. Candidate
admission still verifies the full key. Product raw-root API expansion is not
needed to establish this boundary.

To preserve the current caller's distinct stale/future refusals, the adapter
explicitly admits the full u64 time range during authentication, then checks
the authenticated signing time against the consumer clock and inclusive maximum
age before returning it. The separate delegation window also spans the full
u64 range, matching the caller's documented lack of delegation freshness checks.
This is a qualification compatibility selection, not a new Platform default or
an authorized production Auth migration. Cryptographic delegation authority,
signature, well-formed time and resource checks still apply. The consumer also
demonstrates that selecting a narrow delegation window refuses the same evidence.

Fresh PocketIC certificates now bind the exact Auth-framed message on both root
and application subnets. The additional frozen observations are described in the
[fixture provenance](../../qualification/platform-certificates/fixtures/signatures/README.md#auth-message-observations).
The adapter passes on native Linux and standalone Wasm: exact age and one-past
age, future signing time including clock zero, u64 clock/age extremes, wrong
protected issuer/seed/root, malformed key/root length/envelope, changed digest,
wrong message domain, exact/over signature limits including the hard cap, and
resource refusal. Only a timestamp passes out of the component after its policy
checks; it cannot produce verified claims or authorize a credential.

Native/Wasm Clippy, portable graph admission, frozen replay, workflow lint and
format checks pass. The graph has one local Platform/Types identity and no Auth,
Canic, Runtime, Protocol, Agent or native simulator tools. Qualification-only
`sha2` 0.11.1 was confirmed latest stable/non-yanked in the crates.io sparse index;
the existing locked version was reused without changing the product graph.
Executed Wasm SHA-256:
`9cbf86e4d34eeb307f565e07a007a5504241780899499ec5249308761752bb50`.

This proves that the candidate supports the inspected component's policy shape.
It does not compile the actual Auth crate against the candidate, qualify all
legacy error precedence, prove malformed authenticated-time classification with
observed fixtures, or migrate Auth's registry Types cohort. The candidate's
strict CBOR, delegation-type and shared resource refusals remain intentional
differences requiring actual consumer acceptance. Complete credential/root
authorization, replay state, budget charging, installed verifier execution,
mainnet interoperability and hosted/macOS evidence remain outside this result.
No sibling files or product package contents were changed.
