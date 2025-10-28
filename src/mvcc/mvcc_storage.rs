// MVCC-aware storage layer
//
// Provides multi-version storage operations on top of RocksDB:
// - Versioned writes: Store multiple versions of each key
// - Versioned reads: Query specific versions based on snapshot
// - Version tracking: ALEX index tracks latest version per key
//
// Architecture:
// - RocksDB: Stores (key, txn_id) → (value, begin_ts, end_ts)
// - ALEX: Tracks (key → latest_txn_id) for quick version lookup
// - Oracle: Provides transaction lifecycle and conflict detection

use crate::alex::AlexTree;
use crate::mvcc::{TransactionOracle, VersionedKey, VersionedValue};
use anyhow::{anyhow, Result};
use rocksdb::{WriteBatch, DB};
use std::sync::{Arc, RwLock};
use tracing::{debug, info};

/// MVCC storage layer on top of RocksDB
pub struct MvccStorage {
    /// RocksDB instance for versioned data
    db: Arc<DB>,
    /// ALEX index tracking latest version of each key
    /// Maps: key → latest_txn_id
    alex: Arc<RwLock<AlexTree>>,
    /// Transaction oracle for lifecycle management
    oracle: Arc<TransactionOracle>,
}

impl MvccStorage {
    /// Create new MVCC storage
    pub fn new(db: Arc<DB>, oracle: Arc<TransactionOracle>) -> Self {
        Self {
            db,
            alex: Arc::new(RwLock::new(AlexTree::new())),
            oracle,
        }
    }

    /// Insert a versioned value
    ///
    /// Creates a new version of the key with the given transaction ID.
    /// Updates ALEX to track this as the latest version.
    pub fn insert_version(
        &self,
        key: Vec<u8>,
        value: Vec<u8>,
        txn_id: u64,
    ) -> Result<()> {
        // Create versioned key and value
        let versioned_key = VersionedKey::new(key.clone(), txn_id);
        let versioned_value = VersionedValue::new(value, txn_id);

        // Encode and write to RocksDB
        let encoded_key = versioned_key.encode();
        let encoded_value = versioned_value.encode();

        self.db.put(&encoded_key, &encoded_value)?;

        // Update ALEX to track latest version
        // Convert key bytes to i64 for ALEX
        if key.len() == 8 {
            let key_i64 = i64::from_be_bytes(key[..8].try_into()?);
            let mut alex = self.alex.write()
                .map_err(|e| anyhow!("Lock error: {}", e))?;

            // Store txn_id as marker (ALEX tracks: key → latest_txn_id)
            alex.insert(key_i64, txn_id.to_be_bytes().to_vec())?;
        }

        debug!(key = ?key, txn_id = txn_id, "Inserted versioned value");
        Ok(())
    }

    /// Batch insert versioned values
    ///
    /// Atomically inserts multiple versioned values in a single RocksDB write batch.
    pub fn insert_version_batch(
        &self,
        entries: Vec<(Vec<u8>, Vec<u8>, u64)>,
    ) -> Result<()> {
        if entries.is_empty() {
            return Ok(());
        }

        info!(batch_size = entries.len(), "Batch insert versions started");

        let mut batch = WriteBatch::default();
        let mut alex_updates = Vec::new();

        for (key, value, txn_id) in entries.iter() {
            // Create versioned key and value
            let versioned_key = VersionedKey::new(key.clone(), *txn_id);
            let versioned_value = VersionedValue::new(value.clone(), *txn_id);

            // Add to write batch
            batch.put(versioned_key.encode(), versioned_value.encode());

            // Track for ALEX update
            if key.len() == 8 {
                let key_i64 = i64::from_be_bytes(key[..8].try_into()?);
                alex_updates.push((key_i64, *txn_id));
            }
        }

        // Atomic write to RocksDB
        self.db.write(batch)?;

        // Update ALEX with latest versions
        let mut alex = self.alex.write()
            .map_err(|e| anyhow!("Lock error: {}", e))?;

        for (key_i64, txn_id) in alex_updates {
            alex.insert(key_i64, txn_id.to_be_bytes().to_vec())?;
        }

        info!(batch_size = entries.len(), "Batch insert versions completed");
        Ok(())
    }

    /// Get latest version of a key
    ///
    /// Returns the most recent version visible to any transaction.
    /// Used for simple queries without snapshot isolation.
    pub fn get_latest_version(&self, key: &[u8]) -> Result<Option<VersionedValue>> {
        // Check ALEX for latest version
        if key.len() != 8 {
            return Ok(None);
        }

        let key_i64 = i64::from_be_bytes(key[..8].try_into()?);
        let alex = self.alex.read()
            .map_err(|e| anyhow!("Lock error: {}", e))?;

        let latest_txn_id = match alex.get(key_i64)? {
            Some(txn_bytes) => {
                if txn_bytes.len() != 8 {
                    return Err(anyhow!("Invalid txn_id stored in ALEX"));
                }
                u64::from_be_bytes(txn_bytes[..8].try_into()?)
            }
            None => return Ok(None), // Key doesn't exist
        };

        // Get the versioned value from RocksDB
        let versioned_key = VersionedKey::new(key.to_vec(), latest_txn_id);
        let encoded_key = versioned_key.encode();

        match self.db.get(&encoded_key)? {
            Some(value_bytes) => {
                let versioned_value = VersionedValue::decode(&value_bytes)?;
                Ok(Some(versioned_value))
            }
            None => Ok(None),
        }
    }

    /// Get version visible to a specific snapshot
    ///
    /// Returns the version of the key that should be visible to a transaction
    /// with the given snapshot timestamp.
    ///
    /// Visibility rules:
    /// - Version must have begin_ts <= snapshot_ts
    /// - Version must not be deleted (end_ts = None) OR end_ts > snapshot_ts
    pub fn get_snapshot_version(
        &self,
        key: &[u8],
        snapshot_ts: u64,
    ) -> Result<Option<Vec<u8>>> {
        // Scan all versions of this key (newest first due to inverted txn_id)
        let prefix = VersionedKey::prefix(key);
        let iter = self.db.prefix_iterator(&prefix);

        for item in iter {
            let (key_bytes, value_bytes) = item?;

            // Check if still within our key's prefix
            if !key_bytes.starts_with(&prefix) {
                break;
            }

            let versioned_key = VersionedKey::decode(&key_bytes)?;
            let versioned_value = VersionedValue::decode(&value_bytes)?;

            // Check visibility
            if versioned_value.begin_ts > snapshot_ts {
                // Version created after snapshot - skip
                continue;
            }

            if let Some(end_ts) = versioned_value.end_ts {
                if end_ts <= snapshot_ts {
                    // Version deleted before snapshot - skip
                    continue;
                }
            }

            // This version is visible!
            debug!(
                key = ?key,
                version_txn_id = versioned_key.txn_id,
                snapshot_ts = snapshot_ts,
                "Found visible version"
            );
            return Ok(Some(versioned_value.value));
        }

        // No visible version found
        Ok(None)
    }

    /// Mark a version as deleted
    ///
    /// Sets the end_ts of the latest version to mark it as deleted.
    /// The actual deletion will happen during garbage collection.
    pub fn delete_version(&self, key: Vec<u8>, end_ts: u64) -> Result<()> {
        // Get latest version
        if key.len() != 8 {
            return Err(anyhow!("Invalid key length"));
        }

        let key_i64 = i64::from_be_bytes(key[..8].try_into()?);
        let alex = self.alex.read()
            .map_err(|e| anyhow!("Lock error: {}", e))?;

        let latest_txn_id = match alex.get(key_i64)? {
            Some(txn_bytes) => u64::from_be_bytes(txn_bytes[..8].try_into()?),
            None => return Err(anyhow!("Key not found for deletion")),
        };

        drop(alex);

        // Read current version
        let versioned_key = VersionedKey::new(key.clone(), latest_txn_id);
        let encoded_key = versioned_key.encode();

        let mut versioned_value = match self.db.get(&encoded_key)? {
            Some(bytes) => VersionedValue::decode(&bytes)?,
            None => return Err(anyhow!("Version not found in RocksDB")),
        };

        // Mark as deleted
        versioned_value.mark_deleted(end_ts);

        // Write back
        self.db.put(&encoded_key, versioned_value.encode())?;

        debug!(key = ?key, end_ts = end_ts, "Marked version as deleted");
        Ok(())
    }

    /// Get all versions of a key (for testing/debugging)
    pub fn get_all_versions(&self, key: &[u8]) -> Result<Vec<(u64, VersionedValue)>> {
        let prefix = VersionedKey::prefix(key);
        let iter = self.db.prefix_iterator(&prefix);
        let mut versions = Vec::new();

        for item in iter {
            let (key_bytes, value_bytes) = item?;

            if !key_bytes.starts_with(&prefix) {
                break;
            }

            let versioned_key = VersionedKey::decode(&key_bytes)?;
            let versioned_value = VersionedValue::decode(&value_bytes)?;

            versions.push((versioned_key.txn_id, versioned_value));
        }

        Ok(versions)
    }

    /// Get reference to transaction oracle
    pub fn oracle(&self) -> &Arc<TransactionOracle> {
        &self.oracle
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mvcc::TransactionMode;
    use rocksdb::Options;
    use tempfile::TempDir;

    fn setup_test_storage() -> (MvccStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let mut opts = Options::default();
        opts.create_if_missing(true);

        let db = Arc::new(DB::open(&opts, temp_dir.path()).unwrap());
        let oracle = Arc::new(TransactionOracle::new());
        let storage = MvccStorage::new(db, oracle);

        (storage, temp_dir)
    }

    #[test]
    fn test_insert_and_get_latest() {
        let (storage, _temp) = setup_test_storage();
        let key = 100i64.to_be_bytes().to_vec();
        let value = b"hello".to_vec();
        let txn_id = 1;

        storage.insert_version(key.clone(), value.clone(), txn_id).unwrap();

        let result = storage.get_latest_version(&key).unwrap();
        assert!(result.is_some());
        let versioned_value = result.unwrap();
        assert_eq!(versioned_value.value, value);
        assert_eq!(versioned_value.begin_ts, txn_id);
        assert_eq!(versioned_value.end_ts, None);
    }

    #[test]
    fn test_multiple_versions() {
        let (storage, _temp) = setup_test_storage();
        let key = 200i64.to_be_bytes().to_vec();

        storage.insert_version(key.clone(), b"v1".to_vec(), 1).unwrap();
        storage.insert_version(key.clone(), b"v2".to_vec(), 2).unwrap();
        storage.insert_version(key.clone(), b"v3".to_vec(), 3).unwrap();

        // Latest should be v3
        let latest = storage.get_latest_version(&key).unwrap().unwrap();
        assert_eq!(latest.value, b"v3");
        assert_eq!(latest.begin_ts, 3);

        // Check all versions exist
        let all_versions = storage.get_all_versions(&key).unwrap();
        assert_eq!(all_versions.len(), 3);

        // Versions should be ordered newest first (due to inverted txn_id)
        assert_eq!(all_versions[0].0, 3);
        assert_eq!(all_versions[1].0, 2);
        assert_eq!(all_versions[2].0, 1);
    }

    #[test]
    fn test_snapshot_visibility() {
        let (storage, _temp) = setup_test_storage();
        let key = 300i64.to_be_bytes().to_vec();

        // Insert versions at different timestamps
        storage.insert_version(key.clone(), b"v1".to_vec(), 10).unwrap();
        storage.insert_version(key.clone(), b"v2".to_vec(), 20).unwrap();
        storage.insert_version(key.clone(), b"v3".to_vec(), 30).unwrap();

        // Snapshot at ts=15 should see v1
        let result = storage.get_snapshot_version(&key, 15).unwrap();
        assert_eq!(result, Some(b"v1".to_vec()));

        // Snapshot at ts=25 should see v2
        let result = storage.get_snapshot_version(&key, 25).unwrap();
        assert_eq!(result, Some(b"v2".to_vec()));

        // Snapshot at ts=35 should see v3
        let result = storage.get_snapshot_version(&key, 35).unwrap();
        assert_eq!(result, Some(b"v3".to_vec()));

        // Snapshot at ts=5 (before any version) should see nothing
        let result = storage.get_snapshot_version(&key, 5).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_delete_version() {
        let (storage, _temp) = setup_test_storage();
        let key = 400i64.to_be_bytes().to_vec();

        // Insert and then delete
        storage.insert_version(key.clone(), b"data".to_vec(), 10).unwrap();
        storage.delete_version(key.clone(), 20).unwrap();

        // Snapshot at ts=15 should see the value
        let result = storage.get_snapshot_version(&key, 15).unwrap();
        assert_eq!(result, Some(b"data".to_vec()));

        // Snapshot at ts=25 should NOT see the value (deleted)
        let result = storage.get_snapshot_version(&key, 25).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_batch_insert() {
        let (storage, _temp) = setup_test_storage();

        let entries = vec![
            (100i64.to_be_bytes().to_vec(), b"v1".to_vec(), 1),
            (200i64.to_be_bytes().to_vec(), b"v2".to_vec(), 1),
            (300i64.to_be_bytes().to_vec(), b"v3".to_vec(), 1),
        ];

        storage.insert_version_batch(entries).unwrap();

        // Verify all inserted
        for key in [100i64, 200i64, 300i64] {
            let key_bytes = key.to_be_bytes().to_vec();
            let result = storage.get_latest_version(&key_bytes).unwrap();
            assert!(result.is_some());
        }
    }

    #[test]
    fn test_read_your_own_writes() {
        let (storage, _temp) = setup_test_storage();
        let key = 500i64.to_be_bytes().to_vec();

        // Transaction writes at ts=10
        storage.insert_version(key.clone(), b"my_write".to_vec(), 10).unwrap();

        // Same transaction reads (snapshot_ts=10, should see write)
        let result = storage.get_snapshot_version(&key, 10).unwrap();
        assert_eq!(result, Some(b"my_write".to_vec()));
    }
}
