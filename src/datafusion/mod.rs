//! DataFusion integration for OmenDB
//!
//! Provides TableProvider implementations:
//! - ArrowTableProvider: Modern Table system with ALEX + Arrow/Parquet

pub mod arrow_table_provider;
// redb_table commented out - redb_storage archived to omen-core

pub use arrow_table_provider::ArrowTableProvider;
