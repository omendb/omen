//! Table abstraction - combines schema, storage, and learned index
//! Each table has its own schema and primary key for indexing

use crate::row::Row;
use crate::value::Value;
use crate::table_storage::TableStorage;
use crate::table_index::TableIndex;
use anyhow::{Result, anyhow};
use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use serde::{Serialize, Deserialize};
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

    /// Table schema
    schema: SchemaRef,

    /// Primary key column name (must be orderable type)
    primary_key: String,

    /// Primary key column index
    primary_key_index: usize,

    /// Table directory
    table_dir: PathBuf,

    /// Columnar storage with Arrow/Parquet
    storage: TableStorage,

    /// Learned index over primary key
    index: TableIndex,
}

impl Table {
    /// Create new table
    pub fn new(
        name: String,
        schema: SchemaRef,
        primary_key: String,
        table_dir: PathBuf,
    ) -> Result<Self> {
        // Find primary key index
        let primary_key_index = schema.index_of(&primary_key)?;

        // Validate primary key is orderable
        let pk_field = schema.field(primary_key_index);
        if !crate::value::is_orderable_type(pk_field.data_type()) {
            return Err(anyhow!("Primary key '{}' has non-orderable type", primary_key));
        }

        // Create table directory
        fs::create_dir_all(&table_dir)?;

        // Create storage and index
        let storage = TableStorage::new(schema.clone(), table_dir.clone(), 10000)?;
        let index = TableIndex::new(10000);

        let table = Self {
            name,
            schema,
            primary_key,
            primary_key_index,
            table_dir,
            storage,
            index,
        };

        // Save metadata
        table.save_metadata()?;

        Ok(table)
    }

    /// Load existing table from directory
    pub fn load(name: String, table_dir: PathBuf) -> Result<Self> {
        let metadata_file = table_dir.join("metadata.json");
        let json = fs::read_to_string(&metadata_file)?;
        let metadata: TableMetadata = serde_json::from_str(&json)?;

        // Reconstruct schema
        let fields: Result<Vec<_>> = metadata.schema.iter()
            .map(|f| {
                let data_type = Self::parse_data_type(&f.data_type)?;
                Ok(arrow::datatypes::Field::new(&f.name, data_type, f.nullable))
            })
            .collect();

        let schema = Arc::new(arrow::datatypes::Schema::new(fields?));
        let primary_key_index = schema.index_of(&metadata.primary_key)?;

        // Load storage
        let storage = TableStorage::load(schema.clone(), table_dir.clone())?;

        // Rebuild index from storage
        let mut index = TableIndex::new(storage.row_count());
        let all_rows = storage.scan_all()?;
        for (position, row) in all_rows.iter().enumerate() {
            let pk_value = row.get(primary_key_index)?;
            index.insert(pk_value, position)?;
        }

        Ok(Self {
            name,
            schema,
            primary_key: metadata.primary_key,
            primary_key_index,
            table_dir,
            storage,
            index,
        })
    }

    /// Insert row into table
    pub fn insert(&mut self, row: Row) -> Result<()> {
        // Validate row matches schema
        row.validate(&self.schema)?;

        // Extract primary key value
        let pk_value = row.get(self.primary_key_index)?;

        // Get position where row will be stored
        let position = self.storage.row_count();

        // Add to index
        self.index.insert(pk_value, position)?;

        // Store row
        self.storage.insert(row)?;

        Ok(())
    }

    /// Get row by primary key value
    pub fn get(&self, key_value: &Value) -> Result<Option<Row>> {
        // Use index to find position
        if let Some(position) = self.index.search(key_value)? {
            let row = self.storage.get(position)?;
            return Ok(Some(row));
        }

        Ok(None)
    }

    /// Range query by primary key
    pub fn range_query(&self, start: &Value, end: &Value) -> Result<Vec<Row>> {
        // Use index to find positions
        let positions = self.index.range_query(start, end)?;

        // Retrieve rows
        self.storage.get_many(&positions)
    }

    /// Get all rows as RecordBatch
    pub fn scan(&mut self) -> Result<Vec<RecordBatch>> {
        self.storage.scan_batches()
    }

    /// Get all rows as Row objects
    pub fn scan_all(&self) -> Result<Vec<Row>> {
        self.storage.scan_all()
    }

    /// Table name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Table schema
    pub fn schema(&self) -> &SchemaRef {
        &self.schema
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

    /// Save table metadata
    fn save_metadata(&self) -> Result<()> {
        let schema_fields: Vec<SchemaField> = self.schema.fields().iter()
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
            _ if s.starts_with("Timestamp") => {
                Ok(DataType::Timestamp(TimeUnit::Microsecond, None))
            }
            _ => Err(anyhow!("Unsupported data type: {}", s)),
        }
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

        let table = Table::new(
            "users".to_string(),
            schema,
            "id".to_string(),
            table_dir,
        ).unwrap();

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
        ).unwrap();

        let row = Row::new(vec![
            Value::Int64(1),
            Value::Text("Alice".to_string()),
        ]);

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
        ).unwrap();

        let row = Row::new(vec![
            Value::Int64(1),
            Value::Text("Alice".to_string()),
        ]);

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
        ).unwrap();

        // Insert test data
        for i in 0..10 {
            let row = Row::new(vec![
                Value::Int64(i),
                Value::Float64(i as f64 * 1.5),
            ]);
            table.insert(row).unwrap();
        }

        // Range query
        let results = table.range_query(
            &Value::Int64(3),
            &Value::Int64(7),
        ).unwrap();

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
            ).unwrap();

            for i in 0..3 {
                let row = Row::new(vec![
                    Value::Int64(i),
                    Value::Text(format!("user_{}", i)),
                ]);
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