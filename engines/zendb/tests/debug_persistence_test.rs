//! Debug test for persistence with compression

use zendb::storage::{PageManager, BTree};
use tempfile::TempDir;
use anyhow::Result;
use std::sync::Arc;

#[tokio::test]
async fn test_debug_persistence_simple() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("debug_persist.db");
    
    let root_page_id;
    
    // Phase 1: Create and populate tree
    println!("Phase 1: Creating tree");
    {
        let pm = Arc::new(PageManager::open(&db_path)?);
        let mut btree = BTree::create(pm.clone())?;
        root_page_id = btree.root_page_id();
        
        println!("Root page ID: {}", root_page_id);
        
        // Insert a simple key-value pair
        btree.insert(b"test_key", b"test_value")?;
        
        // Verify it can be read immediately
        let result = btree.search(b"test_key")?;
        assert_eq!(result, Some(b"test_value".to_vec()));
        println!("Immediate read successful");
        
        // Sync to disk
        pm.sync()?;
        println!("Synced to disk");
    }
    
    // Phase 2: Reopen and verify
    println!("Phase 2: Reopening tree");
    {
        let pm = Arc::new(PageManager::open(&db_path)?);
        
        // Read the root page directly to see what's stored
        let root_page = pm.read_page(root_page_id)?;
        println!("Root page first 10 bytes: {:?}", &root_page.data[0..10]);
        
        // Try to open the B+Tree
        match BTree::open(pm.clone(), root_page_id) {
            Ok(btree) => {
                println!("BTree opened successfully");
                
                // Try to search
                match btree.search(b"test_key") {
                    Ok(result) => {
                        println!("Search result: {:?}", result);
                        assert_eq!(result, Some(b"test_value".to_vec()));
                        println!("Persistence test PASSED");
                    }
                    Err(e) => {
                        println!("Search failed: {}", e);
                        return Err(e);
                    }
                }
            }
            Err(e) => {
                println!("BTree open failed: {}", e);
                return Err(e);
            }
        }
    }
    
    Ok(())
}