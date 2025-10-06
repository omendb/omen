//! OmenDB Library - Pure Learned Index Database
//! Production hardening: concurrency, testing, monitoring

// New architecture (proper multi-table database)
pub mod catalog;
pub mod connection_pool;
pub mod logging;
pub mod mvcc;
pub mod row;
pub mod sql_engine;
pub mod table;
pub mod table_index;
pub mod table_storage;
pub mod table_wal;
pub mod value;

// Query routing (Phase 9.2)
pub mod cost_estimator;
pub mod query_classifier;
pub mod query_router;

// Re-exports for common types
pub use connection_pool::{Connection, ConnectionPool, PoolConfig};
pub use logging::{init_from_env, init_logging, LogConfig};
pub use sql_engine::QueryConfig;

// Existing modules (will be refactored)
pub mod alex; // ALEX adaptive learned index (replacement for RMI)
pub mod alex_storage; // Custom mmap-based storage with ALEX (10x faster queries)
pub mod alex_storage_wal; // Write-Ahead Log for AlexStorage durability
pub mod alex_storage_concurrent; // Thread-safe wrapper for AlexStorage
pub mod backup;
pub mod concurrent;
pub mod datafusion;
pub mod index;
pub mod metrics;
pub mod postgres;
pub mod redb_storage;
pub mod rocks_storage;
pub mod rest;
pub mod security;
pub mod server;
pub mod storage;
pub mod wal;

#[cfg(test)]
mod tests;

// Scale testing module available for benchmarking
pub mod scale_tests;

// Integration testing module
pub mod integration_tests;

// Comprehensive multi-table integration tests
#[cfg(test)]
mod multi_table_tests;

use crate::metrics::{record_insert, record_insert_failure, record_search, record_search_failure};
use anyhow::Result;
use std::time::Instant;
use storage::ArrowStorage;

/// Main OmenDB structure combining learned index and Arrow storage
pub struct OmenDB {
    /// Learned index for fast lookups
    pub index: index::RecursiveModelIndex,

    /// Columnar storage backend
    pub storage: ArrowStorage,

    /// Database name
    pub name: String,
}

impl OmenDB {
    /// Create new database instance
    pub fn new(name: &str) -> Self {
        Self {
            index: index::RecursiveModelIndex::new(1_000_000),
            storage: ArrowStorage::new(),
            name: name.to_string(),
        }
    }

    /// Insert time-series data
    pub fn insert(&mut self, timestamp: i64, value: f64, series_id: i64) -> Result<()> {
        let start = Instant::now();

        // Insert into storage
        let result = self.storage.insert(timestamp, value, series_id);

        if result.is_ok() {
            // Update learned index
            self.index.add_key(timestamp);

            // Record success metric
            let duration = start.elapsed().as_secs_f64();
            record_insert(duration);
        } else {
            // Record failure metric
            record_insert_failure();
        }

        result
    }

    /// Point query using learned index
    pub fn get(&self, timestamp: i64) -> Option<f64> {
        let start = Instant::now();

        // Use learned index to find position
        let result = if let Some(_pos) = self.index.search(timestamp) {
            // In real implementation, would fetch from storage
            // For now, return placeholder
            Some(0.0)
        } else {
            None
        };

        // Record metrics
        let duration = start.elapsed().as_secs_f64();
        if result.is_some() {
            record_search(duration);
        } else {
            record_search_failure();
        }

        result
    }

    /// Range query using learned index
    pub fn range_query(&self, start: i64, end: i64) -> Result<Vec<(i64, f64)>> {
        // Use storage's range query which integrates with learned index
        let batches = self.storage.range_query(start, end)?;

        // Convert Arrow batches to simple format
        let mut results = Vec::new();
        for batch in batches {
            // Extract timestamp and value columns
            if let (Some(timestamps), Some(values)) = (
                batch
                    .column(0)
                    .as_any()
                    .downcast_ref::<arrow::array::TimestampMicrosecondArray>(),
                batch
                    .column(1)
                    .as_any()
                    .downcast_ref::<arrow::array::Float64Array>(),
            ) {
                for i in 0..batch.num_rows() {
                    results.push((timestamps.value(i), values.value(i)));
                }
            }
        }

        Ok(results)
    }

    /// Time-series aggregations
    pub fn sum(&self, start: i64, end: i64) -> Result<f64> {
        self.storage.aggregate_sum(start, end)
    }

    pub fn avg(&self, start: i64, end: i64) -> Result<f64> {
        self.storage.aggregate_avg(start, end)
    }
}
