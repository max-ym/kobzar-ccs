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
    pub fn insert(&mut self, key: Key, channel: Channel) -> bool {
        if self.map.contains_key(&key) {
            true
        } else {
            self.map.insert(key, channel);
            false
        }
    }

    /// Remove existing channel from the set. If it exists, true is returned
    /// and false otherwise.
    pub fn remove(&mut self, key: Key) -> bool {
        self.map.remove(&key).is_some()
    }

    /// Channel in the set by the key.
    pub fn get(&self, key: &Key) -> Option<&Channel> {
        self.map.get(&key)
    }

    /// Channel in the set by the key.
    pub fn get_mut(&mut self, key: &Key) -> Option<&mut Channel> {
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

