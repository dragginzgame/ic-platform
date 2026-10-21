#[test]
fn collections_and_traits_keep_original_identities() {
    macro_rules! identity {
        ($kind:ty, $upstream:ty) => {
            let original: Option<$upstream> = None;
            let boundary: Option<$kind> = original;
            let _: Option<$upstream> = boundary;
        };
    }
    use crate::stable_structures as boundary;
    use boundary::DefaultMemoryImpl as Backing;
    use ic_stable_structures as original;
    identity!(Backing, original::DefaultMemoryImpl);
    identity!(boundary::VectorMemory, original::VectorMemory);
    identity!(boundary::BTreeMap<u64, u64, Backing>, original::BTreeMap<u64, u64, Backing>);
    identity!(boundary::BTreeSet<u64, Backing>, original::BTreeSet<u64, Backing>);
    identity!(boundary::Cell<u64, Backing>, original::Cell<u64, Backing>);
    identity!(boundary::Vec<u64, Backing>, original::Vec<u64, Backing>);
    identity!(boundary::MinHeap<u64, Backing>, original::MinHeap<u64, Backing>);
    identity!(boundary::Log<u64, Backing, Backing>, original::Log<u64, Backing, Backing>);
    identity!(
        boundary::RestrictedMemory<Backing>,
        original::RestrictedMemory<Backing>
    );
    identity!(
        boundary::btreemap::Iter<'static, u64, u64, Backing>,
        original::btreemap::Iter<'static, u64, u64, Backing>
    );
    identity!(
        boundary::btreeset::Iter<'static, u64, Backing>,
        original::btreeset::Iter<'static, u64, Backing>
    );
    identity!(
        boundary::log::Iter<'static, u64, Backing, Backing>,
        original::log::Iter<'static, u64, Backing, Backing>
    );
    identity!(boundary::log::WriteError, original::log::WriteError);
    identity!(boundary::storable::Bound, original::storable::Bound);
    identity!(boundary::storable::Blob<8>, original::storable::Blob<8>);
    identity!(
        boundary::storable::BoundedVecN<8>,
        original::storable::BoundedVecN<8>
    );
    identity!(
        boundary::storable::UnboundedVecN<8>,
        original::storable::UnboundedVecN<8>
    );
    identity!(
        boundary::storable::TryFromSliceError,
        original::storable::TryFromSliceError
    );
    let backing = Backing::default();
    let _: &dyn boundary::Memory = &backing as &dyn original::Memory;
    fn requires_original<T: original::Storable>(_: &T) {}
    fn preserves_storable<T: boundary::storable::Storable>(value: &T) {
        requires_original(value);
    }
    preserves_storable(&0_u64);
}
