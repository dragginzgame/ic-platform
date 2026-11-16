# Bounded management snapshots and status

The pending `management-snapshots` Protocol capability supplies the wire boundary
requested by [Backup #11](https://github.com/dragginzgame/ic-platform/issues/11).
Defaults remain empty, Principal is the canonical Types identity, and this
feature selects neither Runtime, Agent, upstream Candid nor management DTOs.
This is a selected finite profile, not a complete management API.

| Management operation | Protocol mapping |
| --- | --- |
| `canister_status` | `CanisterArgs::encode`, `decode_canister_status` |
| `list_canister_snapshots` | `CanisterArgs::encode`, `decode_list_snapshots` |
| `take_canister_snapshot` | `TakeSnapshotArgs::encode`, `decode_take_snapshot` |
| `read_canister_snapshot_metadata` | `SnapshotArgs::encode`, `decode_snapshot_metadata` |
| `read_canister_snapshot_data` | `ReadDataArgs::encode`, `decode_snapshot_data` |
| `upload_canister_snapshot_metadata` | `UploadMetadataArgs::encode`, `decode_snapshot_allocation` |
| `upload_canister_snapshot_data` | `UploadDataArgs::encode`, `decode_empty_response` |
| `load_canister_snapshot` | `LoadSnapshotArgs::encode`, `decode_empty_response` |

Each argument has one declaration for fields, numeric labels, schema and writer.
The shared private writer performs a bounded counting pass before reserving the
complete output, then emits it. No partial request escapes. The static type table
uses fixed stack storage and no runtime schema input. Response declarations
remain decode-only unless that shape is also needed in an argument.

The existing reader retains complete byte/header/type/work/logical-allocation,
vector and skipped-work allowances and the 16-level constructed-type limit.
Selected fields additionally narrow vector ceilings before reserving storage:
ten controllers; 32 module-hash/certified-data/chunk-hash bytes; 256 snapshot-ID
bytes; 1,024 inventory entries/chunk hashes; 4,096 globals; one MiB data chunks.
A field cannot raise the aggregate ceiling; its item-count cap is consumed at
that container rather than incorrectly restricting vectors inside its items.
Consumed work or allocation is never refunded when the field limit is restored.
These are logical accounting limits, not exact process-memory measurements.

Status projects only lifecycle state, controllers and module hash; unknown fields
are validated and skipped without retaining their values. Backup's existing
status bounds map to `Limits` with bytes=1 MiB, header=64 KiB, types=64,
work=2 MiB, allocation=2 MiB, skipping=64 KiB and an explicit vector ceiling.
Metadata uses bytes=1 MiB, header=16 KiB, types=64, work/allocation=2 MiB and
skipping=0. Data replies use bytes=2 MiB, header=16 KiB, types=64, work=8 MiB,
allocation=2 MiB, vector=1 MiB and skipping=0, accommodating a full one-MiB
chunk plus its framing. Callers can further lower every aggregate allowance.

Metadata retains absent optional fields, known null/reserved absence and known
non-option promotion. Unknown present values and variants refuse; they never
become absence. Provenance variants require exactly reserved payloads.
`Float32Bits`/`Float64Bits` preserve every IEEE bit, including NaNs and signed
zero. V128 uses an allocation-free `u128` natural decoder and refuses larger
magnitudes. Backup must migrate its old Nat range check to this earlier typed
refusal; no wrap, truncation or default is permitted. New Rust types and bit-value
Serde are internal API choices, not a migration of persisted consumer bytes.

Snapshot IDs and hashes remain opaque. Nonempty-ID policy, exact digest lengths,
hash comparisons, geometry, the approved 20-MiB workflow, target/source authority,
timers, spending, approvals, journals and retry policy remain with Backup.
The codec does not infer safety to load, replace, uninstall, retry or publish.

## Provenance and focused evidence

Declarations were independently authored from the public management argument and
result interfaces, checked against original registry
`ic-management-canister-types` 0.8.0 and Candid 0.10.40 in the private native
qualification workspace. The lock retains their registry checksums and source
identity. No legacy codec body, generated mapping or crypto algorithm was copied.
Read-only Backup baseline `34e268a866aed8d30ffbdb7d09283d768f36fc65` supplied the
actual finite-profile requirements at `store/capture/status/wire`,
`capture/snapshot`, `capture/transfer/{metadata,data,readback}` and `restore/procedure`.

`make qualify-management-snapshots` checks independent request decoding, exact
and one-byte-short output bounds, all read regions, empty uploads, 20 successive
one-MiB reads/uploads, full-width numbers, float bit patterns, optional promotion,
unknown provenance/global refusals, per-field and aggregate caps, every metadata
truncation, malformed type tables, trailing bytes and empty-result arity.
It also runs strict Clippy/Rustdoc, standalone Wasm replay and a product graph
check. Frozen metadata bytes regenerate only with `REGENERATE_SNAPSHOT_CORPUS=1`
on the `frozen_metadata_corpus_and_empty_result_arity` native test. Ordinary runs
compare those bytes to the original upstream producer.

The focused run passed nine native schema/refusal checks and standalone Wasm with
SHA-256 `9d604477160954bd2a8b6a5407ec67cf5040ff0dde3abfd419c64e003094e8fb`.
All 42 existing Protocol tests and all-feature strict Clippy also passed after the
shared reader changes. These are codec checks, not actual Backup migration,
installed capture/restore, interrupted-transfer, original-result or offline
replay acceptance. #11 remains open for the actual caller/regression qualification
and delivery; #12 independently retains signed-request/recovery acceptance.
