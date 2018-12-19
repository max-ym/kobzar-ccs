use std::collections::BTreeMap;
use std::collections::btree_map::Iter;

/// Type used to identify unique threads.
pub type Key = u32;

/// Thread related metadata. Does not contain architecture-specific
/// information. This information is stored in another struct implemented
/// by the OS core threading module that is connected to this structure
/// instance.
pub struct Thread {
    // TODO
}

/// Thread set. Allows to add, remove and search for threads.
pub struct Set {
    map: BTreeMap<Key, Thread>,

    last_key: Key,
}

impl Set {

    /// Create new empty set.
    pub fn new() -> Self {
        Set::default()
    }

    /// Add new thread to the set and return it's key.
    pub fn add(&mut self, thread: Thread) -> Key {
        let new_key = self.generate_new_key();
        self.map.insert(new_key, thread);
        new_key
    }

    /// Generate new unique key for storing new thread.
    fn generate_new_key(&mut self) -> Key {
        let new_key = self.last_key + 1;
        self.last_key = new_key;
        new_key
    }

    /// Remove thread from the set. Returns removed thread on success and
    /// none if thread was not found.
    pub fn remove(&mut self, key: Key) -> Option<Thread> {
        self.map.remove(&key)
    }

    /// Get thread by it's key.
    pub fn get(&mut self, key: Key) -> Option<&Thread> {
        self.map.get(&key)
    }

    /// Get thread by it's key.
    pub fn get_mut(&mut self, key: Key) -> Option<&mut Thread> {
        self.map.get_mut(&key)
    }

    /// Iterator over elements of the set.
    pub fn iter(&self) -> Iter<Key, Thread> {
        self.map.iter()
    }

    /// Access the internal map of the structure.
    pub fn map(&self) -> &BTreeMap<Key, Thread> {
        &self.map
    }
}

impl Default for Set {

    fn default() -> Self {
        Set {
            map: Default::default(),
            last_key: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set() {
        let mut set = Set::new();
        let thr1 = Thread {
        };
        let thr2 = Thread {
        };

        let k1 = set.add(thr1);
        let k2 = set.add(thr2);

        { let thr1 = set.get(k1).unwrap(); }
        { let thr2 = set.get(k2).unwrap(); }

        assert!(set.get(k2 + 1).is_none());

        set.remove(k2);
        assert!(set.get(k2).is_none());
    }
}
