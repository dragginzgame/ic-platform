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
