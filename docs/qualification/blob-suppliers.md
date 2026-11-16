# Blob signed transport and Caffeine codecs

This implements the supplier surfaces requested by [Platform #17](https://github.com/dragginzgame/ic-platform/issues/17)
and [Platform #18](https://github.com/dragginzgame/ic-platform/issues/18). The changes are
local candidates, not published releases or a completed Blob hard cut.

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

## Remaining gates

The full certificate issuance/CLI custody and management CLI process journeys are
not yet migrated or admitted by these targets. In particular, the native supplier
proof does not replace fresh-process interrupted-dispatch/transfer-custody evidence.
The enrollment example's legacy commercial outcome decoder is not a newly supplied
settlement contract. Those consumer migrations and final qualification keep both
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
