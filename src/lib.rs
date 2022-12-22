include!(concat!(env!("OUT_DIR"), "/code_table.rs"));

pub mod code_table_type;
mod string;

pub use string::*;

/// The type of hashmap used in this crate.
///
/// The hash library may be changed in the future release.
/// Make sure to use only APIs compatible with `std::collections::HashMap`.
pub type OEMCPHashMap<K, V> = phf::Map<K, V>;
