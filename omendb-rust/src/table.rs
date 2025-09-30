//! Table abstraction - combines schema, storage, and learned index
//! Each table has its own schema and primary key for indexing

use crate::row::Row;
use crate::value::Value;
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

    /// In-memory rows (will be replaced with proper storage)
    rows: Vec<Row>,

    /// Learned index keys (will be replaced with TableIndex)
    index_keys: Vec<i64>,
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

        let table = Self {
            name,
            schema,
            primary_key,
            primary_key_index,
            table_dir,
            rows: Vec::new(),
            index_keys: Vec::new(),
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

        Ok(Self {
            name,
            schema,
            primary_key: metadata.primary_key,
            primary_key_index,
            table_dir,
            rows: Vec::new(),
            index_keys: Vec::new(),
        })
    }

    /// Insert row into table
    pub fn insert(&mut self, row: Row) -> Result<()> {
        // Validate row matches schema
        row.validate(&self.schema)?;

        // Extract primary key value
        let pk_value = row.get(self.primary_key_index)?;
        let pk_i64 = pk_value.to_i64()?;

        // Add to index
        self.index_keys.push(pk_i64);

        // Store row
        self.rows.push(row);

        Ok(())
    }

    /// Get row by primary key value
    pub fn get(&self, key_value: &Value) -> Result<Option<&Row>> {
        let search_key = key_value.to_i64()?;

        // Linear search for now (will use learned index later)
        for (i, &index_key) in self.index_keys.iter().enumerate() {
            if index_key == search_key {
                return Ok(Some(&self.rows[i]));
            }
        }

        Ok(None)
    }

    /// Range query by primary key
    pub fn range_query(&self, start: &Value, end: &Value) -> Result<Vec<&Row>> {
        let start_key = start.to_i64()?;
        let end_key = end.to_i64()?;

        let mut results = Vec::new();
        for (i, &index_key) in self.index_keys.iter().enumerate() {
            if index_key >= start_key && index_key <= end_key {
                results.push(&self.rows[i]);
            }
        }

        Ok(results)
    }

    /// Get all rows as RecordBatch
    pub fn scan(&self) -> Result<RecordBatch> {
        Row::rows_to_batch(&self.rows, &self.schema)
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
        self.rows.len()
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
}