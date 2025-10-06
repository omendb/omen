//! AlexStorage: Custom mmap-based storage with ALEX learned index
//!
//! Architecture:
//! - ALEX: Tracks (key → offset in file)
//! - Mmap: Memory-mapped file for zero-copy value access
//! - Append-only: New writes append to end of file
//! - WAL: Write-ahead log for crash recovery and durability
//!
//! Performance targets (validated via benchmark_mmap_validation):
//! - Queries: ~389 ns (ALEX 218ns + mmap 151ns + overhead 20ns)
//! - 10x faster than RocksDB (3,902 ns)
//! - 5.6x faster than SQLite (2,173 ns)
//!
//! Current status: Phase 4 - WAL durability added

use crate::alex::AlexTree;
use crate::alex_storage_wal::AlexStorageWal;
use anyhow::{Context, Result};
use memmap2::{Mmap, MmapMut, MmapOptions};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Entry format in mmap file:
/// [value_len:4 bytes][value:N bytes]
const ENTRY_HEADER_SIZE: usize = 4;

/// Mmap growth chunk size (16MB) - grow in chunks to avoid frequent remaps
const MMAP_GROW_SIZE: u64 = 16 * 1024 * 1024;

/// Tombstone marker for deleted keys (special offset value)
/// Uses u64::MAX to indicate a deleted key in ALEX
const TOMBSTONE: u64 = u64::MAX;

/// AlexStorage: Memory-mapped storage with ALEX learned index
///
/// Design:
/// - ALEX stores (key → file offset)
/// - Mmap provides zero-copy access to values
/// - Append-only writes (no in-place updates)
/// - Deferred remapping: Grow mmap in 16MB chunks to minimize remap overhead
///
/// Trade-offs:
/// - Fast reads: 389ns (mmap + ALEX)
/// - Fast writes: Append-only, deferred remapping
/// - Space overhead: Deleted entries not reclaimed (until compaction)
pub struct AlexStorage {
    /// Path to data file
    data_path: PathBuf,

    /// Base path for all storage files
    base_path: PathBuf,

    /// ALEX index: key → offset in data file
    alex: AlexTree,

    /// Memory-mapped data file (read-only)
    mmap: Option<Mmap>,

    /// Current end of file (for appending new entries)
    file_size: u64,

    /// Size of currently mapped region (may be larger than file_size)
    mapped_size: u64,

    /// Write handle (for appending)
    write_file: File,

    /// Write-Ahead Log for durability
    wal: AlexStorageWal,
}

// Manual Debug implementation (WAL doesn't implement Debug)
impl std::fmt::Debug for AlexStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AlexStorage")
            .field("data_path", &self.data_path)
            .field("base_path", &self.base_path)
            .field("file_size", &self.file_size)
            .field("mapped_size", &self.mapped_size)
            .finish()
    }
}

impl AlexStorage {
    /// Create new AlexStorage at given path
    ///
    /// Automatically replays WAL if present for crash recovery.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let base_path = path.as_ref().to_path_buf();
        let data_path = base_path.join("data.bin");

        // Create or open data file
        let write_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&data_path)
            .context("Failed to create data file")?;

        // Get file size
        let metadata = write_file.metadata()?;
        let file_size = metadata.len();

        // Memory-map the file if it's not empty
        let mmap = if file_size > 0 {
            let read_file = File::open(&data_path)?;
            let mmap = unsafe { Mmap::map(&read_file)? };
            Some(mmap)
        } else {
            None
        };

        // Create WAL (checkpoint threshold: 1000 entries)
        let wal = AlexStorageWal::new(&base_path, 1000)?;

        let mut storage = Self {
            data_path: data_path.clone(),
            base_path: base_path.clone(),
            alex: AlexTree::new(),
            mmap,
            file_size,
            mapped_size: file_size,
            write_file,
            wal,
        };

        // Load existing keys from file if present
        if file_size > 0 {
            storage.load_keys_from_file()?;
        }

        // Replay WAL for crash recovery
        storage.replay_wal()?;

        Ok(storage)
    }

    /// Replay WAL entries (crash recovery)
    fn replay_wal(&mut self) -> Result<()> {
        let entries = AlexStorageWal::replay(&self.base_path)?;

        if entries.is_empty() {
            return Ok(());
        }

        // Replay all entries that aren't already in the storage
        for entry in &entries {
            match entry.entry_type {
                crate::alex_storage_wal::WalEntryType::Insert => {
                    // Check if key already exists in ALEX (already flushed to file)
                    if self.alex.get(entry.key)?.is_some() {
                        // Key already in file, skip replay (already persisted)
                        continue;
                    }
                    // Apply insert without logging to WAL (already logged)
                    self.insert_no_wal(entry.key, &entry.value)?;
                }
                crate::alex_storage_wal::WalEntryType::Delete => {
                    // Mark as deleted in ALEX (set offset to TOMBSTONE)
                    self.alex.insert(entry.key, TOMBSTONE.to_le_bytes().to_vec())?;
                }
                crate::alex_storage_wal::WalEntryType::Checkpoint => {
                    // Checkpoint marker - should not appear in replayed entries
                }
            }
        }

        // After successful replay, checkpoint WAL
        self.wal.checkpoint()?;

        Ok(())
    }

    /// Load keys from existing data file into ALEX
    fn load_keys_from_file(&mut self) -> Result<()> {
        if self.mmap.is_none() {
            return Ok(());
        }

        let mmap = self.mmap.as_ref().unwrap();
        let mut offset = 0u64;
        let mut entries = Vec::new();

        // Scan file and collect (key, offset) pairs
        while offset < self.file_size {
            // Read entry header: [value_len:4]
            if offset + ENTRY_HEADER_SIZE as u64 > self.file_size {
                break; // Incomplete entry
            }

            let value_len_bytes = &mmap[offset as usize..offset as usize + 4];
            let value_len = u32::from_le_bytes(value_len_bytes.try_into().unwrap()) as usize;

            // Skip past header + value
            offset += ENTRY_HEADER_SIZE as u64;

            if offset + value_len as u64 > self.file_size {
                break; // Incomplete entry
            }

            // For now, we need to extract the key from the value
            // TODO: Store key separately in entry format
            // Format: [value_len:4][key:8][value:(N-8)]
            if value_len >= 8 {
                let key_bytes = &mmap[offset as usize..offset as usize + 8];
                let key = i64::from_le_bytes(key_bytes.try_into().unwrap());

                // Store (key → offset) in ALEX
                // Offset points to start of key+value (after length header)
                entries.push((key, offset.to_le_bytes().to_vec()));
            }

            offset += value_len as u64;
        }

        // Bulk insert into ALEX
        if !entries.is_empty() {
            self.alex.insert_batch(entries)?;
        }

        Ok(())
    }

    /// Insert key-value pair (with WAL logging for durability)
    ///
    /// Format written to file:
    /// [value_len:4 bytes][key:8 bytes][value:N bytes]
    ///
    /// ALEX stores: (key → offset of key+value)
    pub fn insert(&mut self, key: i64, value: &[u8]) -> Result<()> {
        // Log to WAL first for durability
        self.wal.log_insert(key, value)?;

        // Apply insert
        self.insert_no_wal(key, value)?;

        // Checkpoint if needed
        if self.wal.needs_checkpoint() {
            self.wal.checkpoint()?;
        }

        Ok(())
    }

    /// Insert key-value pair without WAL logging (internal use for replay)
    fn insert_no_wal(&mut self, key: i64, value: &[u8]) -> Result<()> {
        // Calculate total size: key (8) + value (N)
        let data_len = 8 + value.len();
        let total_len = ENTRY_HEADER_SIZE + data_len;

        // Append to file: [value_len:4][key:8][value:N]
        let current_offset = self.file_size;

        // Write length header
        self.write_file.write_all(&(data_len as u32).to_le_bytes())?;

        // Write key
        self.write_file.write_all(&key.to_le_bytes())?;

        // Write value
        self.write_file.write_all(value)?;

        // Flush to ensure durability
        self.write_file.flush()?;

        // Update file size
        self.file_size += total_len as u64;

        // Remap file only if we've exceeded the mapped region
        if self.file_size > self.mapped_size {
            self.remap_file()?;
        }

        // Store (key → offset) in ALEX
        // Offset points to start of key+value (after length header)
        let value_offset = current_offset + ENTRY_HEADER_SIZE as u64;
        self.alex.insert(key, value_offset.to_le_bytes().to_vec())?;

        Ok(())
    }

    /// Batch insert key-value pairs (optimized, with WAL logging)
    pub fn insert_batch(&mut self, entries: Vec<(i64, Vec<u8>)>) -> Result<()> {
        if entries.is_empty() {
            return Ok(());
        }

        // Log all entries to WAL first for durability
        for (key, value) in &entries {
            self.wal.log_insert(*key, value)?;
        }

        let mut alex_entries = Vec::with_capacity(entries.len());

        for (key, value) in entries {
            let data_len = 8 + value.len();
            let current_offset = self.file_size;

            // Write to file: [value_len:4][key:8][value:N]
            self.write_file.write_all(&(data_len as u32).to_le_bytes())?;
            self.write_file.write_all(&key.to_le_bytes())?;
            self.write_file.write_all(&value)?;

            // Track offset for ALEX
            let value_offset = current_offset + ENTRY_HEADER_SIZE as u64;
            alex_entries.push((key, value_offset.to_le_bytes().to_vec()));

            self.file_size += (ENTRY_HEADER_SIZE + data_len) as u64;
        }

        // Flush all writes
        self.write_file.flush()?;

        // Remap file only if we've exceeded the mapped region
        if self.file_size > self.mapped_size {
            self.remap_file()?;
        }

        // Bulk insert into ALEX
        self.alex.insert_batch(alex_entries)?;

        // Checkpoint if needed
        if self.wal.needs_checkpoint() {
            self.wal.checkpoint()?;
        }

        Ok(())
    }

    /// Delete key-value pair (with WAL logging for durability)
    ///
    /// Marks the key as deleted by setting its offset to TOMBSTONE.
    /// The actual space is not reclaimed until compaction.
    ///
    /// Performance: ~1,500-2,000ns (WAL write + ALEX update)
    pub fn delete(&mut self, key: i64) -> Result<()> {
        // Log to WAL first for durability
        self.wal.log_delete(key)?;

        // Mark as deleted in ALEX (set offset to TOMBSTONE)
        self.alex.insert(key, TOMBSTONE.to_le_bytes().to_vec())?;

        // Checkpoint if needed
        if self.wal.needs_checkpoint() {
            self.wal.checkpoint()?;
        }

        Ok(())
    }

    /// Query value by key (zero-copy - returns slice reference)
    ///
    /// Performance target: ~1,020 ns at 1M scale (vs 1,051 ns with Vec allocation)
    /// Returns a slice reference to avoid allocation overhead (~30ns saved)
    pub fn get(&self, key: i64) -> Result<Option<&[u8]>> {
        // Lookup offset in ALEX
        let offset_bytes = match self.alex.get(key)? {
            Some(bytes) => bytes,
            None => return Ok(None), // Key not found
        };

        // Decode offset
        let offset = u64::from_le_bytes(offset_bytes.as_slice().try_into()?);

        // Check for tombstone (deleted key)
        if offset == TOMBSTONE {
            return Ok(None);
        }

        // Read from mmap
        let mmap = match &self.mmap {
            Some(m) => m,
            None => return Ok(None),
        };

        // Read entry: [key:8][value:N]
        // We need to read the length from BEFORE the offset
        let len_offset = (offset as usize).saturating_sub(ENTRY_HEADER_SIZE);
        if len_offset + ENTRY_HEADER_SIZE > mmap.len() {
            return Ok(None);
        }

        let value_len_bytes = &mmap[len_offset..len_offset + 4];
        let data_len = u32::from_le_bytes(value_len_bytes.try_into()?) as usize;

        // Read key + value
        if offset as usize + data_len > mmap.len() {
            return Ok(None);
        }

        let data = &mmap[offset as usize..offset as usize + data_len];

        // Skip key (first 8 bytes), return value
        if data.len() < 8 {
            return Ok(None);
        }

        // Zero-copy: return slice reference instead of Vec
        Ok(Some(&data[8..]))
    }

    /// Query value by key (owned copy - returns Vec)
    ///
    /// Use this if you need an owned copy of the value.
    /// For most use cases, prefer get() which returns a slice reference.
    pub fn get_owned(&self, key: i64) -> Result<Option<Vec<u8>>> {
        Ok(self.get(key)?.map(|slice| slice.to_vec()))
    }

    /// Remap file after writes (grows in chunks to minimize remap frequency)
    fn remap_file(&mut self) -> Result<()> {
        if self.file_size == 0 {
            self.mmap = None;
            self.mapped_size = 0;
            return Ok(());
        }

        // Calculate new mapped size: round up to next MMAP_GROW_SIZE chunk
        let new_mapped_size = ((self.file_size + MMAP_GROW_SIZE - 1) / MMAP_GROW_SIZE) * MMAP_GROW_SIZE;

        // Grow file to new mapped size
        self.write_file.set_len(new_mapped_size)?;

        // Open read handle
        let read_file = File::open(&self.data_path)?;

        // Create new mmap for entire grown region
        let new_mmap = unsafe { Mmap::map(&read_file)? };

        self.mmap = Some(new_mmap);
        self.mapped_size = new_mapped_size;

        Ok(())
    }

    /// Get storage statistics
    pub fn stats(&self) -> StorageStats {
        StorageStats {
            num_keys: self.alex.len(),
            file_size: self.file_size,
            num_leaves: self.alex.num_leaves(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StorageStats {
    pub num_keys: usize,
    pub file_size: u64,
    pub num_leaves: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_basic_insert_and_get() {
        let dir = tempdir().unwrap();
        let mut storage = AlexStorage::new(dir.path()).unwrap();

        // Insert
        storage.insert(42, b"hello world").unwrap();

        // Query (zero-copy - returns slice)
        let result = storage.get(42).unwrap();
        assert_eq!(result, Some(b"hello world" as &[u8]));

        // Non-existent key
        let result = storage.get(99).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_batch_insert() {
        let dir = tempdir().unwrap();
        let mut storage = AlexStorage::new(dir.path()).unwrap();

        // Batch insert
        let entries = vec![
            (1, b"one".to_vec()),
            (2, b"two".to_vec()),
            (3, b"three".to_vec()),
        ];
        storage.insert_batch(entries).unwrap();

        // Query all (zero-copy - returns slices)
        assert_eq!(storage.get(1).unwrap(), Some(b"one" as &[u8]));
        assert_eq!(storage.get(2).unwrap(), Some(b"two" as &[u8]));
        assert_eq!(storage.get(3).unwrap(), Some(b"three" as &[u8]));
    }

    #[test]
    fn test_persistence() {
        let dir = tempdir().unwrap();

        // Insert data
        {
            let mut storage = AlexStorage::new(dir.path()).unwrap();
            storage.insert(100, b"persistent data").unwrap();
        }

        // Reopen and query (zero-copy - returns slice)
        {
            let storage = AlexStorage::new(dir.path()).unwrap();
            let result = storage.get(100).unwrap();
            assert_eq!(result, Some(b"persistent data" as &[u8]));
        }
    }

    #[test]
    fn test_delete_basic() {
        let dir = tempdir().unwrap();
        let mut storage = AlexStorage::new(dir.path()).unwrap();

        // Insert
        storage.insert(42, b"hello world").unwrap();
        assert_eq!(storage.get(42).unwrap(), Some(b"hello world" as &[u8]));

        // Delete
        storage.delete(42).unwrap();
        assert_eq!(storage.get(42).unwrap(), None);

        // Delete non-existent key (should not error)
        storage.delete(99).unwrap();
    }

    #[test]
    fn test_delete_and_reinsert() {
        let dir = tempdir().unwrap();
        let mut storage = AlexStorage::new(dir.path()).unwrap();

        // Insert
        storage.insert(42, b"hello world").unwrap();
        assert_eq!(storage.get(42).unwrap(), Some(b"hello world" as &[u8]));

        // Delete
        storage.delete(42).unwrap();
        assert_eq!(storage.get(42).unwrap(), None);

        // Reinsert same key
        storage.insert(42, b"new value").unwrap();
        assert_eq!(storage.get(42).unwrap(), Some(b"new value" as &[u8]));
    }

    #[test]
    fn test_delete_persistence() {
        let dir = tempdir().unwrap();

        // Insert and delete
        {
            let mut storage = AlexStorage::new(dir.path()).unwrap();
            storage.insert(100, b"to be deleted").unwrap();
            storage.delete(100).unwrap();
        }

        // Reopen and verify delete persisted
        {
            let storage = AlexStorage::new(dir.path()).unwrap();
            let result = storage.get(100).unwrap();
            assert_eq!(result, None);
        }
    }

    #[test]
    fn test_delete_multiple() {
        let dir = tempdir().unwrap();
        let mut storage = AlexStorage::new(dir.path()).unwrap();

        // Insert multiple
        for i in 0..10 {
            storage.insert(i, format!("value_{}", i).as_bytes()).unwrap();
        }

        // Delete some
        storage.delete(2).unwrap();
        storage.delete(5).unwrap();
        storage.delete(8).unwrap();

        // Verify deletes
        assert_eq!(storage.get(0).unwrap().is_some(), true);
        assert_eq!(storage.get(1).unwrap().is_some(), true);
        assert_eq!(storage.get(2).unwrap(), None); // Deleted
        assert_eq!(storage.get(3).unwrap().is_some(), true);
        assert_eq!(storage.get(4).unwrap().is_some(), true);
        assert_eq!(storage.get(5).unwrap(), None); // Deleted
        assert_eq!(storage.get(6).unwrap().is_some(), true);
        assert_eq!(storage.get(7).unwrap().is_some(), true);
        assert_eq!(storage.get(8).unwrap(), None); // Deleted
        assert_eq!(storage.get(9).unwrap().is_some(), true);
    }
}
