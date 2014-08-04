#![license = "MIT"]
#![deny(missing_doc)]
#![deny(warnings)]
#![allow(unused_variable, dead_code)]

#![feature(struct_variant, macro_rules)]

//! Persistent, immutable, functional data structures in Rust.

pub use self::list::List;
pub use self::map::Map;

/// Contains the list type.
pub mod list;

/// Contains the map type.
pub mod map;

