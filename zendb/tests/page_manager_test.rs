use zendb::storage::{PageManager, Page, PageId};
use tempfile::TempDir;
use std::path::Path;

#[test]
fn test_page_manager_create() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let pm = PageManager::open(&db_path).unwrap();
    assert!(db_path.exists());
    
    pm.sync().unwrap();
}

#[test]
fn test_page_allocation_and_read_write() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let pm = PageManager::open(&db_path).unwrap();
    
    // Allocate a page
    let page_id = pm.allocate_page().unwrap();
    assert_eq!(page_id, 0);
    
    // Create a page with test data
    let mut page = Page::new(page_id);
    let test_data = b"Hello, ZenDB!";
    page.data[..test_data.len()].copy_from_slice(test_data);
    
    // Write the page
    pm.write_page(&page).unwrap();
    
    // Read it back
    let read_page = pm.read_page(page_id).unwrap();
    assert_eq!(read_page.id, page_id);
    assert_eq!(&read_page.data[..test_data.len()], test_data);
}

#[test]
fn test_multiple_pages() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let pm = PageManager::open(&db_path).unwrap();
    
    let mut page_ids = Vec::new();
    
    // Allocate and write multiple pages
    for i in 0..10 {
        let page_id = pm.allocate_page().unwrap();
        page_ids.push(page_id);
        
        let mut page = Page::new(page_id);
        let data = format!("Page {}", i);
        page.data[..data.len()].copy_from_slice(data.as_bytes());
        
        pm.write_page(&page).unwrap();
    }
    
    // Read them back
    for (i, &page_id) in page_ids.iter().enumerate() {
        let page = pm.read_page(page_id).unwrap();
        let expected = format!("Page {}", i);
        assert_eq!(&page.data[..expected.len()], expected.as_bytes());
    }
}

#[test]
fn test_free_and_reuse_pages() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let pm = PageManager::open(&db_path).unwrap();
    
    // Allocate some pages
    let page1 = pm.allocate_page().unwrap();
    let page2 = pm.allocate_page().unwrap();
    let page3 = pm.allocate_page().unwrap();
    
    // Free page2
    pm.free_page(page2).unwrap();
    
    // Next allocation should reuse page2
    let reused = pm.allocate_page().unwrap();
    assert_eq!(reused, page2);
}

#[test]
fn test_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let page_id;
    let test_data = b"Persistent data";
    
    // Write data
    {
        let pm = PageManager::open(&db_path).unwrap();
        page_id = pm.allocate_page().unwrap();
        
        let mut page = Page::new(page_id);
        page.data[..test_data.len()].copy_from_slice(test_data);
        
        pm.write_page(&page).unwrap();
        pm.sync().unwrap();
    }
    
    // Read back after reopening
    {
        let pm = PageManager::open(&db_path).unwrap();
        let page = pm.read_page(page_id).unwrap();
        assert_eq!(&page.data[..test_data.len()], test_data);
    }
}