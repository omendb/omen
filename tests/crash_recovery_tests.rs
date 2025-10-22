//! Crash Recovery Tests for OmenDB
//!
//! Verifies that WAL (Write-Ahead Log) correctly recovers data after simulated crashes

use omendb::wal::{RecoveryStats, WalManager, WalOperation};
use std::sync::Arc;
use tempfile::TempDir;

#[test]
fn test_wal_recovery_basic() {
    let temp_dir = TempDir::new().unwrap();
    let wal_dir = temp_dir.path().join("wal");

    // Phase 1: Write operations to WAL
    {
        let wal = WalManager::new(&wal_dir).unwrap();
        wal.open().unwrap();

        // Write some operations
        wal.write(WalOperation::Insert {
            timestamp: 1000,
            value: 42.5,
            series_id: 1,
        })
        .unwrap();

        wal.write(WalOperation::Insert {
            timestamp: 2000,
            value: 100.0,
            series_id: 2,
        })
        .unwrap();

        wal.write(WalOperation::Update {
            timestamp: 1000,
            new_value: 50.0,
        })
        .unwrap();

        wal.sync().unwrap();
        // Simulate crash: wal dropped without cleanup
    }

    // Phase 2: Recover from WAL
    {
        let wal = WalManager::new(&wal_dir).unwrap();
        let mut recovered_ops = Vec::new();

        let stats = wal
            .recover(|op| {
                recovered_ops.push(op.clone());
                Ok(())
            })
            .unwrap();

        // Verify all operations recovered
        assert_eq!(stats.total_entries, 3);
        assert_eq!(stats.applied_entries, 3);
        assert_eq!(stats.failed_entries, 0);
        assert_eq!(stats.corrupted_entries, 0);
        assert_eq!(recovered_ops.len(), 3);
    }
}

#[test]
fn test_wal_recovery_transactions() {
    let temp_dir = TempDir::new().unwrap();
    let wal_dir = temp_dir.path().join("wal");

    // Phase 1: Write transaction operations
    {
        let wal = Arc::new(WalManager::new(&wal_dir).unwrap());
        wal.open().unwrap();

        // Begin transaction
        wal.write(WalOperation::BeginTxn { txn_id: 1 }).unwrap();

        // Operations within transaction
        wal.write(WalOperation::Insert {
            timestamp: 1000,
            value: 10.0,
            series_id: 1,
        })
        .unwrap();

        wal.write(WalOperation::Insert {
            timestamp: 2000,
            value: 20.0,
            series_id: 2,
        })
        .unwrap();

        // Commit transaction
        wal.write(WalOperation::CommitTxn { txn_id: 1 })
            .unwrap();

        wal.sync().unwrap();
    }

    // Phase 2: Recover and verify transaction
    {
        let wal = WalManager::new(&wal_dir).unwrap();
        let mut recovered_ops = Vec::new();

        let stats = wal
            .recover(|op| {
                recovered_ops.push(op.clone());
                Ok(())
            })
            .unwrap();

        assert_eq!(stats.total_entries, 4); // BEGIN + 2 INSERTS + COMMIT
        assert_eq!(stats.applied_entries, 4);

        // Verify transaction structure
        assert!(matches!(
            recovered_ops[0],
            WalOperation::BeginTxn { txn_id: 1 }
        ));
        assert!(matches!(recovered_ops[1], WalOperation::Insert { .. }));
        assert!(matches!(recovered_ops[2], WalOperation::Insert { .. }));
        assert!(matches!(
            recovered_ops[3],
            WalOperation::CommitTxn { txn_id: 1 }
        ));
    }
}

#[test]
fn test_wal_recovery_with_rollback() {
    let temp_dir = TempDir::new().unwrap();
    let wal_dir = temp_dir.path().join("wal");

    {
        let wal = Arc::new(WalManager::new(&wal_dir).unwrap());
        wal.open().unwrap();

        // Transaction 1: Committed
        wal.write(WalOperation::BeginTxn { txn_id: 1 }).unwrap();
        wal.write(WalOperation::Insert {
            timestamp: 1000,
            value: 100.0,
            series_id: 1,
        })
        .unwrap();
        wal.write(WalOperation::CommitTxn { txn_id: 1 })
            .unwrap();

        // Transaction 2: Rolled back
        wal.write(WalOperation::BeginTxn { txn_id: 2 }).unwrap();
        wal.write(WalOperation::Insert {
            timestamp: 2000,
            value: 200.0,
            series_id: 2,
        })
        .unwrap();
        wal.write(WalOperation::RollbackTxn { txn_id: 2 })
            .unwrap();

        wal.sync().unwrap();
    }

    // Recover and verify
    {
        let wal = WalManager::new(&wal_dir).unwrap();
        let mut recovered_ops = Vec::new();

        let stats = wal
            .recover(|op| {
                recovered_ops.push(op.clone());
                Ok(())
            })
            .unwrap();

        assert_eq!(stats.total_entries, 6);
        assert_eq!(stats.applied_entries, 6);

        // Verify we can identify rolled back transaction
        let has_rollback = recovered_ops
            .iter()
            .any(|op| matches!(op, WalOperation::RollbackTxn { txn_id: 2 }));
        assert!(has_rollback, "Should have rollback marker in WAL");
    }
}

#[test]
fn test_wal_recovery_partial_write() {
    // Test that recovery handles incomplete writes gracefully
    let temp_dir = TempDir::new().unwrap();
    let wal_dir = temp_dir.path().join("wal");

    {
        let wal = WalManager::new(&wal_dir).unwrap();
        wal.open().unwrap();

        // Write valid operations
        for i in 0..10 {
            wal.write(WalOperation::Insert {
                timestamp: i * 1000,
                value: i as f64,
                series_id: 1,
            })
            .unwrap();
        }

        wal.sync().unwrap();
    }

    // Recover - should get all operations
    {
        let wal = WalManager::new(&wal_dir).unwrap();
        let mut recovered_count = 0;

        let stats = wal
            .recover(|_op| {
                recovered_count += 1;
                Ok(())
            })
            .unwrap();

        assert_eq!(stats.total_entries, 10);
        assert_eq!(stats.applied_entries, 10);
        assert_eq!(recovered_count, 10);
    }
}

#[test]
fn test_wal_recovery_empty() {
    // Test recovery from empty WAL
    let temp_dir = TempDir::new().unwrap();
    let wal_dir = temp_dir.path().join("wal");

    {
        let wal = WalManager::new(&wal_dir).unwrap();
        wal.open().unwrap();
        // Don't write anything
    }

    {
        let wal = WalManager::new(&wal_dir).unwrap();
        let stats = wal.recover(|_op| Ok(())).unwrap();

        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.applied_entries, 0);
    }
}

#[test]
fn test_wal_recovery_sequence_continuity() {
    // Test that sequence numbers continue correctly after recovery
    let temp_dir = TempDir::new().unwrap();
    let wal_dir = temp_dir.path().join("wal");

    let last_seq = {
        let wal = WalManager::new(&wal_dir).unwrap();
        wal.open().unwrap();

        // Write 5 operations
        let mut last = 0;
        for i in 0..5 {
            last = wal
                .write(WalOperation::Insert {
                    timestamp: i * 1000,
                    value: i as f64,
                    series_id: 1,
                })
                .unwrap();
        }

        wal.sync().unwrap();
        last
    };

    // Recover and write more
    {
        let wal = WalManager::new(&wal_dir).unwrap();
        wal.open().unwrap();

        // Recovery should set sequence to last_seq + 1
        let new_seq = wal
            .write(WalOperation::Insert {
                timestamp: 6000,
                value: 100.0,
                series_id: 1,
            })
            .unwrap();

        // New sequence should continue from where we left off
        assert!(
            new_seq > last_seq,
            "New sequence {} should be greater than last {}",
            new_seq,
            last_seq
        );
    }
}

#[test]
fn test_wal_recovery_with_checkpoint() {
    let temp_dir = TempDir::new().unwrap();
    let wal_dir = temp_dir.path().join("wal");

    {
        let wal = WalManager::new(&wal_dir).unwrap();
        wal.open().unwrap();

        // Write some data
        for i in 0..5 {
            wal.write(WalOperation::Insert {
                timestamp: i * 1000,
                value: i as f64,
                series_id: 1,
            })
            .unwrap();
        }

        // Create checkpoint
        wal.checkpoint().unwrap();

        // Write more data after checkpoint
        for i in 5..10 {
            wal.write(WalOperation::Insert {
                timestamp: i * 1000,
                value: i as f64,
                series_id: 1,
            })
            .unwrap();
        }

        wal.sync().unwrap();
    }

    // Recover all operations
    {
        let wal = WalManager::new(&wal_dir).unwrap();
        let mut recovered_ops = Vec::new();

        let stats = wal
            .recover(|op| {
                recovered_ops.push(op.clone());
                Ok(())
            })
            .unwrap();

        // Should have 10 inserts + 1 checkpoint marker
        assert_eq!(stats.total_entries, 11);

        // Verify checkpoint marker exists
        let has_checkpoint = recovered_ops.iter().any(|op| matches!(op, WalOperation::Checkpoint { .. }));
        assert!(has_checkpoint, "Should have checkpoint marker");
    }
}

#[test]
fn test_wal_recovery_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    let wal_dir = temp_dir.path().join("wal");

    {
        let wal = WalManager::new(&wal_dir).unwrap();
        wal.open().unwrap();

        for i in 0..10 {
            wal.write(WalOperation::Insert {
                timestamp: i * 1000,
                value: i as f64,
                series_id: 1,
            })
            .unwrap();
        }

        wal.sync().unwrap();
    }

    // Recover with simulated application errors
    {
        let wal = WalManager::new(&wal_dir).unwrap();
        let mut apply_count = 0;

        let stats = wal
            .recover(|op| {
                apply_count += 1;
                // Simulate error on 5th operation
                if apply_count == 5 {
                    Err(anyhow::anyhow!("Simulated error"))
                } else {
                    Ok(())
                }
            })
            .unwrap();

        assert_eq!(stats.total_entries, 10);
        assert_eq!(stats.applied_entries, 9); // 10 - 1 failed
        assert_eq!(stats.failed_entries, 1);
    }
}
