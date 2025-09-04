use zendb::storage::{PageManager, BTree};
use tempfile::TempDir;
use std::sync::Arc;

#[test]
fn test_btree_create_and_search() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("btree_test.db");
    
    let pm = Arc::new(PageManager::open(&db_path).unwrap());
    let mut btree = BTree::create(pm.clone()).unwrap();
    
    // Insert some key-value pairs
    btree.insert(b"key1", b"value1").unwrap();
    btree.insert(b"key2", b"value2").unwrap();
    btree.insert(b"key3", b"value3").unwrap();
    
    // Search for existing keys
    assert_eq!(btree.search(b"key1").unwrap(), Some(b"value1".to_vec()));
    assert_eq!(btree.search(b"key2").unwrap(), Some(b"value2".to_vec()));
    assert_eq!(btree.search(b"key3").unwrap(), Some(b"value3".to_vec()));
    
    // Search for non-existent key
    assert_eq!(btree.search(b"key4").unwrap(), None);
}

#[test]
fn test_btree_update() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("btree_update.db");
    
    let pm = Arc::new(PageManager::open(&db_path).unwrap());
    let mut btree = BTree::create(pm.clone()).unwrap();
    
    // Insert initial value
    btree.insert(b"key1", b"value1").unwrap();
    assert_eq!(btree.search(b"key1").unwrap(), Some(b"value1".to_vec()));
    
    // Update value
    btree.insert(b"key1", b"updated_value").unwrap();
    assert_eq!(btree.search(b"key1").unwrap(), Some(b"updated_value".to_vec()));
}

#[test]
fn test_btree_range_scan() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("btree_range.db");
    
    let pm = Arc::new(PageManager::open(&db_path).unwrap());
    let mut btree = BTree::create(pm.clone()).unwrap();
    
    // Insert ordered keys
    for i in 0..10 {
        let key = format!("key{:02}", i);
        let value = format!("value{}", i);
        btree.insert(key.as_bytes(), value.as_bytes()).unwrap();
    }
    
    // Range scan
    let results = btree.range_scan(b"key03", b"key07").unwrap();
    assert_eq!(results.len(), 5);
    
    // Verify results
    let expected = vec![
        (b"key03".to_vec(), b"value3".to_vec()),
        (b"key04".to_vec(), b"value4".to_vec()),
        (b"key05".to_vec(), b"value5".to_vec()),
        (b"key06".to_vec(), b"value6".to_vec()),
        (b"key07".to_vec(), b"value7".to_vec()),
    ];
    
    for (i, (key, value)) in results.iter().enumerate() {
        assert_eq!(key[..expected[i].0.len()], expected[i].0[..]);
        assert_eq!(value[..], expected[i].1[..]);
    }
}

#[test]
fn test_btree_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("btree_persist.db");
    
    let root_page_id;
    
    // Create and populate tree
    {
        let pm = Arc::new(PageManager::open(&db_path).unwrap());
        let mut btree = BTree::create(pm.clone()).unwrap();
        root_page_id = btree.root_page_id();
        
        btree.insert(b"persistent_key", b"persistent_value").unwrap();
        pm.sync().unwrap();
    }
    
    // Reopen and verify
    {
        let pm = Arc::new(PageManager::open(&db_path).unwrap());
        let btree = BTree::open(pm.clone(), root_page_id).unwrap();
        
        assert_eq!(
            btree.search(b"persistent_key").unwrap(),
            Some(b"persistent_value".to_vec())
        );
    }
}

#[test]
fn test_btree_many_inserts() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("btree_many.db");
    
    let pm = Arc::new(PageManager::open(&db_path).unwrap());
    let mut btree = BTree::create(pm.clone()).unwrap();
    
    // Insert many keys to trigger splits
    for i in 0..100 {
        let key = format!("key{:04}", i);
        let value = format!("value{}", i);
        btree.insert(key.as_bytes(), value.as_bytes()).unwrap();
    }
    
    // Verify all keys
    for i in 0..100 {
        let key = format!("key{:04}", i);
        let expected_value = format!("value{}", i);
        let result = btree.search(key.as_bytes()).unwrap();
        assert!(result.is_some(), "Key {} not found", key);
        
        let value = result.unwrap();
        let value_str = std::str::from_utf8(&value[..expected_value.len()]).unwrap();
        assert_eq!(value_str, expected_value);
    }
}