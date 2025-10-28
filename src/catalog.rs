//! Database catalog - manages multiple tables
//! Provides table creation, lookup, and metadata persistence

use crate::table::Table;
use crate::table_wal::TableWalManager;
use anyhow::{anyhow, Result};
use arrow::datatypes::SchemaRef;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Table metadata for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TableMetadata {
    name: String,
    primary_key: String,
}

/// Catalog manages all tables in the database
pub struct Catalog {
    /// All tables by name
    tables: HashMap<String, Table>,

    /// Directory for storing table data
    data_dir: PathBuf,

    /// Metadata file path
    metadata_file: PathBuf,

    /// Write-ahead log for durability (optional)
    wal: Option<TableWalManager>,
}

impl Catalog {
    /// Create new catalog with given data directory
    pub fn new(data_dir: PathBuf) -> Result<Self> {
        Self::new_with_wal(data_dir, true)
    }

    /// Create new catalog with optional WAL
    pub fn new_with_wal(data_dir: PathBuf, enable_wal: bool) -> Result<Self> {
        // Ensure data directory exists
        fs::create_dir_all(&data_dir)?;

        let metadata_file = data_dir.join("catalog.json");

        // Initialize WAL if enabled
        let wal = if enable_wal {
            let wal_dir = data_dir.join("wal");
            Some(TableWalManager::new(&wal_dir)?)
        } else {
            None
        };

        let mut catalog = Self {
            tables: HashMap::new(),
            data_dir,
            metadata_file,
            wal,
        };

        // Load existing metadata if present
        if catalog.metadata_file.exists() {
            catalog.load_metadata()?;
        }

        Ok(catalog)
    }

    /// Create a new table
    pub fn create_table(
        &mut self,
        name: String,
        schema: SchemaRef,
        primary_key: String,
    ) -> Result<()> {
        // Check if table already exists
        if self.tables.contains_key(&name) {
            return Err(anyhow!("Table '{}' already exists", name));
        }

        // Validate primary key exists in schema
        if schema.index_of(&primary_key).is_err() {
            return Err(anyhow!(
                "Primary key column '{}' not found in schema",
                primary_key
            ));
        }

        // Validate primary key is orderable type
        let pk_field = schema.field_with_name(&primary_key)?;
        if !crate::value::is_orderable_type(pk_field.data_type()) {
            return Err(anyhow!(
                "Primary key column '{}' has non-orderable type {:?}",
                primary_key,
                pk_field.data_type()
            ));
        }

        // Log to WAL before making changes
        if let Some(wal) = &self.wal {
            wal.log_create_table(name.clone(), schema.clone(), primary_key.clone())?;
        }

        // Create table directory
        let table_dir = self.data_dir.join(&name);
        fs::create_dir_all(&table_dir)?;

        // Create table
        let table = Table::new(name.clone(), schema, primary_key, table_dir)?;
        self.tables.insert(name, table);

        // Persist metadata
        self.save_metadata()?;

        Ok(())
    }

    /// Get reference to table
    pub fn get_table(&self, name: &str) -> Result<&Table> {
        self.tables
            .get(name)
            .ok_or_else(|| anyhow!("Table '{}' not found", name))
    }

    /// Get mutable reference to table
    pub fn get_table_mut(&mut self, name: &str) -> Result<&mut Table> {
        self.tables
            .get_mut(name)
            .ok_or_else(|| anyhow!("Table '{}' not found", name))
    }

    /// Drop table
    pub fn drop_table(&mut self, name: &str) -> Result<()> {
        // Check table exists
        if !self.tables.contains_key(name) {
            return Err(anyhow!("Table '{}' not found", name));
        }

        // Log to WAL before making changes
        if let Some(wal) = &self.wal {
            wal.log_drop_table(name.to_string())?;
        }

        // Remove from catalog
        self.tables.remove(name);

        // Remove table directory
        let table_dir = self.data_dir.join(name);
        if table_dir.exists() {
            fs::remove_dir_all(&table_dir)?;
        }

        // Persist metadata
        self.save_metadata()?;

        Ok(())
    }

    /// List all table names
    pub fn list_tables(&self) -> Vec<String> {
        self.tables.keys().cloned().collect()
    }

    /// Check if table exists
    pub fn table_exists(&self, name: &str) -> bool {
        self.tables.contains_key(name)
    }

    /// Save catalog metadata to disk
    fn save_metadata(&self) -> Result<()> {
        let metadata: Vec<TableMetadata> = self
            .tables
            .values()
            .map(|table| TableMetadata {
                name: table.name().to_string(),
                primary_key: table.primary_key().to_string(),
            })
            .collect();

        let json = serde_json::to_string_pretty(&metadata)?;
        fs::write(&self.metadata_file, json)?;

        Ok(())
    }

    /// Load catalog metadata from disk
    fn load_metadata(&mut self) -> Result<()> {
        let json = fs::read_to_string(&self.metadata_file)?;
        let metadata: Vec<TableMetadata> = serde_json::from_str(&json)?;

        // Load each table
        for meta in metadata {
            let table_dir = self.data_dir.join(&meta.name);
            let table = Table::load(meta.name.clone(), table_dir)?;
            self.tables.insert(meta.name, table);
        }

        Ok(())
    }
}

impl Drop for Catalog {
    fn drop(&mut self) {
        // Persist all tables to disk when catalog is dropped
        for table in self.tables.values_mut() {
            let _ = table.persist();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::datatypes::{DataType, Field, Schema};
    use std::sync::Arc;
    use tempfile::TempDir;

    #[test]
    fn test_catalog_create_table() {
        let temp_dir = TempDir::new().unwrap();
        let mut catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();

        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("name", DataType::Utf8, false),
        ]));

        catalog
            .create_table("users".to_string(), schema, "id".to_string())
            .unwrap();

        assert!(catalog.table_exists("users"));
        assert_eq!(catalog.list_tables(), vec!["users"]);
    }

    #[test]
    fn test_catalog_duplicate_table() {
        let temp_dir = TempDir::new().unwrap();
        let mut catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();

        let schema = Arc::new(Schema::new(vec![Field::new("id", DataType::Int64, false)]));

        catalog
            .create_table("users".to_string(), schema.clone(), "id".to_string())
            .unwrap();

        // Try to create duplicate
        let result = catalog.create_table("users".to_string(), schema, "id".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_catalog_invalid_primary_key() {
        let temp_dir = TempDir::new().unwrap();
        let mut catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();

        let schema = Arc::new(Schema::new(vec![Field::new("id", DataType::Int64, false)]));

        // Non-existent column
        let result =
            catalog.create_table("users".to_string(), schema.clone(), "missing".to_string());
        assert!(result.is_err());

        // Non-orderable type
        let schema2 = Arc::new(Schema::new(vec![Field::new("name", DataType::Utf8, false)]));
        let result = catalog.create_table("users".to_string(), schema2, "name".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_catalog_drop_table() {
        let temp_dir = TempDir::new().unwrap();
        let mut catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();

        let schema = Arc::new(Schema::new(vec![Field::new("id", DataType::Int64, false)]));

        catalog
            .create_table("users".to_string(), schema, "id".to_string())
            .unwrap();
        assert!(catalog.table_exists("users"));

        catalog.drop_table("users").unwrap();
        assert!(!catalog.table_exists("users"));
    }
}
