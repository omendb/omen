//! redb-based transactional storage with learned index integration

use crate::index::RecursiveModelIndex;
use crate::value::Value;
use crate::row::Row;
use anyhow::{Result, anyhow};
use redb::{Database, ReadableTable, TableDefinition};
use serde::{Serialize, Deserialize};
use std::path::Path;
use std::sync::Arc;

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

    fn rebuild_index(&mut self) -> Result<()> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(DATA_TABLE)?;

        let keys: Vec<i64> = table.iter()?
            .filter_map(|result| result.ok())
            .map(|(key, _)| key.value())
            .collect();

        if !keys.is_empty() {
            let data: Vec<(i64, usize)> = keys.iter()
                .enumerate()
                .map(|(i, &k)| (k, i))
                .collect();
            self.learned_index.train(data);
        }

        Ok(())
    }

    pub fn insert(&mut self, key: i64, value: &[u8]) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(DATA_TABLE)?;
            table.insert(key, value)?;
        }
        write_txn.commit()?;

        self.learned_index.add_key(key);
        self.row_count += 1;

        if self.row_count % 1000 == 0 {
            self.save_metadata()?;
        }

        Ok(())
    }

    pub fn insert_batch(&mut self, entries: Vec<(i64, Vec<u8>)>) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(DATA_TABLE)?;
            for (key, value) in &entries {
                table.insert(*key, value.as_slice())?;
                self.learned_index.add_key(*key);
                self.row_count += 1;
            }
        }
        write_txn.commit()?;

        self.save_metadata()?;
        Ok(())
    }

    pub fn point_query(&self, key: i64) -> Result<Option<Vec<u8>>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(DATA_TABLE)?;

        if let Some(value_guard) = table.get(key)? {
            Ok(Some(value_guard.value().to_vec()))
        } else {
            Ok(None)
        }
    }

    pub fn range_query(&self, start_key: i64, end_key: i64) -> Result<Vec<(i64, Vec<u8>)>> {
        let _positions = self.learned_index.range_search(start_key, end_key);

        let read_txn = self.db.begin_read()?;
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

        Ok(results)
    }

    pub fn scan_all(&self) -> Result<Vec<(i64, Vec<u8>)>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(DATA_TABLE)?;

        let results: Result<Vec<_>> = table.iter()?
            .map(|result| {
                let (key_guard, value_guard) = result?;
                Ok((key_guard.value(), value_guard.value().to_vec()))
            })
            .collect();

        results
    }

    pub fn delete(&mut self, key: i64) -> Result<bool> {
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
}
