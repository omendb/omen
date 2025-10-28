// Visibility Engine: Snapshot Isolation Rules
//
// Determines which versions of data are visible to a transaction based on:
// - Transaction snapshot (active transactions at begin time)
// - Version timestamps (begin_ts, end_ts)
// - Read-your-own-writes semantics
//
// Implements standard snapshot isolation visibility rules:
// 1. Can see versions created before snapshot AND not deleted before snapshot
// 2. Cannot see versions created by concurrent transactions
// 3. Can always see own writes (read-your-own-writes)

use crate::mvcc::{TransactionState, VersionedValue};

/// Visibility engine for snapshot isolation
///
/// Implements MVCC visibility rules to determine which versions
/// of data are visible to a given transaction.
pub struct VisibilityEngine;

impl VisibilityEngine {
    /// Check if a version is visible to a transaction
    ///
    /// A version is visible if ALL of the following are true:
    /// 1. Version was created before or at snapshot time (begin_ts <= snapshot_ts)
    /// 2. Version is not deleted OR was deleted after snapshot (end_ts = None OR end_ts > snapshot_ts)
    /// 3. Version was not created by a concurrent transaction still active at snapshot time
    ///
    /// Special case: Read-your-own-writes
    /// - If version was created by this transaction (begin_ts == txn_id), always visible
    ///
    /// # Arguments
    /// * `version` - The version to check visibility for
    /// * `txn_state` - Current transaction state (snapshot, txn_id)
    ///
    /// # Returns
    /// * `true` if version is visible to this transaction
    /// * `false` if version should not be visible
    pub fn is_visible(version: &VersionedValue, txn_state: &TransactionState) -> bool {
        let txn_id = txn_state.txn_id;
        let snapshot_ts = txn_state.start_ts;
        let snapshot = &txn_state.snapshot;

        // Rule 1: Read-your-own-writes (highest priority)
        if version.begin_ts == txn_id {
            // This transaction created this version - always visible
            // (even if not yet committed)
            return true;
        }

        // Rule 2: Version must have been created before or at snapshot time
        if version.begin_ts > snapshot_ts {
            // Version created after snapshot - not visible
            return false;
        }

        // Rule 3: Version must not be deleted, or deleted after snapshot
        if let Some(end_ts) = version.end_ts {
            if end_ts <= snapshot_ts {
                // Version was deleted before snapshot - not visible
                return false;
            }
        }

        // Rule 4: Version must not be created by a concurrent transaction
        // A transaction is concurrent if it was active at snapshot time
        if snapshot.contains(&version.begin_ts) {
            // Version created by a concurrent transaction - not visible
            return false;
        }

        // All visibility rules passed - version is visible!
        true
    }

    /// Find the visible version from a list of versions
    ///
    /// Scans through versions (assumed to be ordered newest first)
    /// and returns the first visible version.
    ///
    /// # Arguments
    /// * `versions` - List of versions, ordered newest to oldest
    /// * `txn_state` - Current transaction state
    ///
    /// # Returns
    /// * `Some(value)` - The visible version's value
    /// * `None` - No visible version found
    pub fn find_visible_version(
        versions: &[(u64, VersionedValue)],
        txn_state: &TransactionState,
    ) -> Option<Vec<u8>> {
        for (_txn_id, version) in versions {
            if Self::is_visible(version, txn_state) {
                return Some(version.value.clone());
            }
        }
        None
    }

    /// Check if a key has any visible version
    ///
    /// Similar to find_visible_version but returns boolean instead of value.
    /// Useful for existence checks.
    pub fn has_visible_version(
        versions: &[(u64, VersionedValue)],
        txn_state: &TransactionState,
    ) -> bool {
        versions.iter()
            .any(|(_txn_id, version)| Self::is_visible(version, txn_state))
    }

    /// Get all visible versions (for debugging/testing)
    ///
    /// Returns all versions that are visible to the transaction,
    /// along with their transaction IDs.
    pub fn get_all_visible(
        versions: &[(u64, VersionedValue)],
        txn_state: &TransactionState,
    ) -> Vec<(u64, Vec<u8>)> {
        versions.iter()
            .filter(|(_txn_id, version)| Self::is_visible(version, txn_state))
            .map(|(txn_id, version)| (*txn_id, version.value.clone()))
            .collect()
    }
}

/// Snapshot - A consistent view of the database at a point in time
///
/// Captures the state needed to determine visibility:
/// - snapshot_ts: Timestamp when snapshot was taken
/// - active_txns: Transactions that were active at snapshot time
/// - txn_id: ID of the transaction that owns this snapshot
pub struct Snapshot {
    pub txn_id: u64,
    pub snapshot_ts: u64,
    pub active_txns: Vec<u64>,
}

impl Snapshot {
    /// Create a new snapshot from transaction state
    pub fn from_transaction(txn_state: &TransactionState) -> Self {
        Self {
            txn_id: txn_state.txn_id,
            snapshot_ts: txn_state.start_ts,
            active_txns: txn_state.snapshot.clone(),
        }
    }

    /// Check if a version is visible in this snapshot
    pub fn is_visible(&self, version: &VersionedValue) -> bool {
        // Convert snapshot to transaction state for visibility check
        let txn_state = TransactionState {
            txn_id: self.txn_id,
            start_ts: self.snapshot_ts,
            snapshot: self.active_txns.clone(),
            write_set: Default::default(),
            read_set: Default::default(),
            status: crate::mvcc::TxnStatus::Active,
            mode: crate::mvcc::TransactionMode::ReadOnly,
        };

        VisibilityEngine::is_visible(version, &txn_state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mvcc::{TransactionMode, TxnStatus};
    use std::collections::HashSet;

    fn make_txn_state(txn_id: u64, snapshot_ts: u64, snapshot: Vec<u64>) -> TransactionState {
        TransactionState {
            txn_id,
            start_ts: snapshot_ts,
            snapshot,
            write_set: HashSet::new(),
            read_set: HashSet::new(),
            status: TxnStatus::Active,
            mode: TransactionMode::ReadWrite,
        }
    }

    fn make_version(begin_ts: u64, end_ts: Option<u64>, value: &[u8]) -> VersionedValue {
        VersionedValue {
            value: value.to_vec(),
            begin_ts,
            end_ts,
        }
    }

    #[test]
    fn test_basic_visibility() {
        // Transaction T2 starts at ts=10
        let txn_state = make_txn_state(10, 10, vec![]);

        // Version created at ts=5 (before snapshot) - should be visible
        let v1 = make_version(5, None, b"v1");
        assert!(VisibilityEngine::is_visible(&v1, &txn_state));

        // Version created at ts=15 (after snapshot) - should NOT be visible
        let v2 = make_version(15, None, b"v2");
        assert!(!VisibilityEngine::is_visible(&v2, &txn_state));
    }

    #[test]
    fn test_deleted_version_visibility() {
        let txn_state = make_txn_state(10, 10, vec![]);

        // Version created at ts=5, deleted at ts=8 (before snapshot) - NOT visible
        let v1 = make_version(5, Some(8), b"v1");
        assert!(!VisibilityEngine::is_visible(&v1, &txn_state));

        // Version created at ts=5, deleted at ts=12 (after snapshot) - visible
        let v2 = make_version(5, Some(12), b"v2");
        assert!(VisibilityEngine::is_visible(&v2, &txn_state));

        // Version created at ts=5, not deleted - visible
        let v3 = make_version(5, None, b"v3");
        assert!(VisibilityEngine::is_visible(&v3, &txn_state));
    }

    #[test]
    fn test_read_your_own_writes() {
        let txn_state = make_txn_state(10, 10, vec![]);

        // Version created by this transaction (begin_ts = txn_id) - always visible
        let v1 = make_version(10, None, b"my_write");
        assert!(VisibilityEngine::is_visible(&v1, &txn_state));

        // Even if marked as deleted by same transaction - still visible
        // (transaction hasn't committed yet, so it can still see its own writes)
        let v2 = make_version(10, Some(10), b"my_deleted");
        assert!(VisibilityEngine::is_visible(&v2, &txn_state));
    }

    #[test]
    fn test_concurrent_transaction_invisible() {
        // T2 starts at ts=10, T1 (ts=5) is still active
        let txn_state = make_txn_state(10, 10, vec![5]);

        // Version created by concurrent transaction T1 - NOT visible
        let v1 = make_version(5, None, b"concurrent");
        assert!(!VisibilityEngine::is_visible(&v1, &txn_state));

        // Version created by committed transaction (not in snapshot) - visible
        let v2 = make_version(3, None, b"committed");
        assert!(VisibilityEngine::is_visible(&v2, &txn_state));
    }

    #[test]
    fn test_snapshot_isolation_anomaly_prevention() {
        // Classic snapshot isolation test: two concurrent transactions

        // T1 starts at ts=10
        let t1 = make_txn_state(10, 10, vec![]);

        // T2 starts at ts=20, T1 still active
        let t2 = make_txn_state(20, 20, vec![10]);

        // Initial version at ts=5
        let v_initial = make_version(5, None, b"initial");

        // T1 writes at ts=10
        let v_t1 = make_version(10, None, b"t1_write");

        // T2 writes at ts=20
        let v_t2 = make_version(20, None, b"t2_write");

        // T1 can see initial and its own write
        assert!(VisibilityEngine::is_visible(&v_initial, &t1));
        assert!(VisibilityEngine::is_visible(&v_t1, &t1));
        assert!(!VisibilityEngine::is_visible(&v_t2, &t1)); // Can't see T2

        // T2 cannot see T1's write (concurrent), can see initial
        assert!(VisibilityEngine::is_visible(&v_initial, &t2));
        assert!(!VisibilityEngine::is_visible(&v_t1, &t2)); // Can't see T1 (concurrent)
        assert!(VisibilityEngine::is_visible(&v_t2, &t2));  // Can see own write
    }

    #[test]
    fn test_find_visible_version() {
        let txn_state = make_txn_state(10, 10, vec![]);

        let versions = vec![
            (15, make_version(15, None, b"v3")),     // Too new
            (8, make_version(8, None, b"v2")),       // Visible
            (3, make_version(3, None, b"v1")),       // Also visible, but older
        ];

        // Should return v2 (first visible version)
        let result = VisibilityEngine::find_visible_version(&versions, &txn_state);
        assert_eq!(result, Some(b"v2".to_vec()));
    }

    #[test]
    fn test_find_visible_version_with_deletions() {
        let txn_state = make_txn_state(10, 10, vec![]);

        let versions = vec![
            (15, make_version(15, None, b"v3")),           // Too new
            (8, make_version(8, Some(9), b"v2")),          // Deleted before snapshot
            (3, make_version(3, None, b"v1")),             // Visible
        ];

        // Should skip v3 (too new) and v2 (deleted), return v1
        let result = VisibilityEngine::find_visible_version(&versions, &txn_state);
        assert_eq!(result, Some(b"v1".to_vec()));
    }

    #[test]
    fn test_has_visible_version() {
        let txn_state = make_txn_state(10, 10, vec![]);

        let versions_visible = vec![
            (5, make_version(5, None, b"v1")),
        ];

        let versions_not_visible = vec![
            (15, make_version(15, None, b"v1")),
        ];

        assert!(VisibilityEngine::has_visible_version(&versions_visible, &txn_state));
        assert!(!VisibilityEngine::has_visible_version(&versions_not_visible, &txn_state));
    }

    #[test]
    fn test_get_all_visible() {
        let txn_state = make_txn_state(10, 10, vec![]);

        let versions = vec![
            (15, make_version(15, None, b"v4")),     // Not visible (too new)
            (8, make_version(8, None, b"v3")),       // Visible
            (5, make_version(5, None, b"v2")),       // Visible
            (3, make_version(3, Some(4), b"v1")),    // Not visible (deleted)
        ];

        let visible = VisibilityEngine::get_all_visible(&versions, &txn_state);
        assert_eq!(visible.len(), 2);
        assert_eq!(visible[0], (8, b"v3".to_vec()));
        assert_eq!(visible[1], (5, b"v2".to_vec()));
    }

    #[test]
    fn test_snapshot_wrapper() {
        let txn_state = make_txn_state(10, 10, vec![5]);
        let snapshot = Snapshot::from_transaction(&txn_state);

        let v1 = make_version(5, None, b"concurrent");
        let v2 = make_version(3, None, b"committed");

        assert!(!snapshot.is_visible(&v1)); // Concurrent transaction
        assert!(snapshot.is_visible(&v2));  // Committed before snapshot
    }

    #[test]
    fn test_exact_snapshot_boundary() {
        // Transaction starts at ts=10
        let txn_state = make_txn_state(10, 10, vec![]);

        // Version created at exactly snapshot_ts (begin_ts = snapshot_ts)
        // This should be visible (begin_ts <= snapshot_ts)
        let v1 = make_version(10, None, b"boundary");
        assert!(VisibilityEngine::is_visible(&v1, &txn_state));

        // Version deleted at exactly snapshot_ts (end_ts = snapshot_ts)
        // This should NOT be visible (end_ts <= snapshot_ts)
        let v2 = make_version(5, Some(10), b"deleted_at_boundary");
        assert!(!VisibilityEngine::is_visible(&v2, &txn_state));
    }

    #[test]
    fn test_multiple_active_transactions() {
        // T4 starts at ts=40, T1, T2, T3 still active
        let txn_state = make_txn_state(40, 40, vec![10, 20, 30]);

        let versions = vec![
            (35, make_version(35, None, b"v5")),  // Committed before T4, visible
            (30, make_version(30, None, b"v4")),  // Concurrent (T3), not visible
            (20, make_version(20, None, b"v3")),  // Concurrent (T2), not visible
            (10, make_version(10, None, b"v2")),  // Concurrent (T1), not visible
            (5, make_version(5, None, b"v1")),    // Committed, visible
        ];

        // v5 (ts=35) and v1 (ts=5) should be visible
        // v5 is visible because it committed before T4 started (not in concurrent list)
        // v1 is visible because it committed before all active transactions
        let visible = VisibilityEngine::get_all_visible(&versions, &txn_state);
        assert_eq!(visible.len(), 2);
        assert_eq!(visible[0], (35, b"v5".to_vec()));
        assert_eq!(visible[1], (5, b"v1".to_vec()));
    }

    #[test]
    fn test_write_after_read_scenario() {
        // T1 reads at ts=10, then writes at ts=10
        let txn_state = make_txn_state(10, 10, vec![]);

        // Initial value written at ts=5
        let v_old = make_version(5, None, b"old");

        // T1's own write at ts=10
        let v_new = make_version(10, None, b"new");

        // T1 should see both versions
        // (old from snapshot, new from read-your-own-writes)
        assert!(VisibilityEngine::is_visible(&v_old, &txn_state));
        assert!(VisibilityEngine::is_visible(&v_new, &txn_state));

        // When finding visible version, should prefer newer (own write)
        let versions = vec![
            (10, v_new.clone()),
            (5, v_old.clone()),
        ];
        let result = VisibilityEngine::find_visible_version(&versions, &txn_state);
        assert_eq!(result, Some(b"new".to_vec()));
    }
}
