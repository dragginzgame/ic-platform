//! Bounded management creation, installation and controller settings.
//! Authority, lifecycle, spending and retry decisions remain caller-owned.
use crate::candid::{
    self, Decode, Error, Limits, Reader, Schema,
    encode::{Encode, Writer},
    projection::{absent, projection},
};
pub use crate::candid::{Nat, encode::EncodeError};
pub use crate::management_snapshots::{
    CanisterArgs, CanisterStatus, decode_canister_status, decode_empty_response,
};
use candidate_types::Principal;
use serde::{Deserialize, Serialize};

projection! {
    /// Explicit selected log access policy.
    both enum LogVisibility {
        /// Controllers only.
        #[serde(rename = "controllers")]
        570880087 => Controllers,
        /// Public access.
        #[serde(rename = "public")]
        1727234217 => Public,
        /// Explicit viewer identities.
        #[serde(rename = "allowed_viewers")]
        2235716810 => AllowedViewers(Vec<Principal>),
    }
}

projection! {
    /// Explicit environment entry.
    encoding EnvironmentVariable {
        /// Exact value.
        834174833 => value: String,
        /// Exact name.
        1224700491 => name: String,
    }
}

projection! {
    /// Nine-field CMC creation settings; not the management settings schema.
    #[derive(Default)]
    encoding CreationSettings {
        /// Freezing threshold.
        238856128 => freezing_threshold: Option<Nat>,
        /// Wasm memory threshold.
        241629288 => wasm_memory_threshold: Option<Nat>,
        /// Environment variables.
        356749323 => environment_variables: Option<Vec<EnvironmentVariable>>,
        /// Explicit controller list; policy and count admission belong to the caller.
        570880087 => controllers: Option<Vec<Principal>>,
        /// Reserved cycles limit.
        1095112320 => reserved_cycles_limit: Option<Nat>,
        /// Log visibility.
        2033331437 => log_visibility: Option<LogVisibility>,
        /// Wasm memory limit.
        3380736376 => wasm_memory_limit: Option<Nat>,
        /// Memory allocation.
        3844961758 => memory_allocation: Option<Nat>,
        /// Compute allocation.
        4174053672 => compute_allocation: Option<Nat>,
    }
}

projection! {
    /// Explicit selected snapshot access policy.
    both enum SnapshotVisibility {
        /// Controllers only.
        #[serde(rename = "controllers")]
        570880087 => Controllers,
        /// Public access.
        #[serde(rename = "public")]
        1727234217 => Public,
        /// Explicit viewer identities.
        #[serde(rename = "allowed_viewers")]
        2235716810 => AllowedViewers(Vec<Principal>),
    }
}

projection! {
    /// Explicit selected status access policy.
    both enum StatusVisibility {
        /// Controllers only.
        #[serde(rename = "controllers")]
        570880087 => Controllers,
        /// Public access.
        #[serde(rename = "public")]
        1727234217 => Public,
        /// Explicit viewer identities.
        #[serde(rename = "allowed_viewers")]
        2235716810 => AllowedViewers(Vec<Principal>),
    }
}

projection! {
    /// Complete thirteen-field management settings; distinct from CMC.
    #[derive(Default)]
    encoding CanisterSettings {
        /// Freezing threshold.
        238856128 => freezing_threshold: Option<Nat>,
        /// Wasm memory threshold.
        241629288 => wasm_memory_threshold: Option<Nat>,
        /// Environment variables.
        356749323 => environment_variables: Option<Vec<EnvironmentVariable>>,
        /// Explicit controller list; policy and count admission belong to the caller.
        570880087 => controllers: Option<Vec<Principal>>,
        /// Minimum incoming call cycles.
        793974846 => minimum_incoming_canister_call_cycles: Option<Nat>,
        /// Reserved cycles limit.
        1095112320 => reserved_cycles_limit: Option<Nat>,
        /// Log visibility.
        2033331437 => log_visibility: Option<LogVisibility>,
        /// Log memory limit.
        2503910424 => log_memory_limit: Option<Nat>,
        /// Snapshot access policy.
        2709750061 => snapshot_visibility: Option<SnapshotVisibility>,
        /// Wasm memory limit.
        3380736376 => wasm_memory_limit: Option<Nat>,
        /// Status access policy.
        3469368575 => status_visibility: Option<StatusVisibility>,
        /// Memory allocation.
        3844961758 => memory_allocation: Option<Nat>,
        /// Compute allocation.
        4174053672 => compute_allocation: Option<Nat>,
    }
}

projection! {
    /// One provisional_create_canister_with_cycles argument.
    encoding ProvisionalCreateArgs {
        /// Selected creation settings.
        2336062691 => settings: Option<CanisterSettings>,
        /// Explicit requested identity.
        3348853450 => specified_id: Option<Principal>,
        /// Explicit cycle amount.
        3573748184 => amount: Option<Nat>,
        /// Explicit sender version.
        3599109322 => sender_canister_version: Option<u64>,
    }
}

projection! {
    /// One update_settings argument.
    encoding UpdateSettingsArgs {
        /// Exact target.
        1313628723 => canister_id: Principal,
        /// Explicit settings changes.
        2336062691 => settings: CanisterSettings,
        /// Explicit sender version.
        3599109322 => sender_canister_version: Option<u64>,
    }
}

projection! {
    /// Explicit upgrade heap policy.
    both enum MemoryPersistence {
        /// Keep the heap.
        #[serde(rename = "keep")]
        1191628933 => Keep,
        /// Replace the heap.
        #[serde(rename = "replace")]
        2871543860 => Replace,
    }
}

projection! {
    /// Explicit optional upgrade choices.
    both UpgradeFlags {
        /// Absent versus explicit heap choice.
        793490780 => wasm_memory_persistence: Option<MemoryPersistence> [strict_optional],
        /// Absent versus explicit skip choice.
        1539756096 => skip_pre_upgrade: Option<bool> [strict_optional],
    }
}

projection! {
    /// Installation mode; no implicit reinstall or fallback.
    both enum InstallMode {
        /// Explicit reinstall.
        #[serde(rename = "reinstall")]
        235052488 => Reinstall,
        /// Upgrade with absent or explicit flags.
        #[serde(rename = "upgrade")]
        1664201884 => Upgrade(Option<UpgradeFlags>),
        /// Fresh install.
        #[serde(rename = "install")]
        2375734555 => Install,
    }
}

projection! {
    /// One install_code argument preserving exact Wasm and argument bytes.
    encoding InstallCodeArgs {
        /// Exact initialization/upgrade argument bytes.
        4849238 => arg: Vec<u8>,
        /// Exact Wasm bytes, including caller-selected compression.
        265441191 => wasm_module: Vec<u8>,
        /// Exact operation.
        1214305123 => mode: InstallMode,
        /// Exact target.
        1313628723 => canister_id: Principal,
        /// Explicit sender version.
        3599109322 => sender_canister_version: Option<u64>,
    }
}

projection! {
    /// Creation reply; identity does not establish authorization.
     CreatedCanister {
        /// Returned canonical identity.
        1313628723 => canister_id: Principal [required],
    }
}

/// Decode exactly one bounded canister-ID creation reply.
/// # Errors
/// Refuses malformed, incompatible, trailing or over-budget input.
pub fn decode_created(bytes: &[u8], limits: Limits) -> Result<CreatedCanister, Error> {
    candid::decode(bytes, limits)
}

projection! {
    /// One ordinary create_canister argument; attached cycles remain Runtime-owned.
    encoding CreateCanisterArgs {
        /// Explicit settings; optional presence is preserved.
        2336062691 => settings: Option<CanisterSettings>,
        /// Explicit sender_canister_version; optional presence is preserved.
        3599109322 => sender_canister_version: Option<u64>,
    }
}

projection! {
    /// One install_chunked_code argument; chunk order and exact bytes are retained.
    encoding InstallChunkedCodeArgs {
        /// Exact initialization or upgrade argument bytes.
        4849238 => arg: Vec<u8>,
        /// Exact complete-module hash bytes; no implicit hashing or comparison.
        1176624806 => wasm_module_hash: Vec<u8>,
        /// Explicit installation mode, without fallback.
        1214305123 => mode: InstallMode,
        /// Chunk hashes in the requested assembly order.
        1847393007 => chunk_hashes_list: Vec<crate::management_chunks::ChunkHash>,
        /// Exact installation target.
        2807510037 => target_canister: Principal,
        /// Optional chunk store; absence is preserved for the management contract.
        3052241285 => store_canister: Option<Principal>,
        /// Explicit sender_canister_version; optional presence is preserved.
        3599109322 => sender_canister_version: Option<u64>,
    }
}

projection! {
    /// Execution-cycle charges; this is neither instruction count nor balance loss.
    ExecutionCharges {
        /// Arbitrary natural cycles charged for instructions.
        1_832_883_877 => instructions: Nat [required],
    }
}

projection! {
    /// Selected canister_metrics reply; unrelated metrics are validated and skipped.
    CanisterMetrics {
        /// Reported execution charges; callers own checked numeric conversion.
        2_122_851_290 => cycles_consumed: ExecutionCharges [required],
    }
}

/// Decode one management canister_metrics reply with explicit resource budgets.
/// # Errors
/// Refuses missing/incompatible fields, malformed skipped values, incorrect arity,
/// trailing bytes and exhausted budgets.
pub fn decode_canister_metrics(bytes: &[u8], limits: Limits) -> Result<CanisterMetrics, Error> {
    candid::decode(bytes, limits)
}
