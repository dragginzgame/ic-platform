# Platform certificate boundary — 0.34.0

Platform has empty defaults and two opt-in capabilities. `certificate-decode`
strictly decodes untrusted certificates, borrowing the exact original bytes and
exposing an immutable, explicitly unverified view. `certificate-authentication`
adds direct-root and single-delegation verification with caller-selected root,
target, separate time windows and resource limits. Nested delegations are refused.
There is no implicit root, clock, transport or retry policy.

## Narrow reuse admission

Reuse is limited to the byte-string and hash-tree Serde visitors and wire record
shapes in old Platform's `certification/cbor/strict/mod.rs` at commit
`c24e3a59583eabcf5b6bb87987269c13a209a51f`, source SHA-256
`69f5b9062e939a31ce10a26c868928e0f876c8e35fee57a4555fb74dfdec3dc9`.
That source credits ic-host-tools contributors at Tooling commit
`9ed16c7a480ad80ae6ff606bd17de3350a2b0b97`. Preserve that attribution in the
adapted source. The supplying source and this repository use the Toko Studios
Network Licence 1.0; no algorithm or historical tests are copied.

The retained contract is strict byte-string payloads, exact tree node tags and
arities, 32-byte pruned digests, required fields, duplicate/unknown field refusal,
and opaque delegation bytes. Original whole-input byte admission precedes parsing.
Serde CBOR retains its recursion limit; input bytes are not a heap/work quota.
The new boundary omits generic CBOR encoding, standalone tree decoding and mutable
public certificate fields. Upstream ic-certification 3.2.0 supplies tree values;
serde_cbor 0.11.2 supplies parsing. The adapter contains no unsafe or cryptographic
primitive implementation. Independent literal tests qualify the retained grammar.

The crate is 0.34.0 because crates.io's latest non-yanked Platform entry was
0.33.7 at selection on 2026-11-16, and this replacement deliberately changes APIs.
It is unpublished and belongs to its own Platform release group.

## Direct-root authentication admission

The authentication capability composes exact DER admission with upstream
`ic-verify-bls-signature` 0.6.0 (defaults disabled, `alloc` only). Platform is the
primary external-crypto owner under the reviewed dependency rules; this is not
an extension of the separate qualification-only BLS oracle exception. Point
and subgroup decoding, hash-to-curve and pairing remain entirely upstream.
Checked canonical compressed identity encodings are refused by the adapter.
The upstream generator precomputation is constant; no verification-result or
caller-root cache is introduced. EC signing and Agent remain unselected.

Ordered-forest admission and complete u64 LEB128 time admission are narrowly
adapted from old Platform's historical certificate adapter at
`c24e3a59583eabcf5b6bb87987269c13a209a51f`, file
`certification/historical/mod.rs`, SHA-256
`06ff0a34a3b006b2502381d005cd76b5955569b01162e4da51b13c8bfca25cf0`,
under the same Toko licence.
The destination preserves all visible sibling ordering/ambiguity checks and
root-inclusive depth/node accounting before lookup/hashing. No historical
fixtures or cryptographic bodies are copied. Independent tests cover the direct
path's refusal and boundary cases below.

The public function is deliberately `authenticate_direct_certificate`: any
delegation refuses, even if its contents are malformed. It never authenticates
only the outer signature while ignoring an authority chain. Direct-root evidence
binds immutable original bytes, exact root DER, canonical local target identity,
explicit inclusive time window and checked timestamp. A trusted root has global
canister authority; target binding records supplied context and does not invent
a direct-certificate range check. Request-path/application acceptance is separate.
The general `authenticate_certificate` path additionally checks one delegation;
its contract is detailed below. Canister-signature verification remains unavailable.

## Validation and remaining gates

`make qualify-platform-authentication` includes the decoder gate and passes 28
unit tests, five observed-frame tests, three subgroup tests and nine compile-fail
doctests on Linux. Tests cover literal malformed
CBOR, exact-byte borrowing, wrong roots, altered signatures and time leaves,
identity keys/signatures, malformed DER, signature domain separation, inclusive
time boundaries and overflow, complete u64 LEB128, ambiguous trees and traversal
bounds. A separately calculated SHA-256 tree digest anchors a small fixture;
synthetic fixtures use upstream signing. The additional observed corpus is issued
by PocketIC, as described below. Dedicated non-subgroup point fixtures now pass
through public verification boundaries as recorded below.

Compile-fail checks reject fabricated decoded values, mutable unverified access,
authenticated field/tree mutation, implicit promotion and arbitrary deserialization.
Native/Wasm Clippy, Rustdoc and selected normal/build graph checks pass. Empty
defaults have no dependencies. Authentication selects exactly one local Types,
ic-certification and BLS verifier; Protocol, Runtime and Agent remain absent.
The ordinary gate compiles Wasm; the standalone execution gate below also runs
the verifier. Neither establishes installed verifier execution.

A fresh root-lockfile `cargo audit` on 2026-11-16 reports zero vulnerabilities
against RustSec revision `02e2517f457a359fc6d3440ea4560c8ea188fee8`
(updated 2026-11-15). It reports the existing unmaintained-parser warning
RUSTSEC-2021-0127 for serde_cbor 0.11.2. No advisory is ignored. Parser replacement
needs explicit preservation of strict byte-string, recursion and trailing-input
contracts; the warning is not resolved by these tests. This audit does not cover
separate qualification lockfiles or close crypto/Agent issue #4: neither Agent nor
EC signing is admitted here.

The full authentication gate remains open: frozen direct and delegated mainnet observations now
pass native/Wasm replay (below); broader interoperability and production
consumer qualification remain pending. Canister-signature witness checks are a separate unimplemented surface.
Decode success alone proves syntax; authentication does not accept an application's
request path or status on its behalf.

## Delegation implementation admission

The implementation admits one delegation, reusing the bounded range visitor from
`certification/historical/ranges/mod.rs` at the same reviewed historical commit,
SHA-256 `af57442322bea687cf01c96df186f4be56819d39eb33e7dcf27715e20dbe25ff`,
under Toko Studios Network Licence 1.0. No historical tests are copied. The
adapter parses every selected pair, enforcing exact tuple/byte-string grammar,
29-byte principal bounds, trailing-data refusal and explicit pair count.

The [IC specification](https://docs.internetcomputer.org/references/ic-interface-spec/#canister-ranges)
requires bytewise lexicographic ranges. The historical adapter used Principal's
length-first Rust ordering; the replacement compares significant byte slices,
with unequal-length regression fixtures. Selected shards start at their label;
ordered/nonoverlapping pairs are checked after a match as well. Root and subnet
signatures share the existing upstream verifier, with no new dependency or crypto
implementation. Separate explicit time windows and combined tree-node accounting
apply to both certificates. Embedded certificate byte and selected range-pair
allowances are explicit; the shared node allowance covers both admitted trees
before hashing or lookup. The immutable evidence records delegation subnet,
window and timestamp alongside outer bytes, original root and effective target.

Ten independently authored delegation tests cover both formats, target endpoints
and misses, both signatures and subnet-key admission, nested/malformed/oversized
delegations, exact combined tree and range bounds, separate inclusive windows,
malformed pairs after a match, bytewise order and principal length limits.
A selected malformed, hidden or nonauthorizing shard never falls back to an earlier
shard or legacy ranges. If the sharded namespace is absent or unknown, a separately
found and authenticated legacy leaf may prove authority; unknown itself is never
absence or authorization. Partial-tree lookup distinctions remain upstream-owned.

The same native/Wasm Clippy, Rustdoc and graph checks pass with delegation selected;
no dependencies or feature defaults changed. These fixtures establish local
composition checks; the separate observed-frame checks below extend that evidence.

## Independent replica observations and public consumer

`make qualify-platform-observed` runs the unpublished `platform-certificates`
0.1.0 package in the existing qualification workspace. A small independently
written WAT canister sets 32 zero certified-data bytes and returns the IC-provided
certificate unchanged. PocketIC issues the signatures; the runner has no signing
helper or key. One canister is installed on the NNS subnet and another on an
application subnet. The native client obtains the simulator root explicitly;
this supplies no default root to Platform.

On Linux x86-64 with Rust 1.100.0, ic-testkit 0.106.0 and PocketIC 16.1.0, the
runner passed direct (433 bytes) and delegated (1,144 bytes) observations. It
checks exact-byte custody, root/target/time bindings, certified-data lookup,
delegated subnet identity, wrong-target refusal, direct-only refusal and stale
signing time. The native simulator Principal crosses into local Types through
an explicit checked byte conversion. The consumer compiles and exercises only
Platform's public API; Agent and Runtime are not dependencies of this package.

Frozen observations and hashes live with the product's
[external regression corpus](../../crates/ic-platform/tests/fixtures/README.md).
Five independently authored integration tests cover both original frames and parsed input,
check exact signing/delegation times, reject wrong targets and out-of-window
policies, and reject corruption of each of the 48 outer signature bytes in each
frame. They run in the ordinary authentication target, without a server. Fresh
observations stay under `target/platform-certificates/` and do not rewrite the
corpus. CI runs both replay and the separate PocketIC observation target.

The verifier executes natively against certificates produced by installed Wasm;
this does not claim verifier execution in Wasm, mainnet interoperability or
production consumer adoption. The product graph and defaults are unchanged.

## Selected-subnet signing time — issue #1

[Issue #1](https://github.com/dragginzgame/ic-platform-new/issues/1) supplies the
concrete Query requirement. `authenticate_subnet_time` now returns a narrow,
immutable `AuthenticatedSubnetTime` projection under `certificate-authentication`.
It consumes the original decoded certificate and releases its allocated tree;
only the original borrowed bytes and checked root/subnet/time context remain.
There is no re-encoding, Agent, transport, implicit clock or sampling policy.

The shared private verifier distinguishes canister authority from subnet authority.
Canister authentication retains complete range validation. Subnet authentication
requires the requested subnet to equal the authenticated delegation's subnet ID,
as specified by the
[IC subnet read-state contract](https://docs.internetcomputer.org/references/ic-interface-spec/https-interface/#request-read-state).
It verifies both signatures, checked keys, separate time windows and combined
tree bounds; no canister range is needed for a subnet observation.

Without delegation, a caller must supply a separately trusted root-subnet identity
bound to the selected root key, and it must equal the requested subnet. Missing
binding returns `RootSubnetRequired`; mismatches return `SubnetMismatch`.
Endpoint routing, response data or copying the requested ID do not establish that
trust input. The API never infers child-subnet progress from a root signature.

The result returns signing-certificate time, with delegation time exposed only
as separate context. Complete u64 LEB128, nonminimal-encoding acceptance, typed
malformed/time refusals and explicit inclusive windows are shared with certificate
authentication. Caller-selected decode bytes, combined nodes/depth and embedded
certificate bytes are enforced; these are not allocation quotas. There is no
additional field budget or general response-envelope decoder in this projection.

Tests cover different signing/delegation timestamps, absent range leaves for subnet
observations, root binding refusal, wrong subnet/root, full u64 time, malformed
LEB128, nonminimal encoding and inclusive stale/future boundaries. Frozen observed
frames and the native PocketIC consumer check both root/delegated subnet identity
through the public API. Mutation and direct deserialization fail to compile.
No production dependency was added. The response-type integration described below
completes the supplier projection contract. Query's sampler, endpoint acceptance,
mainnet admission and product delivery remain consumer-owned work.

The open-issue review also found Registry delivery issue #3 and crypto/Agent issue
#4. A fresh qualification-lock audit on 2026-11-16 against RustSec revision
`02e2517f457a359fc6d3440ea4560c8ea188fee8` reports zero vulnerabilities,
with unmaintained warnings for paste 1.0.15 (RUSTSEC-2024-0436) and serde_cbor
0.11.2 (RUSTSEC-2021-0127). No ignores were added. This current selected graph
contains no Agent/derivation remediation and does not close issue #4.

## Explicit non-subgroup rejection

Three external tests in `tests/subgroup.rs` distinguish malformed encodings and
identity points from canonical, nonidentity points outside the required subgroup.
The independently selected inputs are compressed G1 with x=0 and compressed G2
with x.c1=0/x.c0=2 (compression flag 0x80, all other bytes zero except final G2
byte 0x02). These are protocol input data, not imported upstream test vectors.

The admitted external curve implementation `ic_bls12_381` 0.10.1, packaged from
revision `280560da845158bcc96045c2d2e153a6c88e74b7`, classifies each point:
on-curve, not identity, not torsion-free, and exactly round-tripping through its
canonical compressed encoding. Its checked decoder refuses both. Unchecked
upstream admission appears only in the test's input-classification step; these
points are never used for signing, pairing or authenticated evidence there.
No field arithmetic, point codec or subgroup algorithm is implemented locally.

The public prepared-root API refuses the G2 point in otherwise valid DER. The
existing delegated-key fixture also verifies refusal when a valid root-signed
authority supplies that G2 subnet key. Replacing the outer signature in both
PocketIC frames with the G1 point refuses through canister and selected-subnet
authentication. This exercises both direct and delegated public paths without
weakening their other checks.

The curve crate is now an explicit inherited development dependency of the
Platform owner, with defaults off and only `groups` requested. It was already
selected transitively by production BLS verification; no dependency version,
production feature or normal/build graph changed. Native tests, native/Wasm
Clippy, Rustdoc and graph checks pass through the ordinary authentication target.
Wasm is compilation evidence, not execution of the subgroup tests. This subgroup qualification does not complete the separate consumer or
Agent-remediation obligations in #3 and #4. The later supplier delivery for #1
is recorded below.

## Agent response compatibility and supplier completion — issue #1

Review of Agent 0.49.3's public `read_subnet_state_raw` contract confirmed that
it returns an upstream `Certificate` after discarding encoded response bytes.
The initial encoded-only projection was therefore insufficient for Query.
`authenticate_subnet_time_from_certificate` now accepts an immutable borrow of
that exact upstream type and shares the same verification implementation. It
rechecks signatures, exact subnet/root binding, tree bounds and both time windows;
it never relies on a caller's assertion that verification previously succeeded.

The result retains that immutable certificate borrow. `certificate()` returns it;
`original_bytes()` returns None. The encoded entry point instead returns Some
original bytes and no parsed-certificate reference. No re-encoding or manufactured
original-byte evidence is permitted. For parsed input, the caller/upstream decoder
owns HTTP/body limits and outer CBOR admission: original grammar, duplicate fields
and preallocation byte limits cannot be established retrospectively. Embedded
delegation decoding remains strict and byte-bounded. The read-state response and
projection stay tied through the same borrowed object, without a new transport.

An isolated copy of Query at the source identity recorded in
[Registry qualification](registry-settings.md#source-bound-query-settings-acceptance--2026-11-16)
compiled the actual Agent method's returned type into the new projection. One
focused native test exercised both frozen root/delegated frames and checked exact
object borrowing, time and absence of fabricated original bytes. Clippy for that
test passed with warnings denied. The actual Query checkout stayed read-only.

The temporary consumer added local Platform 0.34.0 as a development dependency
and enabled published Platform's `certification` reexport. Query's thiserror
2.0.21 selection conflicted with the candidate's established exact 2.0.18 pin;
the isolated consumer aligned thiserror/thiserror-impl to 2.0.18. This alignment
is a migration requirement, not an applied sibling change. The resulting test
lock SHA-256 is `aee8bc86ad237657fecfc2afe3a15fe4d28acbcf2eabfb00eeec70e832c6756a`;
test source SHA-256 is `4303db5d02ac8f0ea3172be9e82d45a9cb161f27650526a71e5ee7d5e4f5039c`.
Agent and its advisory-bearing dependencies were already in the consumer; none
was added to the replacement product or its qualification workspace.

The maintained native PocketIC consumer also exercises the borrowed projection
for both fresh certificate kinds. Public regression tests verify wrong subnet,
corrupted signature, stale time and tree-bound refusal for parsed input; a
compile-fail test prevents mutation through retained evidence. The encoded-path
full-width/malformed/nonminimal time tests use the same private time verifier.
This supplies issue #1's projection without implementing Query's two-read sampler,
relaxing advisory #4, or claiming production deployment/mainnet qualification.

## Standalone Wasm authentication execution

`make qualify-platform-wasm` executes the public verifier against the existing
PocketIC-issued direct and delegated fixtures, first natively and then as an
optimized `wasm32-unknown-unknown` binary under Node. The small admission binary
lives in the existing private `platform-certificates` qualification package.
No product code, dependency versions or package versions changed.

Both paths check canister authentication, certified-data lookup, exact original
byte custody, selected-subnet time and parsed-object borrowing. They reject wrong
subnets, altered signatures, stale signing times, exhausted tree-node allowances
and oversized encoded input. The runner rejects any Wasm imports and fails on
assertion traps or a nonzero result. Native and Wasm Clippy also pass with warnings
denied. CI runs this target alongside the certificate authentication gate.

On 2026-11-16, Rust 1.100.0 and Node 24.21.0 / V8 13.6.233.17-node.53 passed;
the executed artifact SHA-256 was
`a9934ea2c1261913c291e55ed4e027d482999d09df52d91b3e265d78a0d4af79`.
This establishes execution of the upstream BLS verifier and Platform composition
on 32-bit Wasm for this corpus. It does not establish IC instruction cost,
installed verifier execution, exhaustive refusal coverage or mainnet acceptance.

## Mainnet delegated certificate replay

The same `qualify-platform-wasm` target now runs `mainnet-admission`, using an
independent read-only observation from the ICP documentation canister. Its
[fixture provenance](../../qualification/crates/platform-certificates/fixtures/README.md)
records the URL, public trust anchor, exact certificate hash and distinct signing
and delegation times. The verifier successfully checks the real delegation,
canister ranges and certified-data presence against the pinned mainnet root.

Encoded and parsed selected-subnet projections return the signing timestamp;
exact separate historical windows check delegation freshness independently.
Wrong root, unauthorized canister, wrong subnet, corrupted outer signature and
swapping either time window to the other certificate's time all refuse. The
parsed projection retains the exact object without claiming original bytes.
The root and expected subnet/times are fixed fixture inputs, not discovered from
arbitrary input during replay. No production dependency or API changed.

On 2026-11-16, both native execution and optimized standalone Wasm execution
passed with native/Wasm Clippy. Rust 1.100.0, Node 24.21.0 and V8
13.6.233.17-node.53 executed Wasm SHA-256
`0410194c5d4592ffd22c45c8b1700a9251ee2305782419ee5993ca71aa8deb9a`.
This is bounded mainnet interoperability evidence for one delegated certificate,
not full network qualification, live freshness, HTTP body/witness verification,
installed verifier execution or production consumer adoption. CI replays the
frozen bytes without network access.

## Mainnet direct-root binding replay

The existing `mainnet-admission` binary now also replays a directly root-signed
NNS frontend observation. The fixture record above links the independent
system-canister reference used to select the root-subnet identity. Both the
general and direct-only canister authenticators succeed; certified-data presence
is checked separately. No delegation or range allowance is needed for this path.

Encoded and borrowed selected-subnet projections succeed only with the explicit
NNS binding supplied by the caller. Missing bindings return `RootSubnetRequired`;
requests for another subnet return `SubnetMismatch`. Corrupted signatures and a
one-nanosecond stale signing window also refuse. Exact byte/object custody and
absence of delegation remain observable. This verifies the no-delegation contract
against a mainnet-issued certificate rather than only simulator or synthetic data.

`make qualify-platform-wasm` passes native and Wasm execution and Clippy for both
mainnet observations. The combined mainnet binary's new Wasm SHA-256 is
`1fd711ff1cf4f007c666ad3bc24c37969f16be53312e9558ec1750964882e1b8`
with Rust 1.100.0, Node 24.21.0 and V8 13.6.233.17-node.53 on 2026-11-16.
The earlier delegated-only artifact hash remains historical evidence. These two
observations still do not establish exhaustive mainnet interoperability, live
freshness, HTTP body authentication, installed verifier costs or consumer adoption.
Product contents and versions remain unchanged; publication remains disabled.
