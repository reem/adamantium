use std::sync::Arc;
use std::collections;
use std::default::Default;

/// A key value store, implemented as a persistent, functional
/// size balanced binary search tree.
pub enum Map<K, V> {
    /// A branch node.
    Bin {
        /// The size of this branch.
        pub size: uint,

        /// The key associated with this node.
        pub key: Arc<K>,

        /// The value associated with this node.
        pub value: Arc<V>,

        /// The left branch of this node.
        pub left: Arc<Map<K, V>>,

        /// The right branch of this node.
        pub right: Arc<Map<K, V>>
    },

    /// A leaf node.
    Tip
}

impl<K: Send + Sync, V: Send + Sync> Clone for Map<K, V> {
    fn clone(&self) -> Map<K, V> {
        match *self {
            Tip => Tip,
            Bin { size, ref key, ref value, ref left, ref right } => {
                Map::bin_ref(key, value, left, right)
            }
        }
    }
}

impl<K, V> collections::Collection for Map<K, V> {
    #[inline]
    fn len(&self) -> uint {
        match *self {
            Bin { size, .. } => size,
            Tip => 0
        }
    }
}

impl<K: Ord + Send + Sync, V: Send + Sync> collections::Map<K, V> for Map<K, V> {
    fn find<'a>(&'a self, lookup: &K) -> Option<&'a V> {
        match *self {
            Bin { ref key, ref left, ref right, ref value, .. } => match key.deref().cmp(lookup) {
                Equal   => Some(&**value),
                Less    => left.find(lookup),
                Greater => right.find(lookup)
            },
            Tip => None
        }
    }
}

impl<K: Ord + Send + Sync, V: Send + Sync> collections::Set<K> for Map<K, V> {
    fn contains(&self, lookup: &K) -> bool {
        self.find(lookup).is_some()
    }

    fn is_disjoint(&self, _other: &Map<K, V>) -> bool {
        unimplemented!()
    }

    fn is_subset(&self, _other: &Map<K, V>) -> bool {
        unimplemented!()
    }
}

// Nonstandard lookups
impl<K: Ord, V> Map<K, V> {
    /// Gets the first value in the map whose key is greater than the passed in
    /// key.
    pub fn lookup_greater_than<'a>(&'a self, lookup: &K) -> Option<&'a V> {
        unimplemented!()
    }

    /// Gets the first value in the map whose key is less than the passed in
    /// key.
    pub fn lookup_less_than<'a>(&'a self, lookup: &K) -> Option<&'a V> {
        unimplemented!()
    }

    /// Gets the first value in the map whose key is less than or equal to
    /// the passed in key.
    pub fn lookup_less_equal<'a>(&'a self, lookup: &K) -> Option<&'a V> {
        unimplemented!()
    }

    /// Gets the first value in the map whose key is greater than or equal to
    /// the passed in key.
    pub fn lookup_greater_equal<'a>(&'a self, lookup: &K) -> Option<&'a V> {
        unimplemented!()
    }
}

// Constructors
impl<K: Send + Sync, V: Send + Sync> Map<K, V> {
    /// An empty map.
    #[inline]
    pub fn new() -> Map<K, V> { Tip }

    /// Create a map with one key value pair.
    #[inline]
    pub fn singleton(key: K, value: V) -> Map<K, V> {
        Bin {
            size: 1,
            key: Arc::new(key),
            value: Arc::new(value),
            left: Arc::new(Map::new()),
            right: Arc::new(Map::new())
        }
    }

    /// Bin constructor which takes care of cloning Arcs and size.
    #[inline]
    pub fn bin(key: Arc<K>, value: Arc<V>, left: Arc<Map<K, V>>, right: Arc<Map<K, V>>) -> Map<K, V> {
        Bin {
            size: left.len() + right.len() + 1,
            key: key.clone(),
            value: value.clone(),
            left: left.clone(),
            right: right.clone()
        }
    }

    // Arc-based singleton constructor.
    fn singleton_arc(key: Arc<K>, value: Arc<V>) -> Map<K, V> {
        Bin {
            size: 1,
            key: key,
            value: value,
            left: Arc::new(Map::new()),
            right: Arc::new(Map::new())
        }
    }

    // Bin constructor which takes care of cloning &Arcs and size.
    //
    // This is very useful when destructuring a previous Bin by using `ref left` and such.
    #[inline]
    fn bin_ref(key: &Arc<K>, value: &Arc<V>, left: &Arc<Map<K, V>>, right: &Arc<Map<K, V>>) -> Map<K, V> {
        Bin {
            size: left.len() + right.len() + 1,
            key: key.clone(),
            value: value.clone(),
            left: left.clone(),
            right: right.clone()
        }
    }

    // Bin constructor which Arc's all values.
    //
    // Useful for creating Bin's from constituent parts without writing
    // boilerplate.
    fn bin_no_arc(key: K, value: V, left: Map<K, V>, right: Map<K, V>) -> Map<K, V> {
        Map::bin(Arc::new(key), Arc::new(value), Arc::new(left), Arc::new(right))
    }
}

impl<K: Send + Sync, V: Send + Sync> Default for Map<K, V> {
    #[inline]
    fn default() -> Map<K, V> { Map::new() }
}

// Insertion
impl<K: Send + Sync + Ord, V: Send + Sync> Map<K, V> {
    /// Insert a key value pair into the map. If they key is already present in
    /// the Map, it's value will be replaced.
    pub fn insert(&self, key: Arc<K>, val: Arc<V>) -> Map<K, V> {
        match *self {
            Tip => Map::singleton_arc(key, val),
            Bin { size: ref sizex, key: ref keyx, value: ref valuex,
                  left: ref leftx, right: ref rightx } => {
                match key.cmp(&**keyx) {
                    Equal   => Map::bin_ref(&key, &val, leftx, rightx),
                    Less    => Map::balance(keyx.clone(), valuex.clone(),
                                            Arc::new(leftx.insert(key, val)), rightx.clone()),
                    Greater => Map::balance(keyx.clone(), valuex.clone(),
                                            leftx.clone(), Arc::new(rightx.insert(key, val))),
                }
            }
        }
    }

    /// Insert a new key value pair into the map. If the key is already
    /// present the old value is used.
    pub fn insert_no_replace(&self, key: Arc<K>, val: Arc<V>) -> Map<K, V> {
        match *self {
            Tip => Map::singleton_arc(key, val),
            Bin { size: ref sizex, key: ref keyx, value: ref valuex,
                  left: ref leftx, right: ref rightx } => {
                match key.cmp(&**keyx) {
                    Equal   => self.clone(),
                    Less    => Map::balance(keyx.clone(), valuex.clone(),
                                            Arc::new(leftx.insert(key, val)), rightx.clone()),
                    Greater => Map::balance(keyx.clone(), valuex.clone(),
                                            leftx.clone(), Arc::new(rightx.insert(key, val))),
                }
            }
        }
    }

    /// Insert a key value pair into the map, if the key is already present,
    /// modify it's value with the passed in closure.
    pub fn insert_or_modify_with(&self, key: Arc<K>, val: Arc<V>, modifier: |&V| -> V) -> Map<K, V> {
        match *self {
            Tip => Map::singleton_arc(key, val),
            Bin { size: ref sizex, key: ref keyx, value: ref valuex,
                  left: ref leftx, right: ref rightx } => {
                match key.cmp(&**keyx) {
                    Equal   => Map::bin_ref(&key, &Arc::new(modifier(&**valuex)), leftx, rightx),
                    Less    => Map::balance(keyx.clone(), valuex.clone(),
                                            Arc::new(leftx.insert(key, val)), rightx.clone()),
                    Greater => Map::balance(keyx.clone(), valuex.clone(),
                                            leftx.clone(), Arc::new(rightx.insert(key, val))),
                }
            }
        }
    }
}

static RATIO: uint = 2;
static DELTA: uint = 3;

// Balancing
impl<K: Send + Sync + Ord, V: Send + Sync> Map<K, V> {
    // Create a balanced tree from its constituent parts.
    fn balance(key: Arc<K>, value: Arc<V>, left: Arc<Map<K, V>>, right: Arc<Map<K, V>>) -> Map<K, V> {
        if left.len() + right.len() <= 1 {
            Map::bin(key, value, left, right)
        } else if right.len() > DELTA * left.len() {
            Map::rotate_left(key, value, left, right)
        } else if left.len() > DELTA * right.len() {
            Map::rotate_right(key, value, left, right)
        } else {
            Map::bin(key, value, left, right)
        }
    }

    fn rotate_left(key: Arc<K>, value: Arc<V>, left: Arc<Map<K, V>>, right: Arc<Map<K, V>>) -> Map<K, V> {
        match right.deref() {
            &Tip => fail!("irrefutable pattern match failed."),
            &Bin { left: ref l, right: ref r, .. } => {
                if l.len() < RATIO * r.len() {
                    Map::single_left(key, value, left, right.clone())
                } else {
                    Map::double_left(key, value, left, right.clone())
                }
            }
        }
    }

    fn rotate_right(key: Arc<K>, value: Arc<V>, left: Arc<Map<K, V>>, right: Arc<Map<K, V>>) -> Map<K, V> {
        match left.deref() {
            &Tip => fail!("irrefutable pattern match failed."),
            &Bin { left: ref l, right: ref r, .. } => {
                if r.len() < RATIO * l.len() {
                    Map::single_right(key, value, left.clone(), right)
                } else {
                    Map::double_right(key, value, left.clone(), right)
                }
            }
        }
    }

    fn single_left(key: Arc<K>, value: Arc<V>, left: Arc<Map<K, V>>, right: Arc<Map<K, V>>) -> Map<K, V> {
        match right.deref() {
            &Tip => fail!("irrefutable pattern match failed."),
            &Bin { key: ref kx, value: ref vx, left: ref lx, right: ref rx, .. } => {
                Map::bin_ref(kx, vx, &Arc::new(Map::bin(key, value, left, lx.clone())), rx)
            }
        }
    }

    fn single_right(key: Arc<K>, value: Arc<V>, left: Arc<Map<K, V>>, right: Arc<Map<K, V>>) -> Map<K, V> {
        match left.deref() {
            &Tip => fail!("irrefutable pattern match failed."),
            &Bin { key: ref kx, value: ref vx, left: ref lx, right: ref rx, .. } => {
                Map::bin_ref(kx, vx, lx, &Arc::new(Map::bin(key, value, rx.clone(), right)))
            }
        }
    }

    fn double_left(key: Arc<K>, value: Arc<V>, left: Arc<Map<K, V>>, right: Arc<Map<K, V>>) -> Map<K, V> {
        match right.deref() {
            &Tip => fail!("irrefutable pattern match failed."),
            &Bin { key: ref kx, value: ref vx, left: ref lx, right: ref rx, .. } => {
                match lx.clone().deref() {
                    &Tip => fail!("irrefutable pattern match failed."),
                    &Bin { key: ref ky, value: ref vy, left: ref ly, right: ref ry, .. } => {
                        Map::bin_ref(ky, vy,
                                     &Arc::new(Map::bin(key, value, lx.clone(), ly.clone())),
                                     &Arc::new(Map::bin_ref(kx, vx, ry, rx)))
                    }
                }
            }
        }
    }

    fn double_right(key: Arc<K>, value: Arc<V>, left: Arc<Map<K, V>>, right: Arc<Map<K, V>>) -> Map<K, V> {
        match left.deref() {
            &Tip => fail!("irrefutable pattern match failed."),
            &Bin { key: ref kx, value: ref vx, left: ref lx, right: ref rx, .. } => {
                match rx.clone().deref() {
                    &Tip => fail!("irrefutable pattern match failed."),
                    &Bin { key: ref ky, value: ref vy, left: ref ly, right: ref ry, .. } => {
                        Map::bin_ref(ky, vy,
                                     &Arc::new(Map::bin_ref(kx, vx, lx, ly)),
                                     &Arc::new(Map::bin_ref(&key, &value, ry, &right)))
                    }
                }
            }
        }
    }

    // Create a tree with size and balance restored.
    fn link(key: Arc<K>, value: Arc<V>, left: Arc<Map<K, V>>, right: Arc<Map<K, V>>) -> Map<K, V> {
        match (left.deref(), right.deref()) {
            (&Tip, r) => r.insert(key, value),
            (l, &Tip) => l.insert(key, value),
            (&Bin { size: ref szl, key: ref kl, value: ref vl,
                           left: ref ll, right: ref rl},
             &Bin { size: ref szr, key: ref kr, value: ref vr,
                           left: ref lr, right: ref rr}) => {
                if DELTA * *szl < *szr {
                    Map::balance(kr.clone(), vr.clone(),
                                 Arc::new(Map::link(key, value, left.clone(), lr.clone())),
                                 rr.clone())
                } else if DELTA * *szr < *szl {
                    Map::balance(kl.clone(), vl.clone(), ll.clone(),
                                 Arc::new(Map::link(key, value, rl.clone(), right.clone())))
                } else {
                    Map::bin_ref(&key, &value, &left, &right)
                }
            }
        }
    }

    // Merge two trees and restore their balance.
    fn merge(one: Arc<Map<K, V>>, two: Arc<Map<K, V>>) -> Map<K, V> {
        match (one.deref(), two.deref()) {
            (&Tip, r) => r.clone(),
            (l, &Tip) => l.clone(),
            (&Bin { size: ref szl, key: ref kl, value: ref vl,
                           left: ref ll, right: ref rl},
             &Bin { size: ref szr, key: ref kr, value: ref vr,
                           left: ref lr, right: ref rr}) => {
                if DELTA * *szl < *szr {
                    Map::balance(kr.clone(), vr.clone(),
                                 Arc::new(Map::merge(one.clone(), lr.clone())),
                                 rr.clone())
                } else if DELTA * *szr < *szl {
                    Map::balance(kl.clone(), vl.clone(), ll.clone(),
                                 Arc::new(Map::merge(rl.clone(), two.clone())))
                } else {
                    Map::glue(one.clone(), two.clone())
                }
            }
        }
    }

    // Glue two trees together, assuming that they are balanced with respect to
    // each other (all keys in left are smaller than all keys in right).
    fn glue(left: Arc<Map<K, V>>, right: Arc<Map<K, V>>) -> Map<K, V> {
        match (left.deref(), right.deref()) {
            (&Tip, r) => r.clone(),
            (l, &Tip) => l.clone(),
            (l, r) => {
                if l.len() > r.len() {
                    let (km, max) = l.max().unwrap();
                    let lx = Arc::new(l.delete_max().unwrap());
                    Map::balance(km, max, lx, right.clone())
                } else {
                    let (km, min) = r.min().unwrap();
                    let rx = Arc::new(r.delete_min().unwrap());
                    Map::balance(km, min, left.clone(), rx)
                }
            }
        }
    }
}

// Deletion
impl<K: Send + Sync + Ord, V: Send + Sync> Map<K, V> {
    /// Delete a key and its value from the map.
    ///
    /// If the key is not a member of the map, the original map is returned.
    pub fn delete(&self, key: &K) -> Map<K, V> {
        match *self {
            Tip => Tip,
            Bin { key: ref kx, value: ref vx, left: ref l, right: ref r, .. } => {
                match key.cmp(&**kx) {
                    Less    => Map::balance(kx.clone(), vx.clone(), Arc::new(l.delete(key)), r.clone()),
                    Greater => Map::balance(kx.clone(), vx.clone(), l.clone(), Arc::new(r.delete(key))),
                    Equal   => Map::glue(l.clone(), r.clone())
                }
            }
        }
    }
}

// Updates
impl<K: Send + Sync + Ord, V: Send + Sync> Map<K, V> {
    /// Adjust the value at a specified key with the provided closure.
    ///
    /// If they key is not a member of the map, the original map is returned.
    pub fn adjust(&self, key: &K, modifier: |&V| -> V) -> Map<K, V> {
        match *self {
            Tip => Tip,
            Bin { key: ref kx, value: ref vx, left: ref l, right: ref r, .. } => {
                match key.cmp(&**kx) {
                    Less    => Map::balance(kx.clone(), vx.clone(), Arc::new(l.adjust(key, modifier)), r.clone()),
                    Greater => Map::balance(kx.clone(), vx.clone(), l.clone(), Arc::new(r.adjust(key, modifier))),
                    Equal   => Map::bin(kx.clone(), Arc::new(modifier(&**vx)), l.clone(), r.clone())
                }
            }
        }
    }

    /// Conditionally update the key in the map with the provided closure. If the closure
    /// returns None, then the key value pair is deleted.
    pub fn update(&self, key: &K, modifier: |&V| -> Option<V>) -> Map<K, V> {
        match *self {
            Tip => Tip,
            Bin { key: ref kx, value: ref vx, left: ref l, right: ref r, .. } => {
                match key.cmp(&**kx) {
                    Less    => Map::balance(kx.clone(), vx.clone(), Arc::new(l.update(key, modifier)), r.clone()),
                    Greater => Map::balance(kx.clone(), vx.clone(), l.clone(), Arc::new(r.update(key, modifier))),
                    Equal   => {
                        match modifier(&**vx) {
                            // Alter the key at this value
                            Some(val) => Map::bin(kx.clone(), Arc::new(val), l.clone(), r.clone()),
                            // Delete this key from the map
                            None => Map::glue(l.clone(), r.clone())
                        }
                    }
                }
            }
        }
    }

    /// Alter the value at the provided key, can be used to update, insert, or
    /// delete from the map.
    ///
    /// The provided closure is called with `Some(&key)`, `Some(&value)` if the key is found, and
    /// None if it is not found. If the closure returns Some(value) then that
    /// value replaces the value currently at that key in the map or inserts
    /// the value into the map; if it returns None then that key value pair
    /// will be deleted or will remain not-inserted.
    pub fn alter(&self, key: Arc<K>, modifier: |Option<&K>, Option<&V>| -> Option<V>) -> Map<K, V> {
        match *self {
            Tip => {
                match modifier(None, None) {
                    // Insert this key into the map.
                    Some(val) => Map::singleton_arc(key, Arc::new(val)),
                    // Stay not-inserted.
                    None => Tip
                }
            },
            Bin { key: ref kx, value: ref vx, left: ref l, right: ref r, .. } => {
                match key.cmp(&**kx) {
                    Less    => Map::balance(kx.clone(), vx.clone(), Arc::new(l.alter(key, modifier)), r.clone()),
                    Greater => Map::balance(kx.clone(), vx.clone(), l.clone(), Arc::new(r.alter(key, modifier))),
                    Equal   => {
                        match modifier(Some(&**kx), Some(&**vx)) {
                            // Alter the key at this value
                            Some(val) => Map::bin(kx.clone(), Arc::new(val), l.clone(), r.clone()),
                            // Delete this key from the map
                            None => Map::glue(l.clone(), r.clone())
                        }
                    }
                }
            }
        }
    }
}

// Indexing
impl<K: Send + Sync + Ord, V: Send + Sync> Map<K, V> {
    /// Return the 0-based index of the key in a sorted sequence of all the
    /// keys in the map.
    pub fn find_index(&self, key: &K) -> Option<uint> {
        unimplemented!()
    }

    /// Lookup by the index of the key in a sorted sequence of all the keys.
    pub fn lookup_index<'a>(&'a self, key: uint) -> Option<&'a V> {
        unimplemented!()
    }

    /// Update a value by its keys 0-based index in a sorted sequence of all
    /// the keys in the map.
    ///
    /// See the behavior of alter for the responses to the closure.
    pub fn alter_index(&self, key: uint, modifier: |Option<&V>| -> Option<V>) -> Map<K, V> {
        unimplemented!()
    }

    /// Delete the element at this index.
    pub fn delete_index(&self, key: uint) -> Map<K, V> {
        unimplemented!()
    }
}

// Min/Max
impl<K: Send + Sync + Ord, V: Send + Sync> Map<K, V> {
    /// Find the minimum pair in the map.
    pub fn min(&self) -> Option<(Arc<K>, Arc<V>)> {
        match *self {
            Tip => None,
            Bin { ref left, ref right, ref key, ref value, .. } => {
                match (left.deref(), right.deref()) {
                    // This is a tree with a right pointer only.
                    // Return the current val because it is the min.
                    (&Tip, ref rr) => Some((key.clone(), value.clone())),
                    // This is a tree with a left pointer. Recurse on it.
                    (ref ll, _) => ll.min()
                }
            }
        }
    }

    /// Find the maximum pair in the map.
    pub fn max(&self) -> Option<(Arc<K>, Arc<V>)> {
        match *self {
            Tip => None,
            Bin { ref left, ref right, ref key, ref value, .. } => {
                match (left.deref(), right.deref()) {
                    // This is a tree with a left pointer only.
                    // The current val is the max.
                    (ref ll, &Tip) => Some((key.clone(), value.clone())),
                    // This is a tree with a right pointer. Recurse on it.
                    (_, ref rr) => rr.min()
                }
            }
        }
    }

    /// Delete the minimum element in the map.
    ///
    /// Returns None if the map is empty.
    pub fn delete_min(&self) -> Option<Map<K, V>> {
        match *self {
            Tip => None,
            Bin { ref left, ref right, ref key, ref value, .. } => {
                match (left.deref(), right.deref()) {
                    // This is a leaf, the min is the current value.
                    (&Tip, &Tip) => Some(Tip),
                    // This is a tree with a right pointer only.
                    // Return that right branch, because the
                    // current val is the min.
                    (&Tip, ref rr) => Some((**rr).clone()),
                    // This is a tree with a left pointer. Recurse on it.
                    // ll is not a tip, delete_min cannot fail.
                    (ref ll, ref rr) =>
                        Some(Map::balance(key.clone(), value.clone(),
                                          Arc::new(ll.delete_min().unwrap()),
                                          right.clone()))
                }
            }
        }
    }

    /// Delete the maximum element in the map.
    ///
    /// Returns None if the map is empty.
    pub fn delete_max(&self) -> Option<Map<K, V>> {
        match *self {
            Tip => None,
            Bin { ref left, ref right, ref key, ref value, .. } => {
                match (left.deref(), right.deref()) {
                    // This is a leaf, the min is the current value.
                    (&Tip, &Tip) => Some(Tip),
                    // This is a tree with a left pointer only.
                    // Return that left branch, because the
                    // current val is the max.
                    (ref ll, &Tip) => Some((**ll).clone()),
                    // This is a tree with a right pointer. Recurse on it.
                    // rr is not a tip, delete_max cannot fail.
                    (ref ll, ref rr) =>
                        Some(Map::balance(key.clone(), value.clone(), left.clone(),
                                          Arc::new(rr.delete_max().unwrap())))
                }
            }
        }
    }
}

// Iterators
impl<K: Send + Sync, V: Send + Sync> Map<K, V> {
    /// Get a breadth-first iterator over the items in a map.
    pub fn bfs_iter(&self) -> BfsItems<K, V> {
        unimplemented!()
    }

    /// Get an inorder iterator over the items in a map.
    pub fn inorder_iter(&self) -> OrderItems<Arc<V>> {
        match *self {
            Tip => {
                let iter: Empty<Arc<V>> = Empty;
                OrderItems(box iter as Box<Iterator<Arc<V>>>)
            },
            Bin { ref left, ref right, ref value, .. } => {
                OrderItems(box left.preorder_iter()
                    .chain(Some(value.clone()).move_iter())
                    .chain(right.preorder_iter()) as Box<Iterator<Arc<V>>>)
            }
        }
    }

    /// Get a postorder iterator over the items in a map.
    pub fn preorder_iter(&self) -> OrderItems<Arc<V>> {
        match *self {
            Tip => {
                let iter: Empty<Arc<V>> = Empty;
                OrderItems(box iter as Box<Iterator<Arc<V>>>)
            },
            Bin { ref left, ref right, ref value, .. } => {
                OrderItems(box Some(value.clone()).move_iter()
                    .chain(left.preorder_iter())
                    .chain(right.preorder_iter()) as Box<Iterator<Arc<V>>>)
            }
        }
    }

    /// Get a postorder_iterator iterator over the items in a map.
    pub fn postorder_iter(&self) -> OrderItems<Arc<V>> {
        match *self {
            Tip => {
                let iter: Empty<Arc<V>> = Empty;
                OrderItems(box iter as Box<Iterator<Arc<V>>>)
            },
            Bin { ref left, ref right, ref value, .. } => {
                OrderItems(box left.preorder_iter()
                    .chain(right.preorder_iter())
                    .chain(Some(value.clone()).move_iter()) as Box<Iterator<Arc<V>>>)
            }
        }
    }

}

/// A breadth-first iterator over the pairs of a map.
pub struct BfsItems<'a, K, V> {
    map: &'a Map<K, V>
}

/// An iterator
pub struct OrderItems<V>(Box<Iterator<V>>);

impl<V> Iterator<V> for OrderItems<V> {
    fn next(&mut self) -> Option<V> {
        let OrderItems(ref mut iter) = *self;
        iter.next()
    }
}

/// An empty iterator
pub struct Empty<V>;

impl<T> Iterator<T> for Empty<T> {
    fn next(&mut self) -> Option<T> { None }
}

