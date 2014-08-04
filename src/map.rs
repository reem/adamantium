use std::sync::Arc;
use std::collections;

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

impl<K, V> collections::Collection for Map<K, V> {
    #[inline]
    fn len(&self) -> uint {
        match *self {
            Bin { size, .. } => size,
            Tip => 0
        }
    }
}

impl<K: Ord + Send + Share, V: Send + Share> collections::Map<K, V> for Map<K, V> {
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

impl<K: Ord + Send + Share, V: Send + Share> collections::Set<K> for Map<K, V> {
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
impl<K: Send + Share, V: Send + Share> Map<K, V> {
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

