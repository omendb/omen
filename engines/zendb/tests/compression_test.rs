//! Tests for page compression functionality

use zendb::storage::compression::{PageCompressor, CompressionConfig, CompressedPageFormat};
use zendb::storage::{PageManager, Page};
use tempfile::TempDir;
use anyhow::Result;
use std::sync::Arc;

#[tokio::test]
async fn test_compression_roundtrip() -> Result<()> {
    // Create test page data with a simple pattern
    let mut original_data = vec![0u8; 16384];
    original_data[0] = 1; // Node type: Internal
    original_data[1] = 0; // Key count low byte
    original_data[2] = 1; // Key count high byte  
    for i in 100..200 {
        original_data[i] = b'A'; // Some pattern data
    }
    
    let compressor = PageCompressor::new(CompressionConfig::default());
    
    // Test compression
    let compressed = compressor.compress(&original_data)?;
    println!("Compressed: {:?}", compressed.is_some());
    
    // Test serialization format
    let serialized = CompressedPageFormat::serialize(&original_data, compressed.as_ref());
    println!("Serialized size: {}", serialized.len());
    assert!(serialized.len() <= 16384); // Should fit in page
    
    // Test deserialization  
    let deserialized = CompressedPageFormat::deserialize(&serialized)?;
    println!("Deserialized size: {}", deserialized.len());
    
    // Check that first few bytes are preserved (important for B+Tree node type)
    assert_eq!(deserialized[0], 1, "Node type should be preserved");
    assert_eq!(deserialized[1], 0, "Key count low byte should be preserved");
    assert_eq!(deserialized[2], 1, "Key count high byte should be preserved");
    
    // Check pattern data
    for i in 100..200 {
        assert_eq!(deserialized[i], b'A', "Pattern data should be preserved at index {}", i);
    }
    
    Ok(())
}

#[tokio::test] 
async fn test_page_manager_compression_integration() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("compression_test.db");
    
    let pm = Arc::new(PageManager::open(&db_path)?);
    
    // Create a page with recognizable data
    let page_id = pm.allocate_page()?;
    let mut page = Page::new(page_id);
    
    // Set B+Tree node-like structure
    page.data[0] = 1; // Internal node type
    page.data[1] = 5; // Key count low byte
    page.data[2] = 0; // Key count high byte
    
    // Add some pattern data
    for i in 10..100 {
        page.data[i] = ((i % 256) as u8);
    }
    
    // Write page (should compress internally)
    pm.write_page(&page)?;
    
    // Read page back (should decompress internally)
    let read_page = pm.read_page(page_id)?;
    
    // Verify critical first bytes are preserved
    assert_eq!(read_page.data[0], 1, "Node type should be preserved after compression roundtrip");
    assert_eq!(read_page.data[1], 5, "Key count should be preserved");
    assert_eq!(read_page.data[2], 0, "Key count high byte should be preserved");
    
    // Verify pattern data
    for i in 10..100 {
        assert_eq!(read_page.data[i], ((i % 256) as u8), 
                   "Pattern data should be preserved at index {}", i);
    }
    
    Ok(())
}