use crate::{
    RcPath,
    ThreadKey,
    InterfaceKey,
    InterfaceSet,
};

use std::collections::{
    BTreeSet,
    BTreeMap,
};

/// Key value to identify unique processes.
pub type Key = u32;

/// System process that can implement some interfaces and contains
/// threads that perform tasks.
pub struct Process {
    path: RcPath,
    threads: BTreeSet<ThreadKey>,
    implements: BTreeSet<InterfaceKey>,
}

/// Set that contains processes.
#[derive(Default)]
pub struct Set {
    procs: BTreeMap<Key, Process>,
}

/// Conflicts that were found in interface implementer.
pub struct ImplementationConflicts {
    missing: BTreeSet<InterfaceKey>,
}

impl Process {

    /// Create new empty process.
    pub fn new(path: RcPath) -> Self {
        Process {
            path,
            threads: Default::default(),
            implements: Default::default(),
        }
    }

    /// Threads of this process.
    pub fn threads(&self) -> &BTreeSet<ThreadKey> {
        &self.threads
    }

    /// Interfaces that are implemented by the process.
    pub fn implementations(&self) -> &BTreeSet<InterfaceKey> {
        &self.implements
    }

    /// Attach given thread to this process. Return true if it is
    /// already attached and false otherwise.
    pub fn attach_thread(&mut self, key: ThreadKey) -> bool {
        self.threads.insert(key)
    }

    /// Add new interface that is implemented by this process.
    /// Return true if it is already attached and false otherwise.
    pub fn add_implementation(&mut self, key: InterfaceKey) -> bool {
        self.implements.insert(key)
    }

    /// Check whether there are confliting requirements for interface
    /// implementer.
    ///
    /// Interface set is used to retrieve information about interfaces.
    ///
    /// # Panics
    /// Panic occurs when specified interface key is not found in the set.
    pub fn verify_implementations(&self, interface_set: &InterfaceSet)
            -> Result<(), ImplementationConflicts> {
        let mut prerequisites = BTreeSet::new();

        // Collect list of all prerequisites.
        for interface in &self.implements {
            let interface = interface_set.interface(&interface).unwrap();
            prerequisites.append(&mut interface.prerequisites().clone());
        }

        // Check whether all prerequisites are implemented.
        let mut missing = BTreeSet::new();
        for prerequisite in prerequisites {
            if !self.implements.contains(&prerequisite) {
                missing.insert(prerequisite.clone());
            }
        }

        if !missing.is_empty() {
            Err(ImplementationConflicts {
                missing
            })
        } else {
            Ok(())
        }
    }

    /// Path where this process is located.
    pub fn path(&self) -> &RcPath {
        &self.path
    }
}

impl Set {

    /// Create new empty set of processes.
    pub fn new() -> Self {
        Default::default()
    }

    /// Get a process from the set if it is stored there.
    pub fn get(&self, key: &Key) -> Option<&Process> {
        self.procs.get(key)
    }

    /// Get a process from the set if it is stored there.
    pub fn get_mut(&mut self, key: &Key) -> Option<&mut Process> {
        self.procs.get_mut(key)
    }

    /// Processes in the map.
    pub fn processes(&self) -> &BTreeMap<Key, Process> {
        &self.procs
    }

    pub fn insert(&mut self, key: Key, process: Process) -> bool {
        if self.procs.contains_key(&key) {
            return true;
        }
        self.procs.insert(key, process);
        false
    }

    pub fn remove(&mut self, key: &Key) -> bool {
        match self.procs.remove(&key) {
            Some(_) => true,
            None    => false,
        }
    }
}

impl ImplementationConflicts {

    /// Interfaces that are required but are not implemented.
    ///
    /// This conflict
    /// appears when some interfaces require other interface to be
    /// implemented in first place. Thus, process cannot implement
    /// interface with prerequisites without implementing them in first
    /// place and then this conflict appears.
    pub fn missing(&self) -> &BTreeSet<InterfaceKey> {
        &self.missing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Path,
        Interface,
        Version,
    };

    #[test]
    fn implementation_verification() {
        let p0 = Path::new("a".to_string());
        let p0 = Path::new_from_parent(p0, "b".to_string());

        let ik0 = InterfaceKey::new(p0.clone(), Version::new(1, 0, 0));
        let ik1 = InterfaceKey::new(p0.clone(), Version::new(1, 1, 0));
        let ik2 = InterfaceKey::new(p0.clone(), Version::new(2, 0, 0));

        let mut i = Interface::new();
        i.add_prerequisite(ik1.clone());
        i.add_prerequisite(ik2.clone());

        let mut is = InterfaceSet::new();
        is.add_interface(ik0.clone(), i.clone());
        is.add_interface(ik1.clone(), i.clone());
        is.add_interface(ik2.clone(), i.clone());

        let mut process = Process::new(p0);

        process.add_implementation(ik0);
        process.add_implementation(ik1);

        let result = process.verify_implementations(&is);
        assert!(result.unwrap_err().missing.contains(&ik2));
    }
}
