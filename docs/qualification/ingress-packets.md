# Original ingress packet boundary

Platform 0.34.6's native-only `ingress-packets` capability supplies strict packet
admission for [Backup #12](https://github.com/dragginzgame/ic-platform/issues/12).
It does not create an Agent, HTTP client, signer, clock, retry loop or submission
path. A decoded packet carries both owned fields and a borrowed view of the exact
original bytes, including a self-describing tag when present.

Call and read-state content are distinct. Canonical local Types owns all public
principal fields. Only CBOR byte strings are accepted for opaque fields and path
labels; strings or integer arrays are not byte-string substitutes. Public decode
functions require maps, reject duplicate/unknown fields and trailing data, and
retain the parser's recursion ceiling. Optional key, signature, nonce, sender-info,
delegation, target and permission presence is explicit: present-empty differs
from absent, and explicit null is refused. Delegation ordering is retained.

The caller supplies an inclusive complete-message byte limit, checked before
parsing. Backup's profile selects 2 MiB. This is not a logical-allocation, process
heap or traversal-work budget. No zero-copy claim is made for decoded fields.
Serde's own sequence allocation behavior remains relevant within admitted input.

Request IDs and domain-separated signable bytes call the original
`ic-transport-types` 0.49.3 implementation. Provenance and its Apache-2.0 license
are retained under the existing [native reuse boundary](query-hard-cut.md), with
exact source/checksum in Cargo.lock. No request-ID hash or cryptographic body was
copied or reimplemented. Serde serialization of selected fields is independently
compared with upstream call/read-state content, including sender-info, nonce and
nested label bytes. The feature selects no Agent, reqwest, executor or signing
implementation. Its upstream transport-types dependency retains its native
transitive schema dependencies; these do not cross public identity boundaries.

Synchronous responses distinguish accepted/pending, unreplicated rejection and
opaque replied certificate bytes. Read-state responses likewise preserve opaque
certificate bytes. Rejection codes remain raw numbers for caller policy.
Decoding produces no authenticated evidence. Backup must bind the original
request ID, sender, target, method, arguments and expiry; apply direct-key and
path policy; and authenticate certificates using the original trusted root,
target and explicit time window. A pending response authorizes no resubmission.

`make qualify-ingress-packets` checks strict Clippy/docs, upstream-produced
requests and outcomes, exact request IDs/signable bytes, original-byte retention,
empty/absent distinctions, truncation, duplicate/unknown fields, wrong byte kinds,
oversized input and nested-map refusals. The capability is native-only and is not
a standalone Wasm selection. The combined all-feature Wasm check retains the
existing portable certificate/crypto surface; it does not execute ingress packets.

Actual Backup source adaptation, interruption/offline recovery replay and registry
qualification remain open. The associated snapshot/status codec gap (#11) also
remains independent. Supplier packet tests do not qualify Backup's spending,
authentication, recovery or exact-once policy, and do not close #12.

## Bounded offline construction — 0.34.9 candidate

The additive native-only `encode_envelope(&Envelope, maximum_bytes)` API supplies
Backup's externally signed Call and ReadState construction requirement. It emits
one complete CBOR envelope without a self-describing tag. The returned bytes are
new evidence for the caller to retain before dispatch; never re-encode a retained
request in place. Construction does not authenticate signatures, bind a sender,
validate expiry/paths or authorize a submission. Invalid signatures and explicit
empty delegation lists remain constructible for consumer rejection fixtures.

A shared private serializer counts the complete output under the inclusive byte
ceiling before reserving storage, then writes to a fixed-size slice. Native
Ed25519 signing and signed queries use the same implementation. Public errors
separate byte refusal, allocation and serialization failure. This is an output
bound, not a claim about total process memory or an independent traversal quota.
No generic serializer is made public and no new supplier is selected.

Eight ingress tests, five signing tests and three HTTP/fresh-process recovery
cases pass, alongside Clippy, Rustdoc, the native dependency boundary and combined
Wasm compilation. Independent upstream envelopes cover Call/one-path ReadState,
absent/empty/nonempty delegation, invalid signatures, request IDs/signable bytes,
nonce and sender-info presence. A full 2 MiB encoded envelope passes at its exact
ceiling and refuses one byte below. Logs: `target/encoder-qualification.log`,
`target/encoder-signing-regression.log`, `target/encoder-external-regression.log`.
Platform 0.34.9 is selected for publication. Actual Backup call/lookup, interrupted
allocation and offline spending-preservation acceptance remain required by #12.
