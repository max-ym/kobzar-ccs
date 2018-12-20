use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::cmp::Ordering;

use super::path::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

/// Interface key that identifies unique interface entries in a map.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Key {
    path: RcPath,
    version: Version,
}

/// Information about interface.
#[derive(Debug, Clone)]
pub struct Interface {
    fns: BTreeSet<Func>,

    /// Interfaces that must be implemented first in order to allow this
    /// one's implementation.
    prerequisites: BTreeSet<Key>,
}

/// Function that must be implemented by interface implementator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Func {

    /// ID of this function. Each function has a name which is it's unique
    /// identifier.
    name: String,

    /// Each function is uniquely identified by version. Even there can be
    /// several functions with same name and different versions.
    version: Version,
}

/// Set of all interfaces and their relations.
#[derive(Default)]
pub struct InterfaceSet {
    map: BTreeMap<Key, Rc<Interface>>,
}

impl Key {

    /// Create new interface key.
    pub fn new(path: RcPath, version: Version) -> Self {
        Key {
            path,
            version,
        }
    }

    pub fn path(&self) -> &RcPath {
        &self.path
    }

    pub fn version(&self) -> &Version {
        &self.version
    }
}

impl ToString for Version {

    fn to_string(&self) -> String {
        let maj = self.major.to_string();
        let min = self.minor.to_string();
        let ptc = self.patch.to_string();

        let mut s = String::with_capacity(maj.len() + min.len()
                + ptc.len() + 2);
        s.push_str(&maj);
        s.push('.');
        s.push_str(&min);
        s.push('.');
        s.push_str(&ptc);
        s
    }
}

impl Version {

    /// Create new version instance.
    pub fn new(major: u32, minor: u32, patch: u32) -> Version {
        Version {
            major,
            minor,
            patch,
        }
    }
}

impl PartialOrd for Version {

    fn partial_cmp(&self, other: &Version) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {

    fn cmp(&self, other: &Version) -> Ordering {
        use self::Ordering::*;

        match self.major.cmp(&other.major) {
            Less    => Less,
            Greater => Greater,
            Equal   => {
                match self.minor.cmp(&other.minor) {
                    Less    => Less,
                    Greater => Greater,
                    Equal   => {
                        self.patch.cmp(&other.patch)
                    }
                }
            },
        }
    }
}

impl PartialOrd for Func {

    fn partial_cmp(&self, other: &Func) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Func {

    fn cmp(&self, other: &Func) -> Ordering {
        use self::Ordering::*;

        // Reverse to make comparison alphabetical.
        let cmp = self.name.cmp(&other.name).reverse();

        if cmp == Equal {
            // Names are equal, check versions.
            self.version.cmp(&other.version)
        } else {
            cmp
        }
    }
}

impl Func {

    /// Create new function entry.
    pub fn new(name: String, version: Version) -> Func {
        Func {
            name,
            version,
        }
    }

    /// Function version.
    pub fn version(&self) -> &Version {
        &self.version
    }

    /// Function name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Interface {

    /// Create new interface entry in given package.
    pub fn new() -> Interface {
        Default::default()
    }

    /// Function set.
    pub fn fns(&self) -> &BTreeSet<Func> {
        &self.fns
    }

    /// Set of prerequisite interfaces. These interfaces required to be
    /// implemented by the process in case it want's to implement
    /// given interface.
    pub fn prerequisites(&self) -> &BTreeSet<Key> {
        &self.prerequisites
    }

    /// Add new function to the function set.
    pub fn add_fn(&mut self, func: Func) {
        self.fns.insert(func);
    }

    /// Add new prerequisite to the set.
    pub fn add_prerequisite(&mut self, key: Key) {
        self.prerequisites.insert(key);
    }
}

impl Default for Interface {

    fn default() -> Self {
        Interface {
            fns: Default::default(),
            prerequisites: Default::default(),
        }
    }
}

impl PartialOrd for Key {

    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Key {

    fn cmp(&self, other: &Self) -> Ordering {
        use self::Ordering::*;

        let path = self.path.cmp(&other.path);
        match path {
            Less    => Less,
            Greater => Greater,
            Equal => {
                self.version.cmp(&other.version)
            }
        }
    }
}

impl InterfaceSet {

    /// Create new interface set.
    pub fn new() -> Self {
        Default::default()
    }

    /// Interface map.
    pub fn interfaces(&self) -> &BTreeMap<Key, Rc<Interface>> {
        &self.map
    }

    /// Add new interface to the map. If there is present interface with
    /// same key, new interface will be discarded and Err returned.
    pub fn add_interface(&mut self, key: Key, interface: Interface)
            -> Result<(), ()> {
        if self.map.contains_key(&key) {
            return Err(());
        }

        let rc = Rc::new(interface);
        self.map.insert(key, rc);
        Ok(())
    }

    /// Remove interface from the map. If there is no such key Err is returned.
    pub fn remove_interface(&mut self, key: &Key)
            -> Result<Interface, ()> {
        let i = self.map.remove(key);
        if i.is_none() {
            Err(())
        } else {
            Ok(Rc::try_unwrap(i.unwrap()).unwrap())
        }
    }

    pub fn interface(&self, key: &Key) -> Option<Rc<Interface>> {
        let i = self.map.get(key);
        match i {
            Some(t) => Some(t.clone()),
            None    => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_string() {
        let ver = Version::new(1, 7, 2);
        let s = ver.to_string();
        assert_eq!("1.7.2", s);
    }

    #[test]
    fn version_major_cmp() {
        let ver1 = Version::new(1, 7, 2);
        let ver2 = Version::new(2, 7, 2);

        assert!(ver1 < ver2);
    }

    #[test]
    fn version_minor_cmp() {
        let ver1 = Version::new(1, 6, 2);
        let ver2 = Version::new(1, 7, 2);

        assert!(ver1 < ver2);
    }

    #[test]
    fn version_patch_cmp() {
        let ver1 = Version::new(1, 6, 1);
        let ver2 = Version::new(1, 6, 2);

        assert!(ver1 < ver2);
    }

    #[test]
    fn version_everything_cmp() {
        let ver1 = Version::new(1, 6, 1);
        let ver2 = Version::new(2, 7, 2);

        assert!(ver1 < ver2);
    }

    #[test]
    fn func_name_cmp() {
        let f1 = Func::new("a".to_string(), Version::new(1, 0, 0));
        let f2 = Func::new("b".to_string(), Version::new(1, 0, 0));

        assert!(f1 > f2);
    }

    #[test]
    fn func_ver_cmp() {
        let f1 = Func::new("a".to_string(), Version::new(1, 0, 0));
        let f2 = Func::new("a".to_string(), Version::new(1, 2, 0));

        assert!(f1 < f2);
    }

    #[test]
    fn func_all_cmp() {
        let f1 = Func::new("a".to_string(), Version::new(1, 0, 0));
        let f2 = Func::new("b".to_string(), Version::new(1, 2, 0));

        assert!(f1 > f2);
    }
}
