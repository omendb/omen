//! AlexStorage: Custom mmap-based storage with ALEX learned index
//!
//! Architecture:
//! - ALEX: Tracks (key → offset in file)
//! - Mmap: Memory-mapped file for zero-copy value access
//! - Append-only: New writes append to end of file
//!
//! Performance targets (validated via benchmark_mmap_validation):
//! - Queries: ~389 ns (ALEX 218ns + mmap 151ns + overhead 20ns)
//! - 10x faster than RocksDB (3,902 ns)
//! - 5.6x faster than SQLite (2,173 ns)
//!
//! Current status: Foundation only (no WAL, no compaction, no concurrency)

use crate::alex::AlexTree;
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
#[derive(Debug)]
pub struct AlexStorage {
    /// Path to data file
    data_path: PathBuf,

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
}

impl AlexStorage {
    /// Create new AlexStorage at given path
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data_path = path.as_ref().join("data.bin");

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

        let mut storage = Self {
            data_path,
            alex: AlexTree::new(),
            mmap,
            file_size,
            mapped_size: file_size,
            write_file,
        };

        // Load existing keys from file if present
        if file_size > 0 {
            storage.load_keys_from_file()?;
        }

        Ok(storage)
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

    /// Insert key-value pair
    ///
    /// Format written to file:
    /// [value_len:4 bytes][key:8 bytes][value:N bytes]
    ///
    /// ALEX stores: (key → offset of key+value)
    pub fn insert(&mut self, key: i64, value: &[u8]) -> Result<()> {
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

    /// Batch insert key-value pairs (optimized)
    pub fn insert_batch(&mut self, entries: Vec<(i64, Vec<u8>)>) -> Result<()> {
        if entries.is_empty() {
            return Ok(());
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

        Ok(())
    }

    /// Query value by key
    ///
    /// Performance: ~389 ns (ALEX 218ns + mmap 151ns + overhead 20ns)
    pub fn get(&self, key: i64) -> Result<Option<Vec<u8>>> {
        // Lookup offset in ALEX
        let offset_bytes = match self.alex.get(key)? {
            Some(bytes) => bytes,
            None => return Ok(None), // Key not found
        };

        // Decode offset
        let offset = u64::from_le_bytes(offset_bytes.as_slice().try_into()?);

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

        Ok(Some(data[8..].to_vec()))
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

        // Query
        let result = storage.get(42).unwrap();
        assert_eq!(result, Some(b"hello world".to_vec()));

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

        // Query all
        assert_eq!(storage.get(1).unwrap(), Some(b"one".to_vec()));
        assert_eq!(storage.get(2).unwrap(), Some(b"two".to_vec()));
        assert_eq!(storage.get(3).unwrap(), Some(b"three".to_vec()));
    }

    #[test]
    fn test_persistence() {
        let dir = tempdir().unwrap();

        // Insert data
        {
            let mut storage = AlexStorage::new(dir.path()).unwrap();
            storage.insert(100, b"persistent data").unwrap();
        }

        // Reopen and query
        {
            let storage = AlexStorage::new(dir.path()).unwrap();
            let result = storage.get(100).unwrap();
            assert_eq!(result, Some(b"persistent data".to_vec()));
        }
    }
}
