//! Tests for persistent free list functionality

use zendb::storage::{PageManager, Page};
use tempfile::TempDir;
use anyhow::Result;

#[tokio::test]
async fn test_free_list_persistence() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_free_list.db");
    
    let original_page_ids = {
        // Create a database and allocate some pages
        let pm = PageManager::open(&db_path)?;
        
        // Allocate 5 pages
        let page1 = pm.allocate_page()?;
        let page2 = pm.allocate_page()?;
        let page3 = pm.allocate_page()?;
        let page4 = pm.allocate_page()?;
        let page5 = pm.allocate_page()?;
        
        // Write some data to the pages
        for page_id in &[page1, page2, page3, page4, page5] {
            let mut page = Page::new(*page_id);
            page.data[0..10].copy_from_slice(b"test data ");
            pm.write_page(&page)?;
        }
        
        // Free some pages (page2 and page4)
        pm.free_page(page2)?;
        pm.free_page(page4)?;
        
        // Sync to ensure data is persisted
        pm.sync()?;
        
        vec![page1, page2, page3, page4, page5]
    };
    
    // Reopen the database and verify free pages are available
    {
        let pm = PageManager::open(&db_path)?;
        
        // Allocate two new pages - these should reuse the freed pages
        let reused1 = pm.allocate_page()?;
        let reused2 = pm.allocate_page()?;
        
        // The reused page IDs should be from our freed pages (page2 or page4)
        let freed_pages = [original_page_ids[1], original_page_ids[3]]; // page2, page4
        assert!(freed_pages.contains(&reused1));
        assert!(freed_pages.contains(&reused2));
        assert_ne!(reused1, reused2);
        
        // Verify that we can write to the reused pages
        let mut page1 = Page::new(reused1);
        page1.data[0..12].copy_from_slice(b"reused data1");
        pm.write_page(&page1)?;
        
        let mut page2 = Page::new(reused2);
        page2.data[0..12].copy_from_slice(b"reused data2");
        pm.write_page(&page2)?;
        
        // Read back and verify
        let read_page1 = pm.read_page(reused1)?;
        assert_eq!(&read_page1.data[0..12], b"reused data1");
        
        let read_page2 = pm.read_page(reused2)?;
        assert_eq!(&read_page2.data[0..12], b"reused data2");
    }
    
    Ok(())
}

#[tokio::test]
async fn test_multiple_free_list_operations() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_multiple_free.db");
    
    // First create a database and establish some pages
    let initial_pages = {
        let pm = PageManager::open(&db_path)?;
        
        // Allocate 6 pages initially
        let pages: Vec<_> = (0..6).map(|_| pm.allocate_page().unwrap()).collect();
        
        // Write test data to all pages
        for page_id in &pages {
            let mut page = Page::new(*page_id);
            page.data[0..10].copy_from_slice(b"initial   ");
            pm.write_page(&page)?;
        }
        
        pm.sync()?;
        pages
    };
    
    // Now test free/allocate cycles with persistence
    {
        let pm = PageManager::open(&db_path)?;
        
        // Free some pages (pages 1, 3, 5)
        pm.free_page(initial_pages[1])?;
        pm.free_page(initial_pages[3])?; 
        pm.free_page(initial_pages[5])?;
        pm.sync()?;
    }
    
    // Reopen and verify we can reuse the freed pages
    {
        let pm = PageManager::open(&db_path)?;
        
        // Allocate three pages - should reuse the freed ones
        let reused1 = pm.allocate_page()?;
        let reused2 = pm.allocate_page()?;
        let reused3 = pm.allocate_page()?;
        
        // Write test data to reused pages
        let mut page1 = Page::new(reused1);
        page1.data[0..10].copy_from_slice(b"reused1   ");
        pm.write_page(&page1)?;
        
        let mut page2 = Page::new(reused2);
        page2.data[0..10].copy_from_slice(b"reused2   ");
        pm.write_page(&page2)?;
        
        let mut page3 = Page::new(reused3);
        page3.data[0..10].copy_from_slice(b"reused3   ");
        pm.write_page(&page3)?;
        
        // Verify we can read the data back
        let read1 = pm.read_page(reused1)?;
        assert_eq!(&read1.data[0..10], b"reused1   ");
        
        let read2 = pm.read_page(reused2)?;
        assert_eq!(&read2.data[0..10], b"reused2   ");
        
        let read3 = pm.read_page(reused3)?;
        assert_eq!(&read3.data[0..10], b"reused3   ");
        
        // The reused pages should be from our freed set
        let freed_pages = [initial_pages[1], initial_pages[3], initial_pages[5]];
        assert!(freed_pages.contains(&reused1));
        assert!(freed_pages.contains(&reused2));
        assert!(freed_pages.contains(&reused3));
    }
    
    Ok(())
}

#[tokio::test]
async fn test_empty_free_list_persistence() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_empty_free.db");
    
    // Create a database with no free pages
    {
        let pm = PageManager::open(&db_path)?;
        let page1 = pm.allocate_page()?;
        let page2 = pm.allocate_page()?;
        
        let mut page = Page::new(page1);
        page.data[0..5].copy_from_slice(b"test1");
        pm.write_page(&page)?;
        
        let mut page = Page::new(page2);
        page.data[0..5].copy_from_slice(b"test2");
        pm.write_page(&page)?;
        
        pm.sync()?;
        // Don't free any pages
    }
    
    // Reopen and verify normal operation
    {
        let pm = PageManager::open(&db_path)?;
        
        // Should allocate new page (not reuse any)
        let page3 = pm.allocate_page()?;
        assert_eq!(page3, 2); // Should be the third page (0-indexed)
        
        let mut page = Page::new(page3);
        page.data[0..5].copy_from_slice(b"test3");
        pm.write_page(&page)?;
        
        let read_page = pm.read_page(page3)?;
        assert_eq!(&read_page.data[0..5], b"test3");
    }
    
    Ok(())
}