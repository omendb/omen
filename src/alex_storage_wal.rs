//! Write-Ahead Log (WAL) for AlexStorage
//!
//! Provides crash recovery and durability guarantees for AlexStorage.
//!
//! Design:
//! - Append-only log file for all mutations
//! - Sequential writes for performance
//! - Replay on startup for crash recovery
//! - Periodic checkpointing to compact log
//!
//! File Format:
//! Each entry: [entry_type:1][key:8][value_len:4][value:N]
//!
//! Entry Types:
//! - 0x01: Insert
//! - 0x02: Delete (future)
//! - 0xFF: Checkpoint marker

use anyhow::{Context, Result};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

/// WAL entry type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WalEntryType {
    Insert = 0x01,
    Delete = 0x02,
    Checkpoint = 0xFF,
}

impl WalEntryType {
    fn from_u8(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(WalEntryType::Insert),
            0x02 => Some(WalEntryType::Delete),
            0xFF => Some(WalEntryType::Checkpoint),
            _ => None,
        }
    }
}

/// WAL entry
#[derive(Debug, Clone)]
pub struct WalEntry {
    pub entry_type: WalEntryType,
    pub key: i64,
    pub value: Vec<u8>,
}

/// Write-Ahead Log for AlexStorage
///
/// Provides durability and crash recovery by logging all mutations
/// before applying them to the main storage.
#[derive(Debug)]
pub struct AlexStorageWal {
    /// Path to WAL file
    wal_path: PathBuf,

    /// Write handle for appending entries
    writer: BufWriter<File>,

    /// Number of entries since last checkpoint
    entries_since_checkpoint: usize,

    /// Checkpoint threshold (number of entries)
    checkpoint_threshold: usize,
}

impl AlexStorageWal {
    /// Create or open WAL at given path
    pub fn new<P: AsRef<Path>>(path: P, checkpoint_threshold: usize) -> Result<Self> {
        let wal_path = path.as_ref().join("wal.log");

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&wal_path)
            .context("Failed to create WAL file")?;

        let writer = BufWriter::new(file);

        Ok(Self {
            wal_path,
            writer,
            entries_since_checkpoint: 0,
            checkpoint_threshold,
        })
    }

    /// Log an insert operation
    ///
    /// Format: [0x01][key:8][value_len:4][value:N]
    pub fn log_insert(&mut self, key: i64, value: &[u8]) -> Result<()> {
        // Write entry type
        self.writer.write_all(&[WalEntryType::Insert as u8])?;

        // Write key
        self.writer.write_all(&key.to_le_bytes())?;

        // Write value length
        self.writer.write_all(&(value.len() as u32).to_le_bytes())?;

        // Write value
        self.writer.write_all(value)?;

        // Flush to ensure durability
        self.writer.flush()?;

        self.entries_since_checkpoint += 1;

        Ok(())
    }

    /// Log a delete operation
    ///
    /// Format: [0x02][key:8][value_len:4=0]
    pub fn log_delete(&mut self, key: i64) -> Result<()> {
        // Write entry type
        self.writer.write_all(&[WalEntryType::Delete as u8])?;

        // Write key
        self.writer.write_all(&key.to_le_bytes())?;

        // Write value length (0 for delete)
        self.writer.write_all(&0u32.to_le_bytes())?;

        // Flush to ensure durability
        self.writer.flush()?;

        self.entries_since_checkpoint += 1;

        Ok(())
    }

    /// Write checkpoint marker and truncate log
    ///
    /// Called after successfully flushing data to main storage.
    pub fn checkpoint(&mut self) -> Result<()> {
        // Write checkpoint marker
        self.writer.write_all(&[WalEntryType::Checkpoint as u8])?;
        self.writer.flush()?;

        // Truncate log (start fresh)
        drop(std::mem::replace(
            &mut self.writer,
            BufWriter::new(
                OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(&self.wal_path)?,
            ),
        ));

        self.entries_since_checkpoint = 0;

        Ok(())
    }

    /// Check if checkpoint is needed
    pub fn needs_checkpoint(&self) -> bool {
        self.entries_since_checkpoint >= self.checkpoint_threshold
    }

    /// Replay WAL entries (called during recovery)
    ///
    /// Returns all entries that need to be replayed.
    pub fn replay<P: AsRef<Path>>(path: P) -> Result<Vec<WalEntry>> {
        let wal_path = path.as_ref().join("wal.log");

        // If WAL doesn't exist, nothing to replay
        if !wal_path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&wal_path).context("Failed to open WAL for replay")?;
        let mut reader = BufReader::new(file);

        let mut entries = Vec::new();
        let mut entry_type_buf = [0u8; 1];
        let mut key_buf = [0u8; 8];
        let mut len_buf = [0u8; 4];

        loop {
            // Read entry type
            match reader.read_exact(&mut entry_type_buf) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break, // End of log
                Err(e) => return Err(e.into()),
            }

            let entry_type = WalEntryType::from_u8(entry_type_buf[0])
                .context("Invalid WAL entry type")?;

            // Checkpoint marker - all entries before this have been applied
            if entry_type == WalEntryType::Checkpoint {
                entries.clear(); // Discard entries before checkpoint
                continue;
            }

            // Read key
            reader.read_exact(&mut key_buf)?;
            let key = i64::from_le_bytes(key_buf);

            // Read value length
            reader.read_exact(&mut len_buf)?;
            let value_len = u32::from_le_bytes(len_buf) as usize;

            // Read value
            let mut value = vec![0u8; value_len];
            if value_len > 0 {
                reader.read_exact(&mut value)?;
            }

            entries.push(WalEntry {
                entry_type,
                key,
                value,
            });
        }

        Ok(entries)
    }

    /// Get WAL statistics
    pub fn stats(&self) -> WalStats {
        WalStats {
            entries_since_checkpoint: self.entries_since_checkpoint,
            checkpoint_threshold: self.checkpoint_threshold,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WalStats {
    pub entries_since_checkpoint: usize,
    pub checkpoint_threshold: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_wal_basic() {
        let dir = tempdir().unwrap();
        let mut wal = AlexStorageWal::new(dir.path(), 100).unwrap();

        // Log some inserts
        wal.log_insert(1, b"value1").unwrap();
        wal.log_insert(2, b"value2").unwrap();
        wal.log_insert(3, b"value3").unwrap();

        // Replay
        let entries = AlexStorageWal::replay(dir.path()).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].key, 1);
        assert_eq!(entries[0].value, b"value1");
        assert_eq!(entries[1].key, 2);
        assert_eq!(entries[2].key, 3);
    }

    #[test]
    fn test_wal_checkpoint() {
        let dir = tempdir().unwrap();
        let mut wal = AlexStorageWal::new(dir.path(), 100).unwrap();

        // Log some inserts
        wal.log_insert(1, b"value1").unwrap();
        wal.log_insert(2, b"value2").unwrap();

        // Checkpoint
        wal.checkpoint().unwrap();

        // Log more inserts after checkpoint
        wal.log_insert(3, b"value3").unwrap();

        // Replay - should only see entry after checkpoint
        let entries = AlexStorageWal::replay(dir.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].key, 3);
        assert_eq!(entries[0].value, b"value3");
    }

    #[test]
    fn test_wal_delete() {
        let dir = tempdir().unwrap();
        let mut wal = AlexStorageWal::new(dir.path(), 100).unwrap();

        // Log insert and delete
        wal.log_insert(1, b"value1").unwrap();
        wal.log_delete(1).unwrap();

        // Replay
        let entries = AlexStorageWal::replay(dir.path()).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].entry_type, WalEntryType::Insert);
        assert_eq!(entries[1].entry_type, WalEntryType::Delete);
        assert_eq!(entries[1].key, 1);
    }

    #[test]
    fn test_wal_needs_checkpoint() {
        let dir = tempdir().unwrap();
        let mut wal = AlexStorageWal::new(dir.path(), 3).unwrap();

        assert!(!wal.needs_checkpoint());

        wal.log_insert(1, b"value1").unwrap();
        assert!(!wal.needs_checkpoint());

        wal.log_insert(2, b"value2").unwrap();
        assert!(!wal.needs_checkpoint());

        wal.log_insert(3, b"value3").unwrap();
        assert!(wal.needs_checkpoint());
    }
}
