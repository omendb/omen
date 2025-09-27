//! Arrow-based columnar storage for OmenDB
//! Week 3: Integration with learned indexes for time-series data

use arrow::array::{ArrayRef, Int64Array, Float64Array, TimestampMicrosecondArray};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;
use parquet::arrow::arrow_writer::ArrowWriter;
use parquet::arrow::ArrowReader;
use parquet::arrow::ParquetFileArrowReader;
use parquet::file::properties::WriterProperties;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use anyhow::Result;

/// Time-series optimized columnar storage
pub struct ArrowStorage {
    /// Schema for time-series data
    schema: SchemaRef,

    /// In-memory batches (hot data)
    hot_batches: Vec<RecordBatch>,

    /// File paths for cold data
    cold_files: Vec<String>,

    /// Learned index for timestamp column
    timestamp_index: Option<Box<dyn LearnedIndexTrait>>,
}

/// Trait for learned indexes to integrate with storage
pub trait LearnedIndexTrait: Send + Sync {
    fn train(&mut self, keys: &[i64]);
    fn search(&self, key: i64) -> Option<usize>;
    fn range_search(&self, start: i64, end: i64) -> Vec<usize>;
}

impl ArrowStorage {
    /// Create new storage for time-series data
    pub fn new() -> Self {
        // Standard time-series schema
        let schema = Schema::new(vec![
            Field::new("timestamp", DataType::Timestamp(arrow::datatypes::TimeUnit::Microsecond, None), false),
            Field::new("value", DataType::Float64, false),
            Field::new("series_id", DataType::Int64, false),
            Field::new("tags", DataType::Utf8, true),
        ]);

        Self {
            schema: Arc::new(schema),
            hot_batches: Vec::new(),
            cold_files: Vec::new(),
            timestamp_index: None,
        }
    }

    /// Insert time-series data (integrated with learned index)
    pub fn insert(&mut self, timestamp: i64, value: f64, series_id: i64) -> Result<()> {
        // Create arrays for the batch
        let timestamps = TimestampMicrosecondArray::from(vec![timestamp]);
        let values = Float64Array::from(vec![value]);
        let series_ids = Int64Array::from(vec![series_id]);
        let tags = arrow::array::StringArray::from(vec![Some("")]);

        // Create record batch
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

        // Retrain index periodically (every 1000 inserts)
        if self.hot_batches.len() % 1000 == 0 {
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
                if let Some(timestamps) = batch.column(0).as_any().downcast_ref::<TimestampMicrosecondArray>() {
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
                    if !values.is_null(i) {
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
                    if !values.is_null(i) {
                        sum += values.value(i);
                        count += 1;
                    }
                }
            }
        }

        Ok(if count > 0 { sum / count as f64 } else { 0.0 })
    }

    /// Flush hot data to Parquet files
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

        // Move to cold storage
        self.cold_files.push(path.to_string_lossy().to_string());
        self.hot_batches.clear();

        Ok(())
    }

    /// Load data from Parquet files
    pub fn load_from_parquet(&mut self, path: &Path) -> Result<()> {
        let file = File::open(path)?;
        let reader = ParquetFileArrowReader::new(Arc::new(file));
        let mut arrow_reader = reader.get_record_reader(2048)?;

        while let Some(batch) = arrow_reader.next() {
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
            if let Some(ts_array) = batch.column(0).as_any().downcast_ref::<TimestampMicrosecondArray>() {
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

        let sum = storage.aggregate_sum(base_time, base_time + 100_000).unwrap();
        assert_eq!(sum, 100.0);

        let avg = storage.aggregate_avg(base_time, base_time + 100_000).unwrap();
        assert_eq!(avg, 10.0);
    }
}