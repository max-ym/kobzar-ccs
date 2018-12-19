use super::{
    ThreadKey,
};

use std::collections::BTreeSet;

/// Channel identifier.
pub type Key = u32;

/// The channel-related information.
pub struct Channel {

    /// Participants in channel transactions.
    participants: BTreeSet<ThreadKey>,
}

pub struct WaitDependency {

    /// Thread that waits for a signal.
    waiter: ThreadKey,

    /// Source from which the signal is expected.
    signal_source: Key,
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
