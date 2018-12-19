use std::rc::Rc;
use std::collections::BTreeSet;

pub struct PackageSet {
    paths: BTreeSet<RcPath>,
}

#[derive(Debug, Clone)]
pub struct RcPath(Rc<Path>);

/// Path node.
#[derive(Debug)]
pub struct Path {
    prev_node: Option<RcPath>,
    name: String,
}

/// Iterator over path nodes.
pub struct PathIter {
    nodes: Vec<RcPath>,
    cur: usize,
    curr: usize,
}

impl Path {

    pub fn new(name: String) -> RcPath {
        let rc = Rc::new(Path {
            prev_node: None,
            name,
        });
        RcPath(rc)
    }

    pub fn new_from_parent(parent: RcPath, name: String) -> RcPath {
        let rc = Rc::new(Path {
            prev_node: Some(parent),
            name
        });
        RcPath(rc)
    }

    /// Convert path to string value.
    pub fn to_string(rc: &RcPath) -> String {
        let mut len = rc.name().len();

        let mut cur_node = rc.prev_node.clone();
        while cur_node.is_some() {
            let cur_node_unwrap = &cur_node.unwrap();
            len += cur_node_unwrap.name().len() + 1; // +1 for dot between names.

            cur_node = cur_node_unwrap.prev_node.clone();
        }

        let mut st = String::with_capacity(len);
        st.push_str(&rc.name);

        let mut cur_node = rc.prev_node.clone();
        while cur_node.is_some() {
            let cur_node_unwrap = &cur_node.unwrap();
            st.insert(0, '.');
            st.insert_str(0, &cur_node_unwrap.name); // +1 for dot between names.

            cur_node = cur_node_unwrap.prev_node.clone();
        }

        st
    }

    /// Name of current node.
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl ::std::borrow::Borrow<Path> for RcPath {

    fn borrow(&self) -> &Path {
        self
    }
}

impl ::std::convert::AsRef<Path> for RcPath {

    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl ::std::ops::Deref for RcPath {
    type Target = Rc<Path>;

    fn deref(&self) -> &Rc<Path> {
        &self.0
    }
}

impl Into<Rc<Path>> for RcPath {

    fn into(self) -> Rc<Path> {
        self.0
    }
}

impl PartialEq for RcPath {

    fn eq(&self, other: &RcPath) -> bool {
        let mut a = PathIter::new(self.clone().into());
        let mut b = PathIter::new(other.clone().into());

        loop {
            let a = a.next();
            let b = b.next();

            if a.is_some() && b.is_some() {
                let i = a.unwrap();
                let j = b.unwrap();

                if i.name != j.name {
                    return false;
                }
            } else {
                return false;
            }
        }
    }
}

impl Eq for RcPath {}

impl PartialOrd for RcPath {

    fn partial_cmp(&self, other: &RcPath) -> Option<::std::cmp::Ordering> {
        use std::cmp::Ordering::*;

        let mut a = PathIter::new(self.clone().into());
        let mut b = PathIter::new(other.clone().into());

        loop {
            let i = a.next();
            let j = b.next();

            if i.is_some() {
                let i = i.unwrap();
                if j.is_some() {
                    let j = j.unwrap();

                    let cmp = i.name.cmp(&j.name);
                    if cmp == Equal {
                        continue;
                    } else {
                        // Reverse due to string comparison is reverted to
                        // alphabetical order.
                        return Some(cmp.reverse());
                    }
                } else {
                    // Children is less than parent.
                    return Some(Less);
                }
            } else {
                if j.is_some() {
                    // Root node is greater than children.
                    return Some(Greater);
                } else {
                    // Last nodes are equal as previous. Paths are equal.
                    return Some(Equal);
                }
            }
        }

    }
}

impl Ord for RcPath {

    fn cmp(&self, other: &RcPath) -> ::std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PathIter {

    /// Create path iterator for given path.
    pub fn new(path: RcPath) -> Self {
        use std::collections::LinkedList;

        let mut list = LinkedList::new();

        // Add current node.
        list.push_front(path.clone());

        // Add all subnodes.
        let mut cur = path.prev_node.clone();
        while cur.is_some() {
            let unwrap = cur.unwrap();
            list.push_front(unwrap.clone().into());

            cur = unwrap.prev_node.clone();
        }

        // Transform list to array.
        let mut array = Vec::with_capacity(list.len());
        for i in list.iter() {
            array.push(i.clone());
        }

        PathIter {
            nodes: array,
            cur: 0,
            curr: list.len(),
        }
    }
}

impl Iterator for PathIter {

    type Item = RcPath;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur >= self.nodes.len() {
            None
        } else {
            let val = self.nodes[self.cur].clone();
            self.cur += 1;
            Some(val)
        }
    }
}

impl ExactSizeIterator for PathIter {

    fn len(&self) -> usize {
        self.nodes.len()
    }
}

impl DoubleEndedIterator for PathIter {

    fn next_back(&mut self) -> Option<Self::Item> {
        if self.curr == 0 {
            None
        } else {
            self.curr -= 1;
            Some(self.nodes[self.curr].clone())
        }
    }
}

impl ::std::iter::FusedIterator for PathIter {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_to_str() {
        let p = Path::new("root".to_string());
        let p = Path::new_from_parent(p, "foo".to_string());
        let p = Path::new_from_parent(p, "bar".to_string());
        let p = Path::new_from_parent(p, "baz".to_string());

        assert_eq!(Path::to_string(&p), "root.foo.bar.baz");
    }

    #[test]
    fn path_iter() {
        let p = Path::new("root".to_string());
        let p = Path::new_from_parent(p, "foo".to_string());
        let p = Path::new_from_parent(p, "bar".to_string());
        let p = Path::new_from_parent(p, "baz".to_string());

        let mut iter = PathIter::new(p);
        assert_eq!(iter.next().unwrap().name, "root");
        assert_eq!(iter.next().unwrap().name, "foo");
        assert_eq!(iter.next().unwrap().name, "bar");
        assert_eq!(iter.next().unwrap().name, "baz");
        assert!(iter.next().is_none());

        assert_eq!(iter.next_back().unwrap().name, "baz");
        assert_eq!(iter.next_back().unwrap().name, "bar");
        assert_eq!(iter.next_back().unwrap().name, "foo");
        assert_eq!(iter.next_back().unwrap().name, "root");
        assert!(iter.next_back().is_none());
    }

    #[test]
    fn path_cmp_alphabetical() {
        let p0 = Path::new("a".to_string());
        let p0 = Path::new_from_parent(p0, "b".to_string());
        let p0 = Path::new_from_parent(p0, "c".to_string());
        let p0 = Path::new_from_parent(p0, "d".to_string());

        let p1 = Path::new("a".to_string());
        let p1 = Path::new_from_parent(p1, "c".to_string());
        let p1 = Path::new_from_parent(p1, "c".to_string());
        let p1 = Path::new_from_parent(p1, "d".to_string());

        assert!(p0 > p1);
    }

    #[test]
    fn path_cmp_parent_is_greater() {
        let p0 = Path::new("a".to_string());
        let p0 = Path::new_from_parent(p0, "b".to_string());
        let p0 = Path::new_from_parent(p0, "c".to_string());
        let p0 = Path::new_from_parent(p0, "d".to_string());

        let p1 = Path::new("a".to_string());
        let p1 = Path::new_from_parent(p1, "b".to_string());
        let p1 = Path::new_from_parent(p1, "c".to_string());

        assert!(p0 < p1);
    }
}
