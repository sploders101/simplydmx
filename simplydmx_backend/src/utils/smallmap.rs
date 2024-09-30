//! This provides a place to tinker with different map types
//! and see how they perform. I don't really think hashing is
//! necessary for most of the collection types in the program,
//! so this lets me flesh out features and tinker with optimization
//! later.

use rustc_hash::FxHashMap;

pub type SmallMap<K, V> = FxHashMap<K, V>;
