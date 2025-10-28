// MVCC Integration Tests: Comprehensive Testing of Snapshot Isolation
//
// Tests cover:
// - Concurrent transaction isolation
// - Write conflict detection and resolution
// - Read-your-own-writes semantics
// - Snapshot isolation anomaly prevention
// - Edge cases and boundary conditions
// - Stress testing with many concurrent transactions

use omen::mvcc::{
    MvccStorage, MvccTransactionContext, TransactionMode, TransactionOracle,
};
use rocksdb::{Options, DB};
use std::sync::Arc;
use tempfile::TempDir;

fn setup() -> (Arc<TransactionOracle>, Arc<MvccStorage>, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let mut opts = Options::default();
    opts.create_if_missing(true);

    let db = Arc::new(DB::open(&opts, temp_dir.path()).unwrap());
    let oracle = Arc::new(TransactionOracle::new());
    let storage = Arc::new(MvccStorage::new(db, oracle.clone()));

    (oracle, storage, temp_dir)
}

// ============================================================================
// Basic Isolation Tests
// ============================================================================

#[test]
fn test_basic_snapshot_isolation() {
    let (oracle, storage, _temp) = setup();

    // T1: Write initial value
    let mut t1 = MvccTransactionContext::new(oracle.clone(), storage.clone());
    t1.begin(TransactionMode::ReadWrite).unwrap();
    let key = 100i64.to_be_bytes().to_vec();
    t1.write(key.clone(), b"initial".to_vec()).unwrap();
    t1.commit().unwrap();

    // T2: Start and read
    let mut t2 = MvccTransactionContext::new(oracle.clone(), storage.clone());
    t2.begin(TransactionMode::ReadWrite).unwrap();
    let v1 = t2.read(&key).unwrap();
    assert_eq!(v1, Some(b"initial".to_vec()));

    // T3: Update value
    let mut t3 = MvccTransactionContext::new(oracle.clone(), storage.clone());
    t3.begin(TransactionMode::ReadWrite).unwrap();
    t3.write(key.clone(), b"updated".to_vec()).unwrap();
    t3.commit().unwrap();

    // T2: Should still see "initial" (snapshot isolation)
    let v2 = t2.read(&key).unwrap();
    assert_eq!(v2, Some(b"initial".to_vec()));

    t2.commit().unwrap();

    // T4: New transaction should see "updated"
    let mut t4 = MvccTransactionContext::new(oracle.clone(), storage.clone());
    t4.begin(TransactionMode::ReadOnly).unwrap();
    let v3 = t4.read(&key).unwrap();
    assert_eq!(v3, Some(b"updated".to_vec()));
    t4.commit().unwrap();
}

#[test]
fn test_write_write_conflict() {
    let (oracle, storage, _temp) = setup();
    let key = 100i64.to_be_bytes().to_vec();

    // T1: Write initial value
    let mut t1 = MvccTransactionContext::new(oracle.clone(), storage.clone());
    t1.begin(TransactionMode::ReadWrite).unwrap();
    t1.write(key.clone(), b"value1".to_vec()).unwrap();
    t1.commit().unwrap();

    // T2 and T3 start concurrently
    let mut t2 = MvccTransactionContext::new(oracle.clone(), storage.clone());
    let mut t3 = MvccTransactionContext::new(oracle.clone(), storage.clone());

    t2.begin(TransactionMode::ReadWrite).unwrap();
    t3.begin(TransactionMode::ReadWrite).unwrap();

    // Both write to same key
    t2.write(key.clone(), b"t2_value".to_vec()).unwrap();
    t3.write(key.clone(), b"t3_value".to_vec()).unwrap();

    // First to commit wins
    let commit2 = t2.commit();
    assert!(commit2.is_ok());

    // Second should fail with conflict
    let commit3 = t3.commit();
    assert!(commit3.is_err());
    assert!(commit3.unwrap_err().to_string().contains("conflict"));
}

#[test]
fn test_no_conflict_different_keys() {
    let (oracle, storage, _temp) = setup();

    let key1 = 100i64.to_be_bytes().to_vec();
    let key2 = 200i64.to_be_bytes().to_vec();

    // T1 and T2 start concurrently
    let mut t1 = MvccTransactionContext::new(oracle.clone(), storage.clone());
    let mut t2 = MvccTransactionContext::new(oracle.clone(), storage.clone());

    t1.begin(TransactionMode::ReadWrite).unwrap();
    t2.begin(TransactionMode::ReadWrite).unwrap();

    // Different keys - no conflict
    t1.write(key1, b"value1".to_vec()).unwrap();
    t2.write(key2, b"value2".to_vec()).unwrap();

    // Both should commit successfully
    assert!(t1.commit().is_ok());
    assert!(t2.commit().is_ok());
}

// ============================================================================
// Read-Your-Own-Writes Tests
// ============================================================================

#[test]
fn test_read_own_uncommitted_write() {
    let (oracle, storage, _temp) = setup();
    let mut t1 = MvccTransactionContext::new(oracle, storage);

    t1.begin(TransactionMode::ReadWrite).unwrap();
    let key = 100i64.to_be_bytes().to_vec();

    // Write
    t1.write(key.clone(), b"my_value".to_vec()).unwrap();

    // Read should see own write
    let result = t1.read(&key).unwrap();
    assert_eq!(result, Some(b"my_value".to_vec()));

    t1.commit().unwrap();
}

#[test]
fn test_read_own_updates() {
    let (oracle, storage, _temp) = setup();
    let mut t1 = MvccTransactionContext::new(oracle, storage);

    t1.begin(TransactionMode::ReadWrite).unwrap();
    let key = 100i64.to_be_bytes().to_vec();

    // Multiple updates
    t1.write(key.clone(), b"v1".to_vec()).unwrap();
    assert_eq!(t1.read(&key).unwrap(), Some(b"v1".to_vec()));

    t1.write(key.clone(), b"v2".to_vec()).unwrap();
    assert_eq!(t1.read(&key).unwrap(), Some(b"v2".to_vec()));

    t1.write(key.clone(), b"v3".to_vec()).unwrap();
    assert_eq!(t1.read(&key).unwrap(), Some(b"v3".to_vec()));

    t1.commit().unwrap();
}

#[test]
fn test_read_own_delete() {
    let (oracle, storage, _temp) = setup();

    // Setup: Write initial value
    let mut t1 = MvccTransactionContext::new(oracle.clone(), storage.clone());
    t1.begin(TransactionMode::ReadWrite).unwrap();
    let key = 100i64.to_be_bytes().to_vec();
    t1.write(key.clone(), b"exists".to_vec()).unwrap();
    t1.commit().unwrap();

    // Delete in new transaction
    let mut t2 = MvccTransactionContext::new(oracle, storage);
    t2.begin(TransactionMode::ReadWrite).unwrap();
    t2.delete(key.clone()).unwrap();

    // Should see deletion
    let result = t2.read(&key).unwrap();
    assert_eq!(result, Some(Vec::new())); // Empty value = tombstone

    t2.commit().unwrap();
}

// ============================================================================
// Multi-Transaction Scenarios
// ============================================================================

#[test]
fn test_three_concurrent_transactions() {
    let (oracle, storage, _temp) = setup();
    let key = 100i64.to_be_bytes().to_vec();

    // T1: Write initial
    let mut t1 = MvccTransactionContext::new(oracle.clone(), storage.clone());
    t1.begin(TransactionMode::ReadWrite).unwrap();
    t1.write(key.clone(), b"v0".to_vec()).unwrap();
    t1.commit().unwrap();

    // T2, T3, T4 start concurrently
    let mut t2 = MvccTransactionContext::new(oracle.clone(), storage.clone());
    let mut t3 = MvccTransactionContext::new(oracle.clone(), storage.clone());
    let mut t4 = MvccTransactionContext::new(oracle.clone(), storage.clone());

    t2.begin(TransactionMode::ReadWrite).unwrap();
    t3.begin(TransactionMode::ReadWrite).unwrap();
    t4.begin(TransactionMode::ReadWrite).unwrap();

    // All read initial value
    assert_eq!(t2.read(&key).unwrap(), Some(b"v0".to_vec()));
    assert_eq!(t3.read(&key).unwrap(), Some(b"v0".to_vec()));
    assert_eq!(t4.read(&key).unwrap(), Some(b"v0".to_vec()));

    // All try to write
    t2.write(key.clone(), b"t2".to_vec()).unwrap();
    t3.write(key.clone(), b"t3".to_vec()).unwrap();
    t4.write(key.clone(), b"t4".to_vec()).unwrap();

    // First wins, others fail
    let results = vec![t2.commit(), t3.commit(), t4.commit()];
    let successes = results.iter().filter(|r| r.is_ok()).count();
    let failures = results.iter().filter(|r| r.is_err()).count();

    assert_eq!(successes, 1);
    assert_eq!(failures, 2);
}

#[test]
fn test_read_only_sees_committed_data() {
    let (oracle, storage, _temp) = setup();
    let key = 100i64.to_be_bytes().to_vec();

    // T1: Write initial
    let mut t1 = MvccTransactionContext::new(oracle.clone(), storage.clone());
    t1.begin(TransactionMode::ReadWrite).unwrap();
    t1.write(key.clone(), b"initial".to_vec()).unwrap();
    t1.commit().unwrap();

    // T2: Read-only transaction
    let mut t2 = MvccTransactionContext::new(oracle.clone(), storage.clone());
    t2.begin(TransactionMode::ReadOnly).unwrap();

    // T3: Concurrent update
    let mut t3 = MvccTransactionContext::new(oracle.clone(), storage.clone());
    t3.begin(TransactionMode::ReadWrite).unwrap();
    t3.write(key.clone(), b"updated".to_vec()).unwrap();
    t3.commit().unwrap();

    // T2 should still see initial (snapshot)
    assert_eq!(t2.read(&key).unwrap(), Some(b"initial".to_vec()));
    t2.commit().unwrap();
}

// ============================================================================
// Delete and Tombstone Tests
// ============================================================================

#[test]
fn test_delete_creates_tombstone() {
    let (oracle, storage, _temp) = setup();
    let key = 100i64.to_be_bytes().to_vec();

    // Write initial value
    let mut t1 = MvccTransactionContext::new(oracle.clone(), storage.clone());
    t1.begin(TransactionMode::ReadWrite).unwrap();
    t1.write(key.clone(), b"exists".to_vec()).unwrap();
    t1.commit().unwrap();

    // Delete
    let mut t2 = MvccTransactionContext::new(oracle.clone(), storage.clone());
    t2.begin(TransactionMode::ReadWrite).unwrap();
    t2.delete(key.clone()).unwrap();
    t2.commit().unwrap();

    // Verify deleted
    let mut t3 = MvccTransactionContext::new(oracle, storage);
    t3.begin(TransactionMode::ReadOnly).unwrap();
    let result = t3.read(&key).unwrap();
    assert_eq!(result, None);
    t3.commit().unwrap();
}

#[test]
fn test_delete_conflict() {
    let (oracle, storage, _temp) = setup();
    let key = 100i64.to_be_bytes().to_vec();

    // Write initial value
    let mut t1 = MvccTransactionContext::new(oracle.clone(), storage.clone());
    t1.begin(TransactionMode::ReadWrite).unwrap();
    t1.write(key.clone(), b"exists".to_vec()).unwrap();
    t1.commit().unwrap();

    // Two concurrent deletes
    let mut t2 = MvccTransactionContext::new(oracle.clone(), storage.clone());
    let mut t3 = MvccTransactionContext::new(oracle.clone(), storage.clone());

    t2.begin(TransactionMode::ReadWrite).unwrap();
    t3.begin(TransactionMode::ReadWrite).unwrap();

    t2.delete(key.clone()).unwrap();
    t3.delete(key.clone()).unwrap();

    // First wins, second conflicts
    assert!(t2.commit().is_ok());
    assert!(t3.commit().is_err());
}

// ============================================================================
// Rollback Tests
// ============================================================================

#[test]
fn test_rollback_discards_writes() {
    let (oracle, storage, _temp) = setup();
    let key = 100i64.to_be_bytes().to_vec();

    // T1: Write and rollback
    let mut t1 = MvccTransactionContext::new(oracle.clone(), storage.clone());
    t1.begin(TransactionMode::ReadWrite).unwrap();
    t1.write(key.clone(), b"rolled_back".to_vec()).unwrap();
    t1.rollback().unwrap();

    // T2: Should not see rolled back write
    let mut t2 = MvccTransactionContext::new(oracle, storage);
    t2.begin(TransactionMode::ReadOnly).unwrap();
    let result = t2.read(&key).unwrap();
    assert_eq!(result, None);
    t2.commit().unwrap();
}

#[test]
fn test_rollback_after_conflict() {
    let (oracle, storage, _temp) = setup();
    let key = 100i64.to_be_bytes().to_vec();

    // T1 and T2 start concurrently
    let mut t1 = MvccTransactionContext::new(oracle.clone(), storage.clone());
    let mut t2 = MvccTransactionContext::new(oracle.clone(), storage.clone());

    t1.begin(TransactionMode::ReadWrite).unwrap();
    t2.begin(TransactionMode::ReadWrite).unwrap();

    // Both write to same key
    t1.write(key.clone(), b"t1".to_vec()).unwrap();
    t2.write(key.clone(), b"t2".to_vec()).unwrap();

    // T1 commits first
    t1.commit().unwrap();

    // T2 commit fails (conflict), transaction auto-rolled back
    let result = t2.commit();
    assert!(result.is_err());
    assert!(!t2.is_in_transaction()); // Should be rolled back
}

// ============================================================================
// Multiple Keys Tests
// ============================================================================

#[test]
fn test_multiple_key_writes() {
    let (oracle, storage, _temp) = setup();

    let mut t1 = MvccTransactionContext::new(oracle, storage);
    t1.begin(TransactionMode::ReadWrite).unwrap();

    // Write multiple keys
    for i in 0..100 {
        let key = (i as i64).to_be_bytes().to_vec();
        let value = format!("value_{}", i).into_bytes();
        t1.write(key, value).unwrap();
    }

    assert_eq!(t1.write_buffer_size(), 100);
    t1.commit().unwrap();
}

#[test]
fn test_partial_conflict() {
    let (oracle, storage, _temp) = setup();

    let key1 = 100i64.to_be_bytes().to_vec();
    let key2 = 200i64.to_be_bytes().to_vec();
    let key3 = 300i64.to_be_bytes().to_vec();

    // T1 and T2 concurrent
    let mut t1 = MvccTransactionContext::new(oracle.clone(), storage.clone());
    let mut t2 = MvccTransactionContext::new(oracle, storage);

    t1.begin(TransactionMode::ReadWrite).unwrap();
    t2.begin(TransactionMode::ReadWrite).unwrap();

    // T1: Write key1, key2
    t1.write(key1.clone(), b"t1".to_vec()).unwrap();
    t1.write(key2.clone(), b"t1".to_vec()).unwrap();

    // T2: Write key2, key3 (conflict on key2)
    t2.write(key2.clone(), b"t2".to_vec()).unwrap();
    t2.write(key3, b"t2".to_vec()).unwrap();

    // T1 commits first
    assert!(t1.commit().is_ok());

    // T2 should fail (conflict on key2)
    assert!(t2.commit().is_err());
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_empty_transaction() {
    let (oracle, storage, _temp) = setup();
    let mut t1 = MvccTransactionContext::new(oracle, storage);

    t1.begin(TransactionMode::ReadWrite).unwrap();
    // No operations
    assert!(t1.commit().is_ok());
}

#[test]
fn test_read_nonexistent_key() {
    let (oracle, storage, _temp) = setup();
    let mut t1 = MvccTransactionContext::new(oracle, storage);

    t1.begin(TransactionMode::ReadOnly).unwrap();
    let key = 999i64.to_be_bytes().to_vec();
    let result = t1.read(&key).unwrap();
    assert_eq!(result, None);
    t1.commit().unwrap();
}

#[test]
fn test_write_empty_value() {
    let (oracle, storage, _temp) = setup();
    let key = 100i64.to_be_bytes().to_vec();

    // First, create the key with a value
    let mut t1 = MvccTransactionContext::new(oracle.clone(), storage.clone());
    t1.begin(TransactionMode::ReadWrite).unwrap();
    t1.write(key.clone(), b"exists".to_vec()).unwrap();
    t1.commit().unwrap();

    // Then write empty value (treated as delete)
    let mut t2 = MvccTransactionContext::new(oracle.clone(), storage.clone());
    t2.begin(TransactionMode::ReadWrite).unwrap();
    t2.write(key.clone(), Vec::new()).unwrap();

    // Within transaction, should see own write (empty value)
    let result = t2.read(&key).unwrap();
    assert_eq!(result, Some(Vec::new()));

    t2.commit().unwrap();

    // After commit, empty value acts as delete, so should be None
    let mut t3 = MvccTransactionContext::new(oracle, storage);
    t3.begin(TransactionMode::ReadOnly).unwrap();
    let result2 = t3.read(&key).unwrap();
    assert_eq!(result2, None); // Empty value = tombstone
    t3.commit().unwrap();
}

#[test]
fn test_large_value() {
    let (oracle, storage, _temp) = setup();
    let mut t1 = MvccTransactionContext::new(oracle, storage);

    t1.begin(TransactionMode::ReadWrite).unwrap();
    let key = 100i64.to_be_bytes().to_vec();
    let large_value = vec![42u8; 100_000]; // 100KB

    t1.write(key.clone(), large_value.clone()).unwrap();
    let result = t1.read(&key).unwrap();
    assert_eq!(result, Some(large_value));

    t1.commit().unwrap();
}

// ============================================================================
// Anomaly Prevention Tests
// ============================================================================

#[test]
fn test_no_dirty_reads() {
    let (oracle, storage, _temp) = setup();
    let key = 100i64.to_be_bytes().to_vec();

    // T1: Start and write (but don't commit yet)
    let mut t1 = MvccTransactionContext::new(oracle.clone(), storage.clone());
    t1.begin(TransactionMode::ReadWrite).unwrap();
    t1.write(key.clone(), b"uncommitted".to_vec()).unwrap();

    // T2: Try to read T1's uncommitted write
    let mut t2 = MvccTransactionContext::new(oracle, storage);
    t2.begin(TransactionMode::ReadOnly).unwrap();
    let result = t2.read(&key).unwrap();

    // Should not see uncommitted data
    assert_eq!(result, None);

    t2.commit().unwrap();
    t1.rollback().unwrap();
}

#[test]
fn test_no_lost_updates() {
    let (oracle, storage, _temp) = setup();
    let key = 100i64.to_be_bytes().to_vec();

    // T1: Write initial
    let mut t1 = MvccTransactionContext::new(oracle.clone(), storage.clone());
    t1.begin(TransactionMode::ReadWrite).unwrap();
    t1.write(key.clone(), b"initial".to_vec()).unwrap();
    t1.commit().unwrap();

    // T2 and T3 concurrent (classic lost update scenario)
    let mut t2 = MvccTransactionContext::new(oracle.clone(), storage.clone());
    let mut t3 = MvccTransactionContext::new(oracle, storage);

    t2.begin(TransactionMode::ReadWrite).unwrap();
    t3.begin(TransactionMode::ReadWrite).unwrap();

    // Both read
    t2.read(&key).unwrap();
    t3.read(&key).unwrap();

    // Both write
    t2.write(key.clone(), b"t2_update".to_vec()).unwrap();
    t3.write(key.clone(), b"t3_update".to_vec()).unwrap();

    // First commits
    assert!(t2.commit().is_ok());

    // Second should fail (prevents lost update)
    assert!(t3.commit().is_err());
}

#[test]
fn test_repeatable_reads() {
    let (oracle, storage, _temp) = setup();
    let key = 100i64.to_be_bytes().to_vec();

    // T1: Write initial
    let mut t1 = MvccTransactionContext::new(oracle.clone(), storage.clone());
    t1.begin(TransactionMode::ReadWrite).unwrap();
    t1.write(key.clone(), b"v1".to_vec()).unwrap();
    t1.commit().unwrap();

    // T2: Start and read
    let mut t2 = MvccTransactionContext::new(oracle.clone(), storage.clone());
    t2.begin(TransactionMode::ReadOnly).unwrap();
    let read1 = t2.read(&key).unwrap();
    assert_eq!(read1, Some(b"v1".to_vec()));

    // T3: Update value
    let mut t3 = MvccTransactionContext::new(oracle, storage);
    t3.begin(TransactionMode::ReadWrite).unwrap();
    t3.write(key.clone(), b"v2".to_vec()).unwrap();
    t3.commit().unwrap();

    // T2: Read again - should see same value (repeatable read)
    let read2 = t2.read(&key).unwrap();
    assert_eq!(read2, Some(b"v1".to_vec()));

    t2.commit().unwrap();
}

// ============================================================================
// Stress Tests
// ============================================================================

#[test]
fn test_many_sequential_transactions() {
    let (oracle, storage, _temp) = setup();
    let key = 100i64.to_be_bytes().to_vec();

    // 100 sequential transactions
    for i in 0..100 {
        let mut tx = MvccTransactionContext::new(oracle.clone(), storage.clone());
        tx.begin(TransactionMode::ReadWrite).unwrap();
        let value = format!("value_{}", i).into_bytes();
        tx.write(key.clone(), value).unwrap();
        tx.commit().unwrap();
    }

    // Verify final value
    let mut verify = MvccTransactionContext::new(oracle, storage);
    verify.begin(TransactionMode::ReadOnly).unwrap();
    let result = verify.read(&key).unwrap();
    assert_eq!(result, Some(b"value_99".to_vec()));
    verify.commit().unwrap();
}

#[test]
fn test_many_keys_single_transaction() {
    let (oracle, storage, _temp) = setup();
    let mut tx = MvccTransactionContext::new(oracle, storage);

    tx.begin(TransactionMode::ReadWrite).unwrap();

    // 1000 keys
    for i in 0..1000 {
        let key = (i as i64).to_be_bytes().to_vec();
        let value = format!("val_{}", i).into_bytes();
        tx.write(key, value).unwrap();
    }

    assert_eq!(tx.write_buffer_size(), 1000);
    tx.commit().unwrap();
}
