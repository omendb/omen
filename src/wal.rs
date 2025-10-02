//! Write-Ahead Logging (WAL) for durability and crash recovery
//! Ensures ACID properties and data persistence

use anyhow::{Context, Result};
use bincode;
use chrono::{DateTime, Utc};
use crc32fast::Hasher;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

/// WAL operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalOperation {
    /// Insert a key-value pair
    Insert {
        timestamp: i64,
        value: f64,
        series_id: i64,
    },
    /// Delete a key
    Delete { timestamp: i64 },
    /// Update an existing value
    Update { timestamp: i64, new_value: f64 },
    /// Checkpoint marker
    Checkpoint {
        sequence: u64,
        timestamp: DateTime<Utc>,
    },
    /// Transaction begin
    BeginTxn { txn_id: u64 },
    /// Transaction commit
    CommitTxn { txn_id: u64 },
    /// Transaction rollback
    RollbackTxn { txn_id: u64 },
}

/// WAL entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalEntry {
    /// Unique sequence number
    pub sequence: u64,
    /// Operation to perform
    pub operation: WalOperation,
    /// Timestamp of entry creation
    pub timestamp: DateTime<Utc>,
    /// CRC32 checksum for integrity
    pub checksum: u32,
}

impl WalEntry {
    pub fn new(sequence: u64, operation: WalOperation) -> Self {
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

/// Write-Ahead Log manager
pub struct WalManager {
    /// Path to WAL directory
    wal_dir: PathBuf,
    /// Current WAL file writer
    writer: Arc<Mutex<Option<BufWriter<File>>>>,
    /// Current sequence number
    sequence: Arc<RwLock<u64>>,
    /// Buffer of pending writes
    buffer: Arc<Mutex<VecDeque<WalEntry>>>,
    /// Maximum buffer size before flush
    max_buffer_size: usize,
    /// Sync to disk after each write
    sync_on_write: bool,
    /// Archive old WAL files
    archive_enabled: bool,
}

impl WalManager {
    /// Create new WAL manager
    pub fn new<P: AsRef<Path>>(wal_dir: P) -> Result<Self> {
        let wal_dir = wal_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&wal_dir).context("Failed to create WAL directory")?;

        Ok(Self {
            wal_dir,
            writer: Arc::new(Mutex::new(None)),
            sequence: Arc::new(RwLock::new(0)),
            buffer: Arc::new(Mutex::new(VecDeque::new())),
            max_buffer_size: 1000,
            sync_on_write: true,
            archive_enabled: true,
        })
    }

    /// Open or create WAL file
    pub fn open(&self) -> Result<()> {
        let wal_path = self.current_wal_path();

        // Check if WAL exists before opening
        let exists_before_open = wal_path.exists();

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&wal_path)
            .context("Failed to open WAL file")?;

        let writer = BufWriter::new(file);
        *self.writer.lock().unwrap() = Some(writer);

        // Recover sequence number from existing WAL
        if exists_before_open {
            let last_seq = self.find_last_sequence(&wal_path)?;
            *self.sequence.write().unwrap() = last_seq + 1;
        }

        Ok(())
    }

    /// Write operation to WAL
    pub fn write(&self, operation: WalOperation) -> Result<u64> {
        let sequence = {
            let mut seq = self.sequence.write().unwrap();
            let current = *seq;
            *seq += 1;
            current
        };

        let entry = WalEntry::new(sequence, operation);

        // Add to buffer
        {
            let mut buffer = self.buffer.lock().unwrap();
            buffer.push_back(entry.clone());

            // Flush if buffer is full
            if buffer.len() >= self.max_buffer_size {
                self.flush_buffer_locked(&mut buffer)?;
            }
        }

        // Sync if required
        if self.sync_on_write {
            self.sync()?;
        }

        Ok(sequence)
    }

    /// Flush buffer to disk
    pub fn flush(&self) -> Result<()> {
        let mut buffer = self.buffer.lock().unwrap();
        self.flush_buffer_locked(&mut buffer)
    }

    fn flush_buffer_locked(&self, buffer: &mut VecDeque<WalEntry>) -> Result<()> {
        if let Some(writer) = self.writer.lock().unwrap().as_mut() {
            while let Some(entry) = buffer.pop_front() {
                let data = bincode::serialize(&entry).context("Failed to serialize WAL entry")?;

                // Write length prefix
                let len = data.len() as u32;
                writer.write_all(&len.to_le_bytes())?;

                // Write data
                writer.write_all(&data)?;
            }
            writer.flush()?;
        }
        Ok(())
    }

    /// Sync WAL to disk
    pub fn sync(&self) -> Result<()> {
        self.flush()?;

        if let Some(writer) = self.writer.lock().unwrap().as_mut() {
            writer
                .get_mut()
                .sync_all()
                .context("Failed to sync WAL to disk")?;
        }

        Ok(())
    }

    /// Create checkpoint and rotate WAL
    pub fn checkpoint(&self) -> Result<()> {
        let sequence = *self.sequence.read().unwrap();

        // Write checkpoint marker
        self.write(WalOperation::Checkpoint {
            sequence,
            timestamp: Utc::now(),
        })?;

        // Sync current WAL
        self.sync()?;

        // Rotate to new WAL file
        self.rotate()?;

        Ok(())
    }

    /// Rotate WAL file
    fn rotate(&self) -> Result<()> {
        // Close current writer
        {
            let mut writer = self.writer.lock().unwrap();
            if let Some(w) = writer.as_mut() {
                w.flush()?;
                w.get_mut().sync_all()?;
            }
            *writer = None;
        }

        // Archive current WAL
        if self.archive_enabled {
            let current = self.current_wal_path();
            if current.exists() {
                let archive = self.archive_wal_path();
                std::fs::rename(&current, &archive).context("Failed to archive WAL")?;
            }
        }

        // Open new WAL
        self.open()
    }

    /// Recover from WAL files
    pub fn recover<F>(&self, mut apply_fn: F) -> Result<RecoveryStats>
    where
        F: FnMut(&WalOperation) -> Result<()>,
    {
        let mut stats = RecoveryStats::default();

        // Find all WAL files
        let wal_files = self.find_wal_files()?;

        for wal_path in wal_files {
            let file = File::open(&wal_path).context("Failed to open WAL file for recovery")?;
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

                // Sanity check: WAL entries shouldn't be huge
                if len > 10_000_000 {
                    // 10MB max per entry
                    stats.corrupted_entries += 1;
                    break; // Rest of file likely corrupted
                }

                // Read entry data
                let mut data = vec![0u8; len];
                match reader.read_exact(&mut data) {
                    Ok(_) => {}
                    Err(_) => {
                        stats.corrupted_entries += 1;
                        break; // Can't continue reading
                    }
                }

                // Deserialize entry
                let entry: WalEntry = match bincode::deserialize(&data) {
                    Ok(e) => e,
                    Err(_) => {
                        stats.corrupted_entries += 1;
                        continue;
                    }
                };

                // Verify checksum
                if !entry.verify_checksum() {
                    stats.corrupted_entries += 1;
                    continue;
                }

                // Apply operation
                match apply_fn(&entry.operation) {
                    Ok(_) => stats.applied_entries += 1,
                    Err(_) => stats.failed_entries += 1,
                }

                stats.total_entries += 1;

                // Update sequence
                if entry.sequence > stats.last_sequence {
                    stats.last_sequence = entry.sequence;
                }
            }
        }

        // Update current sequence
        *self.sequence.write().unwrap() = stats.last_sequence + 1;

        Ok(stats)
    }

    /// Find all WAL files in directory
    fn find_wal_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        for entry in std::fs::read_dir(&self.wal_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("wal") {
                files.push(path);
            }
        }

        // Sort by modification time
        files.sort_by_key(|p| {
            std::fs::metadata(p)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        });

        Ok(files)
    }

    /// Find last sequence number in WAL file
    fn find_last_sequence(&self, path: &Path) -> Result<u64> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut last_sequence = 0u64;

        loop {
            // Try to read entry
            let mut len_bytes = [0u8; 4];
            match reader.read_exact(&mut len_bytes) {
                Ok(_) => {}
                Err(_) => break,
            }

            let len = u32::from_le_bytes(len_bytes) as usize;
            let mut data = vec![0u8; len];

            match reader.read_exact(&mut data) {
                Ok(_) => {}
                Err(_) => break,
            }

            if let Ok(entry) = bincode::deserialize::<WalEntry>(&data) {
                if entry.sequence > last_sequence {
                    last_sequence = entry.sequence;
                }
            }
        }

        Ok(last_sequence)
    }

    fn current_wal_path(&self) -> PathBuf {
        self.wal_dir.join("current.wal")
    }

    fn archive_wal_path(&self) -> PathBuf {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        self.wal_dir.join(format!("archive_{}.wal", timestamp))
    }

    /// Clean up old WAL files
    pub fn cleanup(&self, keep_days: i64) -> Result<usize> {
        let cutoff = Utc::now() - chrono::Duration::days(keep_days);
        let mut removed = 0;

        for path in self.find_wal_files()? {
            if let Ok(metadata) = std::fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    let modified: DateTime<Utc> = modified.into();

                    if modified < cutoff && path.file_name() != Some("current.wal".as_ref()) {
                        std::fs::remove_file(&path)?;
                        removed += 1;
                    }
                }
            }
        }

        Ok(removed)
    }
}

/// Recovery statistics
#[derive(Debug, Default)]
pub struct RecoveryStats {
    pub total_entries: usize,
    pub applied_entries: usize,
    pub failed_entries: usize,
    pub corrupted_entries: usize,
    pub last_sequence: u64,
}

/// Transaction manager using WAL
pub struct TransactionManager {
    wal: Arc<WalManager>,
    active_txns: Arc<RwLock<Vec<u64>>>,
    next_txn_id: Arc<RwLock<u64>>,
}

impl TransactionManager {
    pub fn new(wal: Arc<WalManager>) -> Self {
        Self {
            wal,
            active_txns: Arc::new(RwLock::new(Vec::new())),
            next_txn_id: Arc::new(RwLock::new(0)),
        }
    }

    /// Begin new transaction
    pub fn begin(&self) -> Result<Transaction> {
        let txn_id = {
            let mut id = self.next_txn_id.write().unwrap();
            let current = *id;
            *id += 1;
            current
        };

        self.wal.write(WalOperation::BeginTxn { txn_id })?;

        let mut active = self.active_txns.write().unwrap();
        active.push(txn_id);

        Ok(Transaction {
            id: txn_id,
            wal: Arc::clone(&self.wal),
            manager: Arc::new(TransactionManager::new(Arc::clone(&self.wal))),
            committed: false,
        })
    }
}

/// Transaction handle
pub struct Transaction {
    id: u64,
    wal: Arc<WalManager>,
    manager: Arc<TransactionManager>,
    committed: bool,
}

impl Transaction {
    /// Get transaction ID
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Write operation within transaction
    pub fn write(&self, operation: WalOperation) -> Result<u64> {
        self.wal.write(operation)
    }

    /// Commit transaction
    pub fn commit(mut self) -> Result<()> {
        self.wal
            .write(WalOperation::CommitTxn { txn_id: self.id })?;
        self.wal.sync()?;

        let mut active = self.manager.active_txns.write().unwrap();
        active.retain(|&id| id != self.id);

        self.committed = true;
        Ok(())
    }

    /// Rollback transaction
    pub fn rollback(mut self) -> Result<()> {
        self.wal
            .write(WalOperation::RollbackTxn { txn_id: self.id })?;

        let mut active = self.manager.active_txns.write().unwrap();
        active.retain(|&id| id != self.id);

        self.committed = true;
        Ok(())
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        if !self.committed {
            // Auto-rollback on drop
            let _ = self
                .wal
                .write(WalOperation::RollbackTxn { txn_id: self.id });

            let mut active = self.manager.active_txns.write().unwrap();
            active.retain(|&id| id != self.id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_wal_basic_operations() {
        let dir = tempdir().unwrap();
        let wal = WalManager::new(dir.path()).unwrap();
        wal.open().unwrap();

        // Write operations
        let seq1 = wal
            .write(WalOperation::Insert {
                timestamp: 100,
                value: 1.5,
                series_id: 1,
            })
            .unwrap();

        let seq2 = wal
            .write(WalOperation::Update {
                timestamp: 100,
                new_value: 2.5,
            })
            .unwrap();

        assert_eq!(seq1, 0);
        assert_eq!(seq2, 1);

        // Sync to disk
        wal.sync().unwrap();
    }

    #[test]
    fn test_wal_recovery() {
        let dir = tempdir().unwrap();

        // Write some operations
        {
            let wal = WalManager::new(dir.path()).unwrap();
            wal.open().unwrap();

            wal.write(WalOperation::Insert {
                timestamp: 100,
                value: 1.5,
                series_id: 1,
            })
            .unwrap();

            wal.write(WalOperation::Insert {
                timestamp: 200,
                value: 2.5,
                series_id: 2,
            })
            .unwrap();

            wal.sync().unwrap();
        }

        // Recover operations
        {
            let wal = WalManager::new(dir.path()).unwrap();
            let mut recovered = Vec::new();

            let stats = wal
                .recover(|op| {
                    recovered.push(op.clone());
                    Ok(())
                })
                .unwrap();

            assert_eq!(stats.total_entries, 2);
            assert_eq!(stats.applied_entries, 2);
            assert_eq!(recovered.len(), 2);
        }
    }

    #[test]
    fn test_transaction_commit() {
        let dir = tempdir().unwrap();
        let wal = Arc::new(WalManager::new(dir.path()).unwrap());
        wal.open().unwrap();

        let txn_mgr = TransactionManager::new(wal);

        let txn = txn_mgr.begin().unwrap();
        txn.write(WalOperation::Insert {
            timestamp: 100,
            value: 1.5,
            series_id: 1,
        })
        .unwrap();
        txn.commit().unwrap();
    }

    #[test]
    fn test_checkpoint_and_rotation() {
        let dir = tempdir().unwrap();
        let wal = WalManager::new(dir.path()).unwrap();
        wal.open().unwrap();

        // Write some data
        for i in 0..10 {
            wal.write(WalOperation::Insert {
                timestamp: i * 100,
                value: i as f64,
                series_id: 1,
            })
            .unwrap();
        }

        // Create checkpoint
        wal.checkpoint().unwrap();

        // Verify new WAL file created
        let files = wal.find_wal_files().unwrap();
        assert!(files.len() >= 1);
    }

    #[test]
    fn test_corrupted_entry_handling() {
        let dir = tempdir().unwrap();
        let wal_path = dir.path().join("current.wal");

        // Write valid entry followed by corrupted data
        {
            let wal = WalManager::new(dir.path()).unwrap();
            wal.open().unwrap();

            wal.write(WalOperation::Insert {
                timestamp: 100,
                value: 1.5,
                series_id: 1,
            })
            .unwrap();

            wal.sync().unwrap();
        }

        // Append corrupted data directly
        {
            let mut file = OpenOptions::new().append(true).open(&wal_path).unwrap();

            // Write invalid length prefix
            file.write_all(&[255, 255, 255, 255]).unwrap();
            file.write_all(b"corrupted data").unwrap();
        }

        // Recovery should handle corruption gracefully
        {
            let wal = WalManager::new(dir.path()).unwrap();
            let mut recovered = 0;

            let stats = wal
                .recover(|_| {
                    recovered += 1;
                    Ok(())
                })
                .unwrap();

            assert_eq!(recovered, 1); // Only valid entry recovered
            assert_eq!(stats.applied_entries, 1);
        }
    }
}
