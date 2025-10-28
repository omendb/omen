// MVCC Transaction Context: Snapshot Isolation Transaction Management
//
// Integrates all MVCC components for production-ready transactions:
// - TransactionOracle for lifecycle and timestamps
// - MvccStorage for versioned reads and writes
// - VisibilityEngine for snapshot isolation
// - ConflictDetector for first-committer-wins
//
// Provides:
// - BEGIN: Start transaction with snapshot
// - COMMIT: Validate conflicts and persist changes
// - ROLLBACK: Discard changes
// - Read-your-own-writes within transaction

use crate::mvcc::{
    MvccStorage, TransactionMode, TransactionOracle,
};
use anyhow::{anyhow, Result};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// MVCC transaction context for snapshot isolation
///
/// Each transaction gets:
/// - Unique transaction ID from oracle
/// - Snapshot of active transactions at BEGIN time
/// - Write buffer for uncommitted changes
/// - Read/write set tracking for conflict detection
pub struct MvccTransactionContext {
    /// Transaction ID (None if not in transaction)
    txn_id: Option<u64>,
    /// Snapshot timestamp
    snapshot_ts: Option<u64>,
    /// Active transactions at snapshot time
    snapshot: Vec<u64>,
    /// Transaction mode (ReadWrite or ReadOnly)
    mode: TransactionMode,
    /// Write buffer: key → value (uncommitted writes)
    write_buffer: HashMap<Vec<u8>, Vec<u8>>,
    /// Write set: keys modified in this transaction
    write_set: HashSet<Vec<u8>>,
    /// Read set: keys read in this transaction
    read_set: HashSet<Vec<u8>>,
    /// Transaction oracle (shared across all transactions)
    oracle: Arc<TransactionOracle>,
    /// MVCC storage (shared)
    storage: Arc<MvccStorage>,
}

impl MvccTransactionContext {
    /// Create new transaction context
    pub fn new(oracle: Arc<TransactionOracle>, storage: Arc<MvccStorage>) -> Self {
        Self {
            txn_id: None,
            snapshot_ts: None,
            snapshot: Vec::new(),
            mode: TransactionMode::ReadWrite,
            write_buffer: HashMap::new(),
            write_set: HashSet::new(),
            read_set: HashSet::new(),
            oracle,
            storage,
        }
    }

    /// Check if currently in a transaction
    pub fn is_in_transaction(&self) -> bool {
        self.txn_id.is_some()
    }

    /// Get current transaction ID
    pub fn txn_id(&self) -> Option<u64> {
        self.txn_id
    }

    /// Begin a new transaction
    ///
    /// Allocates transaction ID from oracle and captures snapshot.
    pub fn begin(&mut self, mode: TransactionMode) -> Result<u64> {
        if self.is_in_transaction() {
            let txn_id = self.txn_id.unwrap();
            warn!("BEGIN called while transaction {} is active", txn_id);
            // PostgreSQL behavior: BEGIN within transaction continues current tx
            return Ok(txn_id);
        }

        // Start transaction in oracle
        let txn_id = self.oracle.begin(mode)?;
        let snapshot = self.oracle.get_snapshot(txn_id)?;

        self.txn_id = Some(txn_id);
        self.snapshot_ts = Some(txn_id); // Snapshot timestamp = transaction ID
        self.snapshot = snapshot;
        self.mode = mode;
        self.write_buffer.clear();
        self.write_set.clear();
        self.read_set.clear();

        info!("Transaction {} started (mode: {:?}, snapshot: {:?})", txn_id, mode, self.snapshot);
        Ok(txn_id)
    }

    /// Read a key with snapshot isolation
    ///
    /// 1. Check write buffer first (read-your-own-writes)
    /// 2. Query storage with snapshot for visible version
    /// 3. Record in read set for conflict detection
    pub fn read(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let txn_id = self.txn_id.ok_or_else(|| anyhow!("Not in transaction"))?;
        let snapshot_ts = self.snapshot_ts.unwrap();

        // 1. Check write buffer first (read-your-own-writes)
        if let Some(value) = self.write_buffer.get(key) {
            debug!(txn = txn_id, key = ?key, "Read own write from buffer");
            return Ok(Some(value.clone()));
        }

        // 2. Read from storage with snapshot visibility
        let result = self.storage.get_snapshot_version(key, snapshot_ts)?;

        // 3. Track read for conflict detection
        self.read_set.insert(key.to_vec());

        debug!(txn = txn_id, key = ?key, found = result.is_some(), "Snapshot read");
        Ok(result)
    }

    /// Write a key (buffered until commit)
    ///
    /// Writes are buffered in memory until commit. Read-your-own-writes
    /// ensures this transaction sees its own uncommitted writes.
    pub fn write(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        let txn_id = self.txn_id.ok_or_else(|| anyhow!("Not in transaction"))?;

        if self.mode == TransactionMode::ReadOnly {
            return Err(anyhow!("Cannot write in read-only transaction"));
        }

        // Buffer write
        self.write_buffer.insert(key.clone(), value);
        self.write_set.insert(key.clone());

        // Record write in oracle
        self.oracle.record_write(txn_id, key.clone())?;

        debug!(txn = txn_id, key = ?key, "Buffered write");
        Ok(())
    }

    /// Delete a key (creates tombstone)
    ///
    /// Deletion is implemented as a special write that marks the version as deleted.
    pub fn delete(&mut self, key: Vec<u8>) -> Result<()> {
        let txn_id = self.txn_id.ok_or_else(|| anyhow!("Not in transaction"))?;

        if self.mode == TransactionMode::ReadOnly {
            return Err(anyhow!("Cannot delete in read-only transaction"));
        }

        // Mark key for deletion (empty value = tombstone)
        self.write_buffer.insert(key.clone(), Vec::new());
        self.write_set.insert(key.clone());
        self.oracle.record_write(txn_id, key.clone())?;

        debug!(txn = txn_id, key = ?key, "Buffered delete");
        Ok(())
    }

    /// Commit transaction
    ///
    /// 1. Validate no write conflicts (first-committer-wins)
    /// 2. Get commit timestamp from oracle
    /// 3. Write all buffered changes to storage
    /// 4. Return to idle state
    pub fn commit(&mut self) -> Result<u64> {
        let txn_id = self.txn_id.ok_or_else(|| anyhow!("Not in transaction"))?;

        info!(
            txn = txn_id,
            writes = self.write_set.len(),
            reads = self.read_set.len(),
            "Committing transaction"
        );

        // 1. Oracle validates conflicts and assigns commit timestamp
        let commit_ts = match self.oracle.commit(txn_id) {
            Ok(ts) => ts,
            Err(e) => {
                // Conflict detected - rollback
                warn!(txn = txn_id, error = %e, "Commit failed due to conflict");
                self.rollback()?;
                return Err(e);
            }
        };

        // 2. Persist all buffered writes to storage
        let mut entries = Vec::new();
        for (key, value) in self.write_buffer.drain() {
            if value.is_empty() {
                // Tombstone (delete)
                self.storage.delete_version(key, commit_ts)?;
            } else {
                // Regular write
                entries.push((key, value, txn_id));
            }
        }

        if !entries.is_empty() {
            self.storage.insert_version_batch(entries)?;
        }

        info!(txn = txn_id, commit_ts = commit_ts, "Transaction committed");

        // 3. Clear state
        self.txn_id = None;
        self.snapshot_ts = None;
        self.snapshot.clear();
        self.write_set.clear();
        self.read_set.clear();

        Ok(commit_ts)
    }

    /// Rollback transaction
    ///
    /// Discards all buffered writes and aborts transaction in oracle.
    pub fn rollback(&mut self) -> Result<()> {
        match self.txn_id {
            Some(txn_id) => {
                info!(
                    txn = txn_id,
                    writes = self.write_buffer.len(),
                    "Rolling back transaction"
                );

                // Abort in oracle
                self.oracle.abort(txn_id)?;

                // Clear all state
                self.txn_id = None;
                self.snapshot_ts = None;
                self.snapshot.clear();
                self.write_buffer.clear();
                self.write_set.clear();
                self.read_set.clear();

                Ok(())
            }
            None => {
                debug!("ROLLBACK called outside transaction (no-op)");
                Ok(())
            }
        }
    }

    /// Get write buffer size
    pub fn write_buffer_size(&self) -> usize {
        self.write_buffer.len()
    }

    /// Get write set size
    pub fn write_set_size(&self) -> usize {
        self.write_set.len()
    }

    /// Get read set size
    pub fn read_set_size(&self) -> usize {
        self.read_set.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mvcc::TransactionOracle;
    use rocksdb::{Options, DB};
    use std::sync::Arc;
    use tempfile::TempDir;

    fn setup() -> (MvccTransactionContext, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let mut opts = Options::default();
        opts.create_if_missing(true);

        let db = Arc::new(DB::open(&opts, temp_dir.path()).unwrap());
        let oracle = Arc::new(TransactionOracle::new());
        let storage = Arc::new(MvccStorage::new(db, oracle.clone()));
        let ctx = MvccTransactionContext::new(oracle, storage);

        (ctx, temp_dir)
    }

    #[test]
    fn test_begin_commit() {
        let (mut ctx, _temp) = setup();

        assert!(!ctx.is_in_transaction());

        let txn_id = ctx.begin(TransactionMode::ReadWrite).unwrap();
        assert!(ctx.is_in_transaction());
        assert_eq!(ctx.txn_id(), Some(txn_id));

        let commit_ts = ctx.commit().unwrap();
        assert!(!ctx.is_in_transaction());
        assert!(commit_ts > txn_id);
    }

    #[test]
    fn test_begin_rollback() {
        let (mut ctx, _temp) = setup();

        ctx.begin(TransactionMode::ReadWrite).unwrap();
        assert!(ctx.is_in_transaction());

        ctx.rollback().unwrap();
        assert!(!ctx.is_in_transaction());
    }

    #[test]
    fn test_read_your_own_writes() {
        let (mut ctx, _temp) = setup();

        ctx.begin(TransactionMode::ReadWrite).unwrap();

        let key = 100i64.to_be_bytes().to_vec();
        let value = b"test_value".to_vec();

        // Write
        ctx.write(key.clone(), value.clone()).unwrap();

        // Read should see own write
        let result = ctx.read(&key).unwrap();
        assert_eq!(result, Some(value));

        ctx.commit().unwrap();
    }

    #[test]
    fn test_write_buffer() {
        let (mut ctx, _temp) = setup();

        ctx.begin(TransactionMode::ReadWrite).unwrap();

        let key1 = 100i64.to_be_bytes().to_vec();
        let key2 = 200i64.to_be_bytes().to_vec();

        ctx.write(key1, b"value1".to_vec()).unwrap();
        ctx.write(key2, b"value2".to_vec()).unwrap();

        assert_eq!(ctx.write_buffer_size(), 2);
        assert_eq!(ctx.write_set_size(), 2);

        ctx.commit().unwrap();

        // Buffer should be cleared
        assert_eq!(ctx.write_buffer_size(), 0);
    }

    #[test]
    fn test_rollback_clears_buffer() {
        let (mut ctx, _temp) = setup();

        ctx.begin(TransactionMode::ReadWrite).unwrap();

        ctx.write(100i64.to_be_bytes().to_vec(), b"value".to_vec()).unwrap();
        assert_eq!(ctx.write_buffer_size(), 1);

        ctx.rollback().unwrap();

        assert_eq!(ctx.write_buffer_size(), 0);
        assert!(!ctx.is_in_transaction());
    }

    #[test]
    fn test_read_only_transaction() {
        let (mut ctx, _temp) = setup();

        ctx.begin(TransactionMode::ReadOnly).unwrap();

        let key = 100i64.to_be_bytes().to_vec();
        let result = ctx.write(key, b"value".to_vec());

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("read-only"));
    }

    #[test]
    fn test_write_outside_transaction() {
        let (mut ctx, _temp) = setup();

        let key = 100i64.to_be_bytes().to_vec();
        let result = ctx.write(key, b"value".to_vec());

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not in transaction"));
    }

    #[test]
    fn test_snapshot_isolation() {
        let (mut ctx1, _temp) = setup();
        let oracle = ctx1.oracle.clone();
        let storage = ctx1.storage.clone();
        let mut ctx2 = MvccTransactionContext::new(oracle, storage);

        // T1: Write initial value
        ctx1.begin(TransactionMode::ReadWrite).unwrap();
        let key = 100i64.to_be_bytes().to_vec();
        ctx1.write(key.clone(), b"v1".to_vec()).unwrap();
        ctx1.commit().unwrap();

        // T2: Start transaction (captures snapshot)
        ctx2.begin(TransactionMode::ReadWrite).unwrap();
        let v2_read1 = ctx2.read(&key).unwrap();
        assert_eq!(v2_read1, Some(b"v1".to_vec()));

        // T1: Update value
        ctx1.begin(TransactionMode::ReadWrite).unwrap();
        ctx1.write(key.clone(), b"v2".to_vec()).unwrap();
        ctx1.commit().unwrap();

        // T2: Should still see v1 (snapshot isolation)
        let v2_read2 = ctx2.read(&key).unwrap();
        assert_eq!(v2_read2, Some(b"v1".to_vec()));

        ctx2.commit().unwrap();
    }

    #[test]
    fn test_delete() {
        let (mut ctx, _temp) = setup();

        // Write initial value
        ctx.begin(TransactionMode::ReadWrite).unwrap();
        let key = 100i64.to_be_bytes().to_vec();
        ctx.write(key.clone(), b"value".to_vec()).unwrap();
        ctx.commit().unwrap();

        // Delete
        ctx.begin(TransactionMode::ReadWrite).unwrap();
        ctx.delete(key.clone()).unwrap();
        ctx.commit().unwrap();

        // Verify deleted
        ctx.begin(TransactionMode::ReadOnly).unwrap();
        let result = ctx.read(&key).unwrap();
        assert_eq!(result, None);
        ctx.commit().unwrap();
    }

    #[test]
    fn test_multiple_writes_same_key() {
        let (mut ctx, _temp) = setup();

        ctx.begin(TransactionMode::ReadWrite).unwrap();
        let key = 100i64.to_be_bytes().to_vec();

        ctx.write(key.clone(), b"v1".to_vec()).unwrap();
        ctx.write(key.clone(), b"v2".to_vec()).unwrap();
        ctx.write(key.clone(), b"v3".to_vec()).unwrap();

        // Should see latest write
        let result = ctx.read(&key).unwrap();
        assert_eq!(result, Some(b"v3".to_vec()));

        // Write set should only have one entry for this key
        assert_eq!(ctx.write_set_size(), 1);

        ctx.commit().unwrap();
    }

    #[test]
    fn test_nested_begin_ignored() {
        let (mut ctx, _temp) = setup();

        let tx1 = ctx.begin(TransactionMode::ReadWrite).unwrap();
        let tx2 = ctx.begin(TransactionMode::ReadWrite).unwrap();

        // Should return same transaction ID
        assert_eq!(tx1, tx2);

        ctx.commit().unwrap();
    }
}
