//! DataFusion integration for OmenDB
//!
//! Provides TableProvider implementation for redb storage with learned index optimization.

pub mod redb_table;

pub use redb_table::RedbTable;
