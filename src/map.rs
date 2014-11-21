use std::sync::Arc;
use std::default::Default;

pub use self::Map::{Bin, Tip};

/// A key value store, implemented as a persistent, functional
/// size balanced binary search tree.
pub enum Map<K, V> {
    /// A branch node.
    Bin {
        /// The size of this branch.
        size: uint,

        /// The key associated with this node.
        key: Arc<K>,

        /// The value associated with this node.
        value: Arc<V>,

        /// The left branch of this node.
        left: Arc<Map<K, V>>,

        /// The right branch of this node.
        right: Arc<Map<K, V>>
    },

    /// A leaf node.
    Tip
}

impl<K: Send + Sync, V: Send + Sync> Clone for Map<K, V> {
    fn clone(&self) -> Map<K, V> {
        match *self {
            Tip => Tip,
            Bin { ref key, ref value, ref left, ref right, .. } => {
                Map::bin_ref(key, value, left, right)
            }
        }
    }
}

impl<K, V> Map<K, V> {
    /// How many items are in the map.
    #[inline]
    pub fn len(&self) -> uint {
        match *self {
            Bin { size, .. } => size,
            Tip => 0
        }
    }
}

impl<K: Ord + Send + Sync, V: Send + Sync> Map<K, V> {
    /// Lookup a value in the map.
    pub fn get<'a>(&'a self, lookup: &K) -> Option<&'a V> {
        match *self {
            Bin { ref key, ref left, ref right, ref value, .. } => match key.deref().cmp(lookup) {
                Equal   => Some(&**value),
                Less    => left.get(lookup),
                Greater => right.get(lookup)
            },
            Tip => None
        }
    }
}

impl<K: Ord + Send + Sync, V: Send + Sync> Map<K, V> {
    /// Is this key in the map?
    pub fn contains(&self, lookup: &K) -> bool {
        self.get(lookup).is_some()
    }

//     fn is_disjoint(&self, other: &Map<K, V>) -> bool {
//         self.inorder_iter().all(|(k, _)| !other.contains(k.deref()))
//     }
//
//     fn is_subset(&self, other: &Map<K, V>) -> bool {
//         self.inorder_iter().all(|(k, _)| other.contains(k.deref()))
//     }
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
            Bin { key: ref keyx, value: ref valuex,
                  left: ref leftx, right: ref rightx, .. } => {
                match key.cmp(&*keyx) {
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
            Bin { key: ref keyx, value: ref valuex,
                  left: ref leftx, right: ref rightx, .. } => {
                match key.cmp(&*keyx) {
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
            Bin { key: ref keyx, value: ref valuex,
                  left: ref leftx, right: ref rightx, .. } => {
                match key.cmp(&*keyx) {
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
            &Tip => panic!("irrefutable pattern match failed."),
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
            &Tip => panic!("irrefutable pattern match failed."),
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
            &Tip => panic!("irrefutable pattern match failed."),
            &Bin { key: ref kx, value: ref vx, left: ref lx, right: ref rx, .. } => {
                Map::bin_ref(kx, vx, &Arc::new(Map::bin(key, value, left, lx.clone())), rx)
            }
        }
    }

    fn single_right(key: Arc<K>, value: Arc<V>, left: Arc<Map<K, V>>, right: Arc<Map<K, V>>) -> Map<K, V> {
        match left.deref() {
            &Tip => panic!("irrefutable pattern match failed."),
            &Bin { key: ref kx, value: ref vx, left: ref lx, right: ref rx, .. } => {
                Map::bin_ref(kx, vx, lx, &Arc::new(Map::bin(key, value, rx.clone(), right)))
            }
        }
    }

    // FIXME: Something is wrong with this code. It should use left, but it
    // does not.
    fn double_left(key: Arc<K>, value: Arc<V>, _left: Arc<Map<K, V>>, right: Arc<Map<K, V>>) -> Map<K, V> {
        match right.deref() {
            &Tip => panic!("irrefutable pattern match failed."),
            &Bin { key: ref kx, value: ref vx, left: ref lx, right: ref rx, .. } => {
                match lx.clone().deref() {
                    &Tip => panic!("irrefutable pattern match failed."),
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
            &Tip => panic!("irrefutable pattern match failed."),
            &Bin { key: ref kx, value: ref vx, left: ref lx, right: ref rx, .. } => {
                match rx.clone().deref() {
                    &Tip => panic!("irrefutable pattern match failed."),
                    &Bin { key: ref ky, value: ref vy, left: ref ly, right: ref ry, .. } => {
                        Map::bin_ref(ky, vy,
                                     &Arc::new(Map::bin_ref(kx, vx, lx, ly)),
                                     &Arc::new(Map::bin_ref(&key, &value, ry, &right)))
                    }
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
                match key.cmp(&*kx) {
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
                    (&Tip, _) => Some((key.clone(), value.clone())),
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
                    (_, &Tip) => Some((key.clone(), value.clone())),
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
                    (ref ll, _) =>
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
                    (_, ref rr) =>
                        Some(Map::balance(key.clone(), value.clone(), left.clone(),
                                          Arc::new(rr.delete_max().unwrap())))
                }
            }
        }
    }
}

// Iterators
// impl<K: Send + Sync, V: Send + Sync> Map<K, V> {
//     /// Get a breadth-first iterator over the items in a map.
//     pub fn bfs_iter(map: Arc<Map<K, V>>) -> BfsItems<K, V> {
//         let mut queue = collections::RingBuf::new();
//         queue.push(map);
//         BfsItems {
//             queue: queue
//         }
//     }
//
//     /// Get an inorder iterator over the items in a map.
//     pub fn inorder_iter(&self) -> OrderItems<(Arc<K>, Arc<V>)> {
//         match *self {
//             Tip => {
//                 let iter: Empty<(Arc<K>, Arc<V>)> = Empty;
//                 OrderItems(box iter as Box<Iterator<(Arc<K>, Arc<V>)>>)
//             },
//             Bin { ref left, ref right, ref value, ref key, .. } => {
//                 OrderItems(box left.preorder_iter()
//                     .chain(Some((key.clone(), value.clone())).into_iter())
//                     .chain(right.preorder_iter()) as Box<Iterator<(Arc<K>, Arc<V>)>>)
//             }
//         }
//     }
//
//     /// Get a postorder iterator over the items in a map.
//     pub fn preorder_iter(&self) -> OrderItems<(Arc<K>, Arc<V>)> {
//         match *self {
//             Tip => {
//                 let iter: Empty<(Arc<K>, Arc<V>)> = Empty;
//                 OrderItems(box iter as Box<Iterator<(Arc<K>, Arc<V>)>>)
//             },
//             Bin { ref left, ref right, ref value, ref key, .. } => {
//                 OrderItems(box Some((key.clone(), value.clone())).into_iter()
//                     .chain(left.preorder_iter())
//                     .chain(right.preorder_iter()) as Box<Iterator<(Arc<K>, Arc<V>)>>)
//             }
//         }
//     }
//
//     /// Get a postorder_iterator iterator over the items in a map.
//     pub fn postorder_iter(&self) -> OrderItems<(Arc<K>, Arc<V>)> {
//         match *self {
//             Tip => {
//                 let iter: Empty<(Arc<K>, Arc<V>)> = Empty;
//                 OrderItems(box iter as Box<Iterator<(Arc<K>, Arc<V>)>>)
//             },
//             Bin { ref left, ref right, ref value, ref key, .. } => {
//                 OrderItems(box left.preorder_iter()
//                     .chain(right.preorder_iter())
//                     .chain(Some((key.clone(), value.clone())).into_iter()) as Box<Iterator<(Arc<K>, Arc<V>)>>)
//             }
//         }
//     }
//
// }
//
// /// A breadth-first iterator over the pairs of a map.
// pub struct BfsItems<K, V> {
//     queue: collections::RingBuf<Arc<Map<K, V>>>
// }
//
// impl<K: Send + Sync, V: Send + Sync> Iterator<(Arc<K>, Arc<V>)> for BfsItems<K, V> {
//     fn next(&mut self) -> Option<(Arc<K>, Arc<V>)> {
//         match self.queue.pop_front() {
//             Some(next) => {
//                 match *next {
//                     Tip => self.next(),
//                     Bin { ref left, ref right, ref key, ref value, .. } => {
//                         self.queue.push(left.clone());
//                         self.queue.push(right.clone());
//                         Some((key.clone(), value.clone()))
//                     }
//                 }
//             },
//             None => None
//         }
//     }
// }
//
// /// An iterator
// pub struct OrderItems<V>(Box<Iterator<V> + 'static>);
//
// impl<V> Iterator<V> for OrderItems<V> {
//     fn next(&mut self) -> Option<V> {
//         let OrderItems(ref mut iter) = *self;
//         iter.next()
//     }
// }
//
// /// An empty iterator
// pub struct Empty<V>;
//
// impl<T> Iterator<T> for Empty<T> {
//     fn next(&mut self) -> Option<T> { None }
// }
//
