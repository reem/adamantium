#![license = "MIT"]
#![deny(missing_docs)]
#![deny(warnings)]

#![feature(macro_rules)]

//! Persistent, immutable, functional data structures in Rust.

pub use self::list::List;
pub use self::map::Map;

/// Contains the list type.
pub mod list;

/// Contains the map type.
pub mod map;

