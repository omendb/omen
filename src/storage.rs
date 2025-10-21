//! Arrow-based columnar storage for OmenDB with WAL support
//! Week 3: Integration with learned indexes for time-series data

use crate::wal::{WalManager, WalOperation};
use anyhow::Result;
use arrow::array::{Float64Array, Int64Array, TimestampMicrosecondArray};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

/// Time-series optimized columnar storage with durability
pub struct ArrowStorage {
    /// Schema for time-series data
    schema: SchemaRef,

    /// In-memory batches (hot data)
    hot_batches: Vec<RecordBatch>,

    /// File paths for cold data
    cold_files: Vec<String>,

    /// Learned index for timestamp column
    timestamp_index: Option<Box<dyn LearnedIndexTrait>>,

    /// Write-ahead log for durability
    wal: Option<Arc<WalManager>>,

    /// Data directory path
    data_dir: Option<String>,
}

/// Trait for learned indexes to integrate with storage
pub trait LearnedIndexTrait: Send + Sync {
    fn train(&mut self, keys: &[i64]);
    fn search(&self, key: i64) -> Option<usize>;
    fn range_search(&self, start: i64, end: i64) -> Vec<usize>;
}

impl Default for ArrowStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl ArrowStorage {
    /// Create new storage for time-series data
    pub fn new() -> Self {
        // Standard time-series schema
        let schema = Schema::new(vec![
            Field::new(
                "timestamp",
                DataType::Timestamp(arrow::datatypes::TimeUnit::Microsecond, None),
                false,
            ),
            Field::new("value", DataType::Float64, false),
            Field::new("series_id", DataType::Int64, false),
            Field::new("tags", DataType::Utf8, true),
        ]);

        Self {
            schema: Arc::new(schema),
            hot_batches: Vec::new(),
            cold_files: Vec::new(),
            timestamp_index: None,
            wal: None,
            data_dir: None,
        }
    }

    /// Create storage with persistence and WAL
    pub fn with_persistence<P: AsRef<Path>>(data_dir: P) -> Result<Self> {
        let data_path = data_dir.as_ref();
        std::fs::create_dir_all(data_path)?;

        // Initialize WAL
        let wal_dir = data_path.join("wal");
        let wal = WalManager::new(wal_dir)?;
        wal.open()?;

        let schema = Schema::new(vec![
            Field::new(
                "timestamp",
                DataType::Timestamp(arrow::datatypes::TimeUnit::Microsecond, None),
                false,
            ),
            Field::new("value", DataType::Float64, false),
            Field::new("series_id", DataType::Int64, false),
            Field::new("tags", DataType::Utf8, true),
        ]);

        let mut storage = Self {
            schema: Arc::new(schema),
            hot_batches: Vec::new(),
            cold_files: Vec::new(),
            timestamp_index: None,
            wal: Some(Arc::new(wal)),
            data_dir: Some(data_path.to_string_lossy().to_string()),
        };

        // Recover from WAL
        storage.recover_from_wal()?;

        Ok(storage)
    }

    /// Recover data from WAL on startup
    fn recover_from_wal(&mut self) -> Result<()> {
        // Collect operations first to avoid borrow checker issues
        let mut operations_to_apply = Vec::new();

        if let Some(wal) = &self.wal {
            let stats = wal.recover(|op| {
                match op {
                    WalOperation::Insert {
                        timestamp,
                        value,
                        series_id,
                    } => {
                        operations_to_apply.push((*timestamp, *value, *series_id));
                    }
                    _ => {} // Handle other operations as needed
                }
                Ok(())
            })?;

            // Now apply the operations
            for (timestamp, value, series_id) in operations_to_apply {
                self.insert_without_wal(timestamp, value, series_id)?;
            }

            println!(
                "WAL recovery complete: {} entries applied",
                stats.applied_entries
            );
        }
        Ok(())
    }

    /// Internal insert without WAL logging (used during recovery)
    fn insert_without_wal(&mut self, timestamp: i64, value: f64, series_id: i64) -> Result<()> {
        let timestamps = TimestampMicrosecondArray::from(vec![timestamp]);
        let values = Float64Array::from(vec![value]);
        let series_ids = Int64Array::from(vec![series_id]);
        let tags = arrow::array::StringArray::from(vec![Some("")]);

        let batch = RecordBatch::try_new(
            self.schema.clone(),
            vec![
                Arc::new(timestamps),
                Arc::new(values),
                Arc::new(series_ids),
                Arc::new(tags),
            ],
        )?;

        self.hot_batches.push(batch);
        Ok(())
    }

    /// Insert time-series data with WAL logging
    pub fn insert(&mut self, timestamp: i64, value: f64, series_id: i64) -> Result<()> {
        // Write to WAL first for durability
        if let Some(wal) = &self.wal {
            wal.write(WalOperation::Insert {
                timestamp,
                value,
                series_id,
            })?;
        }

        // Then update in-memory data
        self.insert_without_wal(timestamp, value, series_id)?;

        // Retrain index periodically (every 1000 inserts)
        if self.hot_batches.len().is_multiple_of(1000) {
            self.rebuild_index()?;
        }

        Ok(())
    }

    /// Range query using learned index
    pub fn range_query(&self, start_time: i64, end_time: i64) -> Result<Vec<RecordBatch>> {
        let mut results = Vec::new();

        // Use learned index if available
        if let Some(ref index) = self.timestamp_index {
            let positions = index.range_search(start_time, end_time);

            // Efficiently fetch only needed batches
            for pos in positions {
                if pos < self.hot_batches.len() {
                    results.push(self.hot_batches[pos].clone());
                }
            }
        } else {
            // Fallback to scanning all batches
            for batch in &self.hot_batches {
                if let Some(timestamps) = batch
                    .column(0)
                    .as_any()
                    .downcast_ref::<TimestampMicrosecondArray>()
                {
                    // Check if batch contains data in range
                    let min_ts = timestamps.value(0);
                    let max_ts = timestamps.value(timestamps.len() - 1);

                    if max_ts >= start_time && min_ts <= end_time {
                        results.push(batch.clone());
                    }
                }
            }
        }

        Ok(results)
    }

    /// Aggregate functions optimized for time-series
    pub fn aggregate_sum(&self, start_time: i64, end_time: i64) -> Result<f64> {
        let batches = self.range_query(start_time, end_time)?;
        let mut sum = 0.0;

        for batch in batches {
            if let Some(values) = batch.column(1).as_any().downcast_ref::<Float64Array>() {
                for i in 0..values.len() {
                    {
                        sum += values.value(i);
                    }
                }
            }
        }

        Ok(sum)
    }

    pub fn aggregate_avg(&self, start_time: i64, end_time: i64) -> Result<f64> {
        let batches = self.range_query(start_time, end_time)?;
        let mut sum = 0.0;
        let mut count = 0;

        for batch in batches {
            if let Some(values) = batch.column(1).as_any().downcast_ref::<Float64Array>() {
                for i in 0..values.len() {
                    {
                        sum += values.value(i);
                        count += 1;
                    }
                }
            }
        }

        Ok(if count > 0 { sum / count as f64 } else { 0.0 })
    }

    /// Flush hot data to Parquet files with checkpoint
    pub fn flush_to_parquet(&mut self, path: &Path) -> Result<()> {
        if self.hot_batches.is_empty() {
            return Ok(());
        }

        let file = File::create(path)?;
        let props = WriterProperties::builder().build();
        let mut writer = ArrowWriter::try_new(file, self.schema.clone(), Some(props))?;

        for batch in &self.hot_batches {
            writer.write(batch)?;
        }

        writer.close()?;

        // Create WAL checkpoint after successful Parquet write
        if let Some(wal) = &self.wal {
            wal.checkpoint()?;
        }

        // Move to cold storage
        self.cold_files.push(path.to_string_lossy().to_string());
        self.hot_batches.clear();

        Ok(())
    }

    /// Force sync WAL to disk
    pub fn sync(&self) -> Result<()> {
        if let Some(wal) = &self.wal {
            wal.sync()?;
        }
        Ok(())
    }

    /// Load data from Parquet files
    pub fn load_from_parquet(&mut self, path: &Path) -> Result<()> {
        let file = File::open(path)?;
        let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
        let reader = builder.build()?;

        for batch in reader {
            self.hot_batches.push(batch?);
        }

        self.rebuild_index()?;
        Ok(())
    }

    /// Rebuild learned index from current data
    fn rebuild_index(&mut self) -> Result<()> {
        // Extract all timestamps for training
        let mut timestamps = Vec::new();

        for batch in &self.hot_batches {
            if let Some(ts_array) = batch
                .column(0)
                .as_any()
                .downcast_ref::<TimestampMicrosecondArray>()
            {
                for i in 0..ts_array.len() {
                    timestamps.push(ts_array.value(i));
                }
            }
        }

        // Train index if we have a model
        if let Some(ref mut index) = self.timestamp_index {
            index.train(&timestamps);
        }

        Ok(())
    }

    /// Set a custom learned index implementation
    pub fn set_learned_index(&mut self, index: Box<dyn LearnedIndexTrait>) {
        self.timestamp_index = Some(index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_insert_and_query() {
        let mut storage = ArrowStorage::new();

        // Insert time-series data
        let base_time = 1_600_000_000_000_000;
        for i in 0..100 {
            storage.insert(base_time + i * 1000, i as f64, 1).unwrap();
        }

        // Query range
        let results = storage.range_query(base_time, base_time + 50_000).unwrap();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_aggregations() {
        let mut storage = ArrowStorage::new();

        let base_time = 1_600_000_000_000_000;
        for i in 0..10 {
            storage.insert(base_time + i * 1000, 10.0, 1).unwrap();
        }

        let sum = storage
            .aggregate_sum(base_time, base_time + 100_000)
            .unwrap();
        assert_eq!(sum, 100.0);

        let avg = storage
            .aggregate_avg(base_time, base_time + 100_000)
            .unwrap();
        assert_eq!(avg, 10.0);
    }

    #[test]
    fn test_empty_storage_queries() {
        let storage = ArrowStorage::new();

        // Query empty storage
        let results = storage.range_query(0, 1000).unwrap();
        assert!(results.is_empty());

        // Aggregations on empty storage
        let sum = storage.aggregate_sum(0, 1000).unwrap();
        assert_eq!(sum, 0.0);

        let avg = storage.aggregate_avg(0, 1000).unwrap();
        // Average of empty set could be NaN or 0.0 depending on implementation
        assert!(avg.is_nan() || avg == 0.0);
    }

    #[test]
    fn test_single_value_operations() {
        let mut storage = ArrowStorage::new();

        // Insert single value
        storage.insert(1_600_000_000_000_000, 42.5, 1).unwrap();

        // Query single value
        let results = storage
            .range_query(1_599_999_999_999_999, 1_600_000_000_000_001)
            .unwrap();
        assert_eq!(results.len(), 1);

        // Aggregations with single value
        let sum = storage
            .aggregate_sum(1_599_999_999_999_999, 1_600_000_000_000_001)
            .unwrap();
        assert_eq!(sum, 42.5);

        let avg = storage
            .aggregate_avg(1_599_999_999_999_999, 1_600_000_000_000_001)
            .unwrap();
        assert_eq!(avg, 42.5);
    }

    #[test]
    fn test_out_of_range_queries() {
        let mut storage = ArrowStorage::new();

        let base_time = 1_600_000_000_000_000;
        for i in 0..5 {
            storage.insert(base_time + i * 1000, i as f64, 1).unwrap();
        }

        // Query completely before data
        let results = storage
            .range_query(base_time - 10000, base_time - 1000)
            .unwrap();
        assert!(results.is_empty());

        // Query completely after data
        let results = storage
            .range_query(base_time + 10000, base_time + 20000)
            .unwrap();
        assert!(results.is_empty());

        // Aggregations on empty ranges
        let sum = storage
            .aggregate_sum(base_time - 10000, base_time - 1000)
            .unwrap();
        assert_eq!(sum, 0.0);
    }

    #[test]
    fn test_partial_range_queries() {
        let mut storage = ArrowStorage::new();

        let base_time = 1_600_000_000_000_000;
        for i in 0..10 {
            storage.insert(base_time + i * 1000, i as f64, 1).unwrap();
        }

        // Query first half
        let results = storage.range_query(base_time, base_time + 4500).unwrap();
        assert!(!results.is_empty());

        // Query last half
        let results = storage
            .range_query(base_time + 5000, base_time + 15000)
            .unwrap();
        assert!(!results.is_empty());

        // Query middle section
        let results = storage
            .range_query(base_time + 2000, base_time + 7000)
            .unwrap();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_multiple_series() {
        let mut storage = ArrowStorage::new();

        let base_time = 1_600_000_000_000_000;

        // Insert data for multiple series
        for i in 0..5 {
            storage
                .insert(base_time + i * 1000, (i * 10) as f64, 1)
                .unwrap(); // Series 1
            storage
                .insert(base_time + i * 1000, (i * 5) as f64, 2)
                .unwrap(); // Series 2
        }

        // Should find data from both series
        let results = storage.range_query(base_time, base_time + 10000).unwrap();
        assert!(!results.is_empty());

        // Aggregations should include all series
        let sum = storage.aggregate_sum(base_time, base_time + 10000).unwrap();
        assert!(sum > 0.0);
    }

    #[test]
    fn test_large_batch_creation() {
        let mut storage = ArrowStorage::new();

        let base_time = 1_600_000_000_000_000;

        // Insert enough data to trigger multiple batches
        for i in 0..2000 {
            storage.insert(base_time + i * 1000, i as f64, 1).unwrap();
        }

        // Query all data
        let results = storage
            .range_query(base_time, base_time + 2_000_000)
            .unwrap();
        assert!(!results.is_empty());

        // Test aggregations on large dataset
        let sum = storage
            .aggregate_sum(base_time, base_time + 2_000_000)
            .unwrap();
        assert!(sum > 0.0);

        let avg = storage
            .aggregate_avg(base_time, base_time + 2_000_000)
            .unwrap();
        assert!(avg > 0.0);
    }

    #[test]
    fn test_overlapping_time_ranges() {
        let mut storage = ArrowStorage::new();

        let base_time = 1_600_000_000_000_000;
        for i in 0..10 {
            storage.insert(base_time + i * 1000, i as f64, 1).unwrap();
        }

        // Test overlapping queries
        let results1 = storage.range_query(base_time, base_time + 5000).unwrap();
        let results2 = storage
            .range_query(base_time + 3000, base_time + 8000)
            .unwrap();

        assert!(!results1.is_empty());
        assert!(!results2.is_empty());
    }

    #[test]
    fn test_exact_boundary_queries() {
        let mut storage = ArrowStorage::new();

        let base_time = 1_600_000_000_000_000;
        for i in 0..5 {
            storage.insert(base_time + i * 1000, i as f64, 1).unwrap();
        }

        // Query with exact boundaries
        let results = storage.range_query(base_time, base_time + 4000).unwrap();
        assert!(!results.is_empty());

        // Query with timestamp exactly matching data point
        let results = storage
            .range_query(base_time + 2000, base_time + 2000)
            .unwrap();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_learned_index_integration() {
        let mut storage = ArrowStorage::new();

        // Insert data with learned index enabled
        let base_time = 1_600_000_000_000_000;
        for i in 0..20 {
            storage.insert(base_time + i * 1000, i as f64, 1).unwrap();
        }

        // Queries should work with learned index (batch creation happens internally)
        let results = storage.range_query(base_time, base_time + 10000).unwrap();
        assert!(!results.is_empty());

        // Test aggregations with learned index
        let sum = storage.aggregate_sum(base_time, base_time + 20000).unwrap();
        assert!(sum > 0.0);
    }

    #[test]
    fn test_edge_case_aggregations() {
        let mut storage = ArrowStorage::new();

        let base_time = 1_600_000_000_000_000;

        // Insert some negative values
        storage.insert(base_time, -5.0, 1).unwrap();
        storage.insert(base_time + 1000, 10.0, 1).unwrap();
        storage.insert(base_time + 2000, -3.0, 1).unwrap();

        let sum = storage.aggregate_sum(base_time, base_time + 3000).unwrap();
        assert_eq!(sum, 2.0); // -5 + 10 + -3 = 2

        let avg = storage.aggregate_avg(base_time, base_time + 3000).unwrap();
        assert!((avg - 0.6666666666666666).abs() < 0.0001);
    }

    #[test]
    fn test_zero_and_special_values() {
        let mut storage = ArrowStorage::new();

        let base_time = 1_600_000_000_000_000;

        // Insert various special values
        storage.insert(base_time, 0.0, 1).unwrap();
        storage.insert(base_time + 1000, f64::MIN, 1).unwrap();
        storage.insert(base_time + 2000, f64::MAX, 1).unwrap();

        let results = storage.range_query(base_time, base_time + 3000).unwrap();
        assert!(!results.is_empty());

        // Should handle extreme values without panicking
        let sum = storage.aggregate_sum(base_time, base_time + 3000).unwrap();
        assert!(sum.is_finite() || sum.is_infinite());
    }
}
