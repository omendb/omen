//! DataFusion integration for OmenDB
//!
//! Provides TableProvider implementations:
//! - RedbTable: Legacy redb storage with learned index
//! - ArrowTableProvider: Modern Table system with ALEX + Arrow/Parquet

pub mod arrow_table_provider;
pub mod redb_table;

pub use arrow_table_provider::ArrowTableProvider;
pub use redb_table::RedbTable;
