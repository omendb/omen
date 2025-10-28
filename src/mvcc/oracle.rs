// Transaction Oracle: manages transaction lifecycle and timestamp allocation
//
// Provides:
// - Monotonic timestamp allocation
// - Transaction lifecycle (begin, commit, abort)
// - Write conflict detection (snapshot isolation)
// - Garbage collection watermark calculation

use anyhow::{anyhow, Result};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

/// Transaction status
#[derive(Debug, Clone, PartialEq)]
pub enum TxnStatus {
    Active,
    Committed(u64),  // commit timestamp
    Aborted,
}

/// Transaction mode (read-only vs read-write)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransactionMode {
    ReadWrite,
    ReadOnly,
}

/// State of a transaction tracked by the oracle
#[derive(Debug, Clone)]
pub struct TransactionState {
    pub txn_id: u64,
    pub start_ts: u64,
    pub snapshot: Vec<u64>,
    pub write_set: HashSet<Vec<u8>>,
    pub read_set: HashSet<Vec<u8>>,
    pub status: TxnStatus,
    pub mode: TransactionMode,
}

/// Record of a committed transaction (for conflict detection)
#[derive(Debug, Clone)]
pub struct CommittedTransaction {
    pub txn_id: u64,
    pub commit_ts: u64,
    pub write_set: HashSet<Vec<u8>>,
}

/// Maximum number of recent commits to track
const MAX_RECENT_COMMITS: usize = 10000;

/// Transaction Oracle: central MVCC component
pub struct TransactionOracle {
    next_txn_id: AtomicU64,
    active_txns: RwLock<HashMap<u64, TransactionState>>,
    recent_commits: RwLock<VecDeque<CommittedTransaction>>,
    gc_watermark: AtomicU64,
}

impl TransactionOracle {
    pub fn new() -> Self {
        Self {
            next_txn_id: AtomicU64::new(1),
            active_txns: RwLock::new(HashMap::new()),
            recent_commits: RwLock::new(VecDeque::with_capacity(MAX_RECENT_COMMITS)),
            gc_watermark: AtomicU64::new(0),
        }
    }

    /// Begin a new transaction
    pub fn begin(&self, mode: TransactionMode) -> Result<u64> {
        let txn_id = self.next_txn_id.fetch_add(1, Ordering::SeqCst);
        let snapshot_ts = txn_id;

        let active_txns = self.active_txns.read()
            .map_err(|e| anyhow!("Lock error: {}", e))?;

        let snapshot: Vec<u64> = active_txns.keys()
            .filter(|&&id| id < snapshot_ts)
            .copied()
            .collect();

        let state = TransactionState {
            txn_id,
            start_ts: snapshot_ts,
            snapshot,
            write_set: HashSet::new(),
            read_set: HashSet::new(),
            status: TxnStatus::Active,
            mode,
        };

        drop(active_txns);
        let mut active_txns = self.active_txns.write()
            .map_err(|e| anyhow!("Lock error: {}", e))?;
        active_txns.insert(txn_id, state);

        Ok(txn_id)
    }

    /// Get snapshot for a transaction
    pub fn get_snapshot(&self, txn_id: u64) -> Result<Vec<u64>> {
        let active_txns = self.active_txns.read()
            .map_err(|e| anyhow!("Lock error: {}", e))?;

        active_txns.get(&txn_id)
            .map(|state| state.snapshot.clone())
            .ok_or_else(|| anyhow!("Transaction {} not found", txn_id))
    }

    /// Record a write operation
    pub fn record_write(&self, txn_id: u64, key: Vec<u8>) -> Result<()> {
        let mut active_txns = self.active_txns.write()
            .map_err(|e| anyhow!("Lock error: {}", e))?;

        if let Some(state) = active_txns.get_mut(&txn_id) {
            state.write_set.insert(key);
            Ok(())
        } else {
            Err(anyhow!("Transaction {} not found", txn_id))
        }
    }

    /// Record a read operation
    pub fn record_read(&self, txn_id: u64, key: Vec<u8>) -> Result<()> {
        let mut active_txns = self.active_txns.write()
            .map_err(|e| anyhow!("Lock error: {}", e))?;

        if let Some(state) = active_txns.get_mut(&txn_id) {
            state.read_set.insert(key);
            Ok(())
        } else {
            Err(anyhow!("Transaction {} not found", txn_id))
        }
    }

    /// Commit a transaction
    pub fn commit(&self, txn_id: u64) -> Result<u64> {
        let mut active_txns = self.active_txns.write()
            .map_err(|e| anyhow!("Lock error: {}", e))?;

        let state = active_txns.get(&txn_id)
            .ok_or_else(|| anyhow!("Transaction {} not found", txn_id))?
            .clone();

        // Check for write conflicts (read-write transactions only)
        if state.mode == TransactionMode::ReadWrite && !state.write_set.is_empty() {
            drop(active_txns);
            self.check_conflicts(txn_id, &state.write_set, state.start_ts)?;
            active_txns = self.active_txns.write()
                .map_err(|e| anyhow!("Lock error: {}", e))?;
        }

        let commit_ts = self.next_txn_id.fetch_add(1, Ordering::SeqCst);

        if let Some(state) = active_txns.get_mut(&txn_id) {
            state.status = TxnStatus::Committed(commit_ts);
        }

        // Move to recent commits
        if !state.write_set.is_empty() {
            let mut recent = self.recent_commits.write()
                .map_err(|e| anyhow!("Lock error: {}", e))?;

            recent.push_back(CommittedTransaction {
                txn_id,
                commit_ts,
                write_set: state.write_set.clone(),
            });

            while recent.len() > MAX_RECENT_COMMITS {
                recent.pop_front();
            }
        }

        active_txns.remove(&txn_id);

        Ok(commit_ts)
    }

    /// Abort a transaction
    pub fn abort(&self, txn_id: u64) -> Result<()> {
        let mut active_txns = self.active_txns.write()
            .map_err(|e| anyhow!("Lock error: {}", e))?;

        if let Some(state) = active_txns.get_mut(&txn_id) {
            state.status = TxnStatus::Aborted;
        }

        active_txns.remove(&txn_id);
        Ok(())
    }

    /// Check for write conflicts
    fn check_conflicts(&self, txn_id: u64, write_set: &HashSet<Vec<u8>>, start_ts: u64) -> Result<()> {
        let recent = self.recent_commits.read()
            .map_err(|e| anyhow!("Lock error: {}", e))?;

        for committed in recent.iter() {
            if committed.commit_ts <= start_ts {
                continue;
            }

            if !committed.write_set.is_disjoint(write_set) {
                return Err(anyhow!(
                    "Transaction {} conflicts with transaction {} (committed at {})",
                    txn_id,
                    committed.txn_id,
                    committed.commit_ts
                ));
            }
        }

        Ok(())
    }

    /// Get number of active transactions
    pub fn active_count(&self) -> usize {
        self.active_txns.read()
            .map(|txns| txns.len())
            .unwrap_or(0)
    }

    /// Calculate GC watermark
    pub fn calculate_gc_watermark(&self) -> u64 {
        let active = self.active_txns.read()
            .expect("Lock error");

        if active.is_empty() {
            return self.next_txn_id.load(Ordering::SeqCst);
        }

        active.values()
            .map(|state| state.start_ts)
            .min()
            .unwrap_or(0)
    }

    /// Update GC watermark
    pub fn update_gc_watermark(&self) {
        let watermark = self.calculate_gc_watermark();
        self.gc_watermark.store(watermark, Ordering::SeqCst);
    }

    /// Get GC watermark
    pub fn gc_watermark(&self) -> u64 {
        self.gc_watermark.load(Ordering::SeqCst)
    }
}

impl Default for TransactionOracle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oracle_begin() {
        let oracle = TransactionOracle::new();
        let txn1 = oracle.begin(TransactionMode::ReadWrite).unwrap();
        assert_eq!(txn1, 1);

        let txn2 = oracle.begin(TransactionMode::ReadWrite).unwrap();
        assert_eq!(txn2, 2);
        assert_eq!(oracle.active_count(), 2);
    }

    #[test]
    fn test_oracle_commit() {
        let oracle = TransactionOracle::new();
        let txn1 = oracle.begin(TransactionMode::ReadWrite).unwrap();
        oracle.record_write(txn1, b"key1".to_vec()).unwrap();

        let commit_ts = oracle.commit(txn1).unwrap();
        assert!(commit_ts > txn1);
        assert_eq!(oracle.active_count(), 0);
    }

    #[test]
    fn test_oracle_abort() {
        let oracle = TransactionOracle::new();
        let txn1 = oracle.begin(TransactionMode::ReadWrite).unwrap();
        oracle.abort(txn1).unwrap();
        assert_eq!(oracle.active_count(), 0);
    }

    #[test]
    fn test_snapshot_isolation() {
        let oracle = TransactionOracle::new();
        let txn1 = oracle.begin(TransactionMode::ReadWrite).unwrap();
        let snapshot1 = oracle.get_snapshot(txn1).unwrap();
        assert!(snapshot1.is_empty());

        let txn2 = oracle.begin(TransactionMode::ReadWrite).unwrap();
        let snapshot2 = oracle.get_snapshot(txn2).unwrap();
        assert_eq!(snapshot2, vec![txn1]);
    }

    #[test]
    fn test_write_conflict() {
        let oracle = TransactionOracle::new();
        let txn1 = oracle.begin(TransactionMode::ReadWrite).unwrap();
        let txn2 = oracle.begin(TransactionMode::ReadWrite).unwrap();

        oracle.record_write(txn1, b"x".to_vec()).unwrap();
        oracle.record_write(txn2, b"x".to_vec()).unwrap();

        oracle.commit(txn1).unwrap();
        assert!(oracle.commit(txn2).is_err());
    }

    #[test]
    fn test_no_conflict_different_keys() {
        let oracle = TransactionOracle::new();
        let txn1 = oracle.begin(TransactionMode::ReadWrite).unwrap();
        let txn2 = oracle.begin(TransactionMode::ReadWrite).unwrap();

        oracle.record_write(txn1, b"x".to_vec()).unwrap();
        oracle.record_write(txn2, b"y".to_vec()).unwrap();

        oracle.commit(txn1).unwrap();
        oracle.commit(txn2).unwrap();
    }

    #[test]
    fn test_readonly_no_conflict_check() {
        let oracle = TransactionOracle::new();
        let txn1 = oracle.begin(TransactionMode::ReadOnly).unwrap();
        oracle.commit(txn1).unwrap();
    }

    #[test]
    fn test_gc_watermark() {
        let oracle = TransactionOracle::new();
        let watermark1 = oracle.calculate_gc_watermark();
        assert!(watermark1 > 0);

        let txn1 = oracle.begin(TransactionMode::ReadWrite).unwrap();
        let watermark2 = oracle.calculate_gc_watermark();
        assert_eq!(watermark2, txn1);

        let txn2 = oracle.begin(TransactionMode::ReadWrite).unwrap();
        let watermark3 = oracle.calculate_gc_watermark();
        assert_eq!(watermark3, txn1);

        oracle.commit(txn1).unwrap();
        let watermark4 = oracle.calculate_gc_watermark();
        assert_eq!(watermark4, txn2);
    }
}
