// Conflict Detection: Write-Write Conflict Detection for Snapshot Isolation
//
// Implements first-committer-wins rule:
// - If two transactions modify the same key, first to commit wins
// - Second transaction gets a detailed conflict error
//
// Provides detailed conflict information for debugging and user feedback

use anyhow::{anyhow, Result};
use std::collections::HashSet;
use std::fmt;

/// Detailed information about a write-write conflict
#[derive(Debug, Clone, PartialEq)]
pub struct WriteConflict {
    /// Transaction that encountered the conflict (trying to commit)
    pub txn_id: u64,
    /// Transaction that committed first (winner)
    pub conflicting_txn_id: u64,
    /// When the conflicting transaction committed
    pub conflicting_commit_ts: u64,
    /// Keys that both transactions modified
    pub conflicting_keys: Vec<Vec<u8>>,
    /// When this transaction started (for context)
    pub start_ts: u64,
}

impl WriteConflict {
    /// Create a new write conflict
    pub fn new(
        txn_id: u64,
        conflicting_txn_id: u64,
        conflicting_commit_ts: u64,
        conflicting_keys: Vec<Vec<u8>>,
        start_ts: u64,
    ) -> Self {
        Self {
            txn_id,
            conflicting_txn_id,
            conflicting_commit_ts,
            conflicting_keys,
            start_ts,
        }
    }

    /// Get the number of conflicting keys
    pub fn conflict_count(&self) -> usize {
        self.conflicting_keys.len()
    }

    /// Check if a specific key is in the conflict set
    pub fn has_key(&self, key: &[u8]) -> bool {
        self.conflicting_keys.iter().any(|k| k == key)
    }

    /// Convert to anyhow::Error
    pub fn into_error(self) -> anyhow::Error {
        anyhow!("{}", self)
    }
}

impl fmt::Display for WriteConflict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Write conflict: Transaction {} conflicts with transaction {} \
             (committed at ts={}). Conflicting keys: {}. \
             This transaction started at ts={}, but transaction {} \
             modified the same data and committed first (first-committer-wins).",
            self.txn_id,
            self.conflicting_txn_id,
            self.conflicting_commit_ts,
            self.conflict_count(),
            self.start_ts,
            self.conflicting_txn_id
        )
    }
}

impl std::error::Error for WriteConflict {}

/// Detect write-write conflicts between transaction write sets
///
/// Implements first-committer-wins:
/// - Scans recently committed transactions
/// - Checks if any modified the same keys as current transaction
/// - Returns detailed conflict info if overlap found
pub struct ConflictDetector;

impl ConflictDetector {
    /// Detect conflicts between a transaction and recently committed transactions
    ///
    /// # Arguments
    /// * `txn_id` - Transaction trying to commit
    /// * `start_ts` - When this transaction started (snapshot timestamp)
    /// * `write_set` - Keys this transaction modified
    /// * `recent_commits` - Recently committed transactions (ordered by commit time)
    ///
    /// # Returns
    /// * `Ok(())` - No conflicts detected
    /// * `Err(WriteConflict)` - Conflict detected with details
    pub fn detect_conflicts(
        txn_id: u64,
        start_ts: u64,
        write_set: &HashSet<Vec<u8>>,
        recent_commits: &[(u64, u64, HashSet<Vec<u8>>)], // (txn_id, commit_ts, write_set)
    ) -> Result<(), WriteConflict> {
        for (committed_txn_id, commit_ts, committed_write_set) in recent_commits {
            // Only check transactions that committed after this transaction started
            if *commit_ts <= start_ts {
                continue;
            }

            // Find conflicting keys (intersection of write sets)
            let conflicting_keys: Vec<Vec<u8>> = write_set
                .intersection(committed_write_set)
                .cloned()
                .collect();

            if !conflicting_keys.is_empty() {
                return Err(WriteConflict::new(
                    txn_id,
                    *committed_txn_id,
                    *commit_ts,
                    conflicting_keys,
                    start_ts,
                ));
            }
        }

        Ok(())
    }

    /// Check if two write sets conflict
    ///
    /// Returns true if there's any overlap between the sets.
    /// This is a simpler API for quick conflict checks.
    pub fn has_conflict(set_a: &HashSet<Vec<u8>>, set_b: &HashSet<Vec<u8>>) -> bool {
        !set_a.is_disjoint(set_b)
    }

    /// Get all conflicting keys between two write sets
    pub fn get_conflicting_keys(
        set_a: &HashSet<Vec<u8>>,
        set_b: &HashSet<Vec<u8>>,
    ) -> Vec<Vec<u8>> {
        set_a.intersection(set_b).cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_write_set(keys: &[&[u8]]) -> HashSet<Vec<u8>> {
        keys.iter().map(|k| k.to_vec()).collect()
    }

    #[test]
    fn test_no_conflict() {
        let txn_id = 10;
        let start_ts = 10;
        let write_set = make_write_set(&[b"key1", b"key2"]);

        // Recent commits with different keys
        let recent_commits = vec![
            (5, 15, make_write_set(&[b"key3", b"key4"])),
            (6, 20, make_write_set(&[b"key5"])),
        ];

        let result = ConflictDetector::detect_conflicts(txn_id, start_ts, &write_set, &recent_commits);
        assert!(result.is_ok());
    }

    #[test]
    fn test_conflict_detected() {
        let txn_id = 10;
        let start_ts = 10;
        let write_set = make_write_set(&[b"key1", b"key2"]);

        // Recent commit with overlapping key
        let recent_commits = vec![
            (5, 15, make_write_set(&[b"key2", b"key3"])), // Conflicts on key2
        ];

        let result = ConflictDetector::detect_conflicts(txn_id, start_ts, &write_set, &recent_commits);
        assert!(result.is_err());

        let conflict = result.unwrap_err();
        assert_eq!(conflict.txn_id, 10);
        assert_eq!(conflict.conflicting_txn_id, 5);
        assert_eq!(conflict.conflicting_commit_ts, 15);
        assert_eq!(conflict.conflict_count(), 1);
        assert!(conflict.has_key(b"key2"));
    }

    #[test]
    fn test_multiple_conflicting_keys() {
        let txn_id = 10;
        let start_ts = 10;
        let write_set = make_write_set(&[b"key1", b"key2", b"key3"]);

        // Recent commit with multiple overlapping keys
        let recent_commits = vec![
            (5, 15, make_write_set(&[b"key2", b"key3", b"key4"])), // Conflicts on key2, key3
        ];

        let result = ConflictDetector::detect_conflicts(txn_id, start_ts, &write_set, &recent_commits);
        assert!(result.is_err());

        let conflict = result.unwrap_err();
        assert_eq!(conflict.conflict_count(), 2);
        assert!(conflict.has_key(b"key2"));
        assert!(conflict.has_key(b"key3"));
    }

    #[test]
    fn test_first_committer_wins() {
        let txn_id = 10;
        let start_ts = 10;
        let write_set = make_write_set(&[b"key1"]);

        // Two recent commits, both conflict
        let recent_commits = vec![
            (5, 15, make_write_set(&[b"key1"])), // First conflict
            (6, 20, make_write_set(&[b"key1"])), // Second conflict (later)
        ];

        let result = ConflictDetector::detect_conflicts(txn_id, start_ts, &write_set, &recent_commits);
        assert!(result.is_err());

        let conflict = result.unwrap_err();
        // Should report conflict with first committer (txn 5, commit_ts 15)
        assert_eq!(conflict.conflicting_txn_id, 5);
        assert_eq!(conflict.conflicting_commit_ts, 15);
    }

    #[test]
    fn test_ignore_commits_before_start() {
        let txn_id = 10;
        let start_ts = 10;
        let write_set = make_write_set(&[b"key1"]);

        // Commit before start_ts should be ignored
        let recent_commits = vec![
            (5, 8, make_write_set(&[b"key1"])), // Committed before start_ts=10
        ];

        let result = ConflictDetector::detect_conflicts(txn_id, start_ts, &write_set, &recent_commits);
        assert!(result.is_ok()); // No conflict (commit was before snapshot)
    }

    #[test]
    fn test_conflict_at_exact_boundary() {
        let txn_id = 10;
        let start_ts = 10;
        let write_set = make_write_set(&[b"key1"]);

        // Commit at exactly start_ts should be ignored (<=)
        let recent_commits = vec![
            (5, 10, make_write_set(&[b"key1"])),
        ];

        let result = ConflictDetector::detect_conflicts(txn_id, start_ts, &write_set, &recent_commits);
        assert!(result.is_ok()); // No conflict (boundary case)
    }

    #[test]
    fn test_has_conflict_helper() {
        let set_a = make_write_set(&[b"key1", b"key2"]);
        let set_b = make_write_set(&[b"key2", b"key3"]);
        let set_c = make_write_set(&[b"key4"]);

        assert!(ConflictDetector::has_conflict(&set_a, &set_b)); // Overlap on key2
        assert!(!ConflictDetector::has_conflict(&set_a, &set_c)); // No overlap
    }

    #[test]
    fn test_get_conflicting_keys_helper() {
        let set_a = make_write_set(&[b"key1", b"key2", b"key3"]);
        let set_b = make_write_set(&[b"key2", b"key3", b"key4"]);

        let conflicting = ConflictDetector::get_conflicting_keys(&set_a, &set_b);
        assert_eq!(conflicting.len(), 2);
        assert!(conflicting.contains(&b"key2".to_vec()));
        assert!(conflicting.contains(&b"key3".to_vec()));
    }

    #[test]
    fn test_write_conflict_display() {
        let conflict = WriteConflict::new(
            10,  // txn_id
            5,   // conflicting_txn_id
            15,  // conflicting_commit_ts
            vec![b"key1".to_vec()],
            10,  // start_ts
        );

        let msg = format!("{}", conflict);
        assert!(msg.contains("Transaction 10"));
        assert!(msg.contains("transaction 5"));
        assert!(msg.contains("ts=15"));
        assert!(msg.contains("first-committer-wins"));
    }

    #[test]
    fn test_conflict_into_error() {
        let conflict = WriteConflict::new(
            10,
            5,
            15,
            vec![b"key1".to_vec()],
            10,
        );

        let err = conflict.into_error();
        assert!(err.to_string().contains("Write conflict"));
    }

    #[test]
    fn test_multiple_recent_commits_no_conflict() {
        let txn_id = 100;
        let start_ts = 100;
        let write_set = make_write_set(&[b"key_a", b"key_b"]);

        // Many recent commits, none conflict
        let recent_commits = vec![
            (10, 105, make_write_set(&[b"key1", b"key2"])),
            (20, 110, make_write_set(&[b"key3"])),
            (30, 115, make_write_set(&[b"key4", b"key5"])),
            (40, 120, make_write_set(&[b"key6"])),
        ];

        let result = ConflictDetector::detect_conflicts(txn_id, start_ts, &write_set, &recent_commits);
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_write_sets() {
        let txn_id = 10;
        let start_ts = 10;
        let write_set = HashSet::new(); // Empty write set

        let recent_commits = vec![
            (5, 15, make_write_set(&[b"key1"])),
        ];

        let result = ConflictDetector::detect_conflicts(txn_id, start_ts, &write_set, &recent_commits);
        assert!(result.is_ok()); // No conflict (empty write set)
    }

    #[test]
    fn test_large_write_sets() {
        let txn_id = 10;
        let start_ts = 10;

        // Large write sets
        let keys_a: Vec<Vec<u8>> = (0..1000).map(|i| format!("key_a_{}", i).into_bytes()).collect();
        let keys_b: Vec<Vec<u8>> = (500..1500).map(|i| format!("key_a_{}", i).into_bytes()).collect();

        let write_set: HashSet<Vec<u8>> = keys_a.iter().cloned().collect();
        let committed_set: HashSet<Vec<u8>> = keys_b.iter().cloned().collect();

        let recent_commits = vec![(5, 15, committed_set)];

        let result = ConflictDetector::detect_conflicts(txn_id, start_ts, &write_set, &recent_commits);
        assert!(result.is_err());

        let conflict = result.unwrap_err();
        assert_eq!(conflict.conflict_count(), 500); // Overlap on keys 500-999
    }
}
