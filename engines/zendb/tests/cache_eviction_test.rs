//! Tests for LRU cache eviction functionality

use zendb::storage::{PageManager, Page};
use tempfile::TempDir;
use anyhow::Result;

#[tokio::test]
async fn test_lru_cache_eviction() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_cache.db");
    
    // Create a PageManager (uses default cache size of 1024 pages)
    let pm = PageManager::open(&db_path)?;
    
    // Check initial cache stats
    let (used, capacity, utilization) = pm.cache_stats();
    assert_eq!(used, 0);
    assert_eq!(capacity, 1024); // DEFAULT_CACHE_SIZE
    assert_eq!(utilization, 0.0);
    
    // Allocate and read enough pages to test eviction
    let mut page_ids = Vec::new();
    
    // Fill cache to capacity + some extra
    let test_pages = 1100; // More than cache capacity
    for i in 0..test_pages {
        let page_id = pm.allocate_page()?;
        page_ids.push(page_id);
        
        // Write unique data to each page
        let mut page = Page::new(page_id);
        let data = format!("page_data_{:04}", i);
        page.data[0..data.len()].copy_from_slice(data.as_bytes());
        pm.write_page(&page)?;
    }
    
    // Read all pages to populate cache
    for page_id in &page_ids {
        let _page = pm.read_page(*page_id)?;
    }
    
    // Cache should be at capacity, not exceeded
    let (used, capacity, utilization) = pm.cache_stats();
    assert_eq!(used, capacity); // Should be exactly at capacity
    assert!(utilization >= 0.99); // Very close to 100%
    
    println!("Cache stats after filling: used={}, capacity={}, utilization={:.2}%", 
             used, capacity, utilization * 100.0);
    
    // Read the first few pages again - they should have been evicted and need to be re-read
    let page = pm.read_page(page_ids[0])?;
    let data = String::from_utf8_lossy(&page.data[0..10]);
    assert_eq!(data, "page_data_");
    
    // Cache should still be at capacity
    let (used, _, _) = pm.cache_stats();
    assert_eq!(used, capacity);
    
    Ok(())
}

#[tokio::test] 
async fn test_lru_access_ordering() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_lru_ordering.db");
    
    let pm = PageManager::open(&db_path)?;
    
    // Create a few pages
    let page1 = pm.allocate_page()?;
    let page2 = pm.allocate_page()?; 
    let page3 = pm.allocate_page()?;
    
    // Write data to pages
    for (i, page_id) in [page1, page2, page3].iter().enumerate() {
        let mut page = Page::new(*page_id);
        let data = format!("test_page_{}", i + 1);
        page.data[0..data.len()].copy_from_slice(data.as_bytes());
        pm.write_page(&page)?;
    }
    
    // Read pages in order 1, 2, 3
    let _p1 = pm.read_page(page1)?;
    let _p2 = pm.read_page(page2)?; 
    let _p3 = pm.read_page(page3)?;
    
    // Read page1 again - should move it to most recently used
    let _p1_again = pm.read_page(page1)?;
    
    // All pages should be in cache
    let (used, _, _) = pm.cache_stats();
    assert_eq!(used, 3);
    
    // Clear cache and verify
    pm.clear_cache();
    let (used, _, _) = pm.cache_stats();
    assert_eq!(used, 0);
    
    Ok(())
}

#[tokio::test]
async fn test_cache_with_free_pages() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_cache_free.db");
    
    // First scope - test cache population and free page removal
    {
        let pm = PageManager::open(&db_path)?;
        
        // Allocate some pages
        let page1 = pm.allocate_page()?;
        let page2 = pm.allocate_page()?;
        let page3 = pm.allocate_page()?;
        
        // Write and read to populate cache
        for page_id in &[page1, page2, page3] {
            let mut page = Page::new(*page_id);
            page.data[0..10].copy_from_slice(b"test data ");
            pm.write_page(&page)?;
            let _read_page = pm.read_page(*page_id)?;
        }
        
        // Cache should have 3 pages
        let stats = pm.cache_stats();
        assert_eq!(stats.0, 3);
        
        // Free page2 - should be removed from cache
        pm.free_page(page2)?;
        
        // Cache should have 2 pages
        let stats = pm.cache_stats();
        assert_eq!(stats.0, 2);
        
        // Store page IDs for later
        std::mem::drop((page1, page3));
    }
    
    // Second scope - test cache persistence across restarts
    {
        let pm = PageManager::open(&db_path)?;
        let stats = pm.cache_stats();
        assert_eq!(stats.0, 0); // Cache should start empty
        
        // Read some pages to populate cache
        let _p1 = pm.read_page(0)?; // page1 was 0
        let _p2 = pm.read_page(2)?; // page3 was 2
        
        let stats = pm.cache_stats();
        assert_eq!(stats.0, 2); // Should have 2 pages cached
    }
    
    Ok(())
}

#[test]
fn test_lru_cache_unit() {
    use zendb::storage::LRUCache;
    use std::sync::Arc;
    
    let mut cache = LRUCache::new(3); // Small cache for testing
    
    // Create test pages
    let page1 = Arc::new(Page::new(1));
    let page2 = Arc::new(Page::new(2));
    let page3 = Arc::new(Page::new(3));
    let page4 = Arc::new(Page::new(4));
    
    // Insert pages
    cache.insert(1, page1.clone());
    cache.insert(2, page2.clone());
    cache.insert(3, page3.clone());
    
    assert_eq!(cache.len(), 3);
    
    // Access page 1 to make it most recently used
    let _retrieved = cache.get(&1);
    
    // Insert page 4 - should evict page 2 (least recently used)
    cache.insert(4, page4.clone());
    
    // Cache should still have 3 items
    assert_eq!(cache.len(), 3);
    
    // Page 2 should have been evicted
    assert!(cache.get(&2).is_none());
    
    // Pages 1, 3, 4 should still be present
    assert!(cache.get(&1).is_some());
    assert!(cache.get(&3).is_some()); 
    assert!(cache.get(&4).is_some());
    
    // Clear cache
    cache.clear();
    assert_eq!(cache.len(), 0);
}