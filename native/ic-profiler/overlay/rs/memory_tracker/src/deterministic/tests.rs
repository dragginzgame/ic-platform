use super::*;
use ic_logger::replica_logger::no_op_logger;
use nix::sys::mman::{MapFlags, mmap_anonymous, munmap};
use std::num::NonZeroUsize;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicU64, AtomicUsize};

const PAGE_OVERHEAD: u64 = 5_000;
const REGION_INSTRUCTIONS: u64 = WASM_PAGE_SIZE_IN_BYTES as u64 / PAGE_SIZE as u64 * PAGE_OVERHEAD;

struct TestMemory {
    start: NonNull<libc::c_void>,
    size: usize,
}

impl Drop for TestMemory {
    fn drop(&mut self) {
        // SAFETY: This mapping was allocated by `new_tracker` and is no longer used.
        unsafe { munmap(self.start, self.size).unwrap() };
    }
}

fn new_tracker(
    dirty_page_tracking: DirtyPageTracking,
    debited: Arc<AtomicU64>,
) -> (DeterministicMemoryTracker, TestMemory) {
    let size = 3 * WASM_PAGE_SIZE_IN_BYTES;
    // SAFETY: The anonymous mapping has a nonzero page-aligned size.
    let start = unsafe {
        mmap_anonymous(
            None,
            NonZeroUsize::new(size).unwrap(),
            ProtFlags::PROT_NONE,
            MapFlags::MAP_PRIVATE,
        )
        .unwrap()
    };
    let tracker = DeterministicMemoryTracker::new(
        start.as_ptr(),
        NumBytes::new(size as u64),
        no_op_logger(),
        dirty_page_tracking,
        PageMap::new_for_testing(),
        MemoryLimits {
            max_memory_size: NumBytes::new(size as u64),
            max_dirty_pages: NumOsPages::new((size / PAGE_SIZE) as u64),
        },
        PAGE_OVERHEAD,
        Arc::new(SignalMutex::new(move |instructions| {
            debited.fetch_add(instructions, Ordering::Relaxed);
        })),
    )
    .unwrap();
    (tracker, TestMemory { start, size })
}

/// Preallocated storage keeps the observer itself allocation-free and lock-free.
#[derive(Default)]
struct DebitLog {
    count: AtomicUsize,
    kinds: [AtomicU64; 4],
    regions: [AtomicU64; 4],
    instructions: [AtomicU64; 4],
    debited_before_observation: [AtomicU64; 4],
}

impl DebitLog {
    fn record(&self, kind: PageDebitKind, region: u64, instructions: u64, debited: u64) {
        let index = self.count.fetch_add(1, Ordering::Relaxed);
        if index < self.kinds.len() {
            self.kinds[index].store(
                match kind {
                    PageDebitKind::Access => 0,
                    PageDebitKind::Dirty => 1,
                },
                Ordering::Relaxed,
            );
            self.regions[index].store(region, Ordering::Relaxed);
            self.instructions[index].store(instructions, Ordering::Relaxed);
            self.debited_before_observation[index].store(debited, Ordering::Relaxed);
        }
    }

    fn events(&self) -> Vec<(PageDebitKind, u64, u64, u64)> {
        let count = self.count.load(Ordering::Relaxed);
        assert!(
            count <= self.kinds.len(),
            "too many page debit observations"
        );
        (0..count)
            .map(|index| {
                (
                    match self.kinds[index].load(Ordering::Relaxed) {
                        0 => PageDebitKind::Access,
                        1 => PageDebitKind::Dirty,
                        _ => unreachable!(),
                    },
                    self.regions[index].load(Ordering::Relaxed),
                    self.instructions[index].load(Ordering::Relaxed),
                    self.debited_before_observation[index].load(Ordering::Relaxed),
                )
            })
            .collect()
    }
}

#[test]
fn page_observer_reports_actual_access_and_dirty_debits_once_per_region() {
    let debited = Arc::new(AtomicU64::new(0));
    let log = Arc::new(DebitLog::default());
    let (mut tracker, memory) = new_tracker(DirtyPageTracking::Track, debited.clone());
    let observer_log = log.clone();
    let observer_debited = debited.clone();
    tracker.set_page_debit_observer(Arc::new(move |kind, region, instructions| {
        observer_log.record(
            kind,
            region,
            instructions,
            observer_debited.load(Ordering::Relaxed),
        );
    }));

    let base = memory.start.as_ptr().cast::<u8>();
    // SAFETY: These addresses are inside the three-region test mapping.
    let read_address = unsafe { base.add(WASM_PAGE_SIZE_IN_BYTES + 3 * PAGE_SIZE + 17) };
    assert!(tracker.handle_sigsegv(Some(AccessKind::Read), read_address.cast()));
    // SAFETY: The handler mapped all of region 1 as readable. Repeated real
    // accesses to different OS pages must not incur another region debit.
    unsafe {
        for offset in (WASM_PAGE_SIZE_IN_BYTES..2 * WASM_PAGE_SIZE_IN_BYTES).step_by(PAGE_SIZE) {
            assert_eq!(std::ptr::read_volatile(base.add(offset)), 0);
        }
    }
    assert_eq!(debited.load(Ordering::Relaxed), REGION_INSTRUCTIONS);

    assert!(tracker.handle_sigsegv(Some(AccessKind::Write), read_address.cast()));
    // SAFETY: Region 1 is now writable. Repeated writes need no new debit.
    unsafe {
        for offset in (WASM_PAGE_SIZE_IN_BYTES..2 * WASM_PAGE_SIZE_IN_BYTES).step_by(PAGE_SIZE) {
            std::ptr::write_volatile(base.add(offset), 1);
        }
    }
    assert_eq!(debited.load(Ordering::Relaxed), 2 * REGION_INSTRUCTIONS);

    // A first write in a different region pays both the access and dirty debit.
    // SAFETY: This address is inside region 2 of the mapping.
    let write_address = unsafe { base.add(2 * WASM_PAGE_SIZE_IN_BYTES + PAGE_SIZE + 1) };
    assert!(tracker.handle_sigsegv(Some(AccessKind::Write), write_address.cast()));
    // SAFETY: The first-write handler mapped region 2 as writable.
    unsafe { std::ptr::write_volatile(write_address, 2) };
    assert_eq!(debited.load(Ordering::Relaxed), 4 * REGION_INSTRUCTIONS);
    assert_eq!(
        log.events(),
        vec![
            (
                PageDebitKind::Access,
                1,
                REGION_INSTRUCTIONS,
                REGION_INSTRUCTIONS
            ),
            (
                PageDebitKind::Dirty,
                1,
                REGION_INSTRUCTIONS,
                2 * REGION_INSTRUCTIONS
            ),
            (
                PageDebitKind::Access,
                2,
                REGION_INSTRUCTIONS,
                3 * REGION_INSTRUCTIONS
            ),
            (
                PageDebitKind::Dirty,
                2,
                REGION_INSTRUCTIONS,
                4 * REGION_INSTRUCTIONS
            ),
        ]
    );
    assert_eq!(
        tracker.num_accessed_pages(),
        2 * WASM_PAGE_SIZE_IN_BYTES / PAGE_SIZE
    );
    assert_eq!(
        tracker.take_dirty_pages().len(),
        2 * WASM_PAGE_SIZE_IN_BYTES / PAGE_SIZE
    );
}

#[test]
fn page_observer_does_not_add_dirty_debits_when_dirty_tracking_is_ignored() {
    let debited = Arc::new(AtomicU64::new(0));
    let log = Arc::new(DebitLog::default());
    let (mut tracker, memory) = new_tracker(DirtyPageTracking::Ignore, debited.clone());
    let observer_log = log.clone();
    let observer_debited = debited.clone();
    tracker.set_page_debit_observer(Arc::new(move |kind, region, instructions| {
        observer_log.record(
            kind,
            region,
            instructions,
            observer_debited.load(Ordering::Relaxed),
        );
    }));
    assert!(tracker.handle_sigsegv(Some(AccessKind::Write), memory.start.as_ptr()));
    // SAFETY: Ignoring dirty tracking maps the entire accessed region writable.
    unsafe {
        std::ptr::write_volatile(memory.start.as_ptr().cast::<u8>(), 1);
        std::ptr::write_volatile(memory.start.as_ptr().cast::<u8>().add(PAGE_SIZE), 2);
    }
    assert_eq!(debited.load(Ordering::Relaxed), REGION_INSTRUCTIONS);
    assert_eq!(
        log.events(),
        vec![(
            PageDebitKind::Access,
            0,
            REGION_INSTRUCTIONS,
            REGION_INSTRUCTIONS
        )]
    );
    assert!(tracker.take_dirty_pages().is_empty());
}
