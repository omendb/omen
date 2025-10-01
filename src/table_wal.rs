//! Write-Ahead Logging for multi-table database
//! Provides durability and crash recovery for the new table architecture

use crate::row::Row;
use crate::value::Value;
use anyhow::{Result, Context, anyhow};
use arrow::datatypes::SchemaRef;
use chrono::{DateTime, Utc};
use crc32fast::Hasher;
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

/// WAL operation for multi-table database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TableWalOperation {
    /// Create table
    CreateTable {
        table_name: String,
        primary_key: String,
        schema_json: String, // Serialized Arrow schema
    },
    /// Drop table
    DropTable {
        table_name: String,
    },
    /// Insert row into table
    InsertRow {
        table_name: String,
        row_data: Vec<Value>, // Row as vector of values
    },
    /// Checkpoint marker
    Checkpoint {
        sequence: u64,
        timestamp: DateTime<Utc>,
    },
}

/// WAL entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableWalEntry {
    /// Unique sequence number
    pub sequence: u64,
    /// Operation to perform
    pub operation: TableWalOperation,
    /// Timestamp of entry creation
    pub timestamp: DateTime<Utc>,
    /// CRC32 checksum for integrity
    pub checksum: u32,
}

impl TableWalEntry {
    pub fn new(sequence: u64, operation: TableWalOperation) -> Self {
        let timestamp = Utc::now();
        let mut entry = Self {
            sequence,
            operation,
            timestamp,
            checksum: 0,
        };
        entry.checksum = entry.compute_checksum();
        entry
    }

    fn compute_checksum(&self) -> u32 {
        let mut hasher = Hasher::new();
        let data = bincode::serialize(&(self.sequence, &self.operation, &self.timestamp))
            .unwrap_or_default();
        hasher.update(&data);
        hasher.finalize()
    }

    pub fn verify_checksum(&self) -> bool {
        self.checksum == self.compute_checksum()
    }
}

/// Write-Ahead Log manager for multi-table database
pub struct TableWalManager {
    /// Path to WAL directory
    wal_dir: PathBuf,
    /// Current WAL file writer
    writer: Arc<Mutex<Option<BufWriter<File>>>>,
    /// Current sequence number
    sequence: Arc<RwLock<u64>>,
    /// Buffer of pending writes
    buffer: Arc<Mutex<VecDeque<TableWalEntry>>>,
    /// Maximum buffer size before flush
    buffer_size: usize,
    /// Current WAL file name
    current_file: Arc<Mutex<Option<String>>>,
}

impl TableWalManager {
    /// Create new WAL manager
    pub fn new(wal_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(wal_dir)?;

        let mut manager = Self {
            wal_dir: wal_dir.to_path_buf(),
            writer: Arc::new(Mutex::new(None)),
            sequence: Arc::new(RwLock::new(0)),
            buffer: Arc::new(Mutex::new(VecDeque::new())),
            buffer_size: 1000,
            current_file: Arc::new(Mutex::new(None)),
        };

        // Open or create WAL file
        manager.open_wal_file()?;

        // Recover last sequence number
        manager.recover_sequence()?;

        Ok(manager)
    }

    /// Log table creation
    pub fn log_create_table(
        &self,
        table_name: String,
        schema: SchemaRef,
        primary_key: String,
    ) -> Result<()> {
        // Serialize schema as JSON (fields with name, type, nullable)
        let schema_fields: Vec<_> = schema.fields().iter()
            .map(|f| {
                serde_json::json!({
                    "name": f.name(),
                    "data_type": format!("{:?}", f.data_type()),
                    "nullable": f.is_nullable()
                })
            })
            .collect();
        let schema_json = serde_json::to_string(&schema_fields)?;

        let operation = TableWalOperation::CreateTable {
            table_name,
            primary_key,
            schema_json,
        };
        self.write_operation(operation)
    }

    /// Log table drop
    pub fn log_drop_table(&self, table_name: String) -> Result<()> {
        let operation = TableWalOperation::DropTable { table_name };
        self.write_operation(operation)
    }

    /// Log row insertion
    pub fn log_insert_row(&self, table_name: String, row: &Row) -> Result<()> {
        let row_data = row.values().to_vec();
        let operation = TableWalOperation::InsertRow {
            table_name,
            row_data,
        };
        self.write_operation(operation)
    }

    /// Write operation to WAL
    fn write_operation(&self, operation: TableWalOperation) -> Result<()> {
        // Get next sequence number
        let sequence = {
            let mut seq = self.sequence.write().unwrap();
            *seq += 1;
            *seq
        };

        // Create entry
        let entry = TableWalEntry::new(sequence, operation);

        // Add to buffer
        {
            let mut buffer = self.buffer.lock().unwrap();
            buffer.push_back(entry.clone());

            // Flush if buffer is full
            if buffer.len() >= self.buffer_size {
                drop(buffer);
                self.flush()?;
            }
        }

        Ok(())
    }

    /// Flush buffer to disk
    pub fn flush(&self) -> Result<()> {
        let entries: Vec<TableWalEntry> = {
            let mut buffer = self.buffer.lock().unwrap();
            buffer.drain(..).collect()
        };

        if entries.is_empty() {
            return Ok(());
        }

        let mut writer = self.writer.lock().unwrap();
        let writer = writer.as_mut()
            .ok_or_else(|| anyhow!("WAL writer not initialized"))?;

        for entry in entries {
            let data = bincode::serialize(&entry)?;
            let len = data.len() as u32;

            // Write length prefix
            writer.write_all(&len.to_le_bytes())?;
            // Write entry data
            writer.write_all(&data)?;
        }

        writer.flush()?;
        Ok(())
    }

    /// Checkpoint WAL (mark point for recovery)
    pub fn checkpoint(&self) -> Result<()> {
        let sequence = *self.sequence.read().unwrap();
        let operation = TableWalOperation::Checkpoint {
            sequence,
            timestamp: Utc::now(),
        };
        self.write_operation(operation)?;
        self.flush()
    }

    /// Recover from WAL
    pub fn recover(&self) -> Result<Vec<TableWalEntry>> {
        let mut entries = Vec::new();

        // Read all WAL files in directory
        let mut wal_files: Vec<_> = std::fs::read_dir(&self.wal_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("wal_")
            })
            .collect();

        // Sort by filename (timestamp-based)
        wal_files.sort_by_key(|e| e.file_name());

        for entry in wal_files {
            let file = File::open(entry.path())?;
            let mut reader = BufReader::new(file);

            loop {
                // Read length prefix
                let mut len_bytes = [0u8; 4];
                match reader.read_exact(&mut len_bytes) {
                    Ok(_) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                    Err(e) => return Err(e.into()),
                }

                let len = u32::from_le_bytes(len_bytes) as usize;

                // Read entry data
                let mut data = vec![0u8; len];
                reader.read_exact(&mut data)?;

                // Deserialize entry
                let entry: TableWalEntry = bincode::deserialize(&data)?;

                // Verify checksum
                if !entry.verify_checksum() {
                    eprintln!("WAL entry {} failed checksum verification", entry.sequence);
                    continue;
                }

                entries.push(entry);
            }
        }

        Ok(entries)
    }

    /// Open or create WAL file
    fn open_wal_file(&mut self) -> Result<()> {
        let timestamp = Utc::now().timestamp();
        let filename = format!("wal_{}.log", timestamp);
        let file_path = self.wal_dir.join(&filename);

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .context("Failed to open WAL file")?;

        let mut writer = self.writer.lock().unwrap();
        *writer = Some(BufWriter::new(file));

        let mut current = self.current_file.lock().unwrap();
        *current = Some(filename);

        Ok(())
    }

    /// Recover last sequence number from WAL files
    fn recover_sequence(&mut self) -> Result<()> {
        let entries = self.recover()?;
        if let Some(last_entry) = entries.last() {
            let mut seq = self.sequence.write().unwrap();
            *seq = last_entry.sequence;
        }
        Ok(())
    }

    /// Rotate WAL file (create new file for future writes)
    pub fn rotate(&mut self) -> Result<()> {
        // Flush current buffer
        self.flush()?;

        // Close current writer
        {
            let mut writer = self.writer.lock().unwrap();
            *writer = None;
        }

        // Open new file
        self.open_wal_file()
    }

    /// Get current sequence number
    pub fn current_sequence(&self) -> u64 {
        *self.sequence.read().unwrap()
    }
}

impl Drop for TableWalManager {
    fn drop(&mut self) {
        // Flush on drop
        let _ = self.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::datatypes::{DataType, Field, Schema};
    use std::sync::Arc;
    use tempfile::TempDir;

    #[test]
    fn test_wal_create_table() {
        let temp_dir = TempDir::new().unwrap();
        let wal = TableWalManager::new(temp_dir.path()).unwrap();

        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("name", DataType::Utf8, false),
        ]));

        wal.log_create_table("users".to_string(), schema, "id".to_string())
            .unwrap();
        wal.flush().unwrap();

        // Verify entry was written
        let entries = wal.recover().unwrap();
        assert_eq!(entries.len(), 1);

        match &entries[0].operation {
            TableWalOperation::CreateTable { table_name, .. } => {
                assert_eq!(table_name, "users");
            }
            _ => panic!("Expected CreateTable operation"),
        }
    }

    #[test]
    fn test_wal_insert_row() {
        let temp_dir = TempDir::new().unwrap();
        let wal = TableWalManager::new(temp_dir.path()).unwrap();

        let row = Row::new(vec![
            Value::Int64(1),
            Value::Text("Alice".to_string()),
        ]);

        wal.log_insert_row("users".to_string(), &row).unwrap();
        wal.flush().unwrap();

        // Verify entry
        let entries = wal.recover().unwrap();
        assert_eq!(entries.len(), 1);

        match &entries[0].operation {
            TableWalOperation::InsertRow { table_name, row_data } => {
                assert_eq!(table_name, "users");
                assert_eq!(row_data.len(), 2);
            }
            _ => panic!("Expected InsertRow operation"),
        }
    }

    #[test]
    fn test_wal_checkpoint() {
        let temp_dir = TempDir::new().unwrap();
        let wal = TableWalManager::new(temp_dir.path()).unwrap();

        // Write some operations
        let row = Row::new(vec![Value::Int64(1)]);
        wal.log_insert_row("test".to_string(), &row).unwrap();

        // Checkpoint
        wal.checkpoint().unwrap();

        // Verify checkpoint entry
        let entries = wal.recover().unwrap();
        assert_eq!(entries.len(), 2); // INSERT + CHECKPOINT

        let last = entries.last().unwrap();

        match &last.operation {
            TableWalOperation::Checkpoint { sequence, .. } => {
                // Checkpoint stores the sequence of the last operation before it
                assert_eq!(*sequence, 1); // The INSERT had sequence 1
                assert_eq!(last.sequence, 2); // The checkpoint itself has sequence 2
            }
            _ => panic!("Expected Checkpoint operation"),
        }
    }

    #[test]
    fn test_wal_recovery() {
        let temp_dir = TempDir::new().unwrap();

        // Phase 1: Write operations
        {
            let wal = TableWalManager::new(temp_dir.path()).unwrap();

            let schema = Arc::new(Schema::new(vec![
                Field::new("id", DataType::Int64, false),
            ]));

            wal.log_create_table("test".to_string(), schema, "id".to_string())
                .unwrap();

            for i in 0..10 {
                let row = Row::new(vec![Value::Int64(i)]);
                wal.log_insert_row("test".to_string(), &row).unwrap();
            }

            wal.flush().unwrap();
        }

        // Phase 2: Recover
        {
            let wal = TableWalManager::new(temp_dir.path()).unwrap();
            let entries = wal.recover().unwrap();

            assert_eq!(entries.len(), 11); // 1 CREATE + 10 INSERTs
        }
    }

    #[test]
    fn test_wal_checksum_verification() {
        let operation = TableWalOperation::CreateTable {
            table_name: "test".to_string(),
            primary_key: "id".to_string(),
            schema_json: "{}".to_string(),
        };

        let entry = TableWalEntry::new(1, operation);
        assert!(entry.verify_checksum());

        // Tamper with entry
        let mut tampered = entry.clone();
        tampered.sequence = 999;
        assert!(!tampered.verify_checksum());
    }
}