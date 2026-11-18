//! Non-recycled callback identities; values are returned before user destruction.

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

pub(super) struct Registry<T> {
    last: u32,
    entries: BTreeMap<u32, T>,
}

impl<T> Registry<T> {
    pub(super) const fn new() -> Self {
        Self {
            last: 0,
            entries: BTreeMap::new(),
        }
    }

    // The caller retains ownership on exhaustion and drops it outside any
    // enclosing registry borrow. Removal never reuses a previously issued id.
    pub(super) fn insert(&mut self, value: T) -> Result<u32, T> {
        let Some(id) = self.last.checked_add(1) else {
            return Err(value);
        };
        self.last = id;
        let previous = self.entries.insert(id, value);
        assert!(previous.is_none(), "callback id reused");
        Ok(id)
    }

    #[cfg(ic_platform_qualification_accounting)]
    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(super) fn take(&mut self, id: u32) -> Option<T> {
        self.entries.remove(&id)
    }
}
