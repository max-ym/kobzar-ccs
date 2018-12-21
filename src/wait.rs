use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ThreadKey,
    ChannelKey,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WaitDependency {

    /// Thread that waits for a signal.
    waiter: ThreadKey,

    /// Source from which the signal is expected.
    signal_source: ChannelKey,
}

/// Map that contains all awaiting threads.
#[derive(Default)]
pub struct WaitMap {

    /// Map contains channel key. This key identifies the channel that
    /// has waiters.
    map: BTreeMap<ChannelKey, BTreeSet<ThreadKey>>,

    /// This map connects each single thread with a channel where it
    /// waits.
    thr: BTreeMap<ThreadKey, BTreeSet<ChannelKey>>,
}

impl WaitDependency {

    /// Create new dependency based on thread that awaits
    /// the source signal.
    pub fn new(waiter: ThreadKey, signal_source: ChannelKey) -> Self {
        WaitDependency {
            waiter,
            signal_source,
        }
    }

    /// The thread that waits for the signal.
    pub fn waiter(&self) -> &ThreadKey {
        &self.waiter
    }

    /// Source from which the signal is expected.
    pub fn signal_source(&self) -> &ChannelKey {
        &self.signal_source
    }
}

impl WaitMap {

    /// Create new wait map.
    pub fn new() -> Self {
        Default::default()
    }

    /// Add new channel that has waiters. Returns false if the channel
    /// is already present and the existing channel is not changed.
    pub fn add_channel(&mut self, key: ChannelKey, waiters: BTreeSet<ThreadKey>)
            -> bool {
        let present = self.map.contains_key(&key);

        // Add to channel map.
        let added = if present {
            false
        } else {
            self.map.insert(key, waiters.clone());
            true
        };

        // Add to thread map.
        for thread in waiters.iter() {
            // Get set that contains channels connected to thread.
            let set = self.thr.get_mut(&thread);
            let set = if set.is_none() {
                let btreeset = BTreeSet::new();
                self.thr.insert(thread.clone(), btreeset);
                self.thr.get_mut(&thread).unwrap()
            } else {
                set.unwrap()
            };

            // Save this channel as one that thread is connected to.
            set.insert(key.clone());
        }

        added
    }

    /// Add new waiter to registered channel. Returns false if channel
    /// is not registered. It gets registered and waiter is added.
    pub fn add_waiter(&mut self, key: ChannelKey, waiter: ThreadKey) -> bool {
        // Remove from channel map.
        let success = if self.map.contains_key(&key) {
            self.map.get_mut(&key).unwrap().insert(waiter.clone());
            true
        } else {
            let mut waiters = BTreeSet::new();
            waiters.insert(waiter);
            self.map.insert(key, waiters);
            false
        };

        if !success {
            return false;
        }

        self.thr.get_mut(&waiter).unwrap().insert(key);

        true
    }

    /// Remove waiter from the channel. If channel gets zero waiters, it gets
    /// removed from the map. Returns true if waiter was found and false
    /// otherwise.
    pub fn remove_waiter(&mut self, key: ChannelKey, waiter: ThreadKey) -> bool {
        let success = if self.map.contains_key(&key) {
            let (present, set_is_empty) = {
                let set = self.map.get_mut(&key).unwrap();
                let present = set.remove(&waiter);

                (present, set.is_empty())
            };

            if set_is_empty {
                self.map.remove(&key);
            }
            present
        } else {
            false
        };

        if !success {
            return false;
        }

        self.thr.get_mut(&waiter).unwrap().remove(&key);

        true
    }

    /// Map that holds all wait dependencies of the channel.
    pub fn channel_wait_map(&self) -> &BTreeMap<ChannelKey, BTreeSet<ThreadKey>> {
        &self.map
    }

    /// Map that connects each thread with a channel for which it waits.
    pub fn thread_wait_map(&self) -> &BTreeMap<ThreadKey, BTreeSet<ChannelKey>> {
        &self.thr
    }
}
