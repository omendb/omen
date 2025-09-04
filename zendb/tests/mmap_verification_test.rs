use zendb::storage::{PageManager, Page};
use tempfile::TempDir;
use std::fs;

#[test]
fn verify_mmap_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("mmap_test.db");
    
    // Write with mmap
    {
        let pm = PageManager::open(&db_path).unwrap();
        
        for i in 0..10u64 {
            let page_id = pm.allocate_page().unwrap();
            let mut page = Page::new(page_id);
            page.data[0..8].copy_from_slice(&i.to_le_bytes());
            pm.write_page(&page).unwrap();
        }
        
        pm.sync().unwrap();
    }
    
    // Verify file size
    let metadata = fs::metadata(&db_path).unwrap();
    let expected_size = 4096 + (10 * 16384); // header + 10 pages
    assert_eq!(metadata.len(), expected_size as u64);
    
    // Read back without mmap (new instance)
    {
        let pm = PageManager::open(&db_path).unwrap();
        
        for i in 0..10u64 {
            let page = pm.read_page(i).unwrap();
            let value = u64::from_le_bytes(page.data[0..8].try_into().unwrap());
            assert_eq!(value, i);
        }
    }
}

#[test] 
fn verify_free_page_reuse() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("free_test.db");
    
    let pm = PageManager::open(&db_path).unwrap();
    
    // Allocate pages
    let p0 = pm.allocate_page().unwrap();
    let p1 = pm.allocate_page().unwrap();
    let p2 = pm.allocate_page().unwrap();
    let p3 = pm.allocate_page().unwrap();
    
    assert_eq!(p0, 0);
    assert_eq!(p1, 1);
    assert_eq!(p2, 2);
    assert_eq!(p3, 3);
    
    // Free some pages
    pm.free_page(p1).unwrap();
    pm.free_page(p2).unwrap();
    
    // Allocate again - should reuse freed pages
    let reused1 = pm.allocate_page().unwrap();
    let reused2 = pm.allocate_page().unwrap();
    
    // Should reuse in LIFO order
    assert_eq!(reused1, 2); // Last freed
    assert_eq!(reused2, 1); // First freed
    
    // New allocation should continue from where it left off
    let p4 = pm.allocate_page().unwrap();
    assert_eq!(p4, 4);
}