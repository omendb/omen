use zendb::storage::{PageManager, Page};
use tempfile::TempDir;

#[test]
fn stress_test_many_pages() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("stress_test.db");
    
    let pm = PageManager::open(&db_path).unwrap();
    
    // Write many pages to trigger file extension and mmap remapping
    for i in 0..100 {
        let page_id = pm.allocate_page().unwrap();
        assert_eq!(page_id, i);
        
        let mut page = Page::new(page_id);
        let data = format!("Page #{:04} - Testing stress", i);
        page.data[..data.len()].copy_from_slice(data.as_bytes());
        
        pm.write_page(&page).unwrap();
    }
    
    // Sync to ensure durability
    pm.sync().unwrap();
    
    // Read all pages back
    for i in 0..100 {
        let page = pm.read_page(i).unwrap();
        let expected = format!("Page #{:04} - Testing stress", i);
        let actual = std::str::from_utf8(&page.data[..expected.len()]).unwrap();
        assert_eq!(actual, expected);
    }
}

#[test]
fn test_large_page_data() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("large_test.db");
    
    let pm = PageManager::open(&db_path).unwrap();
    
    let page_id = pm.allocate_page().unwrap();
    
    // Fill entire page with pattern
    let mut page = Page::new(page_id);
    for i in 0..page.data.len() {
        page.data[i] = (i % 256) as u8;
    }
    
    pm.write_page(&page).unwrap();
    pm.sync().unwrap();
    
    // Read back and verify
    let read_page = pm.read_page(page_id).unwrap();
    for i in 0..read_page.data.len() {
        assert_eq!(read_page.data[i], (i % 256) as u8, "Mismatch at byte {}", i);
    }
}