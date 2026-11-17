# Blob signed transport and Caffeine codecs

This implements the supplier surfaces requested by [Platform #17](https://github.com/dragginzgame/ic-platform/issues/17)
and [Platform #18](https://github.com/dragginzgame/ic-platform/issues/18). The original supplier slice is published in Platform 0.34.7 and Protocol 0.2.6;
this is not a completed Blob hard cut. The external-signer extension below is
a local Platform 0.34.8 candidate.

## Ownership and public contracts

Platform's opt-in native-only `native-signed` feature extends `native-read`.
`Ed25519Identity::from_pem` accepts explicit bounded PKCS#8 Ed25519 PEM offline.
It supplies canonical self-authenticating Types identities. `prepare_update`
returns immutable exact envelope bytes, request ID, original expiry, signed
recipient and separately selected effective routing target. `from_bytes` verifies
the original direct Ed25519 signature and sender binding without normalizing the
bytes or rejecting an expired request needed for recovery. Delegated signing keys
and sender-info envelopes are outside this selected signer contract.

`dispatch` sends those bytes once, without redirects, proxies or retries (including
429/503). Errors leave delivery uncertain. Accepted, uncertified boundary rejection
and certificate-authenticated observation remain distinct. `observe_request`
signs only a fresh read-state request for the original ID; it cannot renew or
resubmit the update. It makes one observation, with no implicit polling. The
caller owns the whole-operation deadline in addition to HTTP request timeouts.

Request observations retain exact original proof bytes and immutable ID, effective
target, certified time and status. Received, processing, reply, certified rejection,
done, certified absence and pruned/unavailable evidence remain distinct. Absent or
unavailable evidence never establishes nonexecution. Offline `authenticate_request`
requires an explicit historical certificate/delegation time policy; current-network
freshness is not inferred from historical proof. Replies must still pass Blob's
method/root and application checks. Signed queries reuse certified subnet/node-key
authority and verify every response signature against the exact query ID. Their
results are node-authenticated observations, not consensus-certified application
results. The module-hash read supplies a separate authenticated observation, not
atomic continuity or upgrade authority.

Protocol's opt-in `caffeine` feature selects exactly root text, plain certificate
record, `vec blob`, `vec bool`, empty acknowledgement, Cashier account request/result
and top-up arguments. The record declaration owns field/variant IDs and decoding.
The private encoder admits complete output in a counting pass before allocation.
Decode does not require Encode. `Limits` explicitly bounds complete bytes, header,
constructed types, work, logical allocation, vectors and discarded work. Known
Cashier errors with reserved projections validate and discard their payloads under
those same budgets; unknown selected variants refuse. Signed balances and natural
targets use maintained upstream magnitude storage.

The actual enrollment caller at the pinned Blob revision wraps the top-up record
in an **outer option**. `encode_top_up(Option<&TopUpArgs>, limit)` preserves that
form and both inner options. `TopUpArgs::encode` separately emits the bare record
specified in the issue. No reply decoder or settlement meaning is invented for
an enrollment attempt. Blob still checks 32-byte roots, root counts, canonical
root text, method/root equality and returned Cashier owner before effects.

## Provenance and dependencies

No crypto implementation or Agent code is copied. Signing/PEM uses original
`ed25519-dalek 3.0.0` (BSD-3-Clause), enabling its `pem` and `zeroize` features;
request hashing uses original `ic-transport-types 0.49.3` (Apache-2.0). PEM adds
`pkcs8 0.11.0`, `pem-rfc7468 1.0.0` and `base64ct 1.8.3` from the maintained
RustCrypto implementation (Apache-2.0 OR MIT). The existing HTTP and certificate
owners retain their pinned dependencies and verification checks. This explicit
Ed25519 path does not resolve Platform #4's broader legacy Agent identity dependency graph.
`num-bigint 0.5.1` (MIT OR Apache-2.0) supplies ordinary arbitrary integer magnitude
conversion, with scratch and magnitude storage charged before conversion.
Root and qualification lockfiles bind the complete selection.

Product defaults remain empty. Platform does not select Protocol or Runtime;
Protocol selects neither Runtime nor native transport. Both expose the local
canonical Types source. Native Candid schemas, WAT and simulator dependencies stay
in qualification. No unsafe code or cryptographic implementation is introduced.

## Reproduction and evidence

- `make qualify-caffeine`: independent native Candid schemas and literal frames,
  large signed/natural values, exact/short bounds, all truncations, arity, unknown
  variants, malformed discarded payloads and frozen standalone Wasm replay;
  strict Clippy, Rustdoc and production graph admission.
- `make qualify-native-signed`: offline signing and signature refusals, exact
  retained envelope/proof bytes, direct/delegated authority, wrong target/root,
  mismatched request ID/reply, independent time windows and expired originals;
  HTTP lost-response/429/503/redirect/oversize tests; installed direct/delegated
  signed update/recovery, signed query, module hash and effective-routed management
  status on the authenticated local PocketIC server. Strict Clippy and Rustdoc.
- `make qualify-blob-suppliers`: pinned Blob source
  `31dd236814879afd2fd0faf0851e6b46184476ac`, materialized under `target/` only.
  Durable adapters in `qualification/blob-supplier/` migrate the actual signed
  caller and callback/enrollment probes without sibling writes. The real signed
  application/uploader journey retains authority, lost-reply and final-release
  assertions. Two callback-core journeys retain acknowledgements, no-write
  malformed/over-budget refusal and no settlement inference. Four enrollment
  journeys retain one-attempt custody, revocation, held/late replies, opaque
  malformed/oversized observations, same-format stop/upgrade/start and refusal of
  incompatible/foreign persisted state. The selected native test graph rejects
  old Agent/Platform fallback. `target/blob-supplier/verified.json` binds source,
  adapter, lock and artifact hashes after all checks pass.

The Blob source snapshot and retained tests remain under its Toko Studios Network
Licence v1.0. They are disposable consumer qualification inputs, not copied product
implementations. New synthetic reply fixtures use native WAT only. Native Testkit
held fixtures retain their registry Types/Protocol behind an explicit simulator
identity conversion; those are not public product type crossings.

## Gates at the initial supplier qualification

The full certificate issuance/CLI custody and management CLI process journeys are
not yet migrated or admitted by these targets. In particular, the native supplier
proof does not replace fresh-process interrupted-dispatch/transfer-custody evidence.
The enrollment example's legacy commercial outcome decoder is not a newly supplied
settlement contract. At that initial qualification, those consumer migrations kept both
issues open; passing selected supplier and installed probe tests does not close
the complete acceptance checklist.

Blob's Storage 0.74.1 still imports Runtime 0.22.5 raw IC APIs on Wasm. The fixture
keeps that existing storage boundary visible; it does not claim the whole Blob
consumer has completed its hard cut or qualified a second executor. No sibling
checkout, registry publication, provider operation or production state is changed.

## Local result, 16 November 2026

All six independent Caffeine tests and standalone Wasm replay passed. The Wasm
replay digest is `986f7a0c3674f37801d8ec66ad5fc0fb2ca92dde370cdbb6de24448af9073b12`.
The native signing/authority tests, two HTTP fault tests and installed transport
probe passed. The seven selected actual Blob journeys passed with strict native
and Wasm Clippy. The shared Protocol all-feature regression suite passed 42 tests.

Root and qualification lock audits using RustSec revision
`02e2517f457a359fc6d3440ea4560c8ea188fee8` reported zero known vulnerabilities.
The existing unmaintained notices for `paste` and `serde_cbor` remain; this check
introduces no exemption or claim that dependency maintenance is complete.

## Externally signed status requests — Platform 0.34.8 candidate

`PreparedRequestStatus::from_bytes(bytes, &update, maximum)` admits one exact
`request_status/<original-id>` read-state path, the same sender, a valid direct
signature and no delegation. It preserves original CBOR framing and read expiry.
`ReadClient::observe_prepared_request` checks the request/sender/effective-target
binding again, sends those exact bytes once and authenticates the returned
certificate through the existing root/range/time verifier. It takes no identity
or private key. A caller may explicitly admit a newly authorized read, but the
library never renews a read or update, polls, retries or resubmits an update.
The consumer still admits the planned target/method/argument bytes and network.

Direct Ed25519 remains under `native-signed`. The additive
`native-signed-secp256k1` feature admits the strict uncompressed secp256k1 SPKI
and compact ECDSA signature profile through original K256 0.14.0 (MIT/Apache-2.0),
ECDSA 0.17.0, elliptic-curve 0.14.3 and crypto-bigint 0.7.6. No Agent or algorithm
implementation is copied. P-256 and delegated signatures are unsupported and
refuse; this does not resolve Platform #4's retained Agent/derivation graph.

`make qualify-native-signed` passes with actual official ICP CLI 1.7.0
Ed25519/secp256k1 sign-only outputs, a fresh-process retained-status recovery,
exact HTTP bytes, sender/path/route/signature/bounds refusals, direct/delegated
certificates and the existing installed signed update/query/management cases.
Fixtures and generation provenance are in
`qualification/platform-certificates/fixtures/icp-signing/`.
Logs: `target/external-status-tests.log`, `target/external-status-qualification.log`.
Neither signing nor the recovery fixture contacts a live provider or canister.
Platform 0.34.8 is published; see [delivery evidence](../releases.md#external-signing-and-managementledger-release). Full Canic journal
integration and Blob fresh-CLI stop/upgrade/start/custody journeys remain unproved.

## Published Blob consumer acceptance

Read-only review now finds clean Blob source `9c26943`, documenting published
0.103.0 at `75f08fddf6b5f3f6516dded561fa72eb88da5e33`. Its migration commit
`1032653bc14019ff3e5c31520b509f4dcb7bae5b` selects Platform 0.34.8 and product
Protocol 0.2.7; the directly affected CLI/test source is unchanged by the release
commit. Qualification uses Protocol 0.2.6 because Testkit 0.107.3 pins it; that
version already supplies the same selected Caffeine formats. No legacy Platform
remains in Blob's product graph. Remaining private malformed-input oracles,
metrics and opaque top-up outcome decoding are outside these selected supplier
formats. No broader legacy-removal claim is made.

Reviewed original logs under Blob's `target/platform-testkit-adoption/`:

| Evidence | Result / SHA-256 |
| --- | --- |
| `ci-final.log` | 13 CLI cases, including certificate root/reply, expired-original binding and independent Cashier literals; `05b73a4c2ee27ac26bdc03571d58f797e1256da5d2554aef173792f1579db04f` |
| `custody-final.log` | All seven non-browser callbacks pass, including old CLI custody recovery after expiry, transfer/cancellation and no reissuance; `328b2c9cb90ae558c8025839ecf50905b455ef3e602487b02ae681cb0fef9a01` |
| `installed-remaining.log` | Actual signed-uploader authority/no-repeat case passes; `8f17040124073866db545d64fe8671c05033eadc2acbd06db6d9bfe22a1062d4` |
| `management-final.log` | Three lifecycle/custody/overview cases pass; target case failed and was rerun separately; `391508ca46cde25b0074051310fac059664c0169eecd033e5edb995c3bea69af` |
| `target-final.log` | Target/controller/absent-module case passes with strict Clippy; `393329dbbf87b7fe72156a479ade0e5813c08e0e5ce229f00e3257f2c824b623` |

The retained aggregate logs include earlier failures; they are not represented as
all-green runs. The final targeted reruns and source review supply the corrected
assertions and completed cases. `installed-final.log` supplies four enrollment
cases covering one attempt, revocation, lost/late/malformed replies and upgrades.
The browser case remains explicitly ignored and no live provider qualification is
inferred. These results close the remaining Blob acceptance for Platform #17 and
#18. The published external-signing supplier fixtures separately cover Canic's
specified ICP envelopes; full Canic application delivery remains its own work.
Backup's snapshot/recovery obligations in #11/#12 are not closed by Blob evidence.
No sibling checkout was modified or tests executed there during this review.

## Secp256k1 signed queries — issue #28

The compatible published Platform 0.34.14 slice extends the existing
`native-signed-secp256k1` feature with `Secp256k1Identity::from_pem` and
`ReadClient::signed_query_secp256k1`. Input PEM bytes have an inclusive caller
limit; upstream K256 accepts SEC1 and PKCS#8, validates curve/key material and
owns secret-key storage. K256 0.14.0 and elliptic-curve 0.14.3 are unchanged
registry suppliers (Apache-2.0 OR MIT); only their upstream PEM feature is added.
The existing crypto-bigint 0.7.11 security floor remains selected. No cryptographic
algorithm or Agent implementation is copied.

Identity uses the IC uncompressed secp256k1 SPKI representation and canonical
self-authenticating Principal. Signing uses K256’s SHA-256 ECDSA operation on the
original request-ID signable bytes, emitting the fixed-width signature. Ed25519
and secp256k1 queries share one private request/envelope builder and the existing
`query_content` authentication owner. Both retain explicit expiry, configured
HTTP timeout and message limits, certificate/delegation time windows, node-key
and routing authority, and request-bound response signatures. There is no
anonymous fallback. Dropping the future retains the existing cancellation behavior.

Consumer composition:

```rust,ignore
let identity = Secp256k1Identity::from_pem(&pem, 4096)?;
let reply = tokio::time::timeout(
    whole_operation_timeout,
    client.signed_query_secp256k1(
        &identity, canister, method, &arguments, expiry_ns, time_policy,
    ),
).await??;
```

Enable `native-signed-secp256k1`; import both types from `ic_platform::native`.
The consumer still admits the endpoint, explicit trust root/root-subnet binding,
application authorization and whole-operation deadline. Mainnet trust and explicit
loopback-only development bootstrap remain IC DB decisions. Supplier tests do not
claim IC DB/Canic migration, application authorization or unchanged backing proof.

Unit coverage checks SEC1/PKCS#8 principal equivalence, exact/over PEM bounds and
malformed/wrong-algorithm refusal, alongside existing request/response signature,
authority, freshness and resource regressions. The installed signed fixture adds
secp256k1 echo and caller-identity queries on both direct-root and delegated
subnets; the returned caller must equal the explicit key principal and differ
from anonymous. Reproduce with the existing `qualify-native-signed` target.
Platform 0.34.14 is published with [archive verification](../releases.md#exact-offline-signing-and-secp256k1-query-release).
Actual IC DB adoption remains separate from this supplier slice.

Local Linux checks passed: 11 native unit tests (including PEM admission and
shared response-authentication refusals), five HTTP transport/recovery regressions,
product all-feature and installed-fixture Clippy, native-signed-secp256k1 Rustdoc,
and formatting. Installed direct/delegated echo and exact caller checks passed;
log: `target/issue28-installed.log`. The HTTP fixture required loopback access;
its initial sandbox-denied attempt is not a product failure. These checks use
local source with the pre-existing supplier lock updates retained, not a published
0.34.14 archive or a new macOS run.

## Exact offline signer composition — issue #29

Both explicit identity types now provide:

- `prepare_update(UpdateArgs, maximum_bytes) -> PreparedUpdate`.
- `prepare_request_status(request_id, effective, expiry_ns, maximum_bytes) -> PreparedRequestStatus`.

Use `native-signed` for Ed25519 PKCS#8 PEM and `native-signed-secp256k1` for
secp256k1 SEC1/PKCS#8 PEM. No other identity algorithm, encrypted identity format,
delegation chain, key discovery or credential prompt is implemented. Tooling owns
provider selection/unlocking and must preserve the operator’s actual principal;
Canic and Backup retain approval, spending, dispatch and recovery policies.

```rust,ignore
let identity = Secp256k1Identity::from_pem(&explicit_pem, pem_byte_limit)?;
let update = identity.prepare_update(update_args, envelope_byte_limit)?;
// Persist/return exact bytes and public binding metadata for caller admission.
let bytes = update.envelope_bytes();
let status = identity.prepare_request_status(
    *update.request_id(), update.effective_canister(), exact_status_expiry_ns,
    envelope_byte_limit,
)?;
let read_bytes = status.envelope_bytes();
```

Status preparation also works from an original 32-byte request ID without the
original envelope. The caller admits that ID and effective routing target.
`PreparedRequestStatus` exposes `request_id`, `sender`, `effective_canister`,
`expiry` and `envelope_bytes`. Its envelope contains exactly
`[[b"request_status", original_request_id]]`; it adds no `time` path.
The effective target is routing metadata, not part of signed read-state content.
Preparation preserves absolute nanosecond expiry without consulting a clock,
rounding, network operations, renewal or dispatch permission. Complete output is
bounded by the existing counting serializer before allocation. Update fields
retain their pre-copy aggregate bound, recipient/effective distinction and nonce
absence versus empty presence. The existing strict signature and packet admission
owners verify the constructed bytes; no separate CBOR or cryptography is added.

Ed25519’s existing `prepare_update` and HTTP `observe_request` remain compatible;
the latter retains its existing two-path query including `time`. The new offline
status operation is deliberately exact-path. `observe_prepared_request` rechecks
original-update sender/ID/routing bindings and submits only the separately admitted
read. Status admission cannot establish that an update was approved or executed.

Focused tests cover both algorithms, exact non-minute-aligned update/read expiry,
nonce and method/argument fidelity, single-path reads, signature admission,
wrong key/request binding, malformed bytes and exact/one-under envelope ceilings.
Existing status refusal tests retain extra-path and signature coverage. Installed
qualification separately dispatches each algorithm’s update, waits beyond its
original expiry, then prepares and submits only a new exact-path status envelope;
retained update bytes remain unchanged. Both direct-root and delegated subnets
must return the original result. This is supplier proof, not completion of Tooling’s
provider lifecycle or Canic/Backup adoption. Publication remains separate.

Local Linux validation passed 13 native unit cases, strict all-feature and
Ed25519-only Clippy, installed-fixture Clippy, Rustdoc and formatting. Installed
post-expiry original-result lookup passed for both algorithms on both subnet
profiles; retained log: `target/issue29-installed.log`. No new macOS execution,
registry publication, encrypted-provider access or downstream adoption is claimed.


### Independent status-envelope admission — Platform #29 follow-up

Platform 0.34.14 published standalone signing but left parent-side admission
coupled to retained update bytes. The compatible 0.34.15 candidate adds
`PreparedRequestStatus::from_signed_bytes(bytes, original_request_id, expected_sender, effective, maximum_bytes)`.
It delegates to the existing direct-signature binding verifier; no new crypto,
serializer, transport, key discovery or provider lifecycle is introduced.
The parent supplies approved public metadata and the returned envelope only:

```rust,ignore
let status = PreparedRequestStatus::from_signed_bytes(
    &returned_bytes, approved_request_id, approved_sender,
    approved_effective_canister, envelope_byte_limit,
)?;
if status.expiry() != approved_expiry_ns {
    return Err(ExpiryMismatch);
}
```

Admission verifies direct signature/key-to-sender binding and exactly one
`request_status/<original ID>` path. It preserves bytes and authenticated expiry.
A signed expiry differing from approval is valid cryptography but must be refused
by the parent. Effective routing is not signed read-state content: the caller
supplies its approved target. Admission grants neither freshness nor dispatch
permission. Anonymous/delegated profiles cannot fall back through this API.

Independent test envelopes use upstream Ed25519/K256 signing and Serde CBOR,
without the Platform preparation methods or an original update. Both algorithms
exercise exact/over limits, wrong key/ID/sender/path, extra paths, malformed or
missing signatures, tampered expiry, valid-but-unapproved expiry, delegation,
truncation and trailing bytes. The external installed fixture independently
readmits each child-produced envelope before direct-root/delegated original-result
lookup after update expiry, preserving the original update bytes.

Publication of this patch and Tooling's external signer remain separate gates;
this local change does not claim Backup or Canic production integration.

Focused Linux validation passed: all 15 native tests, five HTTP transport/recovery
regressions, native-signed and all-feature strict Clippy, Rustdoc and formatting.
Installed direct-root and delegated journeys passed for both algorithms, including
independent public admission and post-expiry lookup; anonymous transport regression
also passed. Logs: `target/platform29-independent-tests.log`,
`target/platform29-independent-qualification.log` and `target/platform29-clippy.log`.
The pre-existing qualification lock refresh to registry Platform 0.34.14 was
preserved; local candidate entries advance to 0.34.15. These are source-level
checks, not archive publication or new macOS evidence.

Platform 0.34.15 subsequently passed the complete clean-source catalog and was
published with independent archive checksum/VCS verification; see the
[release record](../releases.md#independent-status-admission-and-qualification-flow-release).
The parent-side admission API is now available from crates.io. External signer
publication and actual Tooling/Canic/Backup adoption remain separate.
