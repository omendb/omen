//! Transaction support for OmenDB with MVCC (Multi-Version Concurrency Control)

use rocksdb::{WriteBatch, DB as RocksDB};
use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

/// Transaction ID type
pub type TxnId = u64;

/// Version timestamp
pub type Version = u64;

/// Transaction isolation levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

/// Transaction state
#[derive(Debug, Clone, PartialEq)]
pub enum TxnState {
    Active,
    Committed,
    Aborted,
}

/// Versioned value with MVCC metadata
#[derive(Debug, Clone)]
pub struct VersionedValue {
    pub data: Vec<u8>,
    pub version: Version,
    pub txn_id: TxnId,
    pub deleted: bool,
}

/// Write operation in a transaction
#[derive(Debug, Clone)]
pub enum WriteOp {
    Put(i64, Vec<u8>),
    Delete(i64),
}

/// Transaction manager for MVCC
pub struct TransactionManager {
    /// Next transaction ID
    next_txn_id: AtomicU64,

    /// Next version number
    next_version: AtomicU64,

    /// Active transactions
    active_txns: Arc<RwLock<HashMap<TxnId, Transaction>>>,

    /// Version store (key -> list of versions)
    version_store: Arc<RwLock<BTreeMap<i64, Vec<VersionedValue>>>>,

    /// Commit log for durability
    commit_log: Arc<RwLock<Vec<CommitRecord>>>,

    /// Storage backend
    storage: Arc<RocksDB>,
}

/// Individual transaction
pub struct Transaction {
    pub id: TxnId,
    pub state: TxnState,
    pub isolation_level: IsolationLevel,
    pub start_version: Version,
    pub read_set: HashMap<i64, Version>,
    pub write_set: Vec<WriteOp>,
    pub created_at: SystemTime,
}

/// Commit record for WAL
#[derive(Debug, Clone)]
pub struct CommitRecord {
    pub txn_id: TxnId,
    pub commit_version: Version,
    pub operations: Vec<WriteOp>,
    pub timestamp: SystemTime,
}

impl TransactionManager {
    /// Create a new transaction manager
    pub fn new(storage: Arc<RocksDB>) -> Self {
        TransactionManager {
            next_txn_id: AtomicU64::new(1),
            next_version: AtomicU64::new(1),
            active_txns: Arc::new(RwLock::new(HashMap::new())),
            version_store: Arc::new(RwLock::new(BTreeMap::new())),
            commit_log: Arc::new(RwLock::new(Vec::new())),
            storage,
        }
    }

    /// Begin a new transaction
    pub fn begin(&self, isolation_level: IsolationLevel) -> TxnId {
        let txn_id = self.next_txn_id.fetch_add(1, Ordering::SeqCst);
        let start_version = self.next_version.load(Ordering::SeqCst);

        let txn = Transaction {
            id: txn_id,
            state: TxnState::Active,
            isolation_level,
            start_version,
            read_set: HashMap::new(),
            write_set: Vec::new(),
            created_at: SystemTime::now(),
        };

        self.active_txns.write().unwrap().insert(txn_id, txn);
        txn_id
    }

    /// Read a value within a transaction
    pub fn get(&self, txn_id: TxnId, key: i64) -> Result<Option<Vec<u8>>, String> {
        let active_txns = self.active_txns.read().unwrap();
        let txn = active_txns
            .get(&txn_id)
            .ok_or_else(|| format!("Transaction {} not found", txn_id))?;

        if txn.state != TxnState::Active {
            return Err(format!("Transaction {} is not active", txn_id));
        }

        // Check write set first (read-your-writes)
        for op in &txn.write_set {
            match op {
                WriteOp::Put(k, v) if *k == key => return Ok(Some(v.clone())),
                WriteOp::Delete(k) if *k == key => return Ok(None),
                _ => {}
            }
        }

        // Check version store based on isolation level
        let version_store = self.version_store.read().unwrap();
        if let Some(versions) = version_store.get(&key) {
            // Find the appropriate version based on isolation level
            let version = match txn.isolation_level {
                IsolationLevel::ReadUncommitted => {
                    // Read latest version, even uncommitted
                    versions.last()
                }
                IsolationLevel::ReadCommitted
                | IsolationLevel::RepeatableRead
                | IsolationLevel::Serializable => {
                    // Read latest committed version before transaction start
                    versions
                        .iter()
                        .rev()
                        .find(|v| v.version <= txn.start_version)
                }
            };

            if let Some(v) = version {
                if !v.deleted {
                    // Track read for validation
                    drop(active_txns);
                    self.active_txns
                        .write()
                        .unwrap()
                        .get_mut(&txn_id)
                        .unwrap()
                        .read_set
                        .insert(key, v.version);
                    return Ok(Some(v.data.clone()));
                }
            }
        }

        // Fallback to storage
        match self.storage.get(key.to_le_bytes()) {
            Ok(value) => Ok(value),
            Err(e) => Err(format!("Storage error: {}", e)),
        }
    }

    /// Write a value within a transaction
    pub fn put(&self, txn_id: TxnId, key: i64, value: Vec<u8>) -> Result<(), String> {
        let mut active_txns = self.active_txns.write().unwrap();
        let txn = active_txns
            .get_mut(&txn_id)
            .ok_or_else(|| format!("Transaction {} not found", txn_id))?;

        if txn.state != TxnState::Active {
            return Err(format!("Transaction {} is not active", txn_id));
        }

        txn.write_set.push(WriteOp::Put(key, value));
        Ok(())
    }

    /// Delete a value within a transaction
    pub fn delete(&self, txn_id: TxnId, key: i64) -> Result<(), String> {
        let mut active_txns = self.active_txns.write().unwrap();
        let txn = active_txns
            .get_mut(&txn_id)
            .ok_or_else(|| format!("Transaction {} not found", txn_id))?;

        if txn.state != TxnState::Active {
            return Err(format!("Transaction {} is not active", txn_id));
        }

        txn.write_set.push(WriteOp::Delete(key));
        Ok(())
    }

    /// Commit a transaction
    pub fn commit(&self, txn_id: TxnId) -> Result<(), String> {
        let mut active_txns = self.active_txns.write().unwrap();
        let txn = active_txns
            .get_mut(&txn_id)
            .ok_or_else(|| format!("Transaction {} not found", txn_id))?;

        if txn.state != TxnState::Active {
            return Err(format!("Transaction {} is not active", txn_id));
        }

        // Validate based on isolation level
        if txn.isolation_level == IsolationLevel::Serializable {
            if !self.validate_serializable(txn) {
                txn.state = TxnState::Aborted;
                return Err("Serialization conflict detected".to_string());
            }
        }

        // Get commit version
        let commit_version = self.next_version.fetch_add(1, Ordering::SeqCst);

        // Apply writes to version store
        let mut version_store = self.version_store.write().unwrap();
        let mut batch = WriteBatch::default();

        for op in &txn.write_set {
            match op {
                WriteOp::Put(key, value) => {
                    // Add to version store
                    let versioned = VersionedValue {
                        data: value.clone(),
                        version: commit_version,
                        txn_id,
                        deleted: false,
                    };
                    version_store
                        .entry(*key)
                        .or_insert_with(Vec::new)
                        .push(versioned);

                    // Add to storage batch
                    batch.put(key.to_le_bytes(), value);
                }
                WriteOp::Delete(key) => {
                    // Mark as deleted in version store
                    let versioned = VersionedValue {
                        data: Vec::new(),
                        version: commit_version,
                        txn_id,
                        deleted: true,
                    };
                    version_store
                        .entry(*key)
                        .or_insert_with(Vec::new)
                        .push(versioned);

                    // Add to storage batch
                    batch.delete(key.to_le_bytes());
                }
            }
        }

        // Write to storage
        if let Err(e) = self.storage.write(batch) {
            txn.state = TxnState::Aborted;
            return Err(format!("Storage write failed: {}", e));
        }

        // Log commit
        let commit_record = CommitRecord {
            txn_id,
            commit_version,
            operations: txn.write_set.clone(),
            timestamp: SystemTime::now(),
        };
        self.commit_log.write().unwrap().push(commit_record);

        txn.state = TxnState::Committed;
        Ok(())
    }

    /// Rollback a transaction
    pub fn rollback(&self, txn_id: TxnId) -> Result<(), String> {
        let mut active_txns = self.active_txns.write().unwrap();
        let txn = active_txns
            .get_mut(&txn_id)
            .ok_or_else(|| format!("Transaction {} not found", txn_id))?;

        if txn.state != TxnState::Active {
            return Err(format!("Transaction {} is not active", txn_id));
        }

        txn.state = TxnState::Aborted;
        txn.write_set.clear();
        txn.read_set.clear();
        Ok(())
    }

    /// Validate serializable isolation
    fn validate_serializable(&self, txn: &Transaction) -> bool {
        let version_store = self.version_store.read().unwrap();

        // Check if any read keys have been modified
        for (key, read_version) in &txn.read_set {
            if let Some(versions) = version_store.get(key) {
                // Check if any newer versions exist
                if versions.iter().any(|v| v.version > *read_version) {
                    return false; // Read conflict
                }
            }
        }

        true
    }

    /// Garbage collect old versions
    pub fn gc_old_versions(&self, keep_versions: usize) {
        let mut version_store = self.version_store.write().unwrap();

        for versions in version_store.values_mut() {
            if versions.len() > keep_versions {
                // Keep only the latest N versions
                let remove_count = versions.len() - keep_versions;
                versions.drain(0..remove_count);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_transaction_basic() {
        let dir = tempdir().unwrap();
        let db = Arc::new(RocksDB::open_default(dir.path()).unwrap());
        let tm = TransactionManager::new(db);

        // Begin transaction
        let txn1 = tm.begin(IsolationLevel::ReadCommitted);

        // Write data
        tm.put(txn1, 1, b"value1".to_vec()).unwrap();
        tm.put(txn1, 2, b"value2".to_vec()).unwrap();

        // Read within transaction (read-your-writes)
        assert_eq!(tm.get(txn1, 1).unwrap(), Some(b"value1".to_vec()));

        // Commit
        tm.commit(txn1).unwrap();

        // Read in new transaction
        let txn2 = tm.begin(IsolationLevel::ReadCommitted);
        assert_eq!(tm.get(txn2, 1).unwrap(), Some(b"value1".to_vec()));
        assert_eq!(tm.get(txn2, 2).unwrap(), Some(b"value2".to_vec()));
    }

    #[test]
    fn test_transaction_isolation() {
        let dir = tempdir().unwrap();
        let db = Arc::new(RocksDB::open_default(dir.path()).unwrap());
        let tm = TransactionManager::new(db);

        // Transaction 1 writes
        let txn1 = tm.begin(IsolationLevel::ReadCommitted);
        tm.put(txn1, 1, b"v1".to_vec()).unwrap();

        // Transaction 2 shouldn't see uncommitted writes
        let txn2 = tm.begin(IsolationLevel::ReadCommitted);
        assert_eq!(tm.get(txn2, 1).unwrap(), None);

        // After commit, txn2 should see the value
        tm.commit(txn1).unwrap();

        // New transaction sees committed value
        let txn3 = tm.begin(IsolationLevel::ReadCommitted);
        assert_eq!(tm.get(txn3, 1).unwrap(), Some(b"v1".to_vec()));
    }

    #[test]
    fn test_transaction_rollback() {
        let dir = tempdir().unwrap();
        let db = Arc::new(RocksDB::open_default(dir.path()).unwrap());
        let tm = TransactionManager::new(db);

        let txn = tm.begin(IsolationLevel::ReadCommitted);
        tm.put(txn, 1, b"value".to_vec()).unwrap();
        tm.rollback(txn).unwrap();

        // Rolled back data shouldn't be visible
        let txn2 = tm.begin(IsolationLevel::ReadCommitted);
        assert_eq!(tm.get(txn2, 1).unwrap(), None);
    }
}
