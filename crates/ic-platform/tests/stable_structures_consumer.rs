//! Platform-only substrate consumer; raw backing is confined to isolated tests.
use ic_platform::stable_structures::{
    self, BTreeMap, BTreeSet, Cell, Log, Memory, MinHeap, RestrictedMemory, Vec as StableVec,
    VectorMemory,
};

/// Compile every collection with the caller's memory and retain its bytes on reopen.
pub fn collection_contract<M: Memory + Clone>(memory: [M; 8]) {
    // VectorMemory retains the same target-neutral, clone-shared byte backing.
    let vector = VectorMemory::default();
    let shared = vector.clone();
    assert_eq!(vector.grow(1), 0);
    assert_eq!(shared.size(), 1);
    vector.write(65_532, &[1, 2, 3, 4]);
    let mut bytes = [0; 4];
    shared.read(65_532, &mut bytes);
    assert_eq!(bytes, [1, 2, 3, 4]);
    let mut map = BTreeMap::<u64, u64, _>::init(memory[0].clone());
    assert_eq!(map.insert(1, 2), None);
    let entries: stable_structures::btreemap::Iter<'_, u64, u64, M> = map.iter();
    assert_eq!(
        entries
            .map(|entry| entry.into_pair())
            .collect::<std::vec::Vec<_>>(),
        [(1, 2)]
    );

    let mut set = BTreeSet::<u64, _>::init(memory[1].clone());
    assert!(set.insert(7));
    let entries: stable_structures::btreeset::Iter<'_, u64, M> = set.iter();
    assert_eq!(entries.collect::<std::vec::Vec<_>>(), [7]);
    let mut cell = Cell::init(memory[2].clone(), 0_u64);
    assert_eq!(cell.set(3), 0);
    let vector = StableVec::<u64, _>::init(memory[3].clone());
    vector.push(&4);
    let mut heap = MinHeap::<u64, _>::init(memory[4].clone());
    heap.push(&5);
    let log = Log::<u64, _, _>::init(memory[5].clone(), memory[6].clone());
    let append: Result<u64, stable_structures::log::WriteError> = log.append(&6);
    assert_eq!(append.unwrap(), 0);
    let entries: stable_structures::log::Iter<'_, u64, M, M> = log.iter();
    assert_eq!(entries.collect::<std::vec::Vec<_>>(), [6]);
    let region = RestrictedMemory::new(memory[7].clone(), 1..4);
    let mut region_map = BTreeMap::<u64, u64, _>::init(region);
    region_map.insert(8, 9);
    drop((map, set, cell, vector, heap, log, region_map));

    assert_eq!(
        BTreeMap::<u64, u64, _>::load(memory[0].clone()).get(&1),
        Some(2)
    );
    assert!(BTreeSet::<u64, _>::load(memory[1].clone()).contains(&7));
    assert_eq!(*Cell::init(memory[2].clone(), 0_u64).get(), 3);
    assert_eq!(StableVec::<u64, _>::init(memory[3].clone()).get(0), Some(4));
    assert_eq!(MinHeap::<u64, _>::init(memory[4].clone()).pop(), Some(5));
    assert_eq!(
        Log::<u64, _, _>::init(memory[5].clone(), memory[6].clone()).get(0),
        Some(6)
    );
    assert_eq!(
        BTreeMap::<u64, u64, _>::load(RestrictedMemory::new(memory[7].clone(), 1..4)).get(&8),
        Some(9)
    );
}

#[cfg(not(target_family = "wasm"))]
#[test]
fn collections_reopen_on_original_backing() {
    collection_contract(std::array::from_fn(|_| VectorMemory::default()));
}
