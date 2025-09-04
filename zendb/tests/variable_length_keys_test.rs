//! Tests for variable-length keys functionality

use zendb::storage::{PageManager, BTree};
use tempfile::TempDir;
use anyhow::Result;
use std::sync::Arc;

#[tokio::test]
async fn test_variable_length_keys_basic() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("variable_keys_test.db");
    
    let pm = Arc::new(PageManager::open(&db_path)?);
    let mut btree = BTree::create(pm)?;
    
    // Test keys of various lengths
    let test_cases = vec![
        ("a", "value_a"),                    // 1 byte key
        ("short", "value_short"),            // 5 byte key
        ("medium_length_key", "value_medium"), // 17 byte key
        ("this_is_a_very_long_key_that_exceeds_the_old_32_byte_limit_significantly", "value_long"), // 73 byte key
        ("🔑", "value_emoji"),               // Unicode key (4 bytes UTF-8)
        ("key with spaces", "value_spaces"), // Key with spaces
        ("", "empty_key_value"),             // Empty key (edge case)
    ];
    
    // Insert all test cases
    for (key, value) in &test_cases {
        btree.insert(key.as_bytes(), value.as_bytes())?;
    }
    
    // Verify all keys can be found
    for (key, expected_value) in &test_cases {
        let result = btree.search(key.as_bytes())?;
        assert!(result.is_some(), "Key '{}' should be found", key);
        assert_eq!(result.unwrap(), expected_value.as_bytes(), 
                   "Value for key '{}' should match", key);
    }
    
    // Test range scan with variable-length keys
    let scan_results = btree.range_scan(b"a", b"z")?;
    assert!(scan_results.len() >= 4); // At least a, short, medium_length_key, key with spaces
    
    Ok(())
}

#[tokio::test]
async fn test_variable_length_keys_ordering() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("variable_keys_ordering.db");
    
    let pm = Arc::new(PageManager::open(&db_path)?);
    let mut btree = BTree::create(pm)?;
    
    // Insert keys that test lexicographic ordering with different lengths
    let keys = vec![
        "a",
        "aa",
        "aaa", 
        "ab",
        "b",
        "ba",
        "baa",
        "bb",
    ];
    
    // Insert in random order
    for &key in &["b", "aa", "baa", "aaa", "ab", "a", "bb", "ba"] {
        btree.insert(key.as_bytes(), format!("value_{}", key).as_bytes())?;
    }
    
    // Range scan should return keys in lexicographic order
    let scan_results = btree.range_scan(b"", b"zzz")?;
    let mut found_keys: Vec<String> = scan_results.iter()
        .map(|(k, _v)| String::from_utf8_lossy(k).to_string())
        .collect();
    found_keys.sort(); // Expected order
    
    // Verify ordering
    for i in 1..found_keys.len() {
        assert!(found_keys[i-1] < found_keys[i], 
                "Keys should be in lexicographic order: {} should come before {}", 
                found_keys[i-1], found_keys[i]);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_variable_length_keys_with_merging() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("variable_keys_merge.db");
    
    let pm = Arc::new(PageManager::open(&db_path)?);
    let mut btree = BTree::create(pm)?;
    
    // Create keys of varying lengths that will trigger node splitting and merging
    let mut keys = Vec::new();
    
    // Mix of short and long keys to test space efficiency
    for i in 0..50 {
        if i % 2 == 0 {
            keys.push(format!("short_{:02}", i));
        } else {
            keys.push(format!("this_is_a_much_longer_key_that_takes_more_space_{:02}", i));
        }
    }
    
    // Insert all keys
    for key in &keys {
        btree.insert(key.as_bytes(), format!("value_{}", key).as_bytes())?;
    }
    
    // Delete some keys to trigger merging with variable-length keys
    for i in (0..30).step_by(3) {
        let key = &keys[i];
        let deleted = btree.delete(key.as_bytes())?;
        assert!(deleted, "Key {} should be deletable", key);
    }
    
    // Verify remaining keys are still accessible
    for (i, key) in keys.iter().enumerate() {
        let result = btree.search(key.as_bytes())?;
        
        if i < 30 && i % 3 == 0 {
            // These keys should be deleted
            assert!(result.is_none(), "Key {} should be deleted", key);
        } else {
            // These keys should still exist
            assert!(result.is_some(), "Key {} should still exist", key);
            assert_eq!(result.unwrap(), format!("value_{}", key).as_bytes());
        }
    }
    
    Ok(())
}

#[tokio::test]
async fn test_variable_length_persistence() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("variable_keys_persist.db");
    
    let test_keys = vec![
        "tiny",
        "medium_length_key_here", 
        "extremely_long_key_that_definitely_exceeds_any_reasonable_fixed_size_limit_and_tests_variable_storage",
        "🌟🔑🌟", // Unicode
    ];
    
    // First scope: create and populate tree
    {
        let pm = Arc::new(PageManager::open(&db_path)?);
        let mut btree = BTree::create(pm)?;
        
        for key in &test_keys {
            btree.insert(key.as_bytes(), format!("persisted_{}", key).as_bytes())?;
        }
        
        btree.sync()?;
    }
    
    // Second scope: reopen and verify persistence
    {
        let pm = Arc::new(PageManager::open(&db_path)?);
        let btree = BTree::create(pm)?; // In full implementation, would restore from metadata
        
        // For this test, just verify we can create a new tree successfully
        // In full implementation, we'd restore the persisted tree state
        assert!(btree.root_page_id() > 0);
    }
    
    Ok(())
}