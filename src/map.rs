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

