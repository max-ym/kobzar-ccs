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
#[derive(Default)]
pub struct Network {
    threads: ThreadSet,
    processes: ProcessSet,
    interfaces: InterfaceSet,
    channels: ChannelSet,
    packages: PackageTree,
    wait_deps: WaitMap,

    next_process_key: ProcessKey,
    next_channel_key: ChannelKey,
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
        let process = self.processes.get_mut(process);
        if process.is_none() {
            return None;
        }
        let process = process.unwrap();

        let thread_key = self.threads.add(thread);
        process.attach_thread(thread_key.clone());
        Some(thread_key)
    }

    /// Register new process in the network.
    pub fn new_process(&mut self, process: Process) -> ProcessKey {
        let new_key = self.next_process_key;
        self.next_process_key += 1;
        self.processes.insert(new_key.clone(), process);
        new_key
    }

    /// Register new channel in the network.
    ///
    /// # Returns
    /// None is returned if any of partcipant threads were not found.
    /// Some is returned if channel was successfully registered.
    pub fn new_channel(&mut self, channel: Channel) -> Option<ChannelKey> {
        let participants = channel.participants();

        // Check if all participants are really registered in this network.
        for participant in participants {
            if self.threads.get(participant).is_none() {
                return None;
            }
        }

        let participants = participants.clone();

        let next_channel_key = &mut self.next_channel_key;
        self.channels.insert(next_channel_key.clone(), channel);
        *next_channel_key += 1;

        // Register channel to all threads.
        for participant in participants {
            let thread = self.threads.get_mut(&participant).unwrap();
            thread.channels_mut().insert(next_channel_key.clone());
        }

        Some(next_channel_key.clone())
    }

    /// Try put thread asleep.
    ///
    /// # Returns
    /// Some if thread was found and successfully put asleep.
    /// None if thread was not found.
    pub fn sleep_thread(&mut self, thread: &ThreadKey) -> Option<()> {
        self.change_thread_state_remove_deps(thread, ThreadState::Sleep)
    }

    pub fn active_thread(&mut self, thread: &ThreadKey) -> Option<()> {
        self.change_thread_state_remove_deps(thread, ThreadState::Active)
    }

    pub fn wait_thread(&mut self, thread_key: &ThreadKey,
        signal_source: &ChannelKey, timer: bool
    ) -> Result<Option<()>, ()> {
        if timer == false {
            return Ok(self.change_thread_state_remove_deps(thread_key,
                    ThreadState::WaitWithTimeout));
        }

        if self.channels.get(signal_source).is_none() {
            return Ok(None);
        }

        let thread = self.thread(thread_key);
        if thread.is_none() {
            return Ok(None);
        }
        let thread = thread.unwrap();

        // Register channel relations.
        let wd = unsafe { &mut *(&self.wait_deps as *const _ as *mut WaitMap) };
        wd.add_waiter(signal_source.clone(), thread_key.clone());
        let mut err = false;
        for ch in thread.channels().iter() {
            let result = wd.add_channel_relation(signal_source, ch);
            if result.is_err() {
                err = true;
                break;
            }
        }

        // Revert changes if loop occured.
        if err {
            for ch in thread.channels().iter() {
                wd.remove_channel_relation(signal_source, ch);
            }
            Err(())
        } else {
            Ok(Some(()))
        }
    }

    fn thread_mut(&mut self, thread: &ThreadKey) -> Option<&mut Thread> {
        self.threads.get_mut(thread)
    }

    fn thread(&self, thread: &ThreadKey) -> Option<&Thread> {
        self.threads.get(thread)
    }

    /// Change thread state to given and remove thread from wait dependency.
    fn change_thread_state_remove_deps(&mut self, thread: &ThreadKey,
            state: ThreadState) -> Option<()> {
        let old_state = {
            let thread = self.thread_mut(thread);
            if thread.is_none() {
                return None;
            }
            let thread = thread.unwrap();

            let old_state = thread.state().clone();
            thread.set_state(ThreadState::Sleep);
            old_state
        };

        if old_state == ThreadState::WaitWithoutTimeout {
            self.remove_from_wait_dep(thread);
        }

        Some(())
    }

    /// Remove process from wait dependency.
    fn remove_from_wait_dep(&mut self, thread: &ThreadKey) {
        self.wait_deps.remove_thread(thread);
    }
}

#[cfg(test)]
mod tests {
}
