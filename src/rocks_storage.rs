//! RocksDB-based transactional storage with ALEX learned index integration
//!
//! Architecture:
//! - RocksDB: Fast LSM-tree storage (proven at Facebook scale)
//! - ALEX: Adaptive learned index for key tracking
//!
//! This replaces redb (too slow) as interim solution before custom storage.
//! Expected performance: SQLite parity or better (1-2x faster)

use crate::alex::AlexTree;
use crate::metrics;
use anyhow::Result;
use lru::LruCache;
use rocksdb::{DB, WriteBatch, Options, IteratorMode};
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, instrument, warn};

/// Marker byte for ALEX key tracking (1 byte = minimal overhead)
const KEY_EXISTS_MARKER: &[u8] = &[1];

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StorageMetadata {
    row_count: u64,
    schema_version: u32,
}

/// Transactional storage with ALEX learned index + RocksDB
///
/// Architecture:
/// - ALEX: Tracks which keys exist (key → marker byte)
/// - RocksDB: Stores actual data (key → value)
///
/// Benefits over redb:
/// - RocksDB LSM-tree optimized for writes
/// - Battle-tested at Facebook/CockroachDB/TiDB scale
/// - Expected 5-10x faster than redb
///
/// Benefits of keeping ALEX:
/// - O(1) key existence checks
/// - No need to query RocksDB for missing keys
/// - Query optimization layer above storage
#[derive(Debug)]
pub struct RocksStorage {
    db: Arc<DB>,
    /// ALEX learned index tracks key existence
    alex: AlexTree,
    row_count: u64,
    /// LRU cache for hot values (capacity: 1000 entries)
    value_cache: LruCache<i64, Vec<u8>>,
}

impl RocksStorage {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);

        // TUNING: Larger memtable to batch more writes in memory before flush
        // Reduces compaction overhead for random writes
        opts.set_write_buffer_size(256 * 1024 * 1024); // 64MB → 256MB
        opts.set_max_write_buffer_number(3);

        // TUNING: Larger SST files reduce compaction frequency
        opts.set_target_file_size_base(128 * 1024 * 1024); // 64MB → 128MB

        // TUNING: Delay compaction to batch more writes
        // Higher trigger = fewer compactions = less write amplification
        opts.set_level_zero_file_num_compaction_trigger(8); // Default: 4 → 8

        // TUNING: Larger level base reduces total levels
        opts.set_max_bytes_for_level_base(512 * 1024 * 1024); // 512MB

        // TUNING: Fast compression for upper levels, strong compression for cold data
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        opts.set_bottommost_compression_type(rocksdb::DBCompressionType::Zstd);

        // Optimize for sequential writes (our common pattern)
        opts.set_level_compaction_dynamic_level_bytes(true);
        opts.set_bytes_per_sync(1024 * 1024); // 1MB

        let db = DB::open(&opts, path)?;

        let mut storage = Self {
            db: Arc::new(db),
            alex: AlexTree::new(),
            row_count: 0,
            value_cache: LruCache::new(NonZeroUsize::new(1000).unwrap()),
        };

        storage.load_metadata()?;
        storage.load_keys_from_disk()?;

        Ok(storage)
    }

    fn load_metadata(&mut self) -> Result<()> {
        if let Some(metadata_bytes) = self.db.get(b"__metadata__")? {
            let metadata: StorageMetadata = bincode::deserialize(&metadata_bytes)?;
            self.row_count = metadata.row_count;
        }
        Ok(())
    }

    pub fn save_metadata(&self) -> Result<()> {
        let metadata = StorageMetadata {
            row_count: self.row_count,
            schema_version: 1,
        };

        let metadata_bytes = bincode::serialize(&metadata)?;
        self.db.put(b"__metadata__", metadata_bytes)?;

        Ok(())
    }

    /// Load keys from disk into ALEX (initialization only)
    fn load_keys_from_disk(&mut self) -> Result<()> {
        let mut keys = Vec::new();

        let iter = self.db.iterator(IteratorMode::Start);
        for item in iter {
            let (key_bytes, _) = item?;

            // Skip metadata key
            if key_bytes.as_ref() == b"__metadata__" {
                continue;
            }

            // Decode i64 key
            if key_bytes.len() == 8 {
                let key_array: [u8; 8] = key_bytes[..8].try_into()?;
                let key = i64::from_be_bytes(key_array);
                keys.push(key);
            }
        }

        keys.sort_unstable();

        // Populate ALEX with existing keys (batch mode for fast rebuild)
        let alex_entries: Vec<(i64, Vec<u8>)> = keys
            .into_iter()
            .map(|k| (k, KEY_EXISTS_MARKER.to_vec()))
            .collect();

        self.alex.insert_batch(alex_entries)?;

        // Update metrics
        metrics::set_learned_index_size(self.alex.len());
        metrics::set_learned_index_models(self.alex.num_leaves());

        Ok(())
    }

    #[instrument(skip(self, value), fields(key = key, value_size = value.len()))]
    pub fn insert(&mut self, key: i64, value: &[u8]) -> Result<()> {
        debug!("Insert started");
        let start_time = Instant::now();

        // Insert into RocksDB
        let key_bytes = key.to_be_bytes();
        self.db.put(&key_bytes, value)?;

        // Track key in ALEX (only if new)
        if self.alex.get(key)?.is_none() {
            self.alex.insert(key, KEY_EXISTS_MARKER.to_vec())?;
            self.row_count += 1;

            // Update metrics
            metrics::set_learned_index_size(self.alex.len());
            metrics::set_learned_index_models(self.alex.num_leaves());
        }

        if self.row_count % 1000 == 0 {
            self.save_metadata()?;
        }

        metrics::record_insert(start_time.elapsed().as_secs_f64());

        let duration = start_time.elapsed();
        debug!(duration_ms = duration.as_millis(), "Insert completed");

        Ok(())
    }

    #[instrument(skip(self, entries), fields(batch_size = entries.len()))]
    pub fn insert_batch(&mut self, entries: Vec<(i64, Vec<u8>)>) -> Result<()> {
        if entries.is_empty() {
            return Ok(());
        }

        info!(batch_size = entries.len(), "Batch insert started");
        let start_time = Instant::now();
        let batch_size = entries.len();

        // Single atomic write batch to RocksDB
        let mut batch = WriteBatch::default();
        for (key, value) in &entries {
            let key_bytes = key.to_be_bytes();
            batch.put(&key_bytes, value);
        }
        self.db.write(batch)?;

        // Track all keys in ALEX (batch mode for 10-50x speedup)
        // Filter out existing keys first to avoid duplicate inserts
        let mut new_entries: Vec<(i64, Vec<u8>)> = Vec::new();
        for (key, _) in &entries {
            if self.alex.get(*key)?.is_none() {
                new_entries.push((*key, KEY_EXISTS_MARKER.to_vec()));
            }
        }

        let new_keys = new_entries.len();

        // Batch insert into ALEX (amortizes overhead across all keys)
        if !new_entries.is_empty() {
            self.alex.insert_batch(new_entries)?;
        }

        self.row_count += new_keys as u64;

        // Invalidate cache
        self.value_cache.clear();

        self.save_metadata()?;

        // Update metrics
        metrics::set_learned_index_size(self.alex.len());
        metrics::set_learned_index_models(self.alex.num_leaves());

        let duration = start_time.elapsed().as_secs_f64();

        for _ in 0..batch_size {
            metrics::record_insert(duration / batch_size as f64);
        }

        info!(
            batch_size = batch_size,
            duration_ms = start_time.elapsed().as_millis(),
            throughput_per_sec = (batch_size as f64 / duration).round() as u64,
            "Batch insert completed"
        );

        Ok(())
    }

    #[instrument(skip(self), fields(key = key))]
    pub fn point_query(&mut self, key: i64) -> Result<Option<Vec<u8>>> {
        debug!("Point query started");
        let start_time = Instant::now();

        // Check LRU cache first
        if let Some(cached_value) = self.value_cache.get(&key) {
            metrics::record_query_path("cache_hit");
            metrics::record_search(start_time.elapsed().as_secs_f64());
            return Ok(Some(cached_value.clone()));
        }

        // Check if key exists in ALEX
        if self.alex.get(key)?.is_none() {
            // Key doesn't exist
            metrics::record_query_path("learned_index_miss");
            metrics::record_search(start_time.elapsed().as_secs_f64());
            return Ok(None);
        }

        // Key exists - look up value in RocksDB
        metrics::record_query_path("learned_index");

        let key_bytes = key.to_be_bytes();
        let result = if let Some(value_bytes) = self.db.get(&key_bytes)? {
            let value = value_bytes.to_vec();
            self.value_cache.put(key, value.clone());
            Ok(Some(value))
        } else {
            // Inconsistency: ALEX says key exists but RocksDB doesn't have it
            warn!(key = key, "Inconsistency: ALEX has key but RocksDB doesn't");
            Ok(None)
        };

        metrics::record_search(start_time.elapsed().as_secs_f64());

        let duration = start_time.elapsed();
        debug!(duration_ms = duration.as_millis(), "Point query completed");

        result
    }

    #[instrument(skip(self), fields(start_key = start_key, end_key = end_key))]
    pub fn range_query(&mut self, start_key: i64, end_key: i64) -> Result<Vec<(i64, Vec<u8>)>> {
        debug!("Range query started");
        let start_time = Instant::now();

        // Get keys in range from ALEX
        let keys_in_range = self.alex.range(start_key, end_key)?;

        if !keys_in_range.is_empty() {
            metrics::record_query_path("learned_index");

            // Batch lookup values from RocksDB
            let mut results = Vec::with_capacity(keys_in_range.len());
            for (key, _) in keys_in_range {
                let key_bytes = key.to_be_bytes();
                if let Some(value_bytes) = self.db.get(&key_bytes)? {
                    results.push((key, value_bytes.to_vec()));
                }
            }

            metrics::record_range_query(start_time.elapsed().as_secs_f64(), results.len());

            return Ok(results);
        }

        // Fallback: scan RocksDB directly if ALEX is empty
        metrics::record_query_path("full_scan");

        let mut results = Vec::new();
        let iter = self.db.iterator(IteratorMode::Start);

        for item in iter {
            let (key_bytes, value_bytes) = item?;

            // Skip metadata
            if key_bytes.as_ref() == b"__metadata__" {
                continue;
            }

            if key_bytes.len() == 8 {
                let key_array: [u8; 8] = key_bytes[..8].try_into()?;
                let key = i64::from_be_bytes(key_array);

                if key >= start_key && key <= end_key {
                    results.push((key, value_bytes.to_vec()));
                }

                if key > end_key {
                    break;
                }
            }
        }

        metrics::record_range_query(start_time.elapsed().as_secs_f64(), results.len());

        let duration = start_time.elapsed();
        debug!(
            duration_ms = duration.as_millis(),
            rows_returned = results.len(),
            "Range query completed"
        );

        Ok(results)
    }

    pub fn scan_all(&mut self) -> Result<Vec<(i64, Vec<u8>)>> {
        let mut results = Vec::new();
        let iter = self.db.iterator(IteratorMode::Start);

        for item in iter {
            let (key_bytes, value_bytes) = item?;

            // Skip metadata
            if key_bytes.as_ref() == b"__metadata__" {
                continue;
            }

            if key_bytes.len() == 8 {
                let key_array: [u8; 8] = key_bytes[..8].try_into()?;
                let key = i64::from_be_bytes(key_array);
                results.push((key, value_bytes.to_vec()));
            }
        }

        Ok(results)
    }

    #[instrument(skip(self), fields(key = key))]
    pub fn delete(&mut self, key: i64) -> Result<bool> {
        debug!("Delete started");
        let start_time = Instant::now();

        // Delete from RocksDB
        let key_bytes = key.to_be_bytes();
        let existed = self.db.get(&key_bytes)?.is_some();
        self.db.delete(&key_bytes)?;

        if existed {
            // Note: We don't remove from ALEX for simplicity
            // ALEX will return true for key existence, but RocksDB lookup will return None
            // This is acceptable - ALEX is an optimization, not source of truth
            self.row_count = self.row_count.saturating_sub(1);
            self.save_metadata()?;
        }

        metrics::record_delete(start_time.elapsed().as_secs_f64());

        let duration = start_time.elapsed();
        debug!(duration_ms = duration.as_millis(), deleted = existed, "Delete completed");

        Ok(existed)
    }

    pub fn count(&self) -> u64 {
        self.row_count
    }

    pub fn learned_index_size(&self) -> usize {
        self.alex.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_insert_and_point_query() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let mut storage = RocksStorage::new(&db_path).unwrap();

        // Insert
        storage.insert(100, b"value100").unwrap();
        storage.insert(200, b"value200").unwrap();

        // Point query
        assert_eq!(storage.point_query(100).unwrap(), Some(b"value100".to_vec()));
        assert_eq!(storage.point_query(200).unwrap(), Some(b"value200".to_vec()));
        assert_eq!(storage.point_query(999).unwrap(), None);
    }

    #[test]
    fn test_batch_insert() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let mut storage = RocksStorage::new(&db_path).unwrap();

        // Batch insert
        let entries = vec![
            (1, b"one".to_vec()),
            (2, b"two".to_vec()),
            (3, b"three".to_vec()),
        ];
        storage.insert_batch(entries).unwrap();

        // Verify
        assert_eq!(storage.point_query(1).unwrap(), Some(b"one".to_vec()));
        assert_eq!(storage.point_query(2).unwrap(), Some(b"two".to_vec()));
        assert_eq!(storage.point_query(3).unwrap(), Some(b"three".to_vec()));
        assert_eq!(storage.count(), 3);
    }

    #[test]
    fn test_range_query() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let mut storage = RocksStorage::new(&db_path).unwrap();

        // Insert keys 10, 20, 30, 40, 50
        for i in 1..=5 {
            storage.insert(i * 10, format!("value{}", i * 10).as_bytes()).unwrap();
        }

        // Range query [20, 40]
        let results = storage.range_query(20, 40).unwrap();
        assert_eq!(results.len(), 3); // 20, 30, 40

        assert_eq!(results[0].0, 20);
        assert_eq!(results[1].0, 30);
        assert_eq!(results[2].0, 40);
    }

    #[test]
    fn test_scan_all() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let mut storage = RocksStorage::new(&db_path).unwrap();

        // Insert
        storage.insert(1, b"one").unwrap();
        storage.insert(2, b"two").unwrap();
        storage.insert(3, b"three").unwrap();

        // Scan all
        let results = storage.scan_all().unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_delete() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let mut storage = RocksStorage::new(&db_path).unwrap();

        // Insert and delete
        storage.insert(100, b"value100").unwrap();
        assert_eq!(storage.count(), 1);

        let deleted = storage.delete(100).unwrap();
        assert!(deleted);
        assert_eq!(storage.count(), 0);

        // Verify deleted
        assert_eq!(storage.point_query(100).unwrap(), None);
    }

    #[test]
    fn test_learned_index_integration() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let mut storage = RocksStorage::new(&db_path).unwrap();

        // Insert 1000 keys
        for i in 0..1000 {
            storage.insert(i, format!("value{}", i).as_bytes()).unwrap();
        }

        // Verify ALEX has all keys
        assert_eq!(storage.learned_index_size(), 1000);

        // Point queries should all succeed
        for i in 0..1000 {
            let result = storage.point_query(i).unwrap();
            assert!(result.is_some());
        }
    }

    #[test]
    fn test_persistence() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");

        // Insert and close
        {
            let mut storage = RocksStorage::new(&db_path).unwrap();
            storage.insert(100, b"value100").unwrap();
            storage.insert(200, b"value200").unwrap();
            storage.save_metadata().unwrap();
        }

        // Reopen and verify
        {
            let mut storage = RocksStorage::new(&db_path).unwrap();
            assert_eq!(storage.point_query(100).unwrap(), Some(b"value100".to_vec()));
            assert_eq!(storage.point_query(200).unwrap(), Some(b"value200".to_vec()));
            assert_eq!(storage.count(), 2);
            assert_eq!(storage.learned_index_size(), 2);
        }
    }

    #[test]
    fn test_batch_insert_performance() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let mut storage = RocksStorage::new(&db_path).unwrap();

        // Insert 10K keys in batch
        let entries: Vec<(i64, Vec<u8>)> = (0..10000)
            .map(|i| (i, format!("value{}", i).into_bytes()))
            .collect();

        let start = Instant::now();
        storage.insert_batch(entries).unwrap();
        let elapsed = start.elapsed();

        println!("10K batch insert took {:?}", elapsed);
        assert_eq!(storage.count(), 10000);
    }
}
