//! Public upstream identity and memory-reopen contract for Storage consumers.
#![cfg(feature = "stable-structures")]

use ic_platform::stable_structures::{
    BTreeMap, BTreeSet, Cell, Log, Memory, MinHeap, Storable, Vec, VectorMemory, btreemap,
    btreeset, log, storable,
};

#[test]
fn upstream_identity_and_reopened_memory() {
    fn upstream_memory(memory: &impl ic_stable_structures::Memory) -> u64 {
        memory.size()
    }

    fn storable_contract<T: Storable>() -> storable::Bound {
        T::BOUND
    }
    assert!(matches!(
        storable_contract::<u64>(),
        storable::Bound::Bounded { .. }
    ));
    let memory = VectorMemory::default();
    let mut map = BTreeMap::<u64, u64, _>::init(memory.clone());
    map.insert(7, 42);
    let _: btreemap::Iter<'_, u64, u64, VectorMemory> = map.iter();
    let upstream: ic_stable_structures::BTreeMap<u64, u64, _> = map;
    drop(upstream);
    assert_eq!(upstream_memory(&memory), memory.size());
    assert_eq!(BTreeMap::<u64, u64, _>::init(memory).get(&7), Some(42));

    let mut set = BTreeSet::<u64, _>::init(VectorMemory::default());
    set.insert(9);
    let iter: btreeset::Iter<'_, u64, VectorMemory> = set.iter();
    assert_eq!(iter.collect::<std::vec::Vec<_>>(), [9]);
    let memory = VectorMemory::default();
    let mut cell = Cell::init(memory.clone(), 3_u64);
    cell.set(8);
    drop(cell);
    assert_eq!(*Cell::init(memory, 0_u64).get(), 8);
    let vector = Vec::<u64, _>::init(VectorMemory::default());
    vector.push(&11);
    assert_eq!(vector.get(0), Some(11));
    let mut heap = MinHeap::<u64, _>::init(VectorMemory::default());
    heap.push(&5);
    assert_eq!(heap.pop(), Some(5));
    let log = Log::<u64, _, _>::init(VectorMemory::default(), VectorMemory::default());
    let append: Result<u64, log::WriteError> = log.append(&13);
    assert_eq!(append.unwrap(), 0);
    let iter: log::Iter<'_, u64, VectorMemory, VectorMemory> = log.iter();
    assert_eq!(iter.collect::<std::vec::Vec<_>>(), [13]);
}
