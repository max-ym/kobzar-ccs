use super::{
    ThreadKey,
};

use std::collections::{BTreeSet, BTreeMap};

/// Channel identifier.
pub type Key = u32;

/// The channel-related information.
pub struct Channel {

    /// Participants in channel transactions.
    participants: BTreeSet<ThreadKey>,
}

/// Set that contains all channels.
pub struct ChannelSet {
    map: BTreeMap<Key, Channel>,
}

pub struct WaitDependency {

    /// Thread that waits for a signal.
    waiter: ThreadKey,

    /// Source from which the signal is expected.
    signal_source: Key,
}

/// Map that contains all awaiting threads.
pub struct WaitMap {

    /// Map contains channel key. This key identifies the channel that
    /// has waiters.
    map: BTreeMap<Key, BTreeSet<ThreadKey>>,
}

impl Channel {

    /// Create new channel with only given thread in it.
    pub fn new(creator: ThreadKey) -> Channel {
        let mut participants = BTreeSet::default();

        participants.insert(creator);

        Channel {
            participants
        }
    }

    /// Set of all participants.
    pub fn participants(&self) -> &BTreeSet<ThreadKey> {
        &self.participants
    }

    /// Try adding participant. If it is already present, false is returned.
    pub fn add_participant(&mut self, thread: ThreadKey) -> bool {
        let present = self.participants.insert(thread);
        present
    }

    /// Remove participant from the channel. If it was present, true is
    /// returned.
    pub fn remove_participant(&mut self, thread: ThreadKey) -> bool {
        let present = self.participants.remove(&thread);
        present
    }
}

impl ChannelSet {

    /// Create new empty channel set.
    pub fn new() -> Self {
        Default::default()
    }

    /// Add new channel to the set. If it is already present, existing channel
    /// is not modified and true is returned. False otherwise.
    pub fn add_channel(&mut self, key: Key, channel: Channel) -> bool {
        if self.map.contains_key(&key) {
            true
        } else {
            self.map.insert(key, channel);
            false
        }
    }

    /// Remove existing channel from the set. If it exists, true is returned
    /// and false otherwise.
    pub fn remove_channel(&mut self, key: Key) -> bool {
        self.map.remove(&key).is_some()
    }

    /// Channel in the set by the key.
    pub fn channel(&self, key: Key) -> Option<&Channel> {
        self.map.get(&key)
    }

    /// Channel in the set by the key.
    pub fn channel_mut(&mut self, key: Key) -> Option<&mut Channel> {
        self.map.get_mut(&key)
    }
}

impl Default for ChannelSet {

    fn default() -> Self {
        ChannelSet {
            map: Default::default(),
        }
    }
}

impl WaitDependency {

    /// Create new dependency based on thread that awaits
    /// the source signal.
    pub fn new(waiter: ThreadKey, signal_source: Key) -> Self {
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
    pub fn signal_source(&self) -> &Key {
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
    pub fn add_channel(&mut self, key: Key, waiters: BTreeSet<ThreadKey>)
            -> bool {
        let present = self.map.contains_key(&key);
        if present {
            false
        } else {
            self.map.insert(key, waiters);
            true
        }
    }

    /// Add new waiter to registered channel. Returns false if channel
    /// is not registered. It gets registered and waiter is added.
    pub fn add_waiter(&mut self, key: Key, waiter: ThreadKey) -> bool {
        if self.map.contains_key(&key) {
            self.map.get_mut(&key).unwrap().insert(waiter);
            true
        } else {
            let mut waiters = BTreeSet::new();
            waiters.insert(waiter);
            self.map.insert(key, waiters);
            false
        }
    }

    /// Remove waiter from the channel. If channel gets zero waiters, it gets
    /// removed from the map. Returns true if waiter was found and false
    /// otherwise.
    pub fn remove_waiter(&mut self, key: Key, waiter: ThreadKey) -> bool {
        if self.map.contains_key(&key) {
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
        }
    }
}

impl Default for WaitMap {

    fn default() -> Self {
        WaitMap {
            map: Default::default()
        }
    }
}
