//! Storage engine implementations

use super::{BatchOp, Result, StorageEngine};
use rocksdb::{Options, WriteBatch, DB as RocksDB};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::RwLock;

/// RocksDB storage engine
#[derive(Debug)]
pub struct RocksDBEngine {
    db: Arc<RocksDB>,
}

impl RocksDBEngine {
    pub fn open(path: &str) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        opts.set_write_buffer_size(64 * 1024 * 1024);

        let db = RocksDB::open(&opts, path)?;
        Ok(Self { db: Arc::new(db) })
    }
}

impl StorageEngine for RocksDBEngine {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(self.db.get(key)?)
    }

    fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
        self.db.put(key, value)?;
        Ok(())
    }

    fn delete(&self, key: &[u8]) -> Result<()> {
        self.db.delete(key)?;
        Ok(())
    }

    fn scan(&self, start: &[u8], end: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let mut results = Vec::new();
        let iter = self.db.iterator(rocksdb::IteratorMode::From(
            start,
            rocksdb::Direction::Forward,
        ));

        for item in iter {
            let (key, value) = item?;
            if key.as_ref() > end {
                break;
            }
            results.push((key.to_vec(), value.to_vec()));
        }

        Ok(results)
    }

    fn batch_write(&self, ops: Vec<BatchOp>) -> Result<()> {
        let mut batch = WriteBatch::default();

        for op in ops {
            match op {
                BatchOp::Put(key, value) => batch.put(key, value),
                BatchOp::Delete(key) => batch.delete(key),
            }
        }

        self.db.write(batch)?;
        Ok(())
    }

    fn stats(&self) -> String {
        format!("RocksDB: LSM-tree storage with compression")
    }
}

/// In-memory storage engine (for testing)
#[derive(Debug)]
pub struct InMemoryEngine {
    data: Arc<RwLock<BTreeMap<Vec<u8>, Vec<u8>>>>,
}

impl InMemoryEngine {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }
}

impl StorageEngine for InMemoryEngine {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let data = self.data.read().unwrap();
        Ok(data.get(key).cloned())
    }

    fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
        let mut data = self.data.write().unwrap();
        data.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn delete(&self, key: &[u8]) -> Result<()> {
        let mut data = self.data.write().unwrap();
        data.remove(key);
        Ok(())
    }

    fn scan(&self, start: &[u8], end: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let data = self.data.read().unwrap();
        let results: Vec<_> = data
            .range(start.to_vec()..=end.to_vec())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        Ok(results)
    }

    fn batch_write(&self, ops: Vec<BatchOp>) -> Result<()> {
        let mut data = self.data.write().unwrap();

        for op in ops {
            match op {
                BatchOp::Put(key, value) => {
                    data.insert(key, value);
                }
                BatchOp::Delete(key) => {
                    data.remove(&key);
                }
            }
        }

        Ok(())
    }

    fn stats(&self) -> String {
        let data = self.data.read().unwrap();
        format!("InMemory: {} keys in BTreeMap", data.len())
    }
}
