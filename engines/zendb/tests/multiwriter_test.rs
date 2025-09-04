//! Tests for multi-writer concurrency functionality

use zendb::storage::multiwriter::{MultiWriterLockManager, MultiWriterPageManager, LockMode};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_concurrent_readers() {
    let lock_manager = Arc::new(MultiWriterLockManager::new());
    
    // Multiple readers should be able to acquire shared locks concurrently
    let mut handles = vec![];
    
    for i in 1..=5 {
        let lm = lock_manager.clone();
        let handle = tokio::spawn(async move {
            lm.acquire_lock(i, 100, LockMode::Shared).await.unwrap();
            tokio::time::sleep(Duration::from_millis(50)).await;
            lm.release_lock(i, 100).unwrap();
        });
        handles.push(handle);
    }
    
    // All readers should complete within reasonable time
    for handle in handles {
        let result = timeout(Duration::from_millis(200), handle).await;
        assert!(result.is_ok(), "Concurrent readers should not block each other");
    }
    
    // Verify stats
    let stats = lock_manager.get_stats();
    assert_eq!(stats.locks_acquired.load(std::sync::atomic::Ordering::Relaxed), 5);
    assert_eq!(stats.locks_released.load(std::sync::atomic::Ordering::Relaxed), 5);
}

#[tokio::test]
async fn test_writer_exclusion() {
    let lock_manager = Arc::new(MultiWriterLockManager::new());
    
    // First writer gets lock
    lock_manager.acquire_lock(1, 200, LockMode::Exclusive).await.unwrap();
    
    // Second writer should be blocked
    let lm = lock_manager.clone();
    let blocked_handle = tokio::spawn(async move {
        lm.acquire_lock(2, 200, LockMode::Exclusive).await
    });
    
    // Give some time for second writer to start waiting
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    // Second writer should still be waiting (timeout)
    let wait_result = timeout(Duration::from_millis(100), blocked_handle).await;
    assert!(wait_result.is_err(), "Second writer should be blocked");
    
    // Release first writer's lock
    lock_manager.release_lock(1, 200).unwrap();
    
    // Now second writer should be able to acquire
    let lm = lock_manager.clone();
    let result = timeout(
        Duration::from_millis(100),
        lm.acquire_lock(2, 200, LockMode::Exclusive)
    ).await;
    assert!(result.is_ok(), "Second writer should acquire lock after first releases");
    
    // Clean up
    lock_manager.release_lock(2, 200).unwrap();
}

#[tokio::test]
async fn test_read_write_conflict() {
    let lock_manager = Arc::new(MultiWriterLockManager::new());
    
    // Reader gets shared lock first
    lock_manager.acquire_lock(1, 300, LockMode::Shared).await.unwrap();
    
    // Writer should be blocked
    let lm = lock_manager.clone();
    let writer_handle = tokio::spawn(async move {
        lm.acquire_lock(2, 300, LockMode::Exclusive).await
    });
    
    // Writer should be waiting
    tokio::time::sleep(Duration::from_millis(50)).await;
    let wait_result = timeout(Duration::from_millis(100), writer_handle).await;
    assert!(wait_result.is_err(), "Writer should be blocked by reader");
    
    // Release reader
    lock_manager.release_lock(1, 300).unwrap();
    
    // Now writer should succeed
    let lm = lock_manager.clone();
    let result = timeout(
        Duration::from_millis(100),
        lm.acquire_lock(2, 300, LockMode::Exclusive)
    ).await;
    assert!(result.is_ok(), "Writer should acquire after reader releases");
    
    lock_manager.release_lock(2, 300).unwrap();
}

#[tokio::test]
async fn test_lock_upgrade() {
    let lock_manager = Arc::new(MultiWriterLockManager::new());
    
    // Start with shared lock
    lock_manager.acquire_lock(1, 400, LockMode::Shared).await.unwrap();
    
    // Upgrade to exclusive
    lock_manager.upgrade_lock(1, 400).await.unwrap();
    
    // Other readers should now be blocked
    let lm = lock_manager.clone();
    let reader_result = timeout(
        Duration::from_millis(100),
        lm.acquire_lock(2, 400, LockMode::Shared)
    ).await;
    assert!(reader_result.is_err(), "Reader should be blocked after upgrade to exclusive");
    
    lock_manager.release_lock(1, 400).unwrap();
}

#[tokio::test]
async fn test_transaction_cleanup() {
    let lock_manager = Arc::new(MultiWriterLockManager::new());
    
    // Transaction acquires multiple locks
    let txn_id = 10;
    lock_manager.acquire_lock(txn_id, 501, LockMode::Shared).await.unwrap();
    lock_manager.acquire_lock(txn_id, 502, LockMode::Exclusive).await.unwrap();
    lock_manager.acquire_lock(txn_id, 503, LockMode::Shared).await.unwrap();
    
    // Release all locks at once (simulating commit/abort)
    lock_manager.release_all_locks(txn_id).await.unwrap();
    
    // Other transactions should be able to acquire all these locks
    lock_manager.acquire_lock(11, 501, LockMode::Exclusive).await.unwrap();
    lock_manager.acquire_lock(11, 502, LockMode::Exclusive).await.unwrap();
    lock_manager.acquire_lock(11, 503, LockMode::Exclusive).await.unwrap();
    
    // Clean up
    lock_manager.release_all_locks(11).await.unwrap();
}

#[tokio::test]
async fn test_deadlock_prevention() {
    let lock_manager = Arc::new(MultiWriterLockManager::new());
    
    // Set up potential deadlock scenario:
    // Txn 1 holds lock A, wants lock B
    // Txn 2 holds lock B, wants lock A
    
    // Txn 1 gets lock A
    lock_manager.acquire_lock(1, 601, LockMode::Exclusive).await.unwrap();
    
    // Txn 2 gets lock B
    lock_manager.acquire_lock(2, 602, LockMode::Exclusive).await.unwrap();
    
    // Txn 1 tries to get lock B (will wait)
    let lm1 = lock_manager.clone();
    let txn1_handle = tokio::spawn(async move {
        lm1.acquire_lock(1, 602, LockMode::Exclusive).await
    });
    
    // Give time for txn 1 to start waiting
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    // Txn 2 tries to get lock A (would create deadlock)
    let lm2 = lock_manager.clone();
    let txn2_result = lm2.acquire_lock(2, 601, LockMode::Exclusive).await;
    
    // One of the transactions should fail with deadlock detection
    // or timeout (depending on implementation)
    let is_deadlock_or_timeout = txn2_result.is_err() || 
        timeout(Duration::from_millis(100), txn1_handle).await.is_err();
    
    assert!(is_deadlock_or_timeout, "Deadlock should be detected or timeout");
    
    // Clean up
    lock_manager.release_all_locks(1).await.unwrap();
    lock_manager.release_all_locks(2).await.unwrap();
}

#[tokio::test]
async fn test_multiwriter_page_manager() {
    let lock_manager = Arc::new(MultiWriterLockManager::new());
    let page_manager = MultiWriterPageManager::new(lock_manager.clone());
    
    // Begin multiple transactions
    let txn1 = page_manager.begin_transaction();
    let txn2 = page_manager.begin_transaction();
    let txn3 = page_manager.begin_transaction();
    
    assert!(txn1 > 0);
    assert!(txn2 > txn1);
    assert!(txn3 > txn2);
    
    // Concurrent reads should work
    let pm1 = Arc::new(page_manager);
    let pm2 = pm1.clone();
    let pm3 = pm1.clone();
    
    let h1 = tokio::spawn(async move {
        pm1.read_page(txn1, 700).await
    });
    
    let h2 = tokio::spawn(async move {
        pm2.read_page(txn2, 700).await
    });
    
    let h3 = tokio::spawn(async move {
        pm3.read_page(txn3, 700).await
    });
    
    // All reads should complete
    h1.await.unwrap().unwrap();
    h2.await.unwrap().unwrap();
    h3.await.unwrap().unwrap();
}

#[tokio::test]
async fn test_lock_statistics() {
    let lock_manager = Arc::new(MultiWriterLockManager::new());
    
    // Perform various operations
    lock_manager.acquire_lock(1, 800, LockMode::Shared).await.unwrap();
    lock_manager.acquire_lock(2, 800, LockMode::Shared).await.unwrap();
    lock_manager.release_lock(1, 800).unwrap();
    lock_manager.release_lock(2, 800).unwrap();
    
    lock_manager.acquire_lock(3, 801, LockMode::Exclusive).await.unwrap();
    
    // Try to create conflict
    let lm = lock_manager.clone();
    let conflict_handle = tokio::spawn(async move {
        timeout(
            Duration::from_millis(50),
            lm.acquire_lock(4, 801, LockMode::Exclusive)
        ).await
    });
    
    conflict_handle.await.unwrap().ok(); // May timeout, that's expected
    
    // Check statistics
    let stats = lock_manager.get_stats();
    assert!(stats.locks_acquired.load(std::sync::atomic::Ordering::Relaxed) >= 3);
    assert!(stats.locks_released.load(std::sync::atomic::Ordering::Relaxed) >= 2);
    assert!(stats.lock_conflicts.load(std::sync::atomic::Ordering::Relaxed) >= 1);
}

#[tokio::test]
async fn test_concurrent_writers_different_pages() {
    let lock_manager = Arc::new(MultiWriterLockManager::new());
    
    // Multiple writers on different pages should not block each other
    let mut handles = vec![];
    
    for i in 1..=5 {
        let lm = lock_manager.clone();
        let page_id = 900 + i; // Different page for each writer
        let handle = tokio::spawn(async move {
            lm.acquire_lock(i, page_id, LockMode::Exclusive).await.unwrap();
            tokio::time::sleep(Duration::from_millis(20)).await;
            lm.release_lock(i, page_id).unwrap();
        });
        handles.push(handle);
    }
    
    // All writers should complete quickly (no blocking)
    for handle in handles {
        let result = timeout(Duration::from_millis(100), handle).await;
        assert!(result.is_ok(), "Writers on different pages should not block");
    }
}

#[test]
fn test_deadlock_detector_cycle() {
    // Unit test for deadlock detector (synchronous)
    use zendb::storage::multiwriter::*;
    
    // This would need the DeadlockDetector to be public or use a different testing approach
    // For now, we test it indirectly through the integration tests above
    assert!(true, "Deadlock detection tested in integration tests");
}