use proptest::prelude::*;
use zendb::storage::{PageManager, Page, PageId, BTree};
use zendb::transaction::{TransactionManager, HLC};
use tempfile::TempDir;
use std::sync::Arc;
use std::collections::HashMap;

// Property: PageManager should always return the data that was written
proptest! {
    #[test]
    fn prop_page_manager_read_write(
        data: Vec<u8>,
        page_num in 0u64..100
    ) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("prop_test.db");
        let pm = PageManager::open(&db_path).unwrap();
        
        // Ensure we have enough pages allocated
        for _ in 0..=page_num {
            let _ = pm.allocate_page().unwrap();
        }
        
        // Create page with random data
        let mut page = Page::new(page_num);
        let copy_len = data.len().min(page.data.len());
        page.data[..copy_len].copy_from_slice(&data[..copy_len]);
        
        // Write and read back
        pm.write_page(&page).unwrap();
        let read_page = pm.read_page(page_num).unwrap();
        
        // Should match exactly
        prop_assert_eq!(&page.data[..copy_len], &read_page.data[..copy_len]);
    }
}

// Property: B+Tree should maintain sorted order
proptest! {
    #[test]
    fn prop_btree_sorted_order(
        keys_values in prop::collection::vec((prop::collection::vec(0u8..=255, 1..32), prop::collection::vec(0u8..=255, 1..64)), 1..50)
    ) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("btree_prop.db");
        let pm = Arc::new(PageManager::open(&db_path).unwrap());
        let mut btree = BTree::create(pm.clone()).unwrap();
        
        // Insert all key-value pairs
        for (key, value) in &keys_values {
            btree.insert(key, value).unwrap();
        }
        
        // Verify each key retrieves its value
        for (key, value) in &keys_values {
            let result = btree.search(key).unwrap();
            prop_assert_eq!(result, Some(value.clone()));
        }
        
        // Range scan should return sorted results
        if !keys_values.is_empty() {
            let min_key = keys_values.iter().map(|(k, _)| k).min().unwrap();
            let max_key = keys_values.iter().map(|(k, _)| k).max().unwrap();
            
            let results = btree.range_scan(min_key, max_key).unwrap();
            
            // Verify results are sorted
            for i in 1..results.len() {
                prop_assert!(results[i-1].0 <= results[i].0);
            }
        }
    }
}

// Property: Free pages should be reused
proptest! {
    #[test]
    fn prop_page_manager_free_reuse(
        allocate_count in 1usize..20,
        free_indices in prop::collection::vec(0usize..20, 0..10)
    ) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("free_prop.db");
        let pm = PageManager::open(&db_path).unwrap();
        
        // Allocate pages
        let mut allocated = Vec::new();
        for _ in 0..allocate_count {
            allocated.push(pm.allocate_page().unwrap());
        }
        
        // Free some pages
        let mut freed = Vec::new();
        for &idx in &free_indices {
            if idx < allocated.len() && !freed.contains(&allocated[idx]) {
                pm.free_page(allocated[idx]).unwrap();
                freed.push(allocated[idx]);
            }
        }
        
        // Allocate again - should reuse freed pages
        for _ in 0..freed.len() {
            let new_page = pm.allocate_page().unwrap();
            prop_assert!(freed.contains(&new_page) || new_page >= allocated.len() as u64);
        }
    }
}

// Property: HLC timestamps are always monotonically increasing
proptest! {
    #[test]
    fn prop_hlc_monotonic(
        operations in 1usize..100
    ) {
        let clock = HLC::new(1);
        let mut timestamps = Vec::new();
        
        for _ in 0..operations {
            timestamps.push(clock.now());
        }
        
        // All timestamps should be strictly increasing
        for i in 1..timestamps.len() {
            prop_assert!(timestamps[i] > timestamps[i-1]);
        }
    }
}

// Property: Transaction isolation - no dirty reads
proptest! {
    #[test]
    fn prop_transaction_isolation(
        keys in prop::collection::vec(prop::collection::vec(0u8..=255, 1..10), 1..20),
        values in prop::collection::vec(prop::collection::vec(0u8..=255, 1..50), 1..20)
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let tm = TransactionManager::new();
            
            // Normalize to same length
            let pairs: Vec<_> = keys.iter()
                .zip(values.iter())
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            
            // Start two transactions
            let tx1 = tm.begin().await.unwrap();
            let tx2 = tm.begin().await.unwrap();
            
            // TX1 writes all keys
            for (key, value) in &pairs {
                tm.put(key.clone(), value.clone(), tx1.id).await.unwrap();
            }
            
            // TX2 shouldn't see uncommitted writes
            for (key, _) in &pairs {
                let result = tm.get(key, tx2.id).await.unwrap();
                prop_assert_eq!(result, None);
            }
            
            // Commit TX1
            tm.commit(tx1).await.unwrap();
            
            // TX2 still shouldn't see (snapshot isolation)
            for (key, _) in &pairs {
                let result = tm.get(key, tx2.id).await.unwrap();
                prop_assert_eq!(result, None);
            }
            
            // New TX3 should see everything
            let tx3 = tm.begin().await.unwrap();
            for (key, value) in &pairs {
                let result = tm.get(key, tx3.id).await.unwrap();
                prop_assert_eq!(result, Some(value.clone()));
            }
            
            Ok(())
        }).unwrap();
    }
}

// Property: B+Tree updates should overwrite previous values
proptest! {
    #[test]
    fn prop_btree_update(
        key in prop::collection::vec(0u8..=255, 1..32),
        values in prop::collection::vec(prop::collection::vec(0u8..=255, 1..64), 2..10)
    ) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("update_prop.db");
        let pm = Arc::new(PageManager::open(&db_path).unwrap());
        let mut btree = BTree::create(pm.clone()).unwrap();
        
        // Insert each value in sequence
        for value in &values {
            btree.insert(&key, value).unwrap();
            
            // Should always return the latest value
            let result = btree.search(&key).unwrap();
            prop_assert_eq!(result, Some(value.clone()));
        }
        
        // Final check - should have last value
        let result = btree.search(&key).unwrap();
        prop_assert_eq!(result, Some(values.last().unwrap().clone()));
    }
}

// Property: Page allocation should never return the same page twice (unless freed)
proptest! {
    #[test]
    fn prop_page_unique_allocation(
        allocation_count in 1usize..100
    ) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("unique_prop.db");
        let pm = PageManager::open(&db_path).unwrap();
        
        let mut allocated = HashMap::new();
        
        for i in 0..allocation_count {
            let page_id = pm.allocate_page().unwrap();
            
            // Should not have been allocated before
            prop_assert!(!allocated.contains_key(&page_id),
                        "Page {} was allocated twice at iteration {}", page_id, i);
            
            allocated.insert(page_id, i);
        }
        
        // All pages should be unique
        prop_assert_eq!(allocated.len(), allocation_count);
    }
}

// Property: Persistence - data survives database restart
proptest! {
    #[test]
    fn prop_persistence(
        data in prop::collection::vec((prop::collection::vec(0u8..=255, 1..32), prop::collection::vec(0u8..=255, 1..64)), 1..20)
    ) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("persist_prop.db");
        
        let root_page_id;
        
        // Write data
        {
            let pm = Arc::new(PageManager::open(&db_path).unwrap());
            let mut btree = BTree::create(pm.clone()).unwrap();
            root_page_id = btree.root_page_id();
            
            for (key, value) in &data {
                btree.insert(key, value).unwrap();
            }
            
            pm.sync().unwrap();
        }
        
        // Read back after "restart"
        {
            let pm = Arc::new(PageManager::open(&db_path).unwrap());
            let btree = BTree::open(pm.clone(), root_page_id).unwrap();
            
            for (key, value) in &data {
                let result = btree.search(key).unwrap();
                prop_assert_eq!(result, Some(value.clone()));
            }
        }
    }
}