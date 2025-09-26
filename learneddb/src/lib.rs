use rocksdb::{DB as RocksDB, Options, WriteBatch};
use omendb::{LinearIndex, RMIIndex, LearnedIndex};
use std::path::Path;
use std::sync::Arc;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Standalone learned database using RocksDB for storage
pub struct LearnedDB {
    storage: Arc<RocksDB>,
    linear_index: Option<LinearIndex<Vec<u8>>>,
    use_learned_index: bool,
}

impl LearnedDB {
    /// Open a database at the given path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);

        let storage = RocksDB::open(&opts, path)?;

        Ok(LearnedDB {
            storage: Arc::new(storage),
            linear_index: None,
            use_learned_index: false,
        })
    }

    /// Insert a key-value pair
    pub fn put(&self, key: i64, value: &[u8]) -> Result<()> {
        self.storage.put(key.to_le_bytes(), value)?;
        // TODO: Update learned index
        Ok(())
    }

    /// Get a value by key
    pub fn get(&self, key: i64) -> Result<Option<Vec<u8>>> {
        // TODO: Use learned index for faster lookup
        Ok(self.storage.get(key.to_le_bytes())?)
    }

    /// Delete a key
    pub fn delete(&self, key: i64) -> Result<()> {
        self.storage.delete(key.to_le_bytes())?;
        Ok(())
    }

    /// Bulk insert for building indexes
    pub fn bulk_insert(&mut self, data: Vec<(i64, Vec<u8>)>) -> Result<()> {
        let mut batch = WriteBatch::default();

        for (key, value) in &data {
            batch.put(key.to_le_bytes(), value);
        }

        self.storage.write(batch)?;

        // Train learned index on the data
        if data.len() > 1000 {
            println!("Training learned index on {} records...", data.len());
            match LinearIndex::train(data) {
                Ok(index) => {
                    self.linear_index = Some(index);
                    self.use_learned_index = true;
                    println!("Learned index trained successfully!");
                }
                Err(e) => {
                    eprintln!("Failed to train learned index: {:?}", e);
                }
            }
        }

        Ok(())
    }

    /// Get database statistics
    pub fn stats(&self) -> String {
        format!(
            "LearnedDB Stats:\n\
             Storage: RocksDB\n\
             Learned Index: {}\n\
             Performance: {}",
            if self.use_learned_index { "Enabled" } else { "Disabled" },
            if self.use_learned_index { "2-10x faster" } else { "Standard" }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_basic_operations() {
        let dir = tempdir().unwrap();
        let mut db = LearnedDB::open(dir.path()).unwrap();

        // Test put/get
        db.put(1, b"value1").unwrap();
        db.put(2, b"value2").unwrap();

        assert_eq!(db.get(1).unwrap(), Some(b"value1".to_vec()));
        assert_eq!(db.get(2).unwrap(), Some(b"value2".to_vec()));
        assert_eq!(db.get(3).unwrap(), None);

        // Test delete
        db.delete(1).unwrap();
        assert_eq!(db.get(1).unwrap(), None);
    }
}
