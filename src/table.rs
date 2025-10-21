//! Table abstraction - combines schema, storage, and learned index
//! Each table has its own schema and primary key for indexing
//! Supports MVCC for UPDATE/DELETE operations

use crate::mvcc::{self, MvccIndices};
use crate::row::Row;
use crate::table_index::TableIndex;
use crate::table_storage::TableStorage;
use crate::value::Value;
use anyhow::{anyhow, Result};
use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

/// Table metadata for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TableMetadata {
    name: String,
    primary_key: String,
    schema: Vec<SchemaField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SchemaField {
    name: String,
    data_type: String,
    nullable: bool,
}

/// A table with schema, storage, and learned index
#[derive(Debug)]
pub struct Table {
    /// Table name
    name: String,

    /// User-facing schema (without MVCC columns)
    user_schema: SchemaRef,

    /// Internal schema (with MVCC columns for versioning)
    internal_schema: SchemaRef,

    /// Primary key column name (must be orderable type)
    primary_key: String,

    /// Primary key column index in user schema
    primary_key_index: usize,

    /// MVCC column indices in internal schema
    mvcc_indices: MvccIndices,

    /// Table directory
    table_dir: PathBuf,

    /// Columnar storage with Arrow/Parquet
    storage: TableStorage,

    /// Learned index over primary key
    index: TableIndex,

    /// Version counter for MVCC
    next_version: u64,

    /// Current transaction ID (0 for non-transactional operations)
    current_txn_id: u64,
}

impl Table {
    /// Create new table with MVCC support
    pub fn new(
        name: String,
        user_schema: SchemaRef,
        primary_key: String,
        table_dir: PathBuf,
    ) -> Result<Self> {
        // Find primary key index in user schema
        let primary_key_index = user_schema.index_of(&primary_key)?;

        // Validate primary key is orderable
        let pk_field = user_schema.field(primary_key_index);
        if !crate::value::is_orderable_type(pk_field.data_type()) {
            return Err(anyhow!(
                "Primary key '{}' has non-orderable type",
                primary_key
            ));
        }

        // Create internal schema with MVCC columns
        let internal_schema = mvcc::add_mvcc_columns(user_schema.clone());

        // Get MVCC column indices
        let mvcc_indices = MvccIndices::from_schema(&internal_schema)
            .ok_or_else(|| anyhow!("Failed to add MVCC columns to schema"))?;

        // Create table directory
        fs::create_dir_all(&table_dir)?;

        // Create storage with internal schema (includes MVCC columns)
        let storage = TableStorage::new(internal_schema.clone(), table_dir.clone(), 10000)?;
        let index = TableIndex::new(10000);

        let table = Self {
            name,
            user_schema: user_schema.clone(),
            internal_schema,
            primary_key,
            primary_key_index,
            mvcc_indices,
            table_dir,
            storage,
            index,
            next_version: 0,
            current_txn_id: 0,
        };

        // Save metadata (uses user schema, not internal)
        table.save_metadata()?;

        Ok(table)
    }

    /// Load existing table from directory
    pub fn load(name: String, table_dir: PathBuf) -> Result<Self> {
        let metadata_file = table_dir.join("metadata.json");
        let json = fs::read_to_string(&metadata_file)?;
        let metadata: TableMetadata = serde_json::from_str(&json)?;

        // Reconstruct user schema
        let fields: Result<Vec<_>> = metadata
            .schema
            .iter()
            .map(|f| {
                let data_type = Self::parse_data_type(&f.data_type)?;
                Ok(arrow::datatypes::Field::new(&f.name, data_type, f.nullable))
            })
            .collect();

        let user_schema = Arc::new(arrow::datatypes::Schema::new(fields?));
        let primary_key_index = user_schema.index_of(&metadata.primary_key)?;

        // Create internal schema with MVCC columns
        let internal_schema = mvcc::add_mvcc_columns(user_schema.clone());

        // Get MVCC column indices
        let mvcc_indices = MvccIndices::from_schema(&internal_schema)
            .ok_or_else(|| anyhow!("Failed to add MVCC columns to schema"))?;

        // Load storage (with internal schema)
        let storage = TableStorage::load(internal_schema.clone(), table_dir.clone())?;

        // Find max version from existing data
        let all_rows = storage.scan_all()?;
        let mut max_version = 0u64;
        for row in &all_rows {
            if let Ok(Value::UInt64(version)) = row.get(mvcc_indices.version) {
                max_version = max_version.max(*version);
            }
        }

        // Rebuild index from storage (only non-deleted rows with latest versions)
        let mut index = TableIndex::new(storage.row_count());
        for (position, row) in all_rows.iter().enumerate() {
            // Skip deleted rows
            if let Ok(Value::Boolean(true)) = row.get(mvcc_indices.deleted) {
                continue;
            }

            let pk_value = row.get(primary_key_index)?;
            index.insert(pk_value, position)?;
        }

        Ok(Self {
            name,
            user_schema,
            internal_schema,
            primary_key: metadata.primary_key,
            primary_key_index,
            mvcc_indices,
            table_dir,
            storage,
            index,
            next_version: max_version + 1,
            current_txn_id: 0,
        })
    }

    /// Insert row into table with MVCC metadata
    pub fn insert(&mut self, row: Row) -> Result<()> {
        // Validate row matches user schema
        row.validate(&self.user_schema)?;

        // Extract primary key value
        let pk_value = row.get(self.primary_key_index)?;

        // Add MVCC metadata to row
        let version = self.next_version;
        self.next_version += 1;

        let mut internal_values = row.values().to_vec();
        internal_values.push(Value::UInt64(version)); // __mvcc_version
        internal_values.push(Value::UInt64(self.current_txn_id)); // __mvcc_txn_id
        internal_values.push(Value::Boolean(false)); // __mvcc_deleted

        let internal_row = Row::new(internal_values);

        // Get position where row will be stored
        let position = self.storage.row_count();

        // Add to index
        self.index.insert(pk_value, position)?;

        // Store row with MVCC metadata
        self.storage.insert(internal_row)?;

        Ok(())
    }

    /// Batch insert multiple rows (optimized for learned index)
    ///
    /// Sorts rows by primary key before inserting to maximize ALEX performance.
    /// For random data, this can be 10-100x faster than individual inserts.
    ///
    /// **Use this for bulk loads with random/unordered data.**
    pub fn batch_insert(&mut self, mut rows: Vec<Row>) -> Result<usize> {
        if rows.is_empty() {
            return Ok(0);
        }

        // Validate all rows first
        for row in &rows {
            row.validate(&self.user_schema)?;
        }

        // Sort rows by primary key for optimal ALEX insertion
        // This converts random inserts into sequential inserts
        rows.sort_by(|a, b| {
            let a_pk = a.get(self.primary_key_index).unwrap();
            let b_pk = b.get(self.primary_key_index).unwrap();
            a_pk.partial_cmp(b_pk).unwrap_or(std::cmp::Ordering::Equal)
        });

        let count = rows.len();

        // Insert sorted rows (ALEX will handle this efficiently)
        for row in rows {
            self.insert(row)?;
        }

        Ok(count)
    }

    /// Update row by primary key
    /// Creates new version with updated values, marks old version as deleted
    pub fn update(&mut self, key_value: &Value, updated_row: Row) -> Result<usize> {
        // Validate updated row matches user schema
        updated_row.validate(&self.user_schema)?;

        // Find existing row
        let position = self
            .index
            .search(key_value)?
            .ok_or_else(|| anyhow!("Row with key {:?} not found", key_value))?;

        let existing_internal_row = self.storage.get(position)?;

        // Check if already deleted
        if self.is_deleted(&existing_internal_row) {
            return Ok(0); // Row already deleted, nothing to update
        }

        // Mark old version as deleted (in-place update not supported in append-only storage)
        // Instead, insert new version with updated values
        let version = self.next_version;
        self.next_version += 1;

        let mut internal_values = updated_row.values().to_vec();
        internal_values.push(Value::UInt64(version)); // __mvcc_version
        internal_values.push(Value::UInt64(self.current_txn_id)); // __mvcc_txn_id
        internal_values.push(Value::Boolean(false)); // __mvcc_deleted

        let new_internal_row = Row::new(internal_values);

        // Get new position
        let new_position = self.storage.row_count();

        // Update index to point to new version
        // (Overwrites old position with new position)
        self.index.insert(key_value, new_position)?;

        // Store new version
        self.storage.insert(new_internal_row)?;

        Ok(1) // 1 row updated
    }

    /// Delete row by primary key
    /// Creates new version marked as deleted
    pub fn delete(&mut self, key_value: &Value) -> Result<usize> {
        // Find existing row
        let position = self
            .index
            .search(key_value)?
            .ok_or_else(|| anyhow!("Row with key {:?} not found", key_value))?;

        let existing_internal_row = self.storage.get(position)?;

        // Check if already deleted
        if self.is_deleted(&existing_internal_row) {
            return Ok(0); // Already deleted
        }

        // Create new version marked as deleted
        let version = self.next_version;
        self.next_version += 1;

        // Copy all values from existing row
        let mut internal_values =
            existing_internal_row.values()[..self.user_schema.fields().len()].to_vec();

        // Add MVCC metadata with deleted=true
        internal_values.push(Value::UInt64(version)); // __mvcc_version
        internal_values.push(Value::UInt64(self.current_txn_id)); // __mvcc_txn_id
        internal_values.push(Value::Boolean(true)); // __mvcc_deleted = true

        let deleted_internal_row = Row::new(internal_values);

        // Get new position for deleted row
        let new_position = self.storage.row_count();

        // Store deleted version
        self.storage.insert(deleted_internal_row)?;

        // Update index to point to new deleted version
        // This ensures scan_all and get() see the deleted flag
        self.index.insert(key_value, new_position)?;

        Ok(1) // 1 row deleted
    }

    /// Set transaction ID for next operations
    pub fn set_transaction_id(&mut self, txn_id: u64) {
        self.current_txn_id = txn_id;
    }

    /// Strip MVCC metadata from internal row to create user-facing row
    fn strip_mvcc_columns(&self, internal_row: &Row) -> Row {
        let user_values: Vec<Value> = internal_row
            .values()
            .iter()
            .take(self.user_schema.fields().len())
            .cloned()
            .collect();
        Row::new(user_values)
    }

    /// Check if row is deleted
    fn is_deleted(&self, internal_row: &Row) -> bool {
        matches!(
            internal_row.get(self.mvcc_indices.deleted),
            Ok(Value::Boolean(true))
        )
    }

    /// Get row by primary key value
    pub fn get(&self, key_value: &Value) -> Result<Option<Row>> {
        // Use index to find position
        if let Some(position) = self.index.search(key_value)? {
            let internal_row = self.storage.get(position)?;

            // Check if deleted
            if self.is_deleted(&internal_row) {
                return Ok(None);
            }

            // Strip MVCC columns before returning
            let user_row = self.strip_mvcc_columns(&internal_row);
            return Ok(Some(user_row));
        }

        Ok(None)
    }

    /// Range query by primary key
    pub fn range_query(&self, start: &Value, end: &Value) -> Result<Vec<Row>> {
        // Use index to find positions
        let positions = self.index.range_query(start, end)?;

        // Retrieve rows, filter deleted, strip MVCC columns
        let mut user_rows = Vec::new();
        for internal_row in self.storage.get_many(&positions)? {
            if !self.is_deleted(&internal_row) {
                user_rows.push(self.strip_mvcc_columns(&internal_row));
            }
        }

        Ok(user_rows)
    }

    /// Get all rows as RecordBatch
    pub fn scan(&mut self) -> Result<Vec<RecordBatch>> {
        self.storage.scan_batches()
    }

    /// Get all rows as Row objects (filters deleted, strips MVCC columns)
    /// Only returns current versions (rows pointed to by the index)
    pub fn scan_all(&self) -> Result<Vec<Row>> {
        let all_internal_rows = self.storage.scan_all()?;
        let mut user_rows = Vec::new();

        for (position, internal_row) in all_internal_rows.iter().enumerate() {
            // Extract primary key from this row
            let pk_value = internal_row.get(self.primary_key_index)?;

            // Check if this row is the current version (pointed to by index)
            if let Some(indexed_position) = self.index.search(pk_value)? {
                if indexed_position == position && !self.is_deleted(internal_row) {
                    // This is the current version and not deleted
                    user_rows.push(self.strip_mvcc_columns(internal_row));
                }
            }
        }

        Ok(user_rows)
    }

    /// Table name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Table schema (user-facing, without MVCC columns)
    pub fn schema(&self) -> &SchemaRef {
        &self.user_schema
    }

    /// Primary key column name
    pub fn primary_key(&self) -> &str {
        &self.primary_key
    }

    /// Number of rows
    pub fn row_count(&self) -> usize {
        self.storage.row_count()
    }

    /// Persist table to disk
    pub fn persist(&mut self) -> Result<()> {
        self.storage.persist()?;
        self.save_metadata()?;
        Ok(())
    }

    /// Save table metadata (saves user schema, not internal)
    fn save_metadata(&self) -> Result<()> {
        let schema_fields: Vec<SchemaField> = self
            .user_schema
            .fields()
            .iter()
            .map(|f| SchemaField {
                name: f.name().clone(),
                data_type: Self::format_data_type(f.data_type()),
                nullable: f.is_nullable(),
            })
            .collect();

        let metadata = TableMetadata {
            name: self.name.clone(),
            primary_key: self.primary_key.clone(),
            schema: schema_fields,
        };

        let json = serde_json::to_string_pretty(&metadata)?;
        let metadata_file = self.table_dir.join("metadata.json");
        fs::write(metadata_file, json)?;

        Ok(())
    }

    /// Format Arrow DataType to string
    fn format_data_type(data_type: &arrow::datatypes::DataType) -> String {
        use arrow::datatypes::DataType;
        match data_type {
            DataType::Int64 => "Int64".to_string(),
            DataType::Float64 => "Float64".to_string(),
            DataType::Utf8 => "Utf8".to_string(),
            DataType::Timestamp(unit, tz) => {
                format!("Timestamp({:?}, {:?})", unit, tz)
            }
            DataType::Boolean => "Boolean".to_string(),
            _ => format!("{:?}", data_type),
        }
    }

    /// Parse string to Arrow DataType
    fn parse_data_type(s: &str) -> Result<arrow::datatypes::DataType> {
        use arrow::datatypes::{DataType, TimeUnit};
        match s {
            "Int64" => Ok(DataType::Int64),
            "Float64" => Ok(DataType::Float64),
            "Utf8" => Ok(DataType::Utf8),
            "Boolean" => Ok(DataType::Boolean),
            _ if s.starts_with("Timestamp") => Ok(DataType::Timestamp(TimeUnit::Microsecond, None)),
            _ => Err(anyhow!("Unsupported data type: {}", s)),
        }
    }

    // Public accessors for DataFusion integration

    /// Get user-facing schema (without MVCC columns)
    pub fn user_schema(&self) -> &SchemaRef {
        &self.user_schema
    }

    /// Get all RecordBatches for DataFusion scan (includes MVCC columns)
    pub fn scan_batches(&mut self) -> Result<Vec<RecordBatch>> {
        self.scan()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::datatypes::{DataType, Field, Schema};
    use tempfile::TempDir;

    #[test]
    fn test_table_creation() {
        let temp_dir = TempDir::new().unwrap();
        let table_dir = temp_dir.path().join("users");

        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("name", DataType::Utf8, false),
        ]));

        let table = Table::new("users".to_string(), schema, "id".to_string(), table_dir).unwrap();

        assert_eq!(table.name(), "users");
        assert_eq!(table.primary_key(), "id");
        assert_eq!(table.row_count(), 0);
    }

    #[test]
    fn test_table_insert() {
        let temp_dir = TempDir::new().unwrap();
        let table_dir = temp_dir.path().join("users");

        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("name", DataType::Utf8, false),
        ]));

        let mut table = Table::new(
            "users".to_string(),
            schema.clone(),
            "id".to_string(),
            table_dir,
        )
        .unwrap();

        let row = Row::new(vec![Value::Int64(1), Value::Text("Alice".to_string())]);

        table.insert(row).unwrap();
        assert_eq!(table.row_count(), 1);
    }

    #[test]
    fn test_table_get() {
        let temp_dir = TempDir::new().unwrap();
        let table_dir = temp_dir.path().join("users");

        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("name", DataType::Utf8, false),
        ]));

        let mut table = Table::new(
            "users".to_string(),
            schema.clone(),
            "id".to_string(),
            table_dir,
        )
        .unwrap();

        let row = Row::new(vec![Value::Int64(1), Value::Text("Alice".to_string())]);

        table.insert(row).unwrap();

        let result = table.get(&Value::Int64(1)).unwrap();
        assert!(result.is_some());

        let result = table.get(&Value::Int64(999)).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_table_range_query() {
        let temp_dir = TempDir::new().unwrap();
        let table_dir = temp_dir.path().join("metrics");

        let schema = Arc::new(Schema::new(vec![
            Field::new("timestamp", DataType::Int64, false),
            Field::new("value", DataType::Float64, false),
        ]));

        let mut table = Table::new(
            "metrics".to_string(),
            schema.clone(),
            "timestamp".to_string(),
            table_dir,
        )
        .unwrap();

        // Insert test data
        for i in 0..10 {
            let row = Row::new(vec![Value::Int64(i), Value::Float64(i as f64 * 1.5)]);
            table.insert(row).unwrap();
        }

        // Range query
        let results = table
            .range_query(&Value::Int64(3), &Value::Int64(7))
            .unwrap();

        assert_eq!(results.len(), 5); // 3, 4, 5, 6, 7
    }

    #[test]
    fn test_table_persist_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let table_dir = temp_dir.path().join("users");

        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("name", DataType::Utf8, false),
        ]));

        // Create and populate table
        {
            let mut table = Table::new(
                "users".to_string(),
                schema.clone(),
                "id".to_string(),
                table_dir.clone(),
            )
            .unwrap();

            for i in 0..3 {
                let row = Row::new(vec![Value::Int64(i), Value::Text(format!("user_{}", i))]);
                table.insert(row).unwrap();
            }

            table.persist().unwrap();
        }

        // Load table
        let table = Table::load("users".to_string(), table_dir).unwrap();
        assert_eq!(table.row_count(), 3);

        // Verify data
        let row = table.get(&Value::Int64(1)).unwrap().unwrap();
        assert_eq!(row.get(0).unwrap(), &Value::Int64(1));
        assert_eq!(row.get(1).unwrap(), &Value::Text("user_1".to_string()));
    }
}
