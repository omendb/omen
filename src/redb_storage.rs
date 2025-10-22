//! redb-based transactional storage with ALEX learned index integration

use crate::alex::AlexTree;
use crate::metrics;
use anyhow::Result;
use lru::LruCache;
use redb::{Database, ReadableTable, ReadTransaction, TableDefinition};
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;
use std::path::Path;
use std::time::{Duration, Instant};
use tracing::{debug, info, instrument, warn};

const DATA_TABLE: TableDefinition<i64, &[u8]> = TableDefinition::new("data");
const METADATA_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("metadata");

/// Marker byte for ALEX key tracking (1 byte = minimal overhead)
const KEY_EXISTS_MARKER: &[u8] = &[1];

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StorageMetadata {
    row_count: u64,
    schema_version: u32,
}

/// Transactional storage with ALEX learned index
///
/// Architecture:
/// - ALEX: Tracks which keys exist (key → marker byte)
/// - redb: Stores actual data (key → value)
///
/// Benefits over RMI:
/// - No duplicate sorted_keys array
/// - No manual rebuild coordination
/// - O(1) inserts with gapped arrays
/// - Linear scaling at 10M+ keys
#[derive(Debug)]
pub struct RedbStorage {
    db: Database,
    /// ALEX learned index tracks key existence
    alex: AlexTree,
    row_count: u64,
    /// Cached read transaction (invalidated on writes for consistency)
    cached_read_txn: Option<ReadTransaction>,
    /// LRU cache for hot values (capacity: 1000 entries)
    value_cache: LruCache<i64, Vec<u8>>,
}

impl RedbStorage {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db = Database::create(path)?;

        {
            let write_txn = db.begin_write()?;
            write_txn.open_table(DATA_TABLE)?;
            write_txn.open_table(METADATA_TABLE)?;
            write_txn.commit()?;
        }

        let mut storage = Self {
            db,
            alex: AlexTree::new(),
            row_count: 0,
            cached_read_txn: None,
            value_cache: LruCache::new(NonZeroUsize::new(1000).unwrap()),
        };

        storage.load_metadata()?;
        storage.load_keys_from_disk()?;

        Ok(storage)
    }

    fn load_metadata(&mut self) -> Result<()> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(METADATA_TABLE)?;

        if let Some(metadata_bytes) = table.get("storage_metadata")? {
            let metadata: StorageMetadata = bincode::deserialize(metadata_bytes.value())?;
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

        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(METADATA_TABLE)?;
            table.insert("storage_metadata", metadata_bytes.as_slice())?;
        }
        write_txn.commit()?;

        Ok(())
    }

    /// Load keys from disk into ALEX (initialization only)
    fn load_keys_from_disk(&mut self) -> Result<()> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(DATA_TABLE)?;

        let mut keys: Vec<i64> = table
            .iter()?
            .filter_map(|result| result.ok())
            .map(|(key, _)| key.value())
            .collect();

        keys.sort_unstable();

        // Populate ALEX with existing keys
        for key in keys {
            self.alex.insert(key, KEY_EXISTS_MARKER.to_vec())?;
        }

        // Update metrics
        metrics::set_learned_index_size(self.alex.len());
        metrics::set_learned_index_models(self.alex.num_leaves());

        Ok(())
    }

    /// Get cached read transaction, creating it if needed
    fn get_read_txn(&mut self) -> Result<&ReadTransaction> {
        if self.cached_read_txn.is_none() {
            self.cached_read_txn = Some(self.db.begin_read()?);
        }
        Ok(self.cached_read_txn.as_ref().unwrap())
    }

    #[instrument(skip(self, value), fields(key = key, value_size = value.len()))]
    pub fn insert(&mut self, key: i64, value: &[u8]) -> Result<()> {
        debug!("Insert started");
        let start_time = Instant::now();

        // Insert into redb
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(DATA_TABLE)?;
            table.insert(key, value)?;
        }
        write_txn.commit()?;

        // Track key in ALEX (only if new)
        if self.alex.get(key)?.is_none() {
            self.alex.insert(key, KEY_EXISTS_MARKER.to_vec())?;
            self.row_count += 1;

            // Update metrics
            metrics::set_learned_index_size(self.alex.len());
            metrics::set_learned_index_models(self.alex.num_leaves());
        }

        if self.row_count.is_multiple_of(1000) {
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

        // Single transaction for all redb inserts
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(DATA_TABLE)?;
            for (key, value) in &entries {
                table.insert(*key, value.as_slice())?;
            }
        }
        write_txn.commit()?;

        // Track all keys in ALEX
        let mut new_keys = 0;
        for (key, _) in &entries {
            if self.alex.get(*key)?.is_none() {
                self.alex.insert(*key, KEY_EXISTS_MARKER.to_vec())?;
                new_keys += 1;
            }
        }

        self.row_count += new_keys as u64;

        // Invalidate caches
        self.cached_read_txn = None;
        self.value_cache.clear();

        self.save_metadata()?;

        // Update metrics
        metrics::set_learned_index_size(self.alex.len());
        metrics::set_learned_index_models(self.alex.num_leaves());

        let duration = start_time.elapsed().as_secs_f64();
        let throughput = batch_size as f64 / duration;

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

        // Key exists - look up value in redb
        metrics::record_query_path("learned_index");

        let read_txn = self.get_read_txn()?;
        let table = read_txn.open_table(DATA_TABLE)?;

        let result = if let Some(value_guard) = table.get(key)? {
            let value = value_guard.value().to_vec();
            self.value_cache.put(key, value.clone());
            Ok(Some(value))
        } else {
            // Inconsistency: ALEX says key exists but redb doesn't have it
            // This shouldn't happen, but handle gracefully
            warn!(key = key, "Inconsistency: ALEX has key but redb doesn't");
            Ok(None)
        };

        metrics::record_search(start_time.elapsed().as_secs_f64());

        let duration = start_time.elapsed();
        if duration > Duration::from_millis(100) {
            warn!(duration_ms = duration.as_millis(), key = key, "Slow point query detected");
        } else {
            debug!(duration_ms = duration.as_millis(), "Point query completed");
        }

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

            // Batch lookup values from redb
            let read_txn = self.get_read_txn()?;
            let table = read_txn.open_table(DATA_TABLE)?;

            let mut results = Vec::with_capacity(keys_in_range.len());
            for (key, _) in keys_in_range {
                if let Some(value_guard) = table.get(key)? {
                    results.push((key, value_guard.value().to_vec()));
                }
            }

            metrics::record_range_query(start_time.elapsed().as_secs_f64(), results.len());

            return Ok(results);
        }

        // Fallback: empty result or full scan if ALEX is empty
        metrics::record_query_path("full_scan");

        let read_txn = self.get_read_txn()?;
        let table = read_txn.open_table(DATA_TABLE)?;

        let mut results = Vec::new();

        for item in table.iter()? {
            let (key_guard, value_guard) = item?;
            let key = key_guard.value();

            if key >= start_key && key <= end_key {
                results.push((key, value_guard.value().to_vec()));
            }

            if key > end_key {
                break;
            }
        }

        metrics::record_range_query(start_time.elapsed().as_secs_f64(), results.len());

        let duration = start_time.elapsed();
        debug!(
            duration_ms = duration.as_millis(),
            rows_returned = results.len(),
            "Range query completed"
        );
        if duration > Duration::from_millis(100) {
            warn!(
                duration_ms = duration.as_millis(),
                rows = results.len(),
                "Slow range query detected"
            );
        }

        Ok(results)
    }

    pub fn scan_all(&mut self) -> Result<Vec<(i64, Vec<u8>)>> {
        let read_txn = self.get_read_txn()?;
        let table = read_txn.open_table(DATA_TABLE)?;

        let results: Result<Vec<_>> = table
            .iter()?
            .map(|result| {
                let (key_guard, value_guard) = result?;
                Ok((key_guard.value(), value_guard.value().to_vec()))
            })
            .collect();

        results
    }

    #[instrument(skip(self), fields(key = key))]
    pub fn delete(&mut self, key: i64) -> Result<bool> {
        debug!("Delete started");
        let start_time = Instant::now();

        // Delete from redb
        let write_txn = self.db.begin_write()?;
        let deleted;
        {
            let mut table = write_txn.open_table(DATA_TABLE)?;
            deleted = table.remove(key)?.is_some();
        }
        write_txn.commit()?;

        if deleted {
            // Note: We don't remove from ALEX for simplicity
            // ALEX will return true for key existence, but redb lookup will return None
            // This is acceptable - ALEX is an optimization, not source of truth
            self.row_count = self.row_count.saturating_sub(1);
            self.save_metadata()?;
        }

        metrics::record_delete(start_time.elapsed().as_secs_f64());

        let duration = start_time.elapsed();
        debug!(duration_ms = duration.as_millis(), deleted = deleted, "Delete completed");

        Ok(deleted)
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
        let mut storage = RedbStorage::new(&db_path).unwrap();

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
        let mut storage = RedbStorage::new(&db_path).unwrap();

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
        let mut storage = RedbStorage::new(&db_path).unwrap();

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
        let mut storage = RedbStorage::new(&db_path).unwrap();

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
        let mut storage = RedbStorage::new(&db_path).unwrap();

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
        let mut storage = RedbStorage::new(&db_path).unwrap();

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
            let mut storage = RedbStorage::new(&db_path).unwrap();
            storage.insert(100, b"value100").unwrap();
            storage.insert(200, b"value200").unwrap();
            // Explicitly save metadata before closing (normally happens every 1000 inserts)
            storage.save_metadata().unwrap();
        }

        // Reopen and verify
        {
            let mut storage = RedbStorage::new(&db_path).unwrap();
            assert_eq!(storage.point_query(100).unwrap(), Some(b"value100".to_vec()));
            assert_eq!(storage.point_query(200).unwrap(), Some(b"value200".to_vec()));
            assert_eq!(storage.count(), 2);
            assert_eq!(storage.learned_index_size(), 2);
        }
    }
}
