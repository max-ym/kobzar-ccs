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

    pub fn new() -> Self {
        Default::default()
    }

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
        let channel_key = next_channel_key.clone();
        self.channels.insert(channel_key.clone(), channel);
        self.wait_deps.add_channel(channel_key.clone(), Default::default());
        *next_channel_key += 1;

        // Register channel to all threads.
        for participant in participants {
            let thread = self.threads.get_mut(&participant).unwrap();
            thread.channels_mut().insert(channel_key.clone());
        }

        Some(channel_key)
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
        if timer == true {
            return Ok(self.change_thread_state_remove_deps(thread_key,
                    ThreadState::WaitWithTimeout(signal_source.clone())));
        }

        if self.channels.get(signal_source).is_none() {
            return Ok(None);
        }

        let thread = self.thread(thread_key);
        if thread.is_none() {
            return Ok(None);
        }
        let thread = thread.unwrap();

        // Register channel relations if all threads in channel are locked.
        let wd = unsafe { &mut *(&self.wait_deps as *const _ as *mut WaitMap) };
        wd.add_waiter(signal_source.clone(), thread_key.clone());
        let mut err = false;
        for ch in thread.channels().iter() {
            let participant_count =
                    self.channels.get(ch).unwrap().participants().len();
            if participant_count == wd.channel_wait_map().len() {
                let result = wd.add_channel_relation(signal_source, ch);
                if result.is_err() {
                    err = true;
                    break;
                } else if result.unwrap() == false {
                    panic!("Couldn't find destination channel which is known
                    to be registered. Channel number: {}", ch);
                }
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

    /// Some thread send a message by the channel. It goes to wait mode
    /// and all waiting receivers become sleeping and waiting for processor
    /// time.
    ///
    /// Returns array of threads that wake up from waiting state.
    /// Error is returned if whether channel is not found or sender
    /// is not found or not participating in the channel.
    pub fn channel_signal(&mut self, sender: &ThreadKey,
        channel: &ChannelKey, timer: bool
    ) -> Result<LinkedList<ThreadKey>, ()> {
        // Check whether this thread really is participating in given channel.
        {
            let chan = self.channels.get(channel);
            if chan.is_none() {
                return Err(());
            }
            let chan = chan.unwrap();

            let sender = chan.participants().get(sender);
            if sender.is_none() {
                return Err(());
            }
        }

        // List of all threads to wake up.
        let mut list = LinkedList::new();

        for participant_key in
                self.channels.get(channel).unwrap().participants().iter() {
            let mut_self = unsafe { &mut *(self as *const _ as *mut Self) };
            let thread = self.threads.get(&participant_key).unwrap();
            if thread.is_waiting_channel(&channel) {
                list.push_front(participant_key.clone());
                mut_self.active_thread(&participant_key);
            }
        }

        // Set current thread to wait for signal from channel.
        self.wait_thread(sender, channel, timer).unwrap();

        Ok(list)
    }

    pub fn thread_mut(&mut self, thread: &ThreadKey) -> Option<&mut Thread> {
        self.threads.get_mut(thread)
    }

    pub fn thread(&self, thread: &ThreadKey) -> Option<&Thread> {
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
            thread.set_state(state);
            old_state
        };

        use std::mem::discriminant;
        let without_timeout = discriminant(&ThreadState::WaitWithoutTimeout(0));

        if discriminant(&old_state) == without_timeout {
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
    use super::*;

    #[test]
    fn network_wait_deps() {
        let proc_path1 = Path::new("a".to_string());
        let proc_path2 = Path::new("b".to_string());

        let mut network = Network::new();
        let proc1 = network.new_process(Process::new(proc_path1));
        let proc2 = network.new_process(Process::new(proc_path2));

        let th1 = network.new_thread(Thread::new(), &proc1).unwrap();
        let th2 = network.new_thread(Thread::new(), &proc1).unwrap();
        let th3 = network.new_thread(Thread::new(), &proc2).unwrap();

        let mut ch12 = Channel::new(th1);
        ch12.add_participant(th2);

        let mut ch23 = Channel::new(th2);
        ch23.add_participant(th3);

        let mut ch31 = Channel::new(th3);
        ch31.add_participant(th1);

        let ch12 = network.new_channel(ch12).unwrap();
        let ch23 = network.new_channel(ch23).unwrap();
        let ch31 = network.new_channel(ch31).unwrap();

        assert!(network.wait_thread(&th1, &ch12, false).is_ok());
        //assert!(network.wait_thread(&th2, &ch23, false).is_ok());
        //assert!(network.wait_thread(&th3, &ch31, false).is_err());
    }

    #[test]
    fn network_add_channel() {
        let proc_path1 = Path::new("a".to_string());
        let proc_path2 = Path::new("b".to_string());

        let mut network = Network::new();
        let proc1 = network.new_process(Process::new(proc_path1));
        let proc2 = network.new_process(Process::new(proc_path2));

        let th1 = network.new_thread(Thread::new(), &proc1).unwrap();
        let th2 = network.new_thread(Thread::new(), &proc1).unwrap();
        let th3 = network.new_thread(Thread::new(), &proc2).unwrap();

        let mut ch12 = Channel::new(th1);
        ch12.add_participant(th2);

        let mut ch23 = Channel::new(th2);
        ch23.add_participant(th3);

        let mut ch31 = Channel::new(th3);
        ch31.add_participant(th1);

        let ch12 = network.new_channel(ch12).unwrap();
        let ch23 = network.new_channel(ch23).unwrap();
        let ch31 = network.new_channel(ch31).unwrap();

        assert!(network.thread(&th1).unwrap().channels().contains(&ch12));
        assert!(network.thread(&th2).unwrap().channels().contains(&ch12));
        assert!(network.thread(&th2).unwrap().channels().contains(&ch23));
        assert!(network.thread(&th3).unwrap().channels().contains(&ch23));
        assert!(network.thread(&th1).unwrap().channels().contains(&ch31));
        assert!(network.thread(&th3).unwrap().channels().contains(&ch31));

        assert!(network.channels.get(&ch12).is_some());
        assert!(network.channels.get(&ch23).is_some());
        assert!(network.channels.get(&ch31).is_some());
    }
}
