//! redb-based transactional storage with learned index integration

use crate::index::RecursiveModelIndex;
use crate::metrics;
use crate::row::Row;
use crate::value::Value;
use anyhow::{anyhow, Result};
use redb::{Database, ReadableTable, ReadTransaction, TableDefinition};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, instrument, warn};

const DATA_TABLE: TableDefinition<i64, &[u8]> = TableDefinition::new("data");
const METADATA_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("metadata");

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StorageMetadata {
    row_count: u64,
    schema_version: u32,
}

#[derive(Debug)]
pub struct RedbStorage {
    db: Database,
    learned_index: RecursiveModelIndex,
    row_count: u64,
    /// Sorted array of keys for position-based learned index lookups
    /// Learned index predicts position in this array, then we binary search
    sorted_keys: Vec<i64>,
    /// Flag to track if index needs rebuild (lazy rebuild optimization)
    index_dirty: bool,
    /// Cached error bound (updated after index rebuild)
    cached_error_bound: usize,
    /// Cached read transaction (invalidated on writes for consistency)
    cached_read_txn: Option<ReadTransaction>,
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

        let learned_index = RecursiveModelIndex::new(1_000_000);

        let mut storage = Self {
            db,
            learned_index,
            row_count: 0,
            sorted_keys: Vec::new(),
            index_dirty: false,
            cached_error_bound: 100, // Default
            cached_read_txn: None,   // Will be created on first query
        };

        storage.load_metadata()?;
        storage.rebuild_index()?;

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

    fn save_metadata(&self) -> Result<()> {
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

    #[instrument(skip(self))]
    fn rebuild_index(&mut self) -> Result<()> {
        info!("Index rebuild started");
        let start_time = Instant::now();

        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(DATA_TABLE)?;

        let mut keys: Vec<i64> = table
            .iter()?
            .filter_map(|result| result.ok())
            .map(|(key, _)| key.value())
            .collect();

        // Sort keys for position-based lookup
        keys.sort_unstable();
        self.sorted_keys = keys;

        if !self.sorted_keys.is_empty() {
            // Train learned index with (key, position) pairs
            let data: Vec<(i64, usize)> = self
                .sorted_keys
                .iter()
                .enumerate()
                .map(|(pos, &key)| (key, pos))
                .collect();
            self.learned_index.train(data);

            // Cache error bound to avoid recomputing on every query
            self.cached_error_bound = self.learned_index.max_error_bound();

            // Update learned index size metrics
            metrics::set_learned_index_size(self.sorted_keys.len());
            metrics::set_learned_index_models(self.learned_index.model_count());
        } else {
            self.cached_error_bound = 100; // Default for empty index
            metrics::set_learned_index_size(0);
            metrics::set_learned_index_models(0);
        }

        let duration = start_time.elapsed();
        info!(
            duration_ms = duration.as_millis(),
            keys = self.sorted_keys.len(),
            models = if self.sorted_keys.is_empty() { 0 } else { self.learned_index.model_count() },
            "Index rebuild completed"
        );

        Ok(())
    }

    /// Ensure index is up-to-date before queries (lazy rebuild optimization)
    #[instrument(skip(self))]
    fn ensure_index_fresh(&mut self) -> Result<()> {
        if self.index_dirty {
            debug!("Index dirty, triggering lazy rebuild");
            // Invalidate cached read transaction before rebuild
            self.cached_read_txn = None;
            self.rebuild_index()?;
            self.index_dirty = false;
            // Create fresh read transaction after rebuild
            self.cached_read_txn = Some(self.db.begin_read()?);
        }
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

        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(DATA_TABLE)?;
            table.insert(key, value)?;
        }
        write_txn.commit()?;

        // Maintain sorted_keys array for position-based lookup
        match self.sorted_keys.binary_search(&key) {
            Ok(_) => {
                // Key already exists, no need to insert again
            }
            Err(pos) => {
                // Insert key at correct sorted position
                self.sorted_keys.insert(pos, key);
                self.learned_index.add_key(key);
                self.row_count += 1;
            }
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

        // Single transaction for all inserts (MUCH faster)
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(DATA_TABLE)?;
            for (key, value) in &entries {
                table.insert(*key, value.as_slice())?;
            }
        }
        write_txn.commit()?;

        // Mark index as dirty - DON'T rebuild immediately (lazy rebuild optimization)
        // Index will rebuild on next query for 10-100x insert speedup
        self.index_dirty = true;
        // Invalidate cached read transaction (will see stale data after write)
        self.cached_read_txn = None;
        self.row_count += batch_size as u64;
        self.save_metadata()?;

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

        // Lazy rebuild: ensure index is fresh before querying
        self.ensure_index_fresh()?;

        // Use learned index to predict position in sorted_keys array
        let result = if !self.sorted_keys.is_empty() {
            if let Some(predicted_pos) = self.learned_index.search(key) {
                // Use cached error bound (computed once during rebuild)
                let window_size = self.cached_error_bound;
                let start = predicted_pos
                    .saturating_sub(window_size)
                    .min(self.sorted_keys.len());
                let end = (predicted_pos + window_size).min(self.sorted_keys.len());

                // Binary search in the predicted window
                if let Ok(pos) = self.sorted_keys[start..end].binary_search(&key) {
                    let actual_pos = start + pos;

                    // Record learned index hit with prediction accuracy
                    metrics::record_learned_index_hit(predicted_pos, actual_pos);
                    metrics::record_query_path("learned_index");

                    // Found the key, now look up value in redb using cached transaction
                    let read_txn = self.get_read_txn()?;
                    let table = read_txn.open_table(DATA_TABLE)?;

                    if let Some(value_guard) = table.get(key)? {
                        let result = Ok(Some(value_guard.value().to_vec()));

                        // Record successful search
                        metrics::record_search(start_time.elapsed().as_secs_f64());

                        return result;
                    }
                }

                // Key not found in predicted window
                metrics::record_learned_index_miss();
                metrics::record_query_path("fallback_btree");

                metrics::record_search(start_time.elapsed().as_secs_f64());
                Ok(None)
            } else {
                // Learned index returned None (shouldn't happen)
                metrics::record_learned_index_miss();
                metrics::record_query_path("fallback_btree");

                // Fallback to direct lookup using cached transaction
                let read_txn = self.get_read_txn()?;
                let table = read_txn.open_table(DATA_TABLE)?;

                let result = if let Some(value_guard) = table.get(key)? {
                    Ok(Some(value_guard.value().to_vec()))
                } else {
                    Ok(None)
                };

                metrics::record_search(start_time.elapsed().as_secs_f64());
                result
            }
        } else {
            // No index available, direct lookup using cached transaction
            metrics::record_query_path("fallback_btree");

            let read_txn = self.get_read_txn()?;
            let table = read_txn.open_table(DATA_TABLE)?;

            let result = if let Some(value_guard) = table.get(key)? {
                Ok(Some(value_guard.value().to_vec()))
            } else {
                Ok(None)
            };

            metrics::record_search(start_time.elapsed().as_secs_f64());
            result
        };

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

        // Lazy rebuild: ensure index is fresh before querying
        self.ensure_index_fresh()?;

        // Use learned index to find range positions
        if !self.sorted_keys.is_empty() {
            let positions = self.learned_index.range_search(start_key, end_key);

            if !positions.is_empty() {
                // Use actual model error bound (not hardcoded!)
                let window_size = self.learned_index.max_error_bound();
                let min_pos = positions.iter().min().unwrap_or(&0);
                let max_pos = positions.iter().max().unwrap_or(&0);

                let start_pos = min_pos.saturating_sub(window_size);
                let end_pos = (max_pos + window_size).min(self.sorted_keys.len());

                // Find exact range in sorted_keys using binary search
                let actual_start = self.sorted_keys[start_pos..end_pos]
                    .binary_search(&start_key)
                    .unwrap_or_else(|pos| pos)
                    + start_pos;

                let actual_end = self.sorted_keys[start_pos..end_pos]
                    .binary_search(&end_key)
                    .map(|pos| pos + 1)
                    .unwrap_or_else(|pos| pos)
                    + start_pos;

                // Collect keys in range (copy to Vec to avoid borrow conflicts)
                let keys_in_range: Vec<i64> = self.sorted_keys
                    [actual_start..actual_end.min(self.sorted_keys.len())]
                    .to_vec();

                // Batch lookup values from redb using cached transaction
                let read_txn = self.get_read_txn()?;
                let table = read_txn.open_table(DATA_TABLE)?;

                let mut results = Vec::with_capacity(keys_in_range.len());
                for &key in &keys_in_range {
                    if let Some(value_guard) = table.get(key)? {
                        results.push((key, value_guard.value().to_vec()));
                    }
                }

                // Record learned index range query success
                metrics::record_query_path("learned_index");
                metrics::record_range_query(start_time.elapsed().as_secs_f64(), results.len());

                return Ok(results);
            }
        }

        // Fallback: full scan if no index
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
        // Lazy rebuild: ensure index is fresh
        self.ensure_index_fresh()?;

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

        let write_txn = self.db.begin_write()?;
        let deleted;
        {
            let mut table = write_txn.open_table(DATA_TABLE)?;
            deleted = table.remove(key)?.is_some();
        }
        write_txn.commit()?;

        if deleted {
            self.row_count = self.row_count.saturating_sub(1);
            self.rebuild_index()?;
            self.save_metadata()?;
        }

        metrics::record_delete(start_time.elapsed().as_secs_f64());

        let duration = start_time.elapsed();
        debug!(
            duration_ms = duration.as_millis(),
            deleted = deleted,
            "Delete completed"
        );

        Ok(deleted)
    }

    pub fn count(&self) -> u64 {
        self.row_count
    }

    pub fn close(self) -> Result<()> {
        self.save_metadata()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_redb_storage_basic() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.redb");

        let mut storage = RedbStorage::new(&db_path).unwrap();

        let value = b"test_value".to_vec();
        storage.insert(100, &value).unwrap();

        let result = storage.point_query(100).unwrap();
        assert_eq!(result, Some(value));

        assert_eq!(storage.count(), 1);
    }

    #[test]
    fn test_redb_range_query() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_range.redb");

        let mut storage = RedbStorage::new(&db_path).unwrap();

        for i in 0..100 {
            let value = format!("value_{}", i).into_bytes();
            storage.insert(i, &value).unwrap();
        }

        let results = storage.range_query(20, 30).unwrap();
        assert!(results.len() >= 10 && results.len() <= 11);

        for (key, _) in &results {
            assert!(*key >= 20 && *key <= 30);
        }
    }

    #[test]
    fn test_redb_delete() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_delete.redb");

        let mut storage = RedbStorage::new(&db_path).unwrap();

        storage.insert(42, b"test").unwrap();
        assert_eq!(storage.count(), 1);

        let deleted = storage.delete(42).unwrap();
        assert!(deleted);
        assert_eq!(storage.count(), 0);

        let result = storage.point_query(42).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_redb_persistence() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_persist.redb");

        {
            let mut storage = RedbStorage::new(&db_path).unwrap();
            for i in 0..50 {
                storage.insert(i * 10, b"persistent_data").unwrap();
            }
            storage.close().unwrap();
        }

        {
            let storage = RedbStorage::new(&db_path).unwrap();
            assert_eq!(storage.count(), 50);

            let result = storage.point_query(200).unwrap();
            assert_eq!(result, Some(b"persistent_data".to_vec()));
        }
    }

    #[test]
    fn test_learned_index_integration() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_learned.redb");

        let mut storage = RedbStorage::new(&db_path).unwrap();

        for i in 0..10000 {
            let value = format!("value_{}", i).into_bytes();
            storage.insert(i, &value).unwrap();
        }

        let result = storage.point_query(5000).unwrap();
        assert!(result.is_some());

        let range_results = storage.range_query(4000, 6000).unwrap();
        assert!(range_results.len() >= 2000);
    }

    #[test]
    fn test_observability_integration() {
        use crate::metrics::*;

        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_observability.redb");

        let mut storage = RedbStorage::new(&db_path).unwrap();

        // Record baseline metrics
        let baseline_searches = TOTAL_SEARCHES.get();
        let baseline_inserts = TOTAL_INSERTS.get();
        let baseline_learned_hits = LEARNED_INDEX_HITS.get();

        // Test batch insert (should record metrics and log)
        let batch: Vec<(i64, Vec<u8>)> = (0..1000)
            .map(|i| (i, format!("value_{}", i).into_bytes()))
            .collect();
        storage.insert_batch(batch).unwrap();

        // Verify insert metrics increased
        assert!(TOTAL_INSERTS.get() >= baseline_inserts + 1000);

        // Test point queries (should record learned index metrics)
        for i in (0..1000).step_by(100) {
            let result = storage.point_query(i).unwrap();
            assert!(result.is_some());
        }

        // Verify search metrics increased
        assert!(TOTAL_SEARCHES.get() >= baseline_searches + 10);

        // Verify learned index metrics are being tracked
        // After queries, we should have some hits (index was trained)
        assert!(LEARNED_INDEX_HITS.get() > baseline_learned_hits);

        // Test range query
        let range_results = storage.range_query(100, 200).unwrap();
        assert!(!range_results.is_empty());

        // Test delete
        let deleted = storage.delete(500).unwrap();
        assert!(deleted);

        // Verify learned index size metrics are set
        let index_size = LEARNED_INDEX_SIZE_KEYS.get();
        assert!(index_size > 0, "Learned index size should be tracked");

        let model_count = LEARNED_INDEX_MODELS_COUNT.get();
        assert!(model_count > 0, "Model count should be tracked");

        // Verify hit rate calculation
        let hit_rate = learned_index_hit_rate();
        assert!(
            hit_rate >= 0.0 && hit_rate <= 1.0,
            "Hit rate should be between 0 and 1"
        );
    }
}
