//! Write-Ahead Logging (WAL) for crash recovery
//!
//! The WAL ensures durability by logging all changes before applying them.
//! This allows the database to recover from crashes while maintaining ACID properties.

use anyhow::{Result, Context};
use std::fs::{File, OpenOptions};
use std::io::{Write, Read, Seek, SeekFrom, BufWriter, BufReader};
use std::path::Path;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};

/// WAL entry types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WALEntryType {
    /// Page write operation
    PageWrite {
        page_id: u32,
        data: Vec<u8>,
    },
    /// Transaction begin
    TxnBegin {
        txn_id: u64,
    },
    /// Transaction commit
    TxnCommit {
        txn_id: u64,
    },
    /// Transaction abort
    TxnAbort {
        txn_id: u64,
    },
    /// Checkpoint marker
    Checkpoint {
        lsn: u64,
    },
}

/// WAL entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WALEntry {
    /// Log Sequence Number - monotonically increasing
    pub lsn: u64,
    /// Transaction ID associated with this entry
    pub txn_id: u64,
    /// Timestamp when entry was created
    pub timestamp: u64,
    /// Entry type and data
    pub entry_type: WALEntryType,
    /// Checksum for integrity verification
    pub checksum: u32,
}

impl WALEntry {
    /// Create a new WAL entry
    pub fn new(lsn: u64, txn_id: u64, entry_type: WALEntryType) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
        
        let mut entry = Self {
            lsn,
            txn_id,
            timestamp,
            entry_type,
            checksum: 0,
        };
        
        // Calculate checksum (simple for now)
        entry.checksum = entry.calculate_checksum();
        entry
    }
    
    /// Calculate checksum for this entry
    fn calculate_checksum(&self) -> u32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        self.lsn.hash(&mut hasher);
        self.txn_id.hash(&mut hasher);
        self.timestamp.hash(&mut hasher);
        
        // Hash the entry type
        match &self.entry_type {
            WALEntryType::PageWrite { page_id, data } => {
                "PageWrite".hash(&mut hasher);
                page_id.hash(&mut hasher);
                data.hash(&mut hasher);
            },
            WALEntryType::TxnBegin { txn_id } => {
                "TxnBegin".hash(&mut hasher);
                txn_id.hash(&mut hasher);
            },
            WALEntryType::TxnCommit { txn_id } => {
                "TxnCommit".hash(&mut hasher);
                txn_id.hash(&mut hasher);
            },
            WALEntryType::TxnAbort { txn_id } => {
                "TxnAbort".hash(&mut hasher);
                txn_id.hash(&mut hasher);
            },
            WALEntryType::Checkpoint { lsn } => {
                "Checkpoint".hash(&mut hasher);
                lsn.hash(&mut hasher);
            },
        }
        
        hasher.finish() as u32
    }
    
    /// Verify checksum integrity
    pub fn verify_checksum(&self) -> bool {
        let calculated = self.calculate_checksum();
        self.checksum == calculated
    }
    
    /// Serialize entry to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self)
            .context("Failed to serialize WAL entry")
    }
    
    /// Deserialize entry from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let entry: WALEntry = bincode::deserialize(bytes)
            .context("Failed to deserialize WAL entry")?;
        
        if !entry.verify_checksum() {
            anyhow::bail!("WAL entry checksum verification failed");
        }
        
        Ok(entry)
    }
}

/// Write-Ahead Log manager
pub struct WALManager {
    /// WAL file handle
    file: Arc<Mutex<BufWriter<File>>>,
    /// Current LSN (Log Sequence Number)
    current_lsn: Arc<Mutex<u64>>,
    /// WAL file path
    file_path: String,
    /// Last checkpoint LSN
    last_checkpoint_lsn: Arc<Mutex<u64>>,
}

impl WALManager {
    /// Create a new WAL manager
    pub fn new<P: AsRef<Path>>(wal_path: P) -> Result<Self> {
        let file_path = wal_path.as_ref().to_string_lossy().to_string();
        
        // Open or create WAL file
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&wal_path)
            .with_context(|| format!("Failed to open WAL file: {}", file_path))?;
        
        let buffered_file = BufWriter::new(file);
        
        // Read current LSN from file
        let current_lsn = Self::read_last_lsn(&wal_path)?;
        
        Ok(Self {
            file: Arc::new(Mutex::new(buffered_file)),
            current_lsn: Arc::new(Mutex::new(current_lsn)),
            file_path,
            last_checkpoint_lsn: Arc::new(Mutex::new(0)),
        })
    }
    
    /// Read the last LSN from the WAL file
    fn read_last_lsn<P: AsRef<Path>>(wal_path: P) -> Result<u64> {
        let file = match File::open(&wal_path) {
            Ok(f) => f,
            Err(_) => return Ok(1), // Start from LSN 1 if file doesn't exist
        };
        
        let mut reader = BufReader::new(file);
        let mut last_lsn = 0u64;
        let mut buffer = Vec::new();
        
        // Read all entries to find the highest LSN
        loop {
            // Read entry length (4 bytes)
            let mut len_bytes = [0u8; 4];
            match reader.read_exact(&mut len_bytes) {
                Ok(_) => {},
                Err(_) => break, // End of file
            }
            
            let entry_len = u32::from_be_bytes(len_bytes) as usize;
            
            // Read entry data
            buffer.resize(entry_len, 0);
            reader.read_exact(&mut buffer)?;
            
            // Parse entry to get LSN
            if let Ok(entry) = WALEntry::from_bytes(&buffer) {
                last_lsn = last_lsn.max(entry.lsn);
            }
        }
        
        Ok(last_lsn + 1) // Next LSN to use
    }
    
    /// Write a WAL entry
    pub fn write_entry(&self, txn_id: u64, entry_type: WALEntryType) -> Result<u64> {
        let lsn = {
            let mut current_lsn = self.current_lsn.lock().unwrap();
            let lsn = *current_lsn;
            *current_lsn += 1;
            lsn
        };
        
        let entry = WALEntry::new(lsn, txn_id, entry_type);
        let entry_bytes = entry.to_bytes()?;
        
        // Write to file: [length][entry_data]
        let mut file = self.file.lock().unwrap();
        file.write_all(&(entry_bytes.len() as u32).to_be_bytes())?;
        file.write_all(&entry_bytes)?;
        file.flush()?; // Ensure it's written to disk immediately
        
        Ok(lsn)
    }
    
    /// Force sync to disk
    pub fn sync(&self) -> Result<()> {
        let mut file = self.file.lock().unwrap();
        file.flush()?;
        file.get_mut().sync_all()
            .context("Failed to sync WAL to disk")
    }
    
    /// Create a checkpoint
    pub fn checkpoint(&self) -> Result<u64> {
        let current_lsn = *self.current_lsn.lock().unwrap();
        let checkpoint_lsn = self.write_entry(0, WALEntryType::Checkpoint { lsn: current_lsn })?;
        
        *self.last_checkpoint_lsn.lock().unwrap() = checkpoint_lsn;
        
        // Force sync checkpoint to disk
        self.sync()?;
        
        Ok(checkpoint_lsn)
    }
    
    /// Replay WAL entries from the last checkpoint
    pub fn replay<F>(&self, mut apply_fn: F) -> Result<()> 
    where 
        F: FnMut(&WALEntry) -> Result<()>
    {
        let file = File::open(&self.file_path)?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        let mut uncommitted_txns = std::collections::HashSet::new();
        let mut committed_txns = std::collections::HashSet::new();
        let mut aborted_txns = std::collections::HashSet::new();
        
        // First pass: identify transaction states
        loop {
            // Read entry length
            let mut len_bytes = [0u8; 4];
            match reader.read_exact(&mut len_bytes) {
                Ok(_) => {},
                Err(_) => break, // End of file
            }
            
            let entry_len = u32::from_be_bytes(len_bytes) as usize;
            
            // Read entry data
            buffer.resize(entry_len, 0);
            reader.read_exact(&mut buffer)?;
            
            // Parse entry
            let entry = WALEntry::from_bytes(&buffer)?;
            
            match &entry.entry_type {
                WALEntryType::TxnBegin { txn_id } => {
                    uncommitted_txns.insert(*txn_id);
                },
                WALEntryType::TxnCommit { txn_id } => {
                    uncommitted_txns.remove(txn_id);
                    committed_txns.insert(*txn_id);
                },
                WALEntryType::TxnAbort { txn_id } => {
                    uncommitted_txns.remove(txn_id);
                    aborted_txns.insert(*txn_id);
                },
                _ => {}
            }
        }
        
        // Second pass: replay only committed transactions
        let mut reader = BufReader::new(File::open(&self.file_path)?);
        loop {
            // Read entry length
            let mut len_bytes = [0u8; 4];
            match reader.read_exact(&mut len_bytes) {
                Ok(_) => {},
                Err(_) => break, // End of file
            }
            
            let entry_len = u32::from_be_bytes(len_bytes) as usize;
            
            // Read entry data
            buffer.resize(entry_len, 0);
            reader.read_exact(&mut buffer)?;
            
            // Parse entry
            let entry = WALEntry::from_bytes(&buffer)?;
            
            // Only replay entries from committed transactions
            if committed_txns.contains(&entry.txn_id) || entry.txn_id == 0 {
                apply_fn(&entry)?;
            }
        }
        
        Ok(())
    }
    
    /// Get current LSN
    pub fn current_lsn(&self) -> u64 {
        *self.current_lsn.lock().unwrap()
    }
    
    /// Get last checkpoint LSN
    pub fn last_checkpoint_lsn(&self) -> u64 {
        *self.last_checkpoint_lsn.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_wal_entry_checksum() {
        let entry = WALEntry::new(
            1, 
            100, 
            WALEntryType::PageWrite { 
                page_id: 42, 
                data: vec![1, 2, 3, 4] 
            }
        );
        
        assert!(entry.verify_checksum());
        
        // Corrupt the entry and verify checksum fails
        let mut corrupted = entry.clone();
        corrupted.lsn = 2;
        assert!(!corrupted.verify_checksum());
    }
    
    #[test]
    fn test_wal_serialization() {
        let entry = WALEntry::new(
            1, 
            100, 
            WALEntryType::TxnBegin { txn_id: 100 }
        );
        
        let bytes = entry.to_bytes().unwrap();
        let deserialized = WALEntry::from_bytes(&bytes).unwrap();
        
        assert_eq!(entry.lsn, deserialized.lsn);
        assert_eq!(entry.txn_id, deserialized.txn_id);
        assert!(matches!(deserialized.entry_type, WALEntryType::TxnBegin { txn_id: 100 }));
    }
    
    #[test]
    fn test_wal_manager_basic() {
        let temp_dir = TempDir::new().unwrap();
        let wal_path = temp_dir.path().join("test.wal");
        
        let wal = WALManager::new(&wal_path).unwrap();
        
        // Write some entries
        let lsn1 = wal.write_entry(1, WALEntryType::TxnBegin { txn_id: 1 }).unwrap();
        let lsn2 = wal.write_entry(1, WALEntryType::PageWrite { 
            page_id: 42, 
            data: vec![1, 2, 3] 
        }).unwrap();
        let lsn3 = wal.write_entry(1, WALEntryType::TxnCommit { txn_id: 1 }).unwrap();
        
        assert_eq!(lsn1, 1);
        assert_eq!(lsn2, 2);
        assert_eq!(lsn3, 3);
        
        wal.sync().unwrap();
        
        // Test replay
        let mut replayed_entries = Vec::new();
        wal.replay(|entry| {
            replayed_entries.push(entry.clone());
            Ok(())
        }).unwrap();
        
        assert_eq!(replayed_entries.len(), 3);
        assert_eq!(replayed_entries[0].lsn, 1);
        assert_eq!(replayed_entries[1].lsn, 2);
        assert_eq!(replayed_entries[2].lsn, 3);
    }
}