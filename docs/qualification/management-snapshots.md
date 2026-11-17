# Bounded management snapshots and status

Protocol 0.3.0 updates this boundary: management requests use the complete
thirteen-field CanisterSettings, CMC keeps CreationSettings, and status now
requires version. Ordinary creation and chunked installation are also supplied.
See [current migration and retained-byte evidence](canic-management.md).
The earlier release scope and observations below remain historical evidence.

The `management-snapshots` Protocol capability supplies the wire boundary
requested by [Platform #11](https://github.com/dragginzgame/ic-platform/issues/11).
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

`CaptureStatus` projects only lifecycle state, controllers and module hash; unknown fields
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

## Response fixtures and minimal status

Protocol 0.3.1 adds `CaptureStatus` and `decode_capture_status` for Backup's
minimal preconditions. Missing version is accepted here; a present version is
validated and skipped under the caller's separate skip allowance. This reader
never supplies a synthetic version. `CanisterStatus`/`decode_canister_status`
continues to require a typed `u64` version for upgrade-proof consumers. Both
readers preserve controller/hash caps, complete framing and bounded unknown-field
traversal. Protocol 0.3.3 additionally preserves known bare `vec nat8` promotion
for capture hashes, matching Backup's retained precondition reader. Null,
reserved, absent fields and empty options remain absent; incompatible present
values refuse, including empty vectors whose element type is not `nat8`.
The upgrade-proof reader keeps its strict optional hash contract unchanged.
Capture decoding retains original bytes up to the 32-byte cap; exact hash length,
approved hash comparison and actual controller admission remain Backup policy.

Select `management-snapshot-fixtures` only where response construction is needed.
It enables `management_snapshots::fixtures::{canister_status, capture_status,
list_snapshots, take_snapshot, snapshot_metadata, snapshot_data,
snapshot_allocation, empty_response}`. Pass the corresponding public value and
an inclusive maximum output byte count (only the count for `empty_response`).
The shared two-pass writer counts complete framing before reserving output.
Schemas remain private and fixed; no general codec, runtime schema or transport
is exposed. Defaults stay empty and decode-only selection needs no encoder.

Fixture construction intentionally does not apply the decoder's policy caps:
empty/reused/wrong IDs, unexpected controllers/hashes/provenance, oversized
collections and one-MiB-plus-one data can be emitted within the selected output
budget. Numeric extremes and exact global float bits are preserved. Callers own
the memory of values they construct; the encoder bounds its output allocation.
No fixture is authenticated or safe to restore merely because it encodes.

These typed encoders do not construct malformed schemas. Keep independent
literal goldens for malformed/truncated/trailing/duplicate or unknown labels,
and native qualification-only independently authored Candid schemas for omitted,
promoted, incompatible and extra fields. This is the approach used by
`qualification/engine-view-consumer/tests/management_snapshots.rs`, including
known optional/global/provenance and bounded skipping refusals. Do not replace
those tests with encoder/decoder round trips or introduce a legacy product
fallback. Existing independent metadata fixtures remain unchanged.

Backup caller mapping: use `fixtures::capture_status` for minimal status reply
construction and `decode_capture_status` for its production precondition reader;
use `fixtures::canister_status` only for a reply with an explicit version. The
other fixture functions use the existing snapshot response structs unchanged.
For example, `fixtures::snapshot_allocation(&SnapshotAllocation { snapshot_id:
vec![] }, maximum_bytes)` intentionally emits an empty ID for consumer policy
rejection. A consumer requiring no fixture encoder in its product dependency
closure can select the fixture feature on its test dependency only.

The three `backup-status-*.bin` fixtures retain only original wire bytes from
Backup's `store/capture/status/wire/golden.json` at
`2941ca1324a95c1afdb57e33cef79362c28b0731`, supplied in the issue report. Their
checksums match that source; no Backup implementation or expressive test code
is copied. See the fixture provenance record. Native and Wasm replay require
all three to pass capture projection and fail required-version projection.
Actual Backup caller migration and installed capture/recovery remain downstream
acceptance; this supplier change does not edit or qualify a sibling checkout.

Focused validation on 2026-11-17: `make qualify-management-snapshots` passed
12 native schema/refusal tests, strict Clippy with and without fixture encoding,
Rustdoc, standalone Wasm replay and both production dependency graph checks.
All 42 existing Protocol unit tests also passed with all features enabled.
Wasm SHA-256: `2a6076e9a6329809e47001779ad2d7b64f68d1c8c509785ad154d14dac01f1f6`.
The registry baseline checked for version selection was Protocol 0.3.0; 0.3.1
was selected as a compatible patch. It is now published; see the
[release and registry evidence](../releases.md#backup-fixture-and-projection-release).

The Protocol 0.3.3 capture-hash regression uses independently authored native
Candid declarations for bare/optional bytes, null/reserved/empty options and
incompatible present values. It checks empty/short/exact/oversized hashes,
every truncated prefix, trailing bytes and all resource bounds using Backup's
reported limits. Its frozen promoted-hash frame is also replayed on Wasm;
required-version status continues to refuse that same bare-hash frame even with
a valid version present. This resolves the supplier acceptance incompatibility
reported in [Platform #11](https://github.com/dragginzgame/ic-platform/issues/11);
actual Backup migration and installed recovery remain consumer-owned.

On 2026-11-17, `make qualify-management-snapshots` passed all 13 native cases,
strict Clippy/Rustdoc, both dependency graph checks and Wasm replay (SHA-256
`5301cfabece7eeeddb0371220734365674e8188e4b1d99243103c9a3a6a0e8ce`).
All 42 Protocol unit tests and all-feature strict Clippy passed as well.
Protocol 0.3.3 remains a local, unpublished candidate.
