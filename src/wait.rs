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

    /// Connection between each channel and graph node that represents the
    /// channel.
    chan_to_graph: BTreeMap<ChannelKey, Rc<GraphNode>>,

    /// The graph of dependencies.
    graph: Graph,
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

        // Add to channel map and graph.
        let added = if present {
            false
        } else {
            self.chan.insert(key.clone(), waiters.clone());
            self.chan_to_graph.insert(
                key.clone(), self.graph.new_node());
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

    /// Try removing channel from the graph. It is only removed if it
    /// has no waiters in it.
    ///
    /// True is returned on successful remove and false if there was
    /// some waiters in it and thus channel remains. None is returned
    /// if channel was not found.
    pub fn remove_channel(&mut self, key: &ChannelKey) -> Option<bool> {
        if !self.chan.contains_key(key) {
            return None;
        }

        if !self.chan.get(key).unwrap().is_empty() {
            return Some(false);
        }

        self.chan.remove(key);
        Some(true)
    }

    /// Add new waiter to registered channel. Returns false if channel
    /// is not registered. In this case the channel gets registered
    /// first and waiter is added then. Still false is returned.
    /// True is returned otherwise, when channel already existed.
    pub fn add_waiter(&mut self, key: ChannelKey, waiter: ThreadKey) -> bool {
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

        // Register thread to channel map for this thread if it is not
        // yet created.
        if !self.thr.get(&waiter).is_some() {
            let mut set = BTreeSet::new();
            set.insert(key.clone());
            self.thr.insert(waiter.clone(), set);
        }
        self.thr.get_mut(&waiter).unwrap().insert(key);

        true
    }

    /// Remove waiter from the channel.
    /// Returns true if waiter was found and false
    /// otherwise.
    pub fn remove_waiter(&mut self, key: ChannelKey, waiter: ThreadKey) -> bool {
        let success = if self.chan.contains_key(&key) {
            let present = {
                let set = self.chan.get_mut(&key).unwrap();
                let present = set.remove(&waiter);

                present
            };

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

    /// Remove thread from all channels.
    ///
    /// Returns true if thread was successfully removed and
    /// false if it was not found.
    pub fn remove_thread(&mut self, key: &ThreadKey) -> bool {
        // Collect all channels to remove thread from.
        let channels = self.thr.get(key);
        if channels.is_none() {
            return false;
        }
        let channels = channels.unwrap();

        for chan in channels.iter() {
            self.chan.get_mut(chan).unwrap().remove(key);
        }

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

    /// Create new relation between channels.
    ///
    /// Returns true if relation successfully created.
    /// False is returned when channel was not found by the key.
    /// Err is returned when the relation forms a loop and the changes
    /// are reverted.
    pub fn add_channel_relation(&mut self, to: &ChannelKey,
            from: &ChannelKey) -> Result<bool, ()> {
        let all_exist = {
            let to_exists = self.chan_to_graph.contains_key(to);
            let from_exists = self.chan_to_graph.contains_key(from);
            to_exists && from_exists
        };

        if !all_exist {
            return Ok(false);
        }

        let to = self.chan_to_graph.get(to).unwrap();
        let from = self.chan_to_graph.get(from).unwrap();

        match from.add_relation(&to) {
            Ok(_)   => Ok(true),
            Err(()) => Err(())
        }
    }

    /// Remove channel relations.
    ///
    /// Return None if one of the channels was not found.
    /// Return true if relation was deleted or false if it didn't exist.
    pub fn remove_channel_relation(&mut self, to: &ChannelKey,
        from: &ChannelKey
    ) -> Option<bool> {
        let all_exist = {
            let to_exists = self.chan_to_graph.contains_key(to);
            let from_exists = self.chan_to_graph.contains_key(from);
            to_exists && from_exists
        };

        if !all_exist {
            return None;
        }

        let to = self.chan_to_graph.get(to).unwrap();
        let from = self.chan_to_graph.get(from).unwrap();

        Some(from.remove_relation(to))
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
    pub fn new_node(&mut self) -> Rc<GraphNode> {
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
    pub fn add_relation(&self, node: &Rc<GraphNode>) -> Result<bool, ()> {
        let _self = unsafe { &mut *(self as *const _ as *mut GraphNode) };
        if self.relation_exists(&node) {
            return Ok(false);
        }

        _self.relations.insert(node.id.clone(), node.clone());
        if self.path_has_loop() {
            // Revert changes and return error.
            _self.relations.remove(&node.id);
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

            let already_present = !nodes.insert(cur.id);
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
    pub fn remove_relation(&self, node: &Rc<GraphNode>) -> bool {
        if !self.relation_exists(node) {
            return false;
        }

        let _self = unsafe { &mut *(self as *const _ as *mut GraphNode) };

        _self.relations.remove(&node.id);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graph_loop() {
        let mut graph = Graph::new();
        let mut n1 = graph.new_node();
        let mut n2 = graph.new_node();
        let mut n3 = graph.new_node();

        assert!(n1.add_relation(&n2).is_ok());
        assert!(n2.add_relation(&n3).is_ok());
        assert!(n3.add_relation(&n1).is_err());
    }

    #[test]
    fn wait_map_loop() {
        let mut wm = WaitMap::new();

        let mut c12w: BTreeSet<ThreadKey> = BTreeSet::new();
        c12w.insert(1);
        c12w.insert(2);

        let mut c23w: BTreeSet<ThreadKey> = BTreeSet::new();
        c23w.insert(2);
        c23w.insert(3);

        let mut c31w: BTreeSet<ThreadKey> = BTreeSet::new();
        c31w.insert(3);
        c31w.insert(1);

        let c12 = 1;
        let c23 = 2;
        let c31 = 3;
        wm.add_channel(c12.clone(), c12w);
        wm.add_channel(c23.clone(), c23w);
        wm.add_channel(c31.clone(), c31w);

        assert!(wm.add_channel_relation(&c12, &c23).is_ok());
        assert!(wm.add_channel_relation(&c23, &c31).is_ok());
        assert!(wm.add_channel_relation(&c31, &c12).is_err());
    }

    #[test]
    fn wait_map_self_loop() {
        let mut wm = WaitMap::new();

        let mut c12w: BTreeSet<ThreadKey> = BTreeSet::new();
        c12w.insert(1);
        c12w.insert(2);

        let c12 = 1;
        wm.add_channel(c12.clone(), c12w);

        assert!(wm.add_channel_relation(&c12, &c12).is_err());
    }
}
