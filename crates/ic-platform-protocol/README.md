# ic-platform-protocol

Bounded Cloud Engine request encoding and selected response decoding, Registry settings and marketplace projections. Decode does not require Encode. This is a selected protocol surface; it does not restore the legacy facade or general textual DID tooling. Production Query cohort migration remains separate.

Defaults are empty. Use one exact package/source cohort at public Types and
Runtime boundaries. This release publishes the implemented scope; it does not
claim completion of the full Platform rebuild or migrate applications.

See the [design and qualification records](https://github.com/dragginzgame/ic-platform/tree/main/docs).

The opt-in `http-request` capability encodes one bounded management HTTP GET
argument with explicit response size/replication and optional raw body. It adds
no dependencies and selects no network or cycle policy.

`registry` supplies bounded protobuf envelopes and borrowed record projections;
`registry-placement` supplies target argument encoding and bounded placement
results. Presence, source errors and resource refusals remain explicit. These
codecs perform no transport or authentication. Independent schemas and actual
Query qualification cover the current local package cohort.

`raw-rand` provides allocation-free canonical management seed framing, with an
explicit complete-message bound. `management-chunks` provides bounded
`stored_chunks`/`upload_chunk` request and response codecs using canonical Types.
The HTTP GET encoder also offers explicit pricing version one. Consumer custody,
cache authority, transport and cycle-spending decisions remain outside Protocol.
