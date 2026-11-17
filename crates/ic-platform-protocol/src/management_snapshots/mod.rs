//! Selected management snapshot/status replies for Backup's finite profile.
//!
//! Per-field collection ceilings apply before allocation in addition to the
//! caller's aggregate [`Limits`]. Transport, snapshot ownership, lifecycle,
//! spending, hashes and capture/restore admission remain caller-owned.
pub use crate::candid::encode::EncodeError;
use crate::candid::encode::{Encode, Writer};
use crate::candid::{
    self, Decode, Error, Limits, Reader, Schema,
    projection::{absent, projection},
};
use candidate_types::Principal;
use serde::{Deserialize, Serialize};

projection! {
    /// Required canister record for status and snapshot inventory requests.
    encoding CanisterArgs {
        /// Exact management target, using canonical Types identity.
        1_313_628_723 => canister_id: Principal,
    }
}
projection! {
    /// `take_canister_snapshot` arguments with explicit optional presence.
    encoding TakeSnapshotArgs {
        /// Optional snapshot to replace; absence never selects a replacement.
        435_757_295 => replace_snapshot: Option<Vec<u8>>,
        /// Exact canister target.
        1_313_628_723 => canister_id: Principal,
        /// Optional uninstall choice.
        2_590_416_362 => uninstall_code: Option<bool>,
        /// Optional sender-version check.
        3_599_109_322 => sender_canister_version: Option<u64>,
    }
}
projection! {
    /// Exact target and snapshot for `read_canister_snapshot_metadata`.
    encoding SnapshotArgs {
        /// Exact canister target.
        1_313_628_723 => canister_id: Principal,
        /// Exact opaque snapshot identity.
        4_045_790_262 => snapshot_id: Vec<u8>,
    }
}
projection! {
    /// `load_canister_snapshot`; execution and spending are caller-owned.
    encoding LoadSnapshotArgs {
        /// Exact canister target.
        1_313_628_723 => canister_id: Principal,
        /// Optional sender-version check.
        3_599_109_322 => sender_canister_version: Option<u64>,
        /// Exact opaque snapshot identity.
        4_045_790_262 => snapshot_id: Vec<u8>,
    }
}

projection! {
    /// Reported lifecycle state; this observation grants no upgrade authority.
    both ("management-snapshot-fixtures") enum Status {
        /// Canister is stopped.
        1_130_484_237 => Stopped,
        /// Canister is draining outstanding calls.
        2_990_082_932 => Stopping,
        /// Canister is running.
        3_949_555_199 => Running,
    }
}
projection! {
    /// Selected settings; unselected settings are validated and skipped.
    both ("management-snapshot-fixtures") Settings {
        /// At most ten controller identities, in the returned order.
        570_880_087 => controllers: Vec<Principal> [required, max = 10],
    }
}
projection! {
    /// Status projection that does not allocate discarded management fields.
    both ("management-snapshot-fixtures") CanisterStatus {
        /// Reported lifecycle state.
        100_394_802 => status: Status [required],
        /// Raw observed canister version, required for upgrade proofs.
        2096228120 => version: u64 [required],
        /// Selected controller settings.
        2_336_062_691 => settings: Settings [required],
        /// Absent versus present module hash, at most 32 bytes.
        2_928_387_969 => module_hash: Option<Vec<u8>> [strict_optional, max = 32],
    }
}
projection! {
    /// Lifecycle/controller/module-hash projection for capture preconditions.
    /// Version and other unselected fields are validated under the skip budget.
    both ("management-snapshot-fixtures") CaptureStatus {
        /// Reported lifecycle state.
        100_394_802 => status: Status [required],
        /// Selected controller settings.
        2_336_062_691 => settings: Settings [required],
        /// Absent versus present module hash, at most 32 bytes.
        2_928_387_969 => module_hash: Option<Vec<u8>> [strict_optional, max = 32],
    }
}
/// Decode capture preconditions without requiring a version observation.
/// Use [`decode_canister_status`] when an upgrade proof requires version.
/// # Errors
/// Refuses malformed, trailing or over-budget selected and skipped fields.
pub fn decode_capture_status(bytes: &[u8], limits: Limits) -> Result<CaptureStatus, Error> {
    candid::decode(bytes, limits)
}

projection! {
    /// Snapshot inventory entry; size and timestamp are raw observations.
    both ("management-snapshot-fixtures") Snapshot {
        /// Opaque identifier, at most 256 bytes. Consumer admits nonempty IDs.
        23_515 => id: Vec<u8> [required, max = 256],
        /// Reported total snapshot size.
        1_926_775_388 => total_size: u64 [required],
        /// IC timestamp in nanoseconds.
        3_569_147_778 => taken_at_timestamp: u64 [required],
    }
}
projection! {
    /// Exact returned snapshot bytes; no hash or requested-length claim.
    both ("management-snapshot-fixtures") SnapshotData {
        /// At most one MiB, including a valid empty chunk.
        1_170_392_685 => chunk: Vec<u8> [required, max = 1_048_576],
    }
}
projection! {
    /// Allocated snapshot identity; admission and spending stay with Backup.
    both ("management-snapshot-fixtures") SnapshotAllocation {
        /// Opaque identifier, at most 256 bytes.
        4_045_790_262 => snapshot_id: Vec<u8> [required, max = 256],
    }
}

struct Inventory(Vec<Snapshot>);
impl Decode for Inventory {
    const SCHEMA: Schema = <Vec<Snapshot> as Decode>::SCHEMA;
    fn read(reader: &mut Reader<'_, '_>, id: i64) -> Result<Self, Error> {
        reader.limited_value(id, 1024).map(Self)
    }
}

/// Decode one selected status record under explicit aggregate budgets.
/// # Errors
/// Refuses malformed, incompatible, trailing or over-budget fields, including
/// more than ten controllers or more than 32 module-hash bytes before allocation.
pub fn decode_canister_status(bytes: &[u8], limits: Limits) -> Result<CanisterStatus, Error> {
    candid::decode(bytes, limits)
}
/// Decode one snapshot inventory, capped at 1,024 entries before allocation.
/// # Errors
/// Refuses malformed, incompatible, trailing or over-budget input.
pub fn decode_list_snapshots(bytes: &[u8], limits: Limits) -> Result<Vec<Snapshot>, Error> {
    candid::decode::<Inventory>(bytes, limits).map(|value| value.0)
}
/// Decode the single snapshot returned by `take_canister_snapshot`.
/// # Errors
/// Refuses malformed, incompatible, trailing or over-budget input.
pub fn decode_take_snapshot(bytes: &[u8], limits: Limits) -> Result<Snapshot, Error> {
    candid::decode(bytes, limits)
}
/// Decode exact snapshot data, capped at one MiB before allocation.
/// # Errors
/// Refuses malformed, incompatible, trailing or over-budget input. The complete
/// input allowance must also accommodate the Candid header around a full chunk.
pub fn decode_snapshot_data(bytes: &[u8], limits: Limits) -> Result<SnapshotData, Error> {
    candid::decode(bytes, limits)
}
/// Decode the allocated snapshot identifier without granting restore authority.
/// # Errors
/// Refuses malformed, incompatible, trailing or over-budget input.
pub fn decode_snapshot_allocation(
    bytes: &[u8],
    limits: Limits,
) -> Result<SnapshotAllocation, Error> {
    candid::decode(bytes, limits)
}

/// IEEE-754 binary32 bits, including signed zero and all NaN payload bits.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Float32Bits(pub u32);
/// IEEE-754 binary64 bits, including signed zero and all NaN payload bits.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Float64Bits(pub u64);
macro_rules! float_bits {
    ($name:ident, $float:ty, $id:literal) => {
        impl Decode for $name {
            const SCHEMA: Schema = Schema::Primitive($id);
            fn read(reader: &mut Reader<'_, '_>, id: i64) -> Result<Self, Error> {
                reader
                    .value::<$float>(id)
                    .map(|value| Self(value.to_bits()))
            }
        }
        impl Encode for $name {
            const SCHEMA: Schema = Schema::Primitive($id);
            fn write(&self, writer: &mut Writer<'_>) -> Result<(), EncodeError> {
                writer.bytes(&self.0.to_le_bytes())
            }
        }
    };
}
float_bits!(Float32Bits, f32, -13);
float_bits!(Float64Bits, f64, -14);
/// Exact reserved provenance payload. Non-reserved payloads refuse decoding.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Reserved;
impl Decode for Reserved {
    const SCHEMA: Schema = Schema::Primitive(-16);
    fn read(_: &mut Reader<'_, '_>, id: i64) -> Result<Self, Error> {
        if id == -16 {
            Ok(Self)
        } else {
            Err(Error::Type)
        }
    }
}
impl Encode for Reserved {
    const SCHEMA: Schema = Schema::Primitive(-16);
    fn write(&self, _: &mut Writer<'_>) -> Result<(), EncodeError> {
        Ok(())
    }
}
projection! {
    /// Snapshot global with exact numeric representation. V128 refuses values
    /// outside 128 bits without allocating an arbitrary magnitude.
    both enum Global {
        /// Exact f32 bits.
        5_083_781 => F32(Float32Bits),
        /// Exact f64 bits.
        5_084_452 => F64(Float64Bits),
        /// Signed 32-bit global.
        5_232_968 => I32(i32),
        /// Signed 64-bit global.
        5_233_639 => I64(i64),
        /// Unsigned 128-bit global encoded as Candid nat.
        1_311_016_833 => V128(u128),
    }
}
projection! {
    /// Snapshot provenance; payloads must be exactly Candid reserved.
    both ("management-snapshot-fixtures") enum SnapshotSource {
        /// Snapshot originated from metadata upload.
        833_703_953 => MetadataUpload(Reserved),
        /// Snapshot was taken from a canister.
        2_359_638_948 => TakenFromCanister(Reserved),
    }
}
projection! {
    /// Exact captured timer state, without scheduling policy.
    both enum Timer {
        /// Active deadline in IC nanoseconds.
        373_703_110 => Active(u64),
        /// Inactive timer.
        1_781_536_619 => Inactive,
    }
}
projection! {
    /// Captured low-Wasm-memory hook state.
    both enum LowMemoryHook {
        /// Trigger condition is not met.
        1_043_953_570 => ConditionNotSatisfied,
        /// Hook has executed.
        3_210_470_543 => Executed,
        /// Hook is ready.
        3_870_990_435 => Ready,
    }
}
projection! {
    /// Raw chunk digest under the finite 32-byte profile; no authentication.
    both SnapshotChunk {
        /// At most 32 exact bytes. Digest length and comparison are caller policy.
        1_158_164_430 => hash: Vec<u8> [required, max = 32],
    }
}
projection! {
    /// Selected snapshot metadata. Missing optional fields, explicit known
    /// null/reserved absence and known value promotion are retained; incompatible
    /// present values refuse instead of becoming absence.
    both ("management-snapshot-fixtures") SnapshotMetadata {
        /// At most 4,096 globals; unknown global values never silently disappear.
        339_281_328 => globals: Vec<Option<Global>> [known_elements, max = 4096],
        /// Captured canister version.
        783_802_816 => canister_version: u64 [required],
        /// Optional, strictly typed provenance.
        842_117_339 => source: Option<SnapshotSource> [known_optional],
        /// At most 32 certified-data bytes.
        864_574_658 => certified_data: Vec<u8> [required, max = 32],
        /// Optional timer observation.
        903_275_689 => global_timer: Option<Timer> [known_optional],
        /// Optional hook observation.
        1_282_538_240 => on_low_wasm_memory_hook_status: Option<LowMemoryHook> [known_optional],
        /// Module bytes, not an allocation request.
        1_299_009_433 => wasm_module_size: u64 [required],
        /// Stable-memory bytes.
        1_710_030_651 => stable_memory_size: u64 [required],
        /// At most 1,024 chunk digests, preserving returned order.
        1_767_008_372 => wasm_chunk_store: Vec<SnapshotChunk> [required, max = 1024],
        /// IC timestamp in nanoseconds.
        3_569_147_778 => taken_at_timestamp: u64 [required],
        /// Wasm-memory bytes.
        3_598_099_748 => wasm_memory_size: u64 [required],
    }
}
/// Decode snapshot metadata under the caller's aggregate limits and the finite
/// field profile. Use zero skipped work to refuse unknown fields in Backup.
/// # Errors
/// Refuses malformed, incompatible, trailing, unknown variants, oversized fields
/// and globals outside 128 bits. Success confers no capture/restore authority.
pub fn decode_snapshot_metadata(bytes: &[u8], limits: Limits) -> Result<SnapshotMetadata, Error> {
    candid::decode(bytes, limits)
}
projection! {
    /// Exact extent requested from a selected snapshot region.
    both Extent {
        /// Requested byte count.
        1_280_549_057 => size: u64 [required],
        /// Starting byte offset.
        1_614_780_307 => offset: u64 [required],
    }
}
projection! {
    /// Exact write offset in a selected snapshot region.
    both Offset {
        /// Starting byte offset.
        1_614_780_307 => offset: u64 [required],
    }
}
projection! {
    /// Read region or complete stored chunk; geometry admission is caller-owned.
    both enum ReadKind {
        /// Wasm module extent.
        265_441_191 => WasmModule(Extent),
        /// Wasm memory extent.
        1_405_019_644 => WasmMemory(Extent),
        /// Stable memory extent.
        2_465_514_309 => StableMemory(Extent),
        /// Complete chunk selected by its exact digest.
        4_161_627_090 => WasmChunk(SnapshotChunk),
    }
}
projection! {
    /// Write region; exact chunk bytes are supplied separately.
    both enum WriteKind {
        /// Wasm module offset.
        265_441_191 => WasmModule(Offset),
        /// Wasm memory offset.
        1_405_019_644 => WasmMemory(Offset),
        /// Stable memory offset.
        2_465_514_309 => StableMemory(Offset),
        /// Complete chunk upload.
        4_161_627_090 => WasmChunk,
    }
}
projection! {
    /// `read_canister_snapshot_data` arguments.
    encoding ReadDataArgs {
        /// Exact region selection.
        1_191_829_844 => kind: ReadKind,
        /// Exact canister target.
        1_313_628_723 => canister_id: Principal,
        /// Exact opaque snapshot identifier.
        4_045_790_262 => snapshot_id: Vec<u8>,
    }
}
projection! {
    /// `upload_canister_snapshot_data` arguments.
    encoding UploadDataArgs {
        /// Exact bytes; the complete encoded output is bounded before allocation.
        1_170_392_685 => chunk: Vec<u8>,
        /// Exact write region.
        1_191_829_844 => kind: WriteKind,
        /// Exact canister target.
        1_313_628_723 => canister_id: Principal,
        /// Exact opaque snapshot identifier.
        4_045_790_262 => snapshot_id: Vec<u8>,
    }
}
projection! {
    /// `upload_canister_snapshot_metadata` arguments. Caller must explicitly
    /// admit every restored value; no missing global or timer default is invented.
    encoding UploadMetadataArgs {
        /// Exact global values; absence cannot be uploaded as a value.
        339_281_328 => globals: Vec<Global>,
        /// Optional snapshot to replace.
        435_757_295 => replace_snapshot: Option<Vec<u8>>,
        /// Exact certified data.
        864_574_658 => certified_data: Vec<u8>,
        /// Explicit optional timer state.
        903_275_689 => global_timer: Option<Timer>,
        /// Explicit optional hook state.
        1_282_538_240 => on_low_wasm_memory_hook_status: Option<LowMemoryHook>,
        /// Wasm module bytes.
        1_299_009_433 => wasm_module_size: u64,
        /// Exact canister target.
        1_313_628_723 => canister_id: Principal,
        /// Stable memory bytes.
        1_710_030_651 => stable_memory_size: u64,
        /// Wasm memory bytes.
        3_598_099_748 => wasm_memory_size: u64,
    }
}

/// Decode the empty result tuple of snapshot upload/load operations. Even
/// unused type-table entries retain header, type and traversal validation.
/// # Errors
/// Refuses nonzero arity, malformed framing, trailing bytes or exhausted budgets.
pub fn decode_empty_response(bytes: &[u8], limits: Limits) -> Result<(), Error> {
    candid::decode_empty(bytes, limits)
}

/// Bounded response construction for tests, without policy admission.
/// Collection/identity/hash values deliberately may exceed decoder policy caps.
/// The inclusive complete-output bound is checked before output allocation.
#[cfg(feature = "management-snapshot-fixtures")]
pub mod fixtures;
