use std::collections::{BTreeMap, BTreeSet, LinkedList};
use std::rc::Rc;

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

    /// Map that connects each single channel with a set of
    /// waiting for it's signal threads.
    chan: BTreeMap<ChannelKey, BTreeSet<ThreadKey>>,

    /// This map connects each single thread with a channel where it
    /// waits for a signal.
    thr: BTreeMap<ThreadKey, BTreeSet<ChannelKey>>,
}

type GraphNodeKey = u32;

/// Graph that shows relations between different channels. Used to find a
/// deadlocks.
#[derive(Default)]
pub struct Graph {
    next_id: GraphNodeKey,
}

/// A node of the graph that may be connected to other nodes.
pub struct GraphNode {
    id: GraphNodeKey,
    relations: BTreeMap<GraphNodeKey, Rc<GraphNode>>,
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
        let present = self.chan.contains_key(&key);

        // Add to channel map.
        let added = if present {
            false
        } else {
            self.chan.insert(key, waiters.clone());
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
        let success = if self.chan.contains_key(&key) {
            self.chan.get_mut(&key).unwrap().insert(waiter.clone());
            true
        } else {
            let mut waiters = BTreeSet::new();
            waiters.insert(waiter);
            self.chan.insert(key, waiters);
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
        let success = if self.chan.contains_key(&key) {
            let (present, set_is_empty) = {
                let set = self.chan.get_mut(&key).unwrap();
                let present = set.remove(&waiter);

                (present, set.is_empty())
            };

            if set_is_empty {
                self.chan.remove(&key);
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
        &self.chan
    }

    /// Map that connects each thread with a channel for which it waits.
    pub fn thread_wait_map(&self) -> &BTreeMap<ThreadKey, BTreeSet<ChannelKey>> {
        &self.thr
    }
}

impl Graph {

    /// Create new empty graph.
    pub fn new() -> Self {
        Default::default()
    }

    fn generate_new_node_key(&mut self) -> GraphNodeKey {
        let new_key = self.next_id;
        self.next_id += 1;
        new_key
    }

    /// Create new node that is not connected to any other.
    pub fn create_new_node(&mut self) -> Rc<GraphNode> {
        let node = GraphNode {
            id: self.generate_new_node_key(),
            relations: Default::default(),
        };
        Rc::new(node)
    }
}

impl GraphNode {

    /// Add new relation.
    ///
    /// Returns true on success and false if node is already present.
    /// Error occurs if new relation forms a loop.
    pub fn add_relation(&mut self, node: Rc<GraphNode>) -> Result<bool, ()> {
        if self.relation_exists(&node) {
            return Ok(false);
        }

        self.relations.insert(node.id.clone(), node.clone());
        if self.path_has_loop() {
            // Revert changes and return error.
            self.relations.remove(&node.id);
            return Err(());
        }

        Ok(true)
    }

    /// Check whether teh path that contains this node has a loop.
    fn path_has_loop(&self) -> bool {
        // To check whether there is a loop we need to take each path and
        // follow it to the end. If any of the vertices is repeated then the
        // loop exists.

        // Set of nodes we already gone through.
        let mut nodes = BTreeSet::new();
        // Next nodes to follow through.
        let mut next_nodes = LinkedList::new();
        next_nodes.push_back(self);
        loop {
            let cur = next_nodes.pop_front();
            if cur.is_none() {
                // All path was gone through and no loop was found.
                return false;
            }
            let cur = cur.unwrap();

            let already_present = nodes.insert(cur.id);
            if already_present {
                return true;
            }

            for (_, node) in &cur.relations {
                next_nodes.push_back(&node);
            }
        }
    }

    /// Check whether this node contains relations to given node.
    pub fn relation_exists(&self, node: &Rc<GraphNode>) -> bool {
        self.relations.contains_key(&node.id)
    }

    /// Remove relation to node.
    ///
    /// True on success and false if no such relation was found.
    pub fn remove_relation(&mut self, node: &Rc<GraphNode>) -> bool {
        if !self.relation_exists(node) {
            return false;
        }

        self.relations.remove(&node.id);
        true
    }
}
