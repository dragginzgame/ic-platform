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

`management-snapshots` supplies bounded canister-status and snapshot
list/take/read/upload/load messages for Backup's finite profile. It preserves
exact global float bits and applies per-field collection limits before allocation.
Callers still own identity/hash admission, capture/restore geometry, spending and
recovery; passing codec checks does not qualify those application workflows.

`decode_capture_status` selects lifecycle state, controllers and module hash
without requiring a version. `decode_canister_status` retains required-version
admission for upgrade proofs. The opt-in `management-snapshot-fixtures` feature
exposes narrow bounded response encoders under `management_snapshots::fixtures`.
These permit deliberately invalid policy values for rejection tests and confer
no admission or authentication. See the
[fixture and malformed-shape contract](../../docs/qualification/management-snapshots.md#response-fixtures-and-minimal-status).

`caffeine` supplies selected bounded callback, certificate-record and Cashier
formats, arbitrary signed balances and optional natural top-up targets. The
enrollment outer option has explicit encoding and decoding. Top-up results
validate and discard arbitrary success payloads, or return one of four typed
errors. Application admission and
settlement stay with the caller; see the
[Blob supplier contract](../../docs/qualification/blob-suppliers.md).

`sns-neurons` supplies bounded SNS get/list request encoders and selected raw
neuron/error decoders. IDs, permission codes, stake and dissolve fields remain
consumer evidence. Independently, `ledger` supplies zero-argument ICRC display
metadata calls and bounded symbol/decimals decoding alongside canonical Account
and arbitrary-Nat balance support. See the
[SNS and ICRC supplier contract](../../docs/qualification/sns-neurons.md).

The `ledger::top_up` module encodes exact-fee, timestamped ICP transfers with
eight-byte memos and CMC notifications of the original payment block. Bounded
decoders distinguish duplicates, refunds, other reported refusals and unknown
variants. See the [top-up codec contract](../../docs/qualification/management-ledger.md#icp-and-cmc-top-up).

With `management`, `decode_canister_resources` reads the required natural-valued
`cycles` and `memory_metrics.snapshots_size` from a bounded `canister_status`
response. Callers can check controllers and module evidence on the same bytes.

Protocol 0.3 uses `management::CanisterSettings` for management creation and
settings updates; `CreationSettings` remains the distinct CMC schema. The
`management` feature also encodes ordinary creation and chunked installs, while
the shared status projection requires `version`. See the
[retained-byte migration contract](../../docs/qualification/canic-management.md).

`management::decode_canister_metrics` projects
`cycles_consumed.instructions` as an arbitrary natural execution-cycle charge,
not an instruction count or balance difference. Conversion and interpretation
remain caller-owned. These and the top-up decoders enforce explicit byte, header,
type, work, allocation, vector and skipping budgets. See
[Blob codec qualification](../../docs/qualification/anonymous-and-blob-codecs.md).
