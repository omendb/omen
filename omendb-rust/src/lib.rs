//! OmenDB Library - Pure Learned Index Database
//! Production hardening: concurrency, testing, monitoring

pub mod storage;
pub mod index;
pub mod concurrent;

#[cfg(test)]
mod tests;

use storage::ArrowStorage;
use anyhow::Result;

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
        // Insert into storage
        self.storage.insert(timestamp, value, series_id)?;

        // Update learned index
        self.index.add_key(timestamp);

        Ok(())
    }

    /// Point query using learned index
    pub fn get(&self, timestamp: i64) -> Option<f64> {
        // Use learned index to find position
        if let Some(_pos) = self.index.search(timestamp) {
            // In real implementation, would fetch from storage
            // For now, return placeholder
            Some(0.0)
        } else {
            None
        }
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
                batch.column(0).as_any().downcast_ref::<arrow::array::TimestampMicrosecondArray>(),
                batch.column(1).as_any().downcast_ref::<arrow::array::Float64Array>()
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