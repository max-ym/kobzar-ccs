use std::collections::LinkedList;
use std::collections::BTreeSet;

/// Application thread list operations.
pub mod threads;
pub use crate::threads::{
    Set as ThreadSet,
    Thread,
    Key as ThreadKey,
    State as ThreadState,
};

/// Interface set and operations.
pub mod interfaces;
pub use crate::interfaces:: {
    Version,
    Key as InterfaceKey,
    Interface,
    Func as InterfaceFunc,
    InterfaceSet,
};

/// Paths to packages which contains interfaces and processes.
pub mod path;
pub use crate::path:: {
    Path,
    RcPath,
    PathIter,
    PackageTree,
};

/// Operations on channels between threads.
pub mod channels;
pub use crate::channels::{
    Channel,
    Key as ChannelKey,
    ChannelSet,
};

/// Process data and operations on processes.
pub mod process;
pub use crate::process::{
    Key as ProcessKey,
    Process,
    Set as ProcessSet,
    ImplementationConflicts,
};

/// Operations related to waiting threads and channel lock relations.
pub mod wait;
pub use crate::wait::{
    WaitDependency,
    WaitMap,
    Graph,
    GraphNode,
};

/// Network that contains all threads, channels, packages and interfaces.
pub struct Network {
    threads: ThreadSet,
    processes: ProcessSet,
    interfaces: InterfaceSet,
    channels: ChannelSet,
    packages: PackageTree,
    wait_deps: WaitMap,
}

impl Network {

    /// Threads registered in the network.
    pub fn threads(&self) -> &ThreadSet {
        &self.threads
    }

    /// Processes registered in the network.
    pub fn processes(&self) -> &ProcessSet {
        &self.processes
    }

    /// Interfaces registered in the network.
    pub fn interfaces(&self) -> &InterfaceSet {
        &self.interfaces
    }

    /// Channels created in the network.
    pub fn channels(&self) -> &ChannelSet {
        &self.channels
    }

    /// Packages created in the network.
    pub fn packages(&self) -> &PackageTree {
        &self.packages
    }

    /// Waiting dependencies created in the network.
    pub fn wait_deps(&self) -> &WaitMap {
        &self.wait_deps
    }

    /// Register new thread in given process.
    ///
    /// # Returns
    /// None is returned if no such process was found. Value is
    /// returned when thread was registered successfully.
    pub fn new_thread(&mut self, thread: Thread, process: &ProcessKey)
            -> Option<ThreadKey> {
        unimplemented!()
    }

    /// Register new process in the network.
    pub fn new_process(&mut self, process: Process) -> ProcessKey {
        unimplemented!()
    }

    /// Register new channel in the network.
    ///
    /// # Returns
    /// None is returned if any of partcipant threads were not found.
    /// Some is returned if channel was successfully created.
    pub fn new_channel(&mut self, channel: Channel) -> Option<ChannelKey> {
        unimplemented!()
    }

    /// Try put thread asleep.
    ///
    /// # Returns
    /// Some if thread was found and successfully put asleep.
    /// None if thread was not found.
    pub fn sleep_thread(&mut self, thread: ThreadKey) -> Option<()> {
        unimplemented!()
    }

    pub fn active_thread(&mut self, thread: ThreadKey) -> Option<()> {
        unimplemented!()
    }

    pub fn wait_thread(&mut self, thread: ThreadKey, signal_source: ChannelKey,
        timer: bool
    ) -> Option<()> {
        unimplemented!()
    }

    /// Check if threads in this channel created deadlock of signal waiting.
    /// It also includes verifying other channels that are open by the threads
    /// if they are waiting not for a given channel but are still participating
    /// in it.
    ///
    /// # Returns
    /// True if deadlock was found. False means there is no deadlock.
    /// None if channel was not registered.
    fn check_signal_wait_deadlock(&self, signal_source: ChannelKey)
            -> Option<bool> {

        // To check whether the channel is deadlocked we need to
        // build a graph which will show all relations
        // between different channels. If graph contains
        // a loop then there is a deadlock and otherwise there is
        // no deadlock.
        //
        // When new thread changes it's state on waiting without timer
        // the channels where it paricipates must be checked.
        // The graph relations of these channels must be updated but old
        // results of unaffected channels may be used. This will reduce
        // computation time by removing repeated checks that will produce
        // same results.

        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
}
