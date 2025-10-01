//! Schema-agnostic table storage using Apache Arrow
//! Stores rows in columnar format for efficient queries

use crate::row::Row;
use anyhow::{Result, anyhow};
use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use std::sync::Arc;

/// Schema-agnostic storage for table rows
#[derive(Debug)]
pub struct TableStorage {
    /// Table schema
    schema: SchemaRef,

    /// In-memory batches (will be flushed to disk)
    batches: Vec<RecordBatch>,

    /// Path to Parquet file for persistence
    parquet_file: PathBuf,

    /// Maximum rows per batch before flush
    batch_size: usize,

    /// Current batch being built
    pending_rows: Vec<Row>,
}

impl TableStorage {
    /// Create new table storage
    pub fn new(schema: SchemaRef, data_dir: PathBuf, batch_size: usize) -> Result<Self> {
        let parquet_file = data_dir.join("data.parquet");

        Ok(Self {
            schema,
            batches: Vec::new(),
            parquet_file,
            batch_size,
            pending_rows: Vec::new(),
        })
    }

    /// Load existing storage from Parquet file
    pub fn load(schema: SchemaRef, data_dir: PathBuf) -> Result<Self> {
        let parquet_file = data_dir.join("data.parquet");

        let mut storage = Self {
            schema: schema.clone(),
            batches: Vec::new(),
            parquet_file: parquet_file.clone(),
            batch_size: 10000,
            pending_rows: Vec::new(),
        };

        // Load existing data if file exists
        if parquet_file.exists() {
            let file = File::open(&parquet_file)?;
            let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
            let reader = builder.build()?;

            for batch_result in reader {
                let batch = batch_result?;
                storage.batches.push(batch);
            }
        }

        Ok(storage)
    }

    /// Insert row into storage
    pub fn insert(&mut self, row: Row) -> Result<()> {
        // Validate row
        row.validate(&self.schema)?;

        // Add to pending rows
        self.pending_rows.push(row);

        // Flush if batch is full
        if self.pending_rows.len() >= self.batch_size {
            self.flush()?;
        }

        Ok(())
    }

    /// Insert multiple rows
    pub fn insert_batch(&mut self, rows: Vec<Row>) -> Result<()> {
        for row in rows {
            self.insert(row)?;
        }
        Ok(())
    }

    /// Get row at position
    pub fn get(&self, position: usize) -> Result<Row> {
        // Calculate which batch and offset
        let mut remaining = position;

        // Check batches
        for batch in &self.batches {
            let batch_size = batch.num_rows();
            if remaining < batch_size {
                return Row::from_batch(batch, remaining);
            }
            remaining -= batch_size;
        }

        // Check pending rows
        if remaining < self.pending_rows.len() {
            return Ok(self.pending_rows[remaining].clone());
        }

        Err(anyhow!("Position {} out of bounds", position))
    }

    /// Get multiple rows by positions
    pub fn get_many(&self, positions: &[usize]) -> Result<Vec<Row>> {
        positions.iter()
            .map(|&pos| self.get(pos))
            .collect()
    }

    /// Get all rows
    pub fn scan_all(&self) -> Result<Vec<Row>> {
        let mut rows = Vec::new();

        // Extract from batches
        for batch in &self.batches {
            for i in 0..batch.num_rows() {
                rows.push(Row::from_batch(batch, i)?);
            }
        }

        // Add pending rows
        rows.extend(self.pending_rows.clone());

        Ok(rows)
    }

    /// Get all data as RecordBatches
    pub fn scan_batches(&mut self) -> Result<Vec<RecordBatch>> {
        // Flush pending rows first
        if !self.pending_rows.is_empty() {
            self.flush()?;
        }

        Ok(self.batches.clone())
    }

    /// Total number of rows
    pub fn row_count(&self) -> usize {
        let batch_rows: usize = self.batches.iter()
            .map(|b| b.num_rows())
            .sum();
        batch_rows + self.pending_rows.len()
    }

    /// Flush pending rows to in-memory batch
    pub fn flush(&mut self) -> Result<()> {
        if self.pending_rows.is_empty() {
            return Ok(());
        }

        // Convert to RecordBatch
        let batch = Row::rows_to_batch(&self.pending_rows, &self.schema)?;
        self.batches.push(batch);
        self.pending_rows.clear();

        Ok(())
    }

    /// Write all data to Parquet file
    pub fn persist(&mut self) -> Result<()> {
        // Flush any pending rows
        self.flush()?;

        if self.batches.is_empty() {
            return Ok(());
        }

        // Write to Parquet
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.parquet_file)?;

        let props = WriterProperties::builder().build();
        let mut writer = ArrowWriter::try_new(file, self.schema.clone(), Some(props))?;

        for batch in &self.batches {
            writer.write(batch)?;
        }

        writer.close()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;
    use arrow::datatypes::{DataType, Field, Schema};
    use tempfile::TempDir;

    #[test]
    fn test_storage_insert_and_get() {
        let temp_dir = TempDir::new().unwrap();
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("value", DataType::Float64, false),
        ]));

        let mut storage = TableStorage::new(schema, temp_dir.path().to_path_buf(), 10).unwrap();

        // Insert row
        let row = Row::new(vec![Value::Int64(1), Value::Float64(1.5)]);
        storage.insert(row).unwrap();

        assert_eq!(storage.row_count(), 1);

        // Get row
        let retrieved = storage.get(0).unwrap();
        assert_eq!(retrieved.get(0).unwrap(), &Value::Int64(1));
    }

    #[test]
    fn test_storage_batch_flush() {
        let temp_dir = TempDir::new().unwrap();
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
        ]));

        let mut storage = TableStorage::new(schema, temp_dir.path().to_path_buf(), 5).unwrap();

        // Insert 10 rows (should trigger flush at 5)
        for i in 0..10 {
            let row = Row::new(vec![Value::Int64(i)]);
            storage.insert(row).unwrap();
        }

        assert_eq!(storage.row_count(), 10);
        assert!(storage.batches.len() >= 1); // At least one batch flushed
    }

    #[test]
    fn test_storage_scan_all() {
        let temp_dir = TempDir::new().unwrap();
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
        ]));

        let mut storage = TableStorage::new(schema, temp_dir.path().to_path_buf(), 10).unwrap();

        // Insert rows
        for i in 0..5 {
            let row = Row::new(vec![Value::Int64(i)]);
            storage.insert(row).unwrap();
        }

        // Scan all
        let rows = storage.scan_all().unwrap();
        assert_eq!(rows.len(), 5);
    }

    #[test]
    fn test_storage_persist_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("name", DataType::Utf8, false),
        ]));

        // Create storage and insert data
        {
            let mut storage = TableStorage::new(
                schema.clone(),
                temp_dir.path().to_path_buf(),
                10,
            ).unwrap();

            for i in 0..3 {
                let row = Row::new(vec![
                    Value::Int64(i),
                    Value::Text(format!("name_{}", i)),
                ]);
                storage.insert(row).unwrap();
            }

            storage.persist().unwrap();
        }

        // Load storage
        let storage = TableStorage::load(schema, temp_dir.path().to_path_buf()).unwrap();
        assert_eq!(storage.row_count(), 3);
    }
}