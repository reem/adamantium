use std::sync::Arc;

/// A functional, shareable, persistent singly linked list.
pub enum List<T> {
    /// A list with a head and a tail.
    Cons(T, Arc<List<T>>),

    /// The empty list.
    Nil
}

impl<T> List<T> {
    /// Construct a new, empty list.
    #[inline]
    pub fn new() -> List<T> { Nil }
}

impl<T: Send + Sync> List<T> {
    /// Create a list with one element in it.
    #[inline]
    pub fn singleton(val: T) -> List<T> { Cons(val, Arc::new(Nil)) }

    /// Get the head of a list.
    pub fn head(&self) -> Option<&T> {
        match *self {
            Nil => None,
            Cons(ref head, _) => Some(head)
        }
    }

    /// Get the tail of a list.
    pub fn tail(&self) -> Option<Arc<List<T>>> {
        match *self {
            Nil => None,
            Cons(_, ref tail) => Some(tail.clone())
        }
    }

    /// Get an iterator over the items in a list.
    pub fn iter<'a>(&'a self) -> ListItems<'a, T> {
        ListItems {
            list: self
        }
    }
}

/// An iterator over the items in a list.
pub struct ListItems<'a, T: 'a> {
    list: &'a List<T>
}

impl<'a, T: Send + Sync> Iterator<&'a T> for ListItems<'a, T> {
    fn next(&mut self) -> Option<&'a T> {
        match *self.list {
            Cons(ref head, ref tail) => {
                self.list = &**tail;
                Some(head)
            },
            Nil => None
        }
    }
}

