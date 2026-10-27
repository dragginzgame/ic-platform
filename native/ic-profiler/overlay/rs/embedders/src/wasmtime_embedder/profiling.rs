//! Diagnostic observers for the existing IC instruction meter. No state here is
//! stored in canister memory, and none of the callbacks use the system API fee path.
use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use ic_interfaces::execution_environment::{
    WasmInstructionApiCharge, WasmInstructionBudgetEvent, WasmInstructionChargeLedger,
    WasmInstructionDynamicCharge, WasmInstructionProfile, WasmInstructionProfileBlock,
    WasmInstructionRuleCharge, WasmInstructionStack, WasmInstructionStackCharge,
    WasmInstructionStackFrame,
};
use wasmtime::{Caller, Linker, Module, WasmBacktrace};

use super::{CanisterMemoryType, StoreData};

const MAX_UNIQUE_READS: usize = 131_072;
const MAX_READ_RANGES: usize = 131_072;

type ReadKey = (u8, u8, u64, u64);

#[derive(Default)]
struct MemoryReadState {
    events: HashMap<ReadKey, u64>,
    observed_reads: u64,
    observed_bytes: u64,
    truncated: bool,
    counters_saturated: bool,
}

/// Bounded sparse aggregation of successful logical reads. Repeated reads of
/// the same range need one host record, regardless of their frequency or size.
/// This collector never dereferences guest memory and is never called by the
/// memory fault handler.
pub struct MemoryReadProfileCollector {
    state: Mutex<MemoryReadState>,
    max_unique_reads: usize,
    max_export_ranges: usize,
}

impl MemoryReadProfileCollector {
    pub fn new() -> Self {
        Self::with_limits(MAX_UNIQUE_READS, MAX_READ_RANGES)
    }

    fn with_limits(max_unique_reads: usize, max_export_ranges: usize) -> Self {
        Self {
            state: Mutex::new(MemoryReadState::default()),
            max_unique_reads,
            max_export_ranges,
        }
    }

    /// Paths 0 and 3 are original Wasm and validated host heap reads; both
    /// contribute to the heap channel. Stable paths 1 and 2 distinguish the
    /// direct PageMap copy and native-memory path, not a promised charge.
    pub fn observe_read(&self, memory: i32, address: u64, length: u64, path: i32) {
        if length == 0 {
            return;
        }
        let channel = match (memory, path) {
            (0, 0 | 3) => 0,
            (1, 1) => 1,
            (1, 2) => 2,
            _ => return,
        };
        let mut state = self.state.lock().unwrap();
        let Some(end) = address.checked_add(length) else {
            state.truncated = true;
            return;
        };
        match state.observed_reads.checked_add(1) {
            Some(value) => state.observed_reads = value,
            None => {
                state.observed_reads = u64::MAX;
                state.counters_saturated = true;
            }
        }
        match state.observed_bytes.checked_add(length) {
            Some(value) => state.observed_bytes = value,
            None => {
                state.observed_bytes = u64::MAX;
                state.counters_saturated = true;
            }
        }
        let key = (memory as u8, channel, address, end);
        if let Some(count) = state.events.get_mut(&key) {
            if let Some(value) = count.checked_add(1) {
                *count = value;
            } else {
                state.counters_saturated = true;
            }
        } else if state.events.len() < self.max_unique_reads {
            state.events.insert(key, 1);
        } else {
            state.truncated = true;
        }
    }

    /// Sweep range endpoints outside guest execution. Work and storage depend
    /// on distinct ranges, never on the highest address or the number of bytes.
    pub fn finish(&self) -> ic_interfaces::execution_environment::WasmReadProfile {
        use ic_interfaces::execution_environment::{WasmReadProfile, WasmReadRange};
        let state = self.state.lock().unwrap();
        let mut profile = WasmReadProfile {
            observed_reads: state.observed_reads,
            observed_bytes: state.observed_bytes,
            truncated: state.truncated,
            counters_saturated: state.counters_saturated,
            max_unique_reads: self.max_unique_reads as u64,
            max_export_ranges: self.max_export_ranges as u64,
            coverage: vec![
                "successful_original_wasm_loads".to_string(),
                "successful_original_wasm_memory_copy_sources".to_string(),
                "validated_host_heap_read_ranges".to_string(),
                "successful_stable_read_ranges_by_actual_copy_path".to_string(),
            ],
            ..Default::default()
        };
        let mut endpoints: BTreeMap<(u8, u64), [i128; 3]> = BTreeMap::new();
        for (&(memory, channel, start, end), &count) in &state.events {
            endpoints.entry((memory, start)).or_default()[channel as usize] += count as i128;
            endpoints.entry((memory, end)).or_default()[channel as usize] -= count as i128;
        }
        let mut current_memory = 0;
        let mut previous = 0;
        let mut active = [0_i128; 3];
        for ((memory, address), delta) in endpoints {
            if memory != current_memory {
                current_memory = memory;
                previous = address;
                active = [0; 3];
            }
            if previous < address && active.iter().any(|count| *count > 0) {
                let counts = active.map(|count| {
                    if count > u64::MAX as i128 {
                        profile.counters_saturated = true;
                        u64::MAX
                    } else {
                        count.max(0) as u64
                    }
                });
                let name = if memory == 0 { "heap" } else { "stable" };
                if let Some(last) = profile.ranges.last_mut().filter(|last| {
                    last.memory == name
                        && last.end == previous
                        && last.heap_reads == counts[0]
                        && last.stable_direct_reads == counts[1]
                        && last.stable_native_reads == counts[2]
                }) {
                    last.end = address;
                } else if profile.ranges.len() < self.max_export_ranges {
                    profile.ranges.push(WasmReadRange {
                        memory: name.to_string(),
                        start: previous,
                        end: address,
                        heap_reads: counts[0],
                        stable_direct_reads: counts[1],
                        stable_native_reads: counts[2],
                    });
                } else {
                    profile.truncated = true;
                    break;
                }
            }
            for index in 0..3 {
                active[index] += delta[index];
            }
            previous = address;
        }
        profile
    }
}

impl ic_interfaces::execution_environment::HeapReadObserver for MemoryReadProfileCollector {
    fn read(&self, start: u64, len: u64) {
        self.observe_read(0, start, len, 3);
    }
}

const MAX_UNIQUE_WRITES: usize = 131_072;
const MAX_WRITE_RANGES: usize = 131_072;

type WriteKey = (u8, u64, u64);

#[derive(Default)]
struct MemoryWriteState {
    events: HashMap<WriteKey, u64>,
    observed_writes: u64,
    observed_bytes: u64,
    truncated: bool,
    counters_saturated: bool,
}

/// Sparse successful logical writes. Values are deliberately never inspected:
/// same-value stores and write-then-restore operations increment these counts.
pub struct MemoryWriteProfileCollector {
    state: Mutex<MemoryWriteState>,
    max_unique_writes: usize,
    max_export_ranges: usize,
}

impl MemoryWriteProfileCollector {
    pub fn new() -> Self {
        Self::with_limits(MAX_UNIQUE_WRITES, MAX_WRITE_RANGES)
    }

    fn with_limits(max_unique_writes: usize, max_export_ranges: usize) -> Self {
        Self {
            state: Mutex::new(MemoryWriteState::default()),
            max_unique_writes,
            max_export_ranges,
        }
    }

    /// Heap paths are original Wasm stores, direct stable-copy destinations,
    /// and successful host destinations. Stable path 1 is an API destination.
    /// Path identity affects capture coverage, not the frequency of a byte.
    pub fn observe_write(&self, memory: i32, address: u64, length: u64, path: i32) {
        if length == 0 || !matches!((memory, path), (0, 0 | 2 | 3) | (1, 1)) {
            return;
        }
        let mut state = self.state.lock().unwrap();
        let Some(end) = address.checked_add(length) else {
            state.truncated = true;
            return;
        };
        match state.observed_writes.checked_add(1) {
            Some(value) => state.observed_writes = value,
            None => {
                state.observed_writes = u64::MAX;
                state.counters_saturated = true;
            }
        }
        match state.observed_bytes.checked_add(length) {
            Some(value) => state.observed_bytes = value,
            None => {
                state.observed_bytes = u64::MAX;
                state.counters_saturated = true;
            }
        }
        let key = (memory as u8, address, end);
        if let Some(count) = state.events.get_mut(&key) {
            if let Some(value) = count.checked_add(1) {
                *count = value;
            } else {
                state.counters_saturated = true;
            }
        } else if state.events.len() < self.max_unique_writes {
            state.events.insert(key, 1);
        } else {
            state.truncated = true;
        }
    }

    /// Endpoint sweeps depend on sparse distinct ranges, not address-space size
    /// or write width. All overlapping successful writes add their frequency.
    pub fn finish(&self) -> ic_interfaces::execution_environment::WasmWriteProfile {
        use ic_interfaces::execution_environment::{WasmWriteProfile, WasmWriteRange};
        let state = self.state.lock().unwrap();
        let mut profile = WasmWriteProfile {
            observed_writes: state.observed_writes,
            observed_bytes: state.observed_bytes,
            truncated: state.truncated,
            counters_saturated: state.counters_saturated,
            max_unique_writes: self.max_unique_writes as u64,
            max_export_ranges: self.max_export_ranges as u64,
            coverage: vec![
                "successful_original_wasm_stores".to_string(),
                "successful_original_wasm_bulk_memory_destinations".to_string(),
                "successful_host_heap_write_ranges".to_string(),
                "successful_stable_api_write_ranges".to_string(),
                "same_value_and_write_restore_included".to_string(),
            ],
            ..Default::default()
        };
        let mut endpoints: BTreeMap<(u8, u64), i128> = BTreeMap::new();
        for (&(memory, start, end), &count) in &state.events {
            *endpoints.entry((memory, start)).or_default() += count as i128;
            *endpoints.entry((memory, end)).or_default() -= count as i128;
        }
        let mut current_memory = 0;
        let mut previous = 0;
        let mut active = 0_i128;
        for ((memory, address), delta) in endpoints {
            if memory != current_memory {
                current_memory = memory;
                previous = address;
                active = 0;
            }
            if previous < address && active > 0 {
                let count = if active > u64::MAX as i128 {
                    profile.counters_saturated = true;
                    u64::MAX
                } else {
                    active as u64
                };
                let name = if memory == 0 { "heap" } else { "stable" };
                let (heap_writes, stable_writes) =
                    if memory == 0 { (count, 0) } else { (0, count) };
                if let Some(last) = profile.ranges.last_mut().filter(|last| {
                    last.memory == name
                        && last.end == previous
                        && last.heap_writes == heap_writes
                        && last.stable_writes == stable_writes
                }) {
                    last.end = address;
                } else if profile.ranges.len() < self.max_export_ranges {
                    profile.ranges.push(WasmWriteRange {
                        memory: name.to_string(),
                        start: previous,
                        end: address,
                        heap_writes,
                        stable_writes,
                    });
                } else {
                    profile.truncated = true;
                    break;
                }
            }
            active += delta;
            previous = address;
        }
        profile
    }
}

impl ic_interfaces::execution_environment::HeapWriteObserver for MemoryWriteProfileCollector {
    fn write(&self, start: u64, len: u64) {
        self.observe_write(0, start, len, 3);
    }
}

const MAX_BLOCKS: usize = 65_536;
const MAX_DYNAMIC: usize = 8_192;
const MAX_APIS: usize = 256;
const MAX_BUDGET_EVENTS: usize = 256;
const MAX_RULES: usize = 32;
const MAX_STACKS: usize = 2_048;
const MAX_STACK_CHARGES: usize = 8_192;
const MAX_STACK_OBSERVATIONS: usize = 16_384;
const MAX_STACK_FRAMES: usize = 20;
const MAX_OPERATION_SCOPES: usize = 64;

#[derive(Clone, Copy)]
struct CapturedFrame {
    function: u32,
    offset: Option<u64>,
    matching_module: bool,
}
type StackChargeKey = (
    Option<u32>,
    Option<u32>,
    &'static str,
    Option<&'static str>,
    &'static str,
);
const MAX_PAGE_DETAILS: usize = 8_192;

/// Each slot is written once. Atomics avoid interior-mutation aliasing when
/// the execution thread is interrupted by the signal handler.
#[derive(Default)]
struct PageDebitRecord {
    ready: AtomicBool,
    memory: AtomicU64,
    kind: AtomicU64,
    region_index: AtomicU64,
    instructions: AtomicU64,
    native_pc: AtomicU64,
    host_module_offset: AtomicU64,
    host_operation_active: AtomicBool,
}

/// Signal-safe counters and preallocated evidence slots. The signal handler
/// performs only fixed-size atomic operations; mapping and output allocation
/// happen after the Wasm call returns.
#[derive(Default)]
pub(crate) struct InstructionPageCharges {
    pub enabled: AtomicBool,
    heap: AtomicU64,
    stable: AtomicU64,
    rule_events: [AtomicU64; 4],
    rule_instructions: [AtomicU64; 4],
    fault_native_pc: AtomicU64,
    host_module_offset: AtomicU64,
    host_operation_active: AtomicBool,
    page_records: OnceLock<Box<[PageDebitRecord]>>,
    page_records_used: AtomicU64,
    page_details_truncated: AtomicBool,
}

impl InstructionPageCharges {
    /// Allocate and touch host storage before entering guest code. OnceLock::get
    /// in the handler never waits and cannot initialize or allocate anything.
    pub fn enable_page_details(&self) {
        self.page_records.get_or_init(|| {
            let records = (0..MAX_PAGE_DETAILS)
                .map(|_| PageDebitRecord::default())
                .collect::<Vec<_>>()
                .into_boxed_slice();
            // Touch every slot before guest execution even if allocation used
            // lazily zeroed pages. A repeated enable preserves existing detail.
            for record in records.iter() {
                record.ready.store(false, Ordering::Relaxed);
            }
            records
        });
    }

    /// The guard restores nested fault context on every normal handler return.
    /// It uses atomic operations only and does not acquire a lock.
    pub fn enter_fault(&self, native_pc: Option<u64>) -> FaultPcGuard<'_> {
        FaultPcGuard {
            charges: self,
            previous: self
                .fault_native_pc
                .swap(native_pc.unwrap_or(0), Ordering::Relaxed),
        }
    }

    /// Tags only the dynamic extent of a proven host operation. Holding an Arc
    /// avoids borrowing Caller while the host implementation accesses its heap.
    /// A nested unavailable operation clears the tag until its guard restores it.
    pub fn enter_host_operation(
        self: &Arc<Self>,
        module_offset: Option<u64>,
    ) -> HostOperationGuard {
        let previous_active = self.host_operation_active.swap(false, Ordering::Relaxed);
        let previous = self
            .host_module_offset
            .swap(module_offset.unwrap_or(0), Ordering::Relaxed);
        self.host_operation_active
            .store(module_offset.is_some(), Ordering::Release);
        HostOperationGuard {
            charges: Arc::clone(self),
            previous,
            previous_active,
        }
    }

    pub fn observe(&self, kind: CanisterMemoryType, amount: u64) {
        if self.enabled.load(Ordering::Relaxed) {
            let counter = match kind {
                CanisterMemoryType::Heap => &self.heap,
                CanisterMemoryType::Stable => &self.stable,
            };
            // Counters saturate instead of wrapping; overflow is exposed by the
            // collector as truncation. Practical IC execution limits are smaller.
            Self::add_atomic(counter, amount);
        }
    }

    fn add_atomic(counter: &AtomicU64, amount: u64) {
        let mut current = counter.load(Ordering::Relaxed);
        loop {
            match counter.compare_exchange_weak(
                current,
                current.saturating_add(amount),
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current = actual,
            }
        }
    }

    /// Called at actual page access/dirty debit points. No allocation or lock.
    pub fn observe_page_rule(
        &self,
        memory: CanisterMemoryType,
        kind: memory_tracker::PageDebitKind,
        region_index: u64,
        instructions: u64,
    ) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }
        let index = match memory {
            CanisterMemoryType::Heap => 0,
            CanisterMemoryType::Stable => 2,
        } + match kind {
            memory_tracker::PageDebitKind::Access => 0,
            memory_tracker::PageDebitKind::Dirty => 1,
        };
        Self::add_atomic(&self.rule_events[index], 1);
        Self::add_atomic(&self.rule_instructions[index], instructions);
        let Some(records) = self.page_records.get() else {
            return;
        };
        let slot = self.page_records_used.fetch_add(1, Ordering::Relaxed);
        let Some(record) = usize::try_from(slot)
            .ok()
            .and_then(|slot| records.get(slot))
        else {
            self.page_details_truncated.store(true, Ordering::Relaxed);
            return;
        };
        record.memory.store((index / 2) as u64, Ordering::Relaxed);
        record.kind.store((index % 2) as u64, Ordering::Relaxed);
        record.region_index.store(region_index, Ordering::Relaxed);
        record.instructions.store(instructions, Ordering::Relaxed);
        record.native_pc.store(
            self.fault_native_pc.load(Ordering::Relaxed),
            Ordering::Relaxed,
        );
        record.host_operation_active.store(
            self.host_operation_active.load(Ordering::Acquire),
            Ordering::Relaxed,
        );
        record.host_module_offset.store(
            self.host_module_offset.load(Ordering::Relaxed),
            Ordering::Relaxed,
        );
        record.ready.store(true, Ordering::Release);
    }

    /// Decode evidence after the Wasm call has stopped, outside signal context.
    /// Wasmtime explicitly maps each native interval to an optional Wasm PC;
    /// None intervals and native helper addresses remain unmapped.
    pub fn page_charges(
        &self,
        module: &wasmtime::Module,
        metadata: Option<&ic_interfaces::execution_environment::WasmInstructionModuleMetadata>,
    ) -> (
        Vec<ic_interfaces::execution_environment::WasmInstructionPageCharge>,
        bool,
    ) {
        use ic_interfaces::execution_environment::WasmInstructionPageCharge;
        let Some(records) = self.page_records.get() else {
            return (vec![], false);
        };
        let address_map: Vec<_> = module.address_map().into_iter().flatten().collect();
        let text = module.text();
        let text_start = text.as_ptr() as usize;
        let text_end = text_start.saturating_add(text.len());
        let charges = records
            .iter()
            .filter(|record| record.ready.load(Ordering::Acquire))
            .map(|record| {
                let native_pc = record.native_pc.load(Ordering::Relaxed);
                let native_module_offset =
                    module_offset_from_native_pc(native_pc, text_start..text_end, &address_map);
                let host_module_offset = record
                    .host_operation_active
                    .load(Ordering::Relaxed)
                    .then(|| record.host_module_offset.load(Ordering::Relaxed));
                let (module_offset, host_origin) =
                    select_page_origin(native_module_offset, host_module_offset, metadata);
                WasmInstructionPageCharge {
                    memory: if record.memory.load(Ordering::Relaxed) == 0 {
                        "heap"
                    } else {
                        "stable"
                    }
                    .to_string(),
                    region_index: record.region_index.load(Ordering::Relaxed),
                    kind: if record.kind.load(Ordering::Relaxed) == 0 {
                        "access"
                    } else {
                        "dirty"
                    }
                    .to_string(),
                    instructions: record.instructions.load(Ordering::Relaxed),
                    native_pc: (native_pc != 0).then_some(native_pc),
                    module_offset,
                    origin_id: None,
                    origin_evidence: if host_origin {
                        "exact_host_operation"
                    } else if native_module_offset.is_some() {
                        "native_fault_pc"
                    } else if native_pc == 0 {
                        "fault_pc_unavailable"
                    } else {
                        "native_fault_pc_unmapped"
                    }
                    .to_string(),
                }
            })
            .collect();
        (charges, self.page_details_truncated.load(Ordering::Relaxed))
    }

    pub fn rules(&self) -> impl Iterator<Item = (&'static str, u64, u64)> + '_ {
        [
            "ic.memory.heap.access.v1",
            "ic.memory.heap.dirty.v1",
            "ic.memory.stable.access.v1",
            "ic.memory.stable.dirty.v1",
        ]
        .into_iter()
        .enumerate()
        .filter_map(|(index, id)| {
            let events = self.rule_events[index].load(Ordering::Relaxed);
            (events > 0).then(|| {
                (
                    id,
                    events,
                    self.rule_instructions[index].load(Ordering::Relaxed),
                )
            })
        })
    }

    pub fn totals(&self) -> (u64, u64) {
        (
            self.heap.load(Ordering::Relaxed),
            self.stable.load(Ordering::Relaxed),
        )
    }
}

/// An exact native operation takes precedence. Synthetic Wasm helper code is
/// weaker evidence than a scoped caller whose original operation is known.
fn select_page_origin(
    native: Option<u64>,
    host: Option<u64>,
    metadata: Option<&ic_interfaces::execution_environment::WasmInstructionModuleMetadata>,
) -> (Option<u64>, bool) {
    let resolves_original = |offset: u64| {
        let Some(metadata) = metadata else {
            return false;
        };
        let end = metadata
            .instrumented_ranges
            .partition_point(|range| range.start_offset <= offset);
        end.checked_sub(1)
            .and_then(|index| metadata.instrumented_ranges.get(index))
            .is_some_and(|range| offset < range.end_offset && range.origin_id.is_some())
    };
    if native.is_some_and(resolves_original) {
        (native, false)
    } else if host.is_some_and(resolves_original) {
        (host, true)
    } else {
        // Retain synthetic native evidence for later explicit labeling, never
        // extend a neighboring source range or accept an unproven host tag.
        (native, false)
    }
}

fn module_offset_from_native_pc(
    native_pc: u64,
    text: std::ops::Range<usize>,
    address_map: &[(usize, Option<u32>)],
) -> Option<u64> {
    let pc = usize::try_from(native_pc)
        .ok()
        .filter(|pc| text.contains(pc))?;
    let native_offset = pc - text.start;
    let end = address_map.partition_point(|(offset, _)| *offset <= native_offset);
    end.checked_sub(1)
        .and_then(|index| address_map[index].1)
        .map(u64::from)
}

pub(crate) struct FaultPcGuard<'a> {
    charges: &'a InstructionPageCharges,
    previous: u64,
}

impl Drop for FaultPcGuard<'_> {
    fn drop(&mut self) {
        self.charges
            .fault_native_pc
            .store(self.previous, Ordering::Relaxed);
    }
}

pub(crate) struct HostOperationGuard {
    charges: Arc<InstructionPageCharges>,
    previous: u64,
    previous_active: bool,
}

impl Drop for HostOperationGuard {
    fn drop(&mut self) {
        self.charges
            .host_operation_active
            .store(false, Ordering::Relaxed);
        self.charges
            .host_module_offset
            .store(self.previous, Ordering::Relaxed);
        self.charges
            .host_operation_active
            .store(self.previous_active, Ordering::Release);
    }
}

#[derive(Default)]
struct ApiAggregate {
    calls: u64,
    base: u64,
    bytes: u64,
    direct: u64,
}
struct BudgetEvent {
    before: i64,
    after: Option<i64>,
    cause: &'static str,
}

/// Bounded host allocations are reserved before execution. Reaching a detail
/// limit drops additional detail while preserving category totals.
pub struct InstructionChargeProfileCollector {
    blocks: HashMap<u32, WasmInstructionProfileBlock>,
    dynamic: HashMap<(i32, i64, i32, i32), (u64, u64)>,
    apis: HashMap<&'static str, ApiAggregate>,
    budget_events: Vec<BudgetEvent>,
    rules: HashMap<&'static str, (u64, u64)>,
    ledger: WasmInstructionChargeLedger,
    truncated: bool,
    module: Option<Arc<ic_interfaces::execution_environment::WasmInstructionModuleMetadata>>,
    runtime_module: Option<Module>,
    page_charges: Option<Arc<InstructionPageCharges>>,
    stacks: Vec<WasmInstructionStack>,
    stack_keys: HashMap<Vec<(u32, Option<u64>, bool)>, u32>,
    stack_charges: HashMap<StackChargeKey, (u64, u64)>,
    stack_observations: usize,
    stack_details_truncated: bool,
    operation_scopes: Vec<HostOperationGuard>,
    overflow_scope: Option<HostOperationGuard>,
    operation_scope_overflow: usize,
}

impl Default for InstructionChargeProfileCollector {
    fn default() -> Self {
        Self {
            blocks: HashMap::with_capacity(MAX_BLOCKS),
            dynamic: HashMap::with_capacity(MAX_DYNAMIC),
            apis: HashMap::with_capacity(MAX_APIS),
            budget_events: Vec::with_capacity(MAX_BUDGET_EVENTS),
            rules: HashMap::with_capacity(MAX_RULES),
            ledger: WasmInstructionChargeLedger::default(),
            truncated: false,
            module: None,
            runtime_module: None,
            page_charges: None,
            stacks: Vec::with_capacity(MAX_STACKS),
            stack_keys: HashMap::with_capacity(MAX_STACKS),
            stack_charges: HashMap::with_capacity(MAX_STACK_CHARGES + MAX_APIS),
            stack_observations: 0,
            stack_details_truncated: false,
            operation_scopes: Vec::with_capacity(MAX_OPERATION_SCOPES),
            overflow_scope: None,
            operation_scope_overflow: 0,
        }
    }
}

impl InstructionChargeProfileCollector {
    pub(crate) fn new(
        module: Option<Arc<ic_interfaces::execution_environment::WasmInstructionModuleMetadata>>,
        runtime_module: Option<Module>,
        page_charges: Option<Arc<InstructionPageCharges>>,
    ) -> Self {
        Self {
            module,
            runtime_module,
            page_charges,
            ..Default::default()
        }
    }
    fn range(
        &self,
        frame: CapturedFrame,
    ) -> Option<&ic_interfaces::execution_environment::WasmInstructionOriginRange> {
        if !frame.matching_module {
            return None;
        }
        let offset = frame.offset?;
        let metadata = self.module.as_ref()?;
        let index = metadata
            .instrumented_ranges
            .partition_point(|range| range.start_offset <= offset)
            .checked_sub(1)?;
        metadata.instrumented_ranges.get(index).filter(|range| {
            range.instrumented_function_index == frame.function && offset < range.end_offset
        })
    }

    fn mapped_frame(&self, frame: CapturedFrame) -> WasmInstructionStackFrame {
        let range = self.range(frame);
        let origin = range
            .and_then(|range| range.origin_id)
            .and_then(|id| self.module.as_ref()?.operators.get(id as usize));
        WasmInstructionStackFrame {
            instrumented_function_index: frame.function,
            module_offset: frame.offset,
            function_index: origin.map(|op| op.function_index),
            origin_id: origin.map(|op| op.origin_id),
            body_offset: origin.map(|op| op.body_offset),
            operator_ordinal: origin.map(|op| op.operator_ordinal),
            synthetic_kind: range
                .filter(|range| !range.kind.starts_with("original"))
                .map(|range| range.kind.clone()),
        }
    }

    fn operation_origin(&self, frames: &[CapturedFrame]) -> Option<(u32, u64)> {
        for frame in frames {
            // Only known generated helper frames can be crossed. An unmapped
            // physical frame must never turn an outer caller into a guessed
            // API callsite.
            let range = self.range(*frame)?;
            match range.kind.as_str() {
                "original_call" | "dynamic_meter" | "memory_grow" | "operation_scope" => {
                    return Some((range.origin_id?, frame.offset?));
                }
                "runtime_helper" | "stable_replacement" => continue,
                _ => return None,
            }
        }
        None
    }

    fn record_stack_charge(
        &mut self,
        frames: Option<Vec<CapturedFrame>>,
        category: &'static str,
        api: Option<&'static str>,
        instructions: u64,
        explicit_origin: Option<(u32, u64, u32)>,
        evidence: &'static str,
    ) {
        self.stack_observations += 1;
        let explicit_id = explicit_origin.and_then(|(function, offset, ordinal)| {
            let operators = &self.module.as_ref()?.operators;
            let index = operators
                .binary_search_by_key(&(function, ordinal), |op| {
                    (op.function_index, op.operator_ordinal)
                })
                .ok()?;
            let origin = &operators[index];
            (origin.body_offset == offset).then_some(origin.origin_id)
        });
        let inferred_id = frames
            .as_ref()
            .and_then(|frames| self.operation_origin(frames))
            .map(|(id, _)| id);
        let origin = explicit_id.or(inferred_id);
        let evidence = if explicit_id.is_some() {
            evidence
        } else if inferred_id.is_some() {
            "exact_operation"
        } else {
            "unavailable"
        };
        let stack_id = frames.and_then(|frames| {
            if frames.is_empty() {
                return None;
            }
            if frames.len() >= MAX_STACK_FRAMES {
                self.stack_details_truncated = true;
            }
            let key: Vec<_> = frames
                .iter()
                .map(|frame| (frame.function, frame.offset, frame.matching_module))
                .collect();
            if let Some(id) = self.stack_keys.get(&key) {
                return Some(*id);
            }
            if self.stacks.len() >= MAX_STACKS {
                self.stack_details_truncated = true;
                return None;
            }
            let id = self.stacks.len() as u32;
            let mapped = frames
                .iter()
                .map(|frame| self.mapped_frame(*frame))
                .collect();
            self.stacks.push(WasmInstructionStack {
                stack_id: id,
                truncated: frames.len() >= MAX_STACK_FRAMES,
                frames: mapped,
            });
            self.stack_keys.insert(key, id);
            Some(id)
        });
        if stack_id.is_none() {
            self.stack_details_truncated = true;
        }
        let mut key = (stack_id, origin, category, api, evidence);
        if !self.stack_charges.contains_key(&key) && self.stack_charges.len() >= MAX_STACK_CHARGES {
            self.stack_details_truncated = true;
            key = (None, None, category, api, "unavailable");
        }
        let row = self.stack_charges.entry(key).or_insert((0, 0));
        Self::add(&mut row.0, 1, &mut self.truncated);
        Self::add(&mut row.1, instructions, &mut self.truncated);
    }

    pub fn observe_unmapped_page_charge(
        &mut self,
        category: &'static str,
        events: u64,
        instructions: u64,
    ) {
        let row = self
            .stack_charges
            .entry((None, None, category, None, "unavailable"))
            .or_insert((0, 0));
        Self::add(&mut row.0, events, &mut self.truncated);
        Self::add(&mut row.1, instructions, &mut self.truncated);
    }

    fn push_operation_scope(&mut self, guard: Option<HostOperationGuard>) {
        if self.operation_scopes.len() >= MAX_OPERATION_SCOPES || self.operation_scope_overflow > 0
        {
            drop(guard);
            self.operation_scope_overflow += 1;
            if self.overflow_scope.is_none() {
                self.overflow_scope = self
                    .page_charges
                    .as_ref()
                    .map(|charges| charges.enter_host_operation(None));
            }
            self.stack_details_truncated = true;
        } else if let Some(guard) = guard {
            self.operation_scopes.push(guard);
        }
    }

    fn pop_operation_scope(&mut self) {
        if self.operation_scope_overflow > 0 {
            self.operation_scope_overflow -= 1;
            if self.operation_scope_overflow == 0 {
                self.overflow_scope.take();
            }
        } else {
            self.operation_scopes.pop();
        }
    }

    pub fn clear_operation_scopes(&mut self) {
        self.overflow_scope.take();
        self.operation_scope_overflow = 0;
        while self.operation_scopes.pop().is_some() {}
    }

    pub fn observe_rule(&mut self, rule_id: &'static str, instructions: u64) {
        self.observe_rule_count(rule_id, 1, instructions);
    }

    pub fn observe_rule_count(&mut self, rule_id: &'static str, events: u64, instructions: u64) {
        if !self.rules.contains_key(rule_id) && self.rules.len() >= MAX_RULES {
            self.truncated = true;
            return;
        }
        let row = self.rules.entry(rule_id).or_insert((0, 0));
        Self::add(&mut row.0, events, &mut self.truncated);
        Self::add(&mut row.1, instructions, &mut self.truncated);
    }

    fn add(total: &mut u64, amount: u64, truncated: &mut bool) {
        match total.checked_add(amount) {
            Some(value) => *total = value,
            None => {
                *total = u64::MAX;
                *truncated = true;
            }
        }
    }

    pub fn observe_block(
        &mut self,
        id: i32,
        function: i32,
        offset: i64,
        end: i64,
        ordinal: i32,
        weight: i64,
        entry: i64,
    ) {
        let weight = weight as u64;
        let entry = entry as u64;
        self.observe_rule("ic.wasm.static.v1", weight.saturating_sub(entry));
        if entry > 0 {
            self.observe_rule("ic.wasm.function_entry.v1", entry);
        }
        Self::add(
            &mut self.ledger.static_instructions,
            weight,
            &mut self.truncated,
        );
        let id = id as u32;
        if !self.blocks.contains_key(&id) && self.blocks.len() >= MAX_BLOCKS {
            self.truncated = true;
            return;
        }
        let block = self
            .blocks
            .entry(id)
            .or_insert_with(|| WasmInstructionProfileBlock {
                block_id: id as u64,
                function_index: function as u32,
                body_offset: offset as u64,
                end_body_offset: end as u64,
                operator_ordinal: ordinal as u64,
                weight,
                entry_weight: entry as u64,
                hits: 0,
                instructions: 0,
            });
        Self::add(&mut block.hits, 1, &mut self.truncated);
        Self::add(&mut block.instructions, weight, &mut self.truncated);
    }

    pub fn observe_dynamic(
        &mut self,
        amount: u64,
        function: i32,
        offset: i64,
        ordinal: i32,
        category: i32,
    ) {
        if category >= 100 {
            let rule = match category {
                101 => "ic.stable.read.native.v1",
                102 => "ic.stable64.read.zero_length.v1",
                103 => "ic.stable64.write.zero_length.v1",
                _ => "ic.runtime.other.v1",
            };
            self.observe_rule(rule, 0);
            return;
        }
        self.observe_rule(
            match category {
                0 => "ic.wasm.bulk_length.v1",
                1 => "ic.stable.api.v1",
                _ => "ic.runtime.other.v1",
            },
            amount,
        );
        let total = match category {
            0 => &mut self.ledger.dynamic_instructions,
            1 => &mut self.ledger.stable_api_instructions,
            2 => &mut self.ledger.stable_page_instructions,
            _ => &mut self.ledger.other_known_instructions,
        };
        Self::add(total, amount, &mut self.truncated);
        let key = (function, offset, ordinal, category);
        if !self.dynamic.contains_key(&key) && self.dynamic.len() >= MAX_DYNAMIC {
            self.truncated = true;
            return;
        }
        let row = self.dynamic.entry(key).or_insert((0, 0));
        Self::add(&mut row.0, 1, &mut self.truncated);
        Self::add(&mut row.1, amount, &mut self.truncated);
    }

    pub fn observe_api(&mut self, api: &'static str, base: u64, bytes: u64, direct: u64) {
        self.observe_rule("ic.api.base.v1", base);
        if bytes > 0 {
            self.observe_rule("ic.api.bytes.v1", bytes);
        }
        if direct > 0 {
            self.observe_rule("ic.api.direct.v1", direct);
        }
        for amount in [base, bytes, direct] {
            Self::add(
                &mut self.ledger.api_instructions,
                amount,
                &mut self.truncated,
            );
        }
        if !self.apis.contains_key(api) && self.apis.len() >= MAX_APIS {
            self.truncated = true;
            return;
        }
        let row = self.apis.entry(api).or_default();
        Self::add(&mut row.calls, 1, &mut self.truncated);
        Self::add(&mut row.base, base, &mut self.truncated);
        Self::add(&mut row.bytes, bytes, &mut self.truncated);
        Self::add(&mut row.direct, direct, &mut self.truncated);
    }

    pub fn observe_budget_refill(&mut self, before: i64, after: Option<i64>, cause: &'static str) {
        if after.is_some() {
            self.observe_rule("ic.budget.dts_refill.v1", 0);
        }
        if self.budget_events.len() >= MAX_BUDGET_EVENTS {
            self.truncated = true;
            return;
        }
        self.budget_events.push(BudgetEvent {
            before,
            after,
            cause,
        });
    }

    pub fn finish(mut self, heap: u64, stable: u64) -> WasmInstructionProfile {
        self.ledger.heap_page_instructions = heap;
        Self::add(
            &mut self.ledger.stable_page_instructions,
            stable,
            &mut self.truncated,
        );
        for amount in [
            self.ledger.static_instructions,
            self.ledger.dynamic_instructions,
            self.ledger.api_instructions,
            self.ledger.stable_api_instructions,
            self.ledger.heap_page_instructions,
            self.ledger.stable_page_instructions,
            self.ledger.other_known_instructions,
        ] {
            Self::add(
                &mut self.ledger.raw_instructions,
                amount,
                &mut self.truncated,
            );
        }
        let mut blocks: Vec<_> = self.blocks.into_values().collect();
        blocks.sort_by_key(|block| block.block_id);
        let mut dynamic_charges: Vec<_> = self
            .dynamic
            .into_iter()
            .map(
                |((function, offset, ordinal, category), (events, instructions))| {
                    WasmInstructionDynamicCharge {
                        function_index: u32::try_from(function).ok(),
                        body_offset: u64::try_from(offset).ok(),
                        operator_ordinal: u64::try_from(ordinal).ok(),
                        category: match category {
                            0 => "bulk_length",
                            1 => "stable_api",
                            2 => "stable_page",
                            _ => "runtime",
                        }
                        .to_string(),
                        events,
                        instructions,
                    }
                },
            )
            .collect();
        dynamic_charges.sort_by(|a, b| {
            (&a.function_index, &a.body_offset, &a.category).cmp(&(
                &b.function_index,
                &b.body_offset,
                &b.category,
            ))
        });
        let mut api_calls: Vec<_> = self
            .apis
            .into_iter()
            .map(|(api, row)| WasmInstructionApiCharge {
                api: api.to_string(),
                calls: row.calls,
                base_instructions: row.base,
                byte_instructions: row.bytes,
                direct_instructions: row.direct,
            })
            .collect();
        api_calls.sort_by(|a, b| a.api.cmp(&b.api));
        let mut stack_charges: Vec<_> = self
            .stack_charges
            .into_iter()
            .map(
                |((stack_id, origin_id, category, api, evidence), (events, instructions))| {
                    WasmInstructionStackCharge {
                        stack_id,
                        origin_id,
                        category: category.to_string(),
                        api: api.map(str::to_string),
                        events,
                        instructions,
                        origin_evidence: evidence.to_string(),
                    }
                },
            )
            .collect();
        stack_charges.sort_by(|a, b| {
            (&a.stack_id, &a.origin_id, &a.category, &a.api).cmp(&(
                &b.stack_id,
                &b.origin_id,
                &b.category,
                &b.api,
            ))
        });
        let mut rules: Vec<_> = self
            .rules
            .into_iter()
            .map(|(id, (events, instructions))| WasmInstructionRuleCharge {
                rule_id: id.to_string(),
                events,
                instructions,
            })
            .collect();
        rules.sort_by(|a, b| a.rule_id.cmp(&b.rule_id));
        let module_origin_map_truncated = self
            .module
            .as_ref()
            .is_some_and(|metadata| metadata.origin_map_truncated);
        let module_origin_map_unavailable = self.module.is_none();
        WasmInstructionProfile {
            version: 1,
            stacks: self.stacks, stack_charges, stack_details_truncated: self.stack_details_truncated,
            module_origin_map_truncated, module_origin_map_unavailable,
            page_charges: Vec::new(), page_details_truncated: false,
            rules, module: self.module.map(Arc::unwrap_or_clone), blocks, dynamic_charges, api_calls,
            budget_events: self.budget_events.into_iter().map(|event| WasmInstructionBudgetEvent { before: event.before, after: event.after, cause: event.cause.to_string() }).collect(), ledger: self.ledger, truncated: self.truncated,
            limitations: vec![
                "Block totals allocate prepaid IC charges; they do not count executed operators.".to_string(),
                "Stacks contain bounded physical Wasm frames; Rust source and inline frames require matching external artifacts. Unmapped operation origins remain unavailable.".to_string(),
                "Physical attempts retain observed debits across DTS slices; aborted attempts may lack guest memory-page sets. Slice round accounting is separate from this guest ledger.".to_string(),
            ],
        }
    }
}

/// Capture is limited to selected executions and a fixed number of normal
/// callback sites. It never runs from a signal handler.
pub(crate) fn observe_charge_stack(
    caller: &mut Caller<'_, StoreData>,
    category: &'static str,
    api: Option<&'static str>,
    instructions: u64,
    explicit: Option<(u32, u64, u32)>,
    evidence: &'static str,
) {
    let Some(profile) = caller.data().instruction_charge_profile.as_ref() else {
        return;
    };
    let frames = if profile.stack_observations < MAX_STACK_OBSERVATIONS {
        let backtrace = WasmBacktrace::force_capture(&*caller);
        Some(
            backtrace
                .frames()
                .iter()
                .take(MAX_STACK_FRAMES)
                .map(|frame| CapturedFrame {
                    function: frame.func_index(),
                    offset: frame.module_offset().map(|pc| pc as u64),
                    matching_module: profile
                        .runtime_module
                        .as_ref()
                        .is_some_and(|module| Module::same(module, frame.module())),
                })
                .collect(),
        )
    } else {
        None
    };
    if let Some(profile) = &mut caller.data_mut().instruction_charge_profile {
        profile.record_stack_charge(frames, category, api, instructions, explicit, evidence);
    }
}

/// Scope a host copy to its proven current guest caller. A missing origin
/// clears any outer tag for this nested scope instead of retaining a stale one.
pub(crate) fn host_operation_scope(caller: &Caller<'_, StoreData>) -> Option<HostOperationGuard> {
    let profile = caller.data().instruction_charge_profile.as_ref()?;
    let charges = profile.page_charges.as_ref()?;
    let offset = if profile.stack_observations < MAX_STACK_OBSERVATIONS {
        let backtrace = WasmBacktrace::force_capture(caller);
        let frames: Vec<_> = backtrace
            .frames()
            .iter()
            .take(MAX_STACK_FRAMES)
            .map(|frame| CapturedFrame {
                function: frame.func_index(),
                offset: frame.module_offset().map(|pc| pc as u64),
                matching_module: profile
                    .runtime_module
                    .as_ref()
                    .is_some_and(|module| Module::same(module, frame.module())),
            })
            .collect();
        profile.operation_origin(&frames).map(|(_, offset)| offset)
    } else {
        None
    };
    Some(charges.enter_host_operation(offset))
}

pub(crate) fn link_observers(linker: &mut Linker<StoreData>) {
    linker
        .func_wrap(
            "__",
            "instruction_profile_block",
            |mut caller: Caller<'_, StoreData>,
             id: i32,
             function: i32,
             offset: i64,
             end: i64,
             ordinal: i32,
             weight: i64,
             entry: i64| {
                if let Some(profile) = &mut caller.data_mut().instruction_charge_profile {
                    profile.observe_block(id, function, offset, end, ordinal, weight, entry);
                }
                observe_charge_stack(
                    &mut caller,
                    "static_prepaid",
                    None,
                    weight as u64,
                    Some((function as u32, offset as u64, ordinal as u32)),
                    "block",
                );
            },
        )
        .expect("link private block observer");
    linker
        .func_wrap(
            "__",
            "instruction_profile_dynamic",
            |mut caller: Caller<'_, StoreData>,
             amount: i64,
             function: i32,
             offset: i64,
             ordinal: i32,
             category: i32|
             -> i64 {
                if category == 110 {
                    let guard = host_operation_scope(&caller);
                    if let Some(profile) = &mut caller.data_mut().instruction_charge_profile {
                        profile.push_operation_scope(guard);
                    }
                    return amount;
                }
                if category == 111 {
                    if let Some(profile) = &mut caller.data_mut().instruction_charge_profile {
                        profile.pop_operation_scope();
                    }
                    return amount;
                }
                if let Some(profile) = &mut caller.data_mut().instruction_charge_profile {
                    profile.observe_dynamic(amount as u64, function, offset, ordinal, category);
                }
                if category < 100 {
                    let explicit = (function >= 0 && offset >= 0 && ordinal >= 0).then_some((
                        function as u32,
                        offset as u64,
                        ordinal as u32,
                    ));
                    observe_charge_stack(
                        &mut caller,
                        if category == 0 {
                            "bulk_length"
                        } else {
                            "stable_api"
                        },
                        None,
                        amount as u64,
                        explicit,
                        "exact_operation",
                    );
                }
                amount
            },
        )
        .expect("link private dynamic observer");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proven_host_operation_precedes_a_synthetic_native_helper() {
        use ic_interfaces::execution_environment::{
            WasmInstructionModuleMetadata, WasmInstructionOriginRange,
        };
        let metadata = WasmInstructionModuleMetadata {
            instrumented_ranges: vec![
                WasmInstructionOriginRange {
                    instrumented_function_index: 1,
                    start_offset: 10,
                    end_offset: 20,
                    origin_id: None,
                    kind: "stable_helper".into(),
                },
                WasmInstructionOriginRange {
                    instrumented_function_index: 2,
                    start_offset: 30,
                    end_offset: 40,
                    origin_id: Some(7),
                    kind: "original_call".into(),
                },
                WasmInstructionOriginRange {
                    instrumented_function_index: 3,
                    start_offset: 50,
                    end_offset: 60,
                    origin_id: Some(8),
                    kind: "original".into(),
                },
            ],
            ..Default::default()
        };
        assert_eq!(
            select_page_origin(Some(12), Some(32), Some(&metadata)),
            (Some(32), true)
        );
        assert_eq!(
            select_page_origin(Some(52), Some(32), Some(&metadata)),
            (Some(52), false)
        );
        assert_eq!(
            select_page_origin(Some(12), Some(42), Some(&metadata)),
            (Some(12), false)
        );
        assert_eq!(
            select_page_origin(None, Some(32), Some(&metadata)),
            (Some(32), true)
        );
        assert_eq!(
            select_page_origin(None, Some(42), Some(&metadata)),
            (None, false)
        );
        assert_eq!(
            select_page_origin(Some(12), Some(32), None),
            (Some(12), false)
        );
    }

    #[test]
    fn native_pc_mapping_preserves_unknown_intervals_and_text_bounds() {
        let address_map = [(0, Some(42)), (4, None), (8, Some(77))];
        assert_eq!(
            module_offset_from_native_pc(1001, 1000..1012, &address_map),
            Some(42)
        );
        assert_eq!(
            module_offset_from_native_pc(1005, 1000..1012, &address_map),
            None
        );
        assert_eq!(
            module_offset_from_native_pc(1008, 1000..1012, &address_map),
            Some(77)
        );
        assert_eq!(
            module_offset_from_native_pc(999, 1000..1012, &address_map),
            None
        );
        assert_eq!(
            module_offset_from_native_pc(1012, 1000..1012, &address_map),
            None
        );
    }

    #[test]
    fn host_operation_tags_are_scoped_and_unavailable_nested_calls_clear_them() {
        let charges = Arc::new(InstructionPageCharges::default());
        charges.enabled.store(true, Ordering::Relaxed);
        charges.enable_page_details();
        let observe = || {
            charges.observe_page_rule(
                CanisterMemoryType::Heap,
                memory_tracker::PageDebitKind::Access,
                0,
                80_000,
            )
        };
        {
            let _outer = charges.enter_host_operation(Some(42));
            observe();
            {
                let _unknown = charges.enter_host_operation(None);
                observe();
            }
            {
                let _inner = charges.enter_host_operation(Some(77));
                observe();
            }
            observe();
        }
        observe();
        let records = charges.page_records.get().unwrap();
        let tags: Vec<_> = records
            .iter()
            .take(5)
            .map(|record| {
                record
                    .host_operation_active
                    .load(Ordering::Relaxed)
                    .then(|| record.host_module_offset.load(Ordering::Relaxed))
            })
            .collect();
        assert_eq!(tags, vec![Some(42), None, Some(77), Some(42), None]);
    }

    #[test]
    fn page_details_are_bounded_and_restore_nested_fault_context() {
        let charges = InstructionPageCharges::default();
        charges.enabled.store(true, Ordering::Relaxed);
        charges.enable_page_details();
        {
            let _outer = charges.enter_fault(Some(0x1234));
            charges.observe_page_rule(
                CanisterMemoryType::Heap,
                memory_tracker::PageDebitKind::Access,
                7,
                80_000,
            );
            {
                let _inner = charges.enter_fault(Some(0x5678));
                charges.observe_page_rule(
                    CanisterMemoryType::Stable,
                    memory_tracker::PageDebitKind::Dirty,
                    8,
                    80_000,
                );
            }
            charges.observe_page_rule(
                CanisterMemoryType::Heap,
                memory_tracker::PageDebitKind::Dirty,
                7,
                80_000,
            );
        }
        charges.observe_page_rule(
            CanisterMemoryType::Heap,
            memory_tracker::PageDebitKind::Access,
            9,
            80_000,
        );
        let records = charges.page_records.get().unwrap();
        assert_eq!(records[0].native_pc.load(Ordering::Relaxed), 0x1234);
        assert_eq!(records[1].native_pc.load(Ordering::Relaxed), 0x5678);
        assert_eq!(records[2].native_pc.load(Ordering::Relaxed), 0x1234);
        assert_eq!(records[3].native_pc.load(Ordering::Relaxed), 0);
        assert_eq!(records[0].region_index.load(Ordering::Relaxed), 7);
        assert_eq!(records[1].memory.load(Ordering::Relaxed), 1);
        assert_eq!(records[1].kind.load(Ordering::Relaxed), 1);
        for region in 4..=MAX_PAGE_DETAILS {
            charges.observe_page_rule(
                CanisterMemoryType::Heap,
                memory_tracker::PageDebitKind::Access,
                region as u64,
                80_000,
            );
        }
        assert_eq!(records.len(), MAX_PAGE_DETAILS);
        assert!(charges.page_details_truncated.load(Ordering::Relaxed));
        assert!(
            records
                .iter()
                .all(|record| record.ready.load(Ordering::Acquire))
        );
        // Dropping detail does not drop actual debit aggregates.
        assert_eq!(
            charges.rule_events[0].load(Ordering::Relaxed),
            MAX_PAGE_DETAILS as u64 - 1
        );
        assert_eq!(
            charges.rule_instructions[0].load(Ordering::Relaxed),
            (MAX_PAGE_DETAILS as u64 - 1) * 80_000
        );
    }

    #[test]
    fn module_origin_map_availability_does_not_truncate_one_shallow_stack() {
        use ic_interfaces::execution_environment::{
            WasmInstructionModuleMetadata, WasmInstructionOperator, WasmInstructionOriginRange,
        };

        for state in ["complete", "truncated", "unavailable"] {
            let module = (state != "unavailable").then(|| {
                Arc::new(WasmInstructionModuleMetadata {
                    operators: vec![WasmInstructionOperator {
                        origin_id: 0,
                        function_index: 1,
                        body_offset: 9,
                        operator_ordinal: 0,
                        weight: 1,
                    }],
                    instrumented_ranges: vec![WasmInstructionOriginRange {
                        instrumented_function_index: 1,
                        start_offset: 10,
                        end_offset: 20,
                        origin_id: Some(0),
                        kind: "original_call".into(),
                    }],
                    origin_map_truncated: state == "truncated",
                    ..Default::default()
                })
            });
            let mut collector = InstructionChargeProfileCollector::new(module, None, None);
            collector.observe_dynamic(7, 1, 9, 0, 0);
            // For the truncated map, use a coordinate beyond its retained
            // prefix. The physical frame is complete; its original operator
            // must remain unavailable instead of truncating the stack.
            collector.record_stack_charge(
                Some(vec![CapturedFrame {
                    function: 1,
                    offset: Some(if state == "truncated" { 25 } else { 12 }),
                    matching_module: true,
                }]),
                "bulk_length",
                None,
                7,
                None,
                "exact_operation",
            );
            let profile = collector.finish(80_000, 160_000);
            assert_eq!(profile.module_origin_map_truncated, state == "truncated");
            assert_eq!(
                profile.module_origin_map_unavailable,
                state == "unavailable"
            );
            assert!(!profile.stack_details_truncated, "{state}");
            assert!(!profile.truncated, "{state}");
            assert_eq!(profile.stacks.len(), 1);
            assert!(!profile.stacks[0].truncated);
            assert_eq!(profile.stacks[0].frames.len(), 1);
            let expected_origin = (state == "complete").then_some(0);
            assert_eq!(profile.stacks[0].frames[0].origin_id, expected_origin);
            assert_eq!(profile.stack_charges[0].origin_id, expected_origin);
            assert_eq!(profile.stack_charges[0].instructions, 7);
            // Map availability never changes existing instruction/page debits.
            assert_eq!(profile.ledger.dynamic_instructions, 7);
            assert_eq!(profile.ledger.heap_page_instructions, 80_000);
            assert_eq!(profile.ledger.stable_page_instructions, 160_000);
            assert_eq!(profile.ledger.raw_instructions, 240_007);
        }
    }

    #[test]
    fn stack_sampling_and_dictionary_caps_keep_full_charge_totals() {
        use ic_interfaces::execution_environment::{
            WasmInstructionModuleMetadata, WasmInstructionOperator,
        };
        let count = MAX_STACK_OBSERVATIONS + 32;
        let module = Arc::new(WasmInstructionModuleMetadata {
            operators: (0..count)
                .map(|i| WasmInstructionOperator {
                    origin_id: i as u32,
                    function_index: 0,
                    body_offset: i as u64 + 1,
                    operator_ordinal: i as u32,
                    weight: 1,
                })
                .collect(),
            ..Default::default()
        });
        let mut collector = InstructionChargeProfileCollector::new(Some(module), None, None);
        for i in 0..count {
            collector.observe_dynamic(1, 0, i as i64 + 1, i as i32, 0);
            let frames = (i < MAX_STACK_OBSERVATIONS).then(|| {
                vec![CapturedFrame {
                    function: i as u32,
                    offset: Some(i as u64),
                    matching_module: false,
                }]
            });
            collector.record_stack_charge(
                frames,
                "bulk_length",
                None,
                1,
                Some((0, i as u64 + 1, i as u32)),
                "exact_operation",
            );
        }
        let profile = collector.finish(0, 0);
        assert!(profile.stack_details_truncated);
        assert!(!profile.module_origin_map_truncated);
        assert!(!profile.module_origin_map_unavailable);
        assert_eq!(profile.stacks.len(), MAX_STACKS);
        assert!(profile.stack_charges.len() <= MAX_STACK_CHARGES + MAX_APIS);
        assert_eq!(
            profile
                .stack_charges
                .iter()
                .map(|charge| charge.instructions)
                .sum::<u64>(),
            count as u64
        );
        assert_eq!(profile.ledger.raw_instructions, count as u64);
    }

    #[test]
    fn paired_guest_operation_scopes_restore_parent_after_return_trap_and_overflow() {
        let charges = Arc::new(InstructionPageCharges::default());
        let outer = charges.enter_host_operation(Some(10));
        let mut collector =
            InstructionChargeProfileCollector::new(None, None, Some(Arc::clone(&charges)));
        collector.push_operation_scope(Some(charges.enter_host_operation(Some(20))));
        collector.push_operation_scope(Some(charges.enter_host_operation(Some(30))));
        assert_eq!(charges.host_module_offset.load(Ordering::Relaxed), 30);
        collector.pop_operation_scope();
        assert_eq!(charges.host_module_offset.load(Ordering::Relaxed), 20);
        collector.clear_operation_scopes();
        assert_eq!(charges.host_module_offset.load(Ordering::Relaxed), 10);
        for index in 0..=MAX_OPERATION_SCOPES {
            collector
                .push_operation_scope(Some(charges.enter_host_operation(Some(100 + index as u64))));
        }
        assert!(!charges.host_operation_active.load(Ordering::Relaxed));
        collector.pop_operation_scope();
        assert!(charges.host_operation_active.load(Ordering::Relaxed));
        assert_eq!(
            charges.host_module_offset.load(Ordering::Relaxed),
            100 + MAX_OPERATION_SCOPES as u64 - 1
        );
        collector.clear_operation_scopes();
        assert_eq!(charges.host_module_offset.load(Ordering::Relaxed), 10);
        drop(outer);
        assert!(!charges.host_operation_active.load(Ordering::Relaxed));
    }

    #[test]
    fn exact_operation_resolution_never_skips_an_unmapped_or_noncall_frame() {
        use ic_interfaces::execution_environment::{
            WasmInstructionModuleMetadata, WasmInstructionOriginRange,
        };
        let collector = InstructionChargeProfileCollector::new(
            Some(Arc::new(WasmInstructionModuleMetadata {
                instrumented_ranges: vec![
                    WasmInstructionOriginRange {
                        instrumented_function_index: 1,
                        start_offset: 1,
                        end_offset: 2,
                        origin_id: Some(0),
                        kind: "original_call".to_string(),
                    },
                    WasmInstructionOriginRange {
                        instrumented_function_index: 2,
                        start_offset: 3,
                        end_offset: 4,
                        origin_id: None,
                        kind: "runtime_helper".to_string(),
                    },
                    WasmInstructionOriginRange {
                        instrumented_function_index: 3,
                        start_offset: 5,
                        end_offset: 6,
                        origin_id: Some(1),
                        kind: "original".to_string(),
                    },
                ],
                ..Default::default()
            })),
            None,
            None,
        );
        let caller = CapturedFrame {
            function: 1,
            offset: Some(1),
            matching_module: true,
        };
        assert_eq!(
            collector.operation_origin(&[
                CapturedFrame {
                    function: 2,
                    offset: Some(3),
                    matching_module: true
                },
                caller
            ]),
            Some((0, 1))
        );
        for first in [
            CapturedFrame {
                function: 9,
                offset: None,
                matching_module: true,
            },
            CapturedFrame {
                function: 1,
                offset: Some(2),
                matching_module: true,
            },
            CapturedFrame {
                function: 3,
                offset: Some(5),
                matching_module: true,
            },
            CapturedFrame {
                function: 1,
                offset: Some(1),
                matching_module: false,
            },
        ] {
            assert_eq!(collector.operation_origin(&[first, caller]), None);
        }
    }

    #[test]
    fn detail_limits_preserve_observed_category_totals() {
        let mut collector = InstructionChargeProfileCollector::default();
        // A sparse high module-global ID must be retained before the cap.
        collector.observe_block(70_000, 0, 0, 1, 0, 7, 1);
        for id in 0..MAX_BLOCKS {
            collector.observe_block(id as i32, 0, 0, 1, 0, 2, 1);
        }
        // Existing rows keep accumulating after new detail has been limited.
        collector.observe_block(70_000, 0, 0, 1, 0, 7, 1);
        for origin in 0..=MAX_DYNAMIC {
            collector.observe_dynamic(3, 0, origin as i64, origin as i32, 0);
        }
        let profile = collector.finish(11, 13);
        assert!(profile.truncated);
        assert_eq!(profile.blocks.len(), MAX_BLOCKS);
        let high_block = profile
            .blocks
            .iter()
            .find(|row| row.block_id == 70_000)
            .unwrap();
        assert_eq!(high_block.hits, 2);
        assert_eq!(high_block.instructions, 14);
        assert_eq!(profile.dynamic_charges.len(), MAX_DYNAMIC);
        assert_eq!(
            profile.ledger.static_instructions,
            14 + MAX_BLOCKS as u64 * 2
        );
        assert_eq!(
            profile.ledger.dynamic_instructions,
            (MAX_DYNAMIC as u64 + 1) * 3
        );
        assert_eq!(profile.ledger.heap_page_instructions, 11);
        assert_eq!(profile.ledger.stable_page_instructions, 13);
        assert_eq!(
            profile.ledger.raw_instructions,
            38 + MAX_BLOCKS as u64 * 2 + (MAX_DYNAMIC as u64 + 1) * 3
        );
    }

    #[test]
    fn overflow_is_visible_and_cannot_wrap_a_counter() {
        let mut collector = InstructionChargeProfileCollector::default();
        collector.observe_dynamic(u64::MAX, -1, -1, -1, 1);
        collector.observe_dynamic(1, -1, -1, -1, 1);
        let profile = collector.finish(0, 0);
        assert!(profile.truncated);
        assert_eq!(profile.ledger.raw_instructions, u64::MAX);
        assert_eq!(profile.ledger.stable_api_instructions, u64::MAX);
        assert_eq!(profile.dynamic_charges[0].function_index, None);
        assert_eq!(profile.dynamic_charges[0].instructions, u64::MAX);
    }
}

#[cfg(test)]
mod read_mask_tests {
    use super::MemoryReadProfileCollector;

    #[test]
    fn read_mask_overlapping_reads_preserve_width_frequency_and_stable_paths() {
        let collector = MemoryReadProfileCollector::new();
        collector.observe_read(0, 10, 8, 0);
        collector.observe_read(0, 10, 8, 0);
        collector.observe_read(0, 14, 8, 3);
        collector.observe_read(1, 10, 4, 1);
        collector.observe_read(1, 12, 4, 2);
        collector.observe_read(0, 42, 0, 0);
        let profile = collector.finish();
        assert_eq!(profile.observed_reads, 5);
        assert_eq!(profile.observed_bytes, 32);
        assert!(!profile.truncated);
        let runs: Vec<_> = profile
            .ranges
            .iter()
            .map(|r| {
                (
                    r.memory.as_str(),
                    r.start,
                    r.end,
                    r.heap_reads,
                    r.stable_direct_reads,
                    r.stable_native_reads,
                )
            })
            .collect();
        assert_eq!(
            runs,
            vec![
                ("heap", 10, 14, 2, 0, 0),
                ("heap", 14, 18, 3, 0, 0),
                ("heap", 18, 22, 1, 0, 0),
                ("stable", 10, 12, 0, 1, 0),
                ("stable", 12, 14, 0, 1, 1),
                ("stable", 14, 16, 0, 0, 1),
            ]
        );
    }

    #[test]
    fn read_mask_bounds_unique_ranges_and_export_without_dense_allocation() {
        let collector = MemoryReadProfileCollector::with_limits(2, 1);
        collector.observe_read(1, 1_u64 << 48, 1_u64 << 32, 1);
        collector.observe_read(1, 1_u64 << 48, 1_u64 << 32, 1);
        collector.observe_read(0, 0, 8, 0);
        collector.observe_read(0, 16, 8, 0);
        let profile = collector.finish();
        assert_eq!(profile.observed_reads, 4);
        assert!(profile.truncated);
        assert_eq!(profile.ranges.len(), 1);
        assert_eq!(profile.max_unique_reads, 2);
        assert_eq!(profile.max_export_ranges, 1);
    }

    #[test]
    fn read_mask_saturation_and_invalid_addresses_are_declared() {
        let collector = MemoryReadProfileCollector::new();
        collector.observe_read(0, 0, u64::MAX, 0);
        collector.observe_read(0, 0, 1, 0);
        collector.observe_read(0, u64::MAX, 1, 0);
        let profile = collector.finish();
        assert!(profile.counters_saturated);
        assert!(profile.truncated);
        assert_eq!(profile.observed_bytes, u64::MAX);
        assert_eq!(profile.ranges[0].heap_reads, 2);
    }
}

#[cfg(test)]
mod write_mask_tests {
    use super::MemoryWriteProfileCollector;

    #[test]
    fn write_mask_repeated_overlapping_writes_preserve_width_and_frequency() {
        let collector = MemoryWriteProfileCollector::new();
        collector.observe_write(0, 10, 8, 0);
        collector.observe_write(0, 10, 8, 0);
        collector.observe_write(0, 14, 8, 3);
        collector.observe_write(1, 10, 4, 1);
        collector.observe_write(1, 12, 4, 1);
        collector.observe_write(0, 42, 0, 0);
        let profile = collector.finish();
        assert_eq!(profile.observed_writes, 5);
        assert_eq!(profile.observed_bytes, 32);
        assert!(!profile.truncated);
        let runs: Vec<_> = profile
            .ranges
            .iter()
            .map(|r| {
                (
                    r.memory.as_str(),
                    r.start,
                    r.end,
                    r.heap_writes,
                    r.stable_writes,
                )
            })
            .collect();
        assert_eq!(
            runs,
            vec![
                ("heap", 10, 14, 2, 0),
                ("heap", 14, 18, 3, 0),
                ("heap", 18, 22, 1, 0),
                ("stable", 10, 12, 0, 1),
                ("stable", 12, 14, 0, 2),
                ("stable", 14, 16, 0, 1),
            ]
        );
    }

    #[test]
    fn write_mask_bounds_sparse_ranges_and_export_without_dense_allocation() {
        let collector = MemoryWriteProfileCollector::with_limits(2, 1);
        collector.observe_write(1, 1_u64 << 48, 1_u64 << 32, 1);
        collector.observe_write(1, 1_u64 << 48, 1_u64 << 32, 1);
        collector.observe_write(0, 0, 8, 0);
        collector.observe_write(0, 16, 8, 0);
        let profile = collector.finish();
        assert_eq!(profile.observed_writes, 4);
        assert!(profile.truncated);
        assert_eq!(profile.ranges.len(), 1);
        assert_eq!(profile.max_unique_writes, 2);
        assert_eq!(profile.max_export_ranges, 1);
    }

    #[test]
    fn write_mask_saturation_and_invalid_addresses_are_declared() {
        let collector = MemoryWriteProfileCollector::new();
        collector.observe_write(0, 0, u64::MAX, 0);
        collector.observe_write(0, 0, 1, 0);
        collector.observe_write(0, u64::MAX, 1, 0);
        let profile = collector.finish();
        assert!(profile.counters_saturated);
        assert!(profile.truncated);
        assert_eq!(profile.observed_bytes, u64::MAX);
        assert_eq!(profile.ranges[0].heap_writes, 2);
    }
}
