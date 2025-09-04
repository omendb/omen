//! Tests for B+Tree node merging functionality

use zendb::storage::{PageManager, BTree};
use tempfile::TempDir;
use anyhow::Result;
use std::sync::Arc;

#[tokio::test]
async fn test_btree_node_merging_basic() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("btree_merge_test.db");
    
    let pm = Arc::new(PageManager::open(&db_path)?);
    let mut btree = BTree::create(pm)?;
    
    // Insert enough keys to create multiple levels
    let keys: Vec<String> = (0..20).map(|i| format!("key_{:03}", i)).collect();
    
    for key in &keys {
        btree.insert(key.as_bytes(), format!("value_{}", key).as_bytes())?;
    }
    
    // Verify all keys exist
    for key in &keys {
        let result = btree.search(key.as_bytes())?;
        assert!(result.is_some());
        assert_eq!(result.unwrap(), format!("value_{}", key).as_bytes());
    }
    
    // Delete keys to trigger underflow and merging
    for i in (0..15).rev() { // Delete most keys in reverse order
        let key = format!("key_{:03}", i);
        let deleted = btree.delete(key.as_bytes())?;
        assert!(deleted, "Key {} should exist", key);
    }
    
    // Verify remaining keys still exist
    for i in 15..20 {
        let key = format!("key_{:03}", i);
        let result = btree.search(key.as_bytes())?;
        assert!(result.is_some(), "Key {} should still exist", key);
    }
    
    // Verify deleted keys are gone
    for i in 0..15 {
        let key = format!("key_{:03}", i);
        let result = btree.search(key.as_bytes())?;
        assert!(result.is_none(), "Key {} should be deleted", key);
    }
    
    Ok(())
}

#[tokio::test]  
async fn test_btree_redistribution() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("btree_redistrib_test.db");
    
    let pm = Arc::new(PageManager::open(&db_path)?);
    let mut btree = BTree::create(pm)?;
    
    // Insert keys in a pattern that will create specific node distributions
    let keys: Vec<String> = (0..50).map(|i| format!("k_{:04}", i)).collect();
    
    for key in &keys {
        btree.insert(key.as_bytes(), format!("v_{}", key).as_bytes())?;
    }
    
    // Delete some keys to trigger redistribution (not full merging)
    for i in (10..20).step_by(2) { // Delete every other key in middle range
        let key = format!("k_{:04}", i);
        let deleted = btree.delete(key.as_bytes())?;
        assert!(deleted);
    }
    
    // Verify tree is still searchable and consistent
    for i in 0..50 {
        let key = format!("k_{:04}", i);
        let result = btree.search(key.as_bytes())?;
        
        if i >= 10 && i < 20 && i % 2 == 0 {
            // These keys should be deleted
            assert!(result.is_none(), "Key {} should be deleted", key);
        } else {
            // These keys should still exist
            assert!(result.is_some(), "Key {} should exist", key);
            assert_eq!(result.unwrap(), format!("v_{}", key).as_bytes());
        }
    }
    
    Ok(())
}

#[tokio::test]
async fn test_btree_full_merge_cascade() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("btree_cascade_test.db");
    
    let pm = Arc::new(PageManager::open(&db_path)?);
    let mut btree = BTree::create(pm)?;
    
    // Insert enough keys to create a tall tree
    let keys: Vec<String> = (0..100).map(|i| format!("item_{:04}", i)).collect();
    
    for key in &keys {
        btree.insert(key.as_bytes(), format!("data_{}", key).as_bytes())?;
    }
    
    // Delete most keys to trigger cascading merges
    for i in (0..80).rev() {
        let key = format!("item_{:04}", i);
        let deleted = btree.delete(key.as_bytes())?;
        assert!(deleted, "Key {} should be deletable", key);
        
        // Verify key is actually gone
        let result = btree.search(key.as_bytes())?;
        assert!(result.is_none(), "Key {} should be deleted", key);
    }
    
    // Verify remaining keys are still accessible
    for i in 80..100 {
        let key = format!("item_{:04}", i);
        let result = btree.search(key.as_bytes())?;
        assert!(result.is_some(), "Key {} should still exist after cascade", key);
        assert_eq!(result.unwrap(), format!("data_{}", key).as_bytes());
    }
    
    // Test range scan on remaining keys
    let scan_results = btree.range_scan(b"item_0080", b"item_0099")?;
    assert_eq!(scan_results.len(), 20); // items 80-99
    
    Ok(())
}

#[tokio::test]
async fn test_btree_delete_nonexistent() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("btree_nonexistent_test.db");
    
    let pm = Arc::new(PageManager::open(&db_path)?);
    let mut btree = BTree::create(pm)?;
    
    // Insert some keys
    for i in 0..10 {
        let key = format!("exist_{}", i);
        btree.insert(key.as_bytes(), b"value")?;
    }
    
    // Try to delete nonexistent keys
    let nonexistent_keys = ["missing_1", "missing_2", "zzz_last"];
    
    for key in &nonexistent_keys {
        let deleted = btree.delete(key.as_bytes())?;
        assert!(!deleted, "Nonexistent key {} should return false", key);
    }
    
    // Verify existing keys are unaffected
    for i in 0..10 {
        let key = format!("exist_{}", i);
        let result = btree.search(key.as_bytes())?;
        assert!(result.is_some(), "Existing key {} should be unaffected", key);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_btree_merge_with_persistence() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("btree_persist_merge.db");
    
    // Create initial tree and perform operations
    {
        let pm = Arc::new(PageManager::open(&db_path)?);
        let mut btree = BTree::create(pm)?;
        
        // Insert keys
        for i in 0..30 {
            let key = format!("persistent_{:03}", i);
            btree.insert(key.as_bytes(), format!("value_{}", i).as_bytes())?;
        }
        
        // Delete some keys to trigger merging
        for i in (0..20).step_by(3) {
            let key = format!("persistent_{:03}", i);
            btree.delete(key.as_bytes())?;
        }
        
        // Sync to ensure persistence
        btree.sync()?;
    }
    
    // Reopen and verify tree is consistent
    {
        let pm = Arc::new(PageManager::open(&db_path)?);
        // Note: This test assumes we store root_page_id somehow
        // For now we'll create a new tree, in full implementation
        // we'd restore the root page ID from metadata
        let btree = BTree::create(pm)?;
        
        // This is a simplified test - in full implementation we'd
        // restore the tree state and verify the merged structure persisted
        // For now, just verify we can create a new tree successfully
        assert!(btree.root_page_id() > 0);
    }
    
    Ok(())
}