//! WAL (Write-Ahead Logging) tests for durability
//! Validates crash recovery and data persistence

use crate::wal::{WalManager, WalOperation, TransactionManager};
use crate::storage::ArrowStorage;
use crate::concurrent::ConcurrentOmenDB;
use tempfile::tempdir;
use std::sync::Arc;

#[test]
fn test_wal_basic_write_and_recovery() {
    let dir = tempdir().unwrap();
    let wal = WalManager::new(dir.path()).unwrap();
    wal.open().unwrap();

    // Write some operations
    let seq1 = wal.write(WalOperation::Insert {
        timestamp: 1000,
        value: 42.0,
        series_id: 1,
    }).unwrap();

    let seq2 = wal.write(WalOperation::Insert {
        timestamp: 2000,
        value: 43.0,
        series_id: 1,
    }).unwrap();

    assert_eq!(seq1, 0);
    assert_eq!(seq2, 1);

    // Sync and verify
    wal.sync().unwrap();
}

#[test]
fn test_storage_with_wal_recovery() {
    let dir = tempdir().unwrap();

    // Write data with WAL
    {
        let mut storage = ArrowStorage::with_persistence(&dir).unwrap();

        // Insert data
        for i in 0..100 {
            storage.insert(i * 1000, i as f64, 1).unwrap();
        }

        // Force sync
        storage.sync().unwrap();
    }

    // Simulate crash and recovery
    {
        let storage = ArrowStorage::with_persistence(&dir).unwrap();

        // Data should be recovered from WAL
        let results = storage.range_query(0, 100_000).unwrap();
        assert!(!results.is_empty(), "Data should be recovered from WAL");
    }
}

#[test]
fn test_concurrent_db_with_persistence() {
    let dir = tempdir().unwrap();

    // Create database with persistence
    let db = Arc::new(ConcurrentOmenDB::with_persistence(10_000, &dir).unwrap());

    // Insert data concurrently
    let mut handles = vec![];

    for thread_id in 0..5 {
        let db_clone = Arc::clone(&db);

        let handle = std::thread::spawn(move || {
            for i in 0..20 {
                let timestamp = (thread_id * 100 + i) * 1000;
                db_clone.insert(timestamp, i as f64, thread_id).unwrap();
            }
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Sync to disk
    db.sync().unwrap();

    // Verify data persisted
    let metrics = db.metrics();
    assert_eq!(metrics.write_count, 100);
}

#[test]
fn test_wal_checkpoint_and_rotation() {
    let dir = tempdir().unwrap();
    let wal = WalManager::new(dir.path()).unwrap();
    wal.open().unwrap();

    // Write data
    for i in 0..100 {
        wal.write(WalOperation::Insert {
            timestamp: i * 1000,
            value: i as f64,
            series_id: 1,
        }).unwrap();
    }

    // Create checkpoint
    wal.checkpoint().unwrap();

    // Write more data after checkpoint
    for i in 100..150 {
        wal.write(WalOperation::Insert {
            timestamp: i * 1000,
            value: i as f64,
            series_id: 1,
        }).unwrap();
    }

    // Recover and verify all data
    let mut recovered_count = 0;
    let stats = wal.recover(|_| {
        recovered_count += 1;
        Ok(())
    }).unwrap();

    assert!(recovered_count >= 50, "Should recover post-checkpoint data");
}

#[test]
fn test_transaction_commit_and_rollback() {
    let dir = tempdir().unwrap();
    let wal = Arc::new(WalManager::new(dir.path()).unwrap());
    wal.open().unwrap();

    let txn_mgr = TransactionManager::new(Arc::clone(&wal));

    // Test commit
    {
        let txn = txn_mgr.begin().unwrap();
        txn.write(WalOperation::Insert {
            timestamp: 1000,
            value: 42.0,
            series_id: 1,
        }).unwrap();
        txn.commit().unwrap();
    }

    // Test rollback
    {
        let txn = txn_mgr.begin().unwrap();
        txn.write(WalOperation::Insert {
            timestamp: 2000,
            value: 43.0,
            series_id: 2,
        }).unwrap();
        txn.rollback().unwrap();
    }

    // Recover and check operations
    let mut operations = Vec::new();
    wal.recover(|op| {
        operations.push(op.clone());
        Ok(())
    }).unwrap();

    // Should have begin, insert, commit for first txn
    // And begin, insert, rollback for second txn
    assert!(operations.len() >= 6);
}

#[test]
fn test_wal_corruption_handling() {
    let dir = tempdir().unwrap();
    let wal_path = dir.path();

    // Write valid data
    {
        let wal = WalManager::new(&wal_path).unwrap();
        wal.open().unwrap();

        for i in 0..10 {
            wal.write(WalOperation::Insert {
                timestamp: i * 1000,
                value: i as f64,
                series_id: 1,
            }).unwrap();
        }

        wal.sync().unwrap();
    }

    // Corrupt the WAL file by appending garbage
    {
        use std::fs::OpenOptions;
        use std::io::Write;

        let mut file = OpenOptions::new()
            .append(true)
            .open(wal_path.join("current.wal"))
            .unwrap();

        // Write corrupted data
        file.write_all(b"CORRUPTED_DATA_HERE").unwrap();
    }

    // Recovery should handle corruption gracefully
    {
        let wal = WalManager::new(&wal_path).unwrap();

        let mut recovered = 0;
        let stats = wal.recover(|_| {
            recovered += 1;
            Ok(())
        }).unwrap();

        // Should recover valid entries before corruption
        assert_eq!(recovered, 10, "Should recover valid entries");
        assert_eq!(stats.applied_entries, 10);
    }
}

#[test]
fn test_wal_cleanup_old_files() {
    let dir = tempdir().unwrap();
    let wal = WalManager::new(dir.path()).unwrap();
    wal.open().unwrap();

    // Create multiple checkpoints to generate archive files
    for checkpoint in 0..3 {
        for i in 0..10 {
            wal.write(WalOperation::Insert {
                timestamp: (checkpoint * 10 + i) * 1000,
                value: i as f64,
                series_id: checkpoint,
            }).unwrap();
        }
        wal.checkpoint().unwrap();
    }

    // Clean up files older than 0 days (all archives)
    let removed = wal.cleanup(0).unwrap();
    assert!(removed > 0, "Should remove old WAL files");
}

#[test]
fn test_concurrent_wal_writes() {
    let dir = tempdir().unwrap();
    let wal = Arc::new(WalManager::new(dir.path()).unwrap());
    wal.open().unwrap();

    let mut handles = vec![];

    // Spawn multiple threads writing to WAL
    for thread_id in 0..10 {
        let wal_clone = Arc::clone(&wal);

        let handle = std::thread::spawn(move || {
            for i in 0..100 {
                wal_clone.write(WalOperation::Insert {
                    timestamp: (thread_id * 1000 + i) as i64,
                    value: i as f64,
                    series_id: thread_id,
                }).unwrap();
            }
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Sync and verify
    wal.sync().unwrap();

    // Recover and count
    let mut count = 0;
    wal.recover(|_| {
        count += 1;
        Ok(())
    }).unwrap();

    assert_eq!(count, 1000, "Should have all 1000 operations");
}

#[test]
#[ignore] // Long running test
fn test_sustained_wal_operations() {
    use std::time::{Duration, Instant};

    let dir = tempdir().unwrap();
    let wal = WalManager::new(dir.path()).unwrap();
    wal.open().unwrap();

    let start = Instant::now();
    let mut operations = 0;

    // Run for 5 seconds
    while start.elapsed() < Duration::from_secs(5) {
        wal.write(WalOperation::Insert {
            timestamp: operations,
            value: operations as f64,
            series_id: 1,
        }).unwrap();

        operations += 1;

        // Checkpoint every 10000 operations
        if operations % 10000 == 0 {
            wal.checkpoint().unwrap();
        }
    }

    // Final sync
    wal.sync().unwrap();

    let ops_per_sec = operations as f64 / 5.0;
    println!("WAL sustained operations: {} ops/sec", ops_per_sec);

    // Should maintain good throughput
    assert!(ops_per_sec > 10000.0, "WAL performance degraded");
}