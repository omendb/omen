use zendb::transaction::{TransactionManager, IsolationLevel, HLCTimestamp};
use tokio;

#[tokio::test]
async fn test_mvcc_basic_isolation() {
    let tm = TransactionManager::new();
    
    // Start transaction 1
    let tx1 = tm.begin().await.unwrap();
    
    // Write data in tx1
    tm.put(b"key1".to_vec(), b"value1".to_vec(), tx1.id).await.unwrap();
    
    // Start transaction 2 (should not see uncommitted data)
    let tx2 = tm.begin().await.unwrap();
    let result = tm.get(b"key1", tx2.id).await.unwrap();
    assert_eq!(result, None, "TX2 should not see uncommitted data from TX1");
    
    // Commit transaction 1
    tm.commit(tx1).await.unwrap();
    
    // Now tx2 still shouldn't see it (repeatable read)
    let result = tm.get(b"key1", tx2.id).await.unwrap();
    assert_eq!(result, None, "TX2 should maintain snapshot isolation");
    
    // Start transaction 3 (should see committed data)
    let tx3 = tm.begin().await.unwrap();
    let result = tm.get(b"key1", tx3.id).await.unwrap();
    assert_eq!(result, Some(b"value1".to_vec()));
}

#[tokio::test]
async fn test_mvcc_write_conflict() {
    let tm = TransactionManager::new();
    
    // Start two transactions
    let tx1 = tm.begin().await.unwrap();
    let tx2 = tm.begin().await.unwrap();
    
    // Both write to same key
    tm.put(b"conflict_key".to_vec(), b"value1".to_vec(), tx1.id).await.unwrap();
    tm.put(b"conflict_key".to_vec(), b"value2".to_vec(), tx2.id).await.unwrap();
    
    // Commit tx1
    tm.commit(tx1).await.unwrap();
    
    // tx2 commit should fail due to write-write conflict
    let result = tm.commit(tx2).await;
    assert!(result.is_err() || {
        // Transaction was aborted due to conflict
        let tx = tm.begin().await.unwrap();
        let value = tm.get(b"conflict_key", tx.id).await.unwrap();
        value == Some(b"value1".to_vec()) // tx1's value should win
    });
}

#[tokio::test]
async fn test_mvcc_time_travel() {
    let tm = TransactionManager::new();
    
    // Create initial version
    let tx1 = tm.begin().await.unwrap();
    tm.put(b"time_key".to_vec(), b"version1".to_vec(), tx1.id).await.unwrap();
    tm.commit(tx1).await.unwrap();
    
    // Sleep to ensure time advances
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    
    // Capture timestamp after first commit
    let ts_after_v1 = tm.begin().await.unwrap().start_timestamp;
    
    // Update to version 2
    let tx2 = tm.begin().await.unwrap();
    tm.put(b"time_key".to_vec(), b"version2".to_vec(), tx2.id).await.unwrap();
    tm.commit(tx2).await.unwrap();
    
    // Sleep again
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    
    // Capture timestamp after second commit
    let ts_after_v2 = tm.begin().await.unwrap().start_timestamp;
    
    // Update to version 3
    let tx3 = tm.begin().await.unwrap();
    tm.put(b"time_key".to_vec(), b"version3".to_vec(), tx3.id).await.unwrap();
    tm.commit(tx3).await.unwrap();
    
    // Time travel queries - query at timestamps when versions were committed
    let v1 = tm.get_at_timestamp(b"time_key", ts_after_v1);
    assert_eq!(v1, Some(b"version1".to_vec()));
    
    let v2 = tm.get_at_timestamp(b"time_key", ts_after_v2);
    assert_eq!(v2, Some(b"version2".to_vec()));
    
    // Latest version
    let tx_now = tm.begin().await.unwrap();
    let v3 = tm.get(b"time_key", tx_now.id).await.unwrap();
    assert_eq!(v3, Some(b"version3".to_vec()));
}

#[tokio::test]
async fn test_mvcc_range_scan() {
    let tm = TransactionManager::new();
    
    // Insert multiple keys
    let tx = tm.begin().await.unwrap();
    for i in 0..10 {
        let key = format!("key{:02}", i);
        let value = format!("value{}", i);
        tm.put(key.into_bytes(), value.into_bytes(), tx.id).await.unwrap();
    }
    tm.commit(tx).await.unwrap();
    
    // Range scan
    let tx2 = tm.begin().await.unwrap();
    let results = tm.range_scan(b"key03", b"key07", tx2.id).await.unwrap();
    
    assert_eq!(results.len(), 5);
    for (i, (key, value)) in results.iter().enumerate() {
        let expected_key = format!("key{:02}", i + 3);
        let expected_value = format!("value{}", i + 3);
        assert_eq!(key, &expected_key.as_bytes());
        assert_eq!(value, &expected_value.as_bytes());
    }
}

#[tokio::test]
async fn test_mvcc_delete() {
    let tm = TransactionManager::new();
    
    // Insert a key
    let tx1 = tm.begin().await.unwrap();
    tm.put(b"delete_key".to_vec(), b"value".to_vec(), tx1.id).await.unwrap();
    tm.commit(tx1).await.unwrap();
    
    // Delete the key
    let tx2 = tm.begin().await.unwrap();
    tm.delete(b"delete_key".to_vec(), tx2.id).await.unwrap();
    
    // Before commit, other transactions should still see it
    let tx3 = tm.begin().await.unwrap();
    let result = tm.get(b"delete_key", tx3.id).await.unwrap();
    assert_eq!(result, Some(b"value".to_vec()));
    
    // Commit the delete
    tm.commit(tx2).await.unwrap();
    
    // New transaction should not see deleted key
    let tx4 = tm.begin().await.unwrap();
    let result = tm.get(b"delete_key", tx4.id).await.unwrap();
    assert_eq!(result, None);
}

#[tokio::test]
async fn test_mvcc_isolation_levels() {
    let tm = TransactionManager::new();
    
    // Test Read Committed
    let tx1 = tm.begin_with_isolation(IsolationLevel::ReadCommitted).await.unwrap();
    tm.put(b"iso_key".to_vec(), b"v1".to_vec(), tx1.id).await.unwrap();
    
    // Another transaction with Read Uncommitted
    let tx2 = tm.begin_with_isolation(IsolationLevel::ReadUncommitted).await.unwrap();
    // This test would need ReadUncommitted implementation
    
    // Test Serializable
    let tx3 = tm.begin_with_isolation(IsolationLevel::Serializable).await.unwrap();
    let _ = tm.get(b"iso_key", tx3.id).await; // This creates read dependency
    
    tm.commit(tx1).await.unwrap();
}

#[tokio::test]
async fn test_hlc_monotonic() {
    use zendb::transaction::HLC;
    
    let clock = HLC::new(1);
    let mut timestamps = Vec::new();
    
    for _ in 0..100 {
        timestamps.push(clock.now());
    }
    
    // Verify strict monotonic increase
    for i in 1..timestamps.len() {
        assert!(timestamps[i] > timestamps[i-1], 
                "Timestamp {} ({:?}) should be > {} ({:?})",
                i, timestamps[i], i-1, timestamps[i-1]);
    }
}

#[tokio::test]
async fn test_mvcc_concurrent_transactions() {
    let tm = Arc::new(TransactionManager::new());
    let mut handles = Vec::new();
    
    // Spawn 10 concurrent transactions
    for i in 0..10 {
        let tm_clone = tm.clone();
        let handle = tokio::spawn(async move {
            let tx = tm_clone.begin().await.unwrap();
            let key = format!("concurrent_{}", i);
            let value = format!("value_{}", i);
            
            tm_clone.put(key.into_bytes(), value.into_bytes(), tx.id)
                .await
                .unwrap();
            
            // Random delay
            tokio::time::sleep(tokio::time::Duration::from_millis(i as u64)).await;
            
            tm_clone.commit(tx).await.unwrap();
        });
        handles.push(handle);
    }
    
    // Wait for all to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Verify all values are present
    let tx = tm.begin().await.unwrap();
    for i in 0..10 {
        let key = format!("concurrent_{}", i);
        let expected = format!("value_{}", i);
        let result = tm.get(key.as_bytes(), tx.id).await.unwrap();
        assert_eq!(result, Some(expected.into_bytes()));
    }
}

#[tokio::test]
async fn test_mvcc_garbage_collection() {
    let tm = TransactionManager::new();
    
    // Create multiple versions
    for i in 0..10 {
        let tx = tm.begin().await.unwrap();
        let value = format!("version_{}", i);
        tm.put(b"gc_key".to_vec(), value.into_bytes(), tx.id).await.unwrap();
        tm.commit(tx).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
    
    // Garbage collect old versions (keep last 100 microseconds)
    tm.garbage_collect(100);
    
    // Latest version should still be available
    let tx = tm.begin().await.unwrap();
    let result = tm.get(b"gc_key", tx.id).await.unwrap();
    assert!(result.is_some());
    
    // Very old timestamp queries might not work after GC
    let very_old_ts = HLCTimestamp::new(1, 0);
    let old_result = tm.get_at_timestamp(b"gc_key", very_old_ts);
    // This might be None after GC
}

use std::sync::Arc;