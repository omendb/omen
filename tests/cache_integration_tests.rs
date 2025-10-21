use omendb::{
    catalog::Catalog,
    row::Row,
    table::Table,
    value::Value,
};
use arrow::datatypes::{DataType, Field, Schema};
use std::sync::Arc;
use tempfile::TempDir;
use anyhow::Result;

#[test]
fn test_table_with_cache_basic() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;

    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
    ]));

    let table_dir = temp_dir.path().join("users_cached");
    let mut table = Table::new_with_cache(
        "users".to_string(),
        schema,
        "id".to_string(),
        table_dir,
        1000,  // 1000 entry cache
    )?;

    // Insert row
    let row = Row::new(vec![Value::Int64(1), Value::Text("Alice".to_string())]);
    table.insert(row.clone())?;

    // First get: cache miss (cold)
    let result1 = table.get(&Value::Int64(1))?;
    assert!(result1.is_some());

    // Second get: cache hit (warm)
    let result2 = table.get(&Value::Int64(1))?;
    assert!(result2.is_some());

    // Check cache stats
    let stats = table.cache_stats().unwrap();
    assert_eq!(stats.hits, 1, "Second get should be cache hit");
    assert_eq!(stats.misses, 1, "First get should be cache miss");
    assert_eq!(stats.hit_rate, 50.0, "Hit rate should be 50% (1/2)");

    Ok(())
}

#[test]
fn test_table_cache_invalidation_on_update() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;

    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("value", DataType::Int64, false),
    ]));

    let table_dir = temp_dir.path().join("test_update");
    let mut table = Table::new_with_cache(
        "test".to_string(),
        schema,
        "id".to_string(),
        table_dir,
        1000,
    )?;

    // Insert and populate cache
    let row = Row::new(vec![Value::Int64(1), Value::Int64(100)]);
    table.insert(row.clone())?;
    table.get(&Value::Int64(1))?;  // Populate cache

    let stats_before = table.cache_stats().unwrap();
    assert_eq!(stats_before.size, 1, "Cache should have 1 entry");

    // Update should invalidate cache
    let updated_row = Row::new(vec![Value::Int64(1), Value::Int64(200)]);
    table.update(&Value::Int64(1), updated_row)?;

    // Get after update should be cache miss (cache was invalidated)
    let result = table.get(&Value::Int64(1))?;
    assert_eq!(result.unwrap().get(1)?, &Value::Int64(200));

    let stats_after = table.cache_stats().unwrap();
    // After update: cache was invalidated, then repopulated
    assert_eq!(stats_after.size, 1, "Cache should have 1 entry again");

    Ok(())
}

#[test]
fn test_table_cache_invalidation_on_delete() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;

    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("value", DataType::Int64, false),
    ]));

    let table_dir = temp_dir.path().join("test_delete");
    let mut table = Table::new_with_cache(
        "test".to_string(),
        schema,
        "id".to_string(),
        table_dir,
        1000,
    )?;

    // Insert and populate cache
    let row = Row::new(vec![Value::Int64(1), Value::Int64(100)]);
    table.insert(row.clone())?;
    table.get(&Value::Int64(1))?;  // Populate cache

    let stats_before = table.cache_stats().unwrap();
    assert_eq!(stats_before.size, 1, "Cache should have 1 entry");

    // Delete should invalidate cache
    table.delete(&Value::Int64(1))?;

    // Get after delete should return None (row deleted)
    let result = table.get(&Value::Int64(1))?;
    assert!(result.is_none(), "Deleted row should not be found");

    let stats_after = table.cache_stats().unwrap();
    // Cache was invalidated by delete, and get returned None (not cached)
    assert_eq!(stats_after.size, 0, "Cache should be empty after delete");

    Ok(())
}

#[test]
fn test_table_cache_hit_rate() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;

    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("value", DataType::Int64, false),
    ]));

    let table_dir = temp_dir.path().join("test_hit_rate");
    let mut table = Table::new_with_cache(
        "test".to_string(),
        schema,
        "id".to_string(),
        table_dir,
        100,
    )?;

    // Insert 10 rows
    for i in 0..10 {
        let row = Row::new(vec![Value::Int64(i), Value::Int64(i * 100)]);
        table.insert(row)?;
    }

    // First access: all misses
    for i in 0..10 {
        table.get(&Value::Int64(i))?;
    }

    let stats_after_first = table.cache_stats().unwrap();
    assert_eq!(stats_after_first.hits, 0, "First access should all be misses");
    assert_eq!(stats_after_first.misses, 10, "All 10 accesses should be misses");

    // Second access: all hits
    for i in 0..10 {
        table.get(&Value::Int64(i))?;
    }

    let stats_after_second = table.cache_stats().unwrap();
    assert_eq!(stats_after_second.hits, 10, "Second access should all be hits");
    assert_eq!(stats_after_second.misses, 10, "Misses should remain 10");
    assert_eq!(stats_after_second.hit_rate, 50.0, "Hit rate should be 50% (10/20)");

    Ok(())
}

#[test]
fn test_table_cache_lru_eviction() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;

    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("value", DataType::Int64, false),
    ]));

    let table_dir = temp_dir.path().join("test_lru");
    let mut table = Table::new_with_cache(
        "test".to_string(),
        schema,
        "id".to_string(),
        table_dir,
        2,  // Only 2 entries max
    )?;

    // Insert 3 rows
    for i in 0..3 {
        let row = Row::new(vec![Value::Int64(i), Value::Int64(i * 100)]);
        table.insert(row)?;
    }

    // Access rows 0, 1 (both get cached)
    table.get(&Value::Int64(0))?;
    table.get(&Value::Int64(1))?;

    let stats_after_2 = table.cache_stats().unwrap();
    assert_eq!(stats_after_2.size, 2, "Cache should have 2 entries");

    // Access row 2 (should evict row 0, the LRU)
    table.get(&Value::Int64(2))?;

    let stats_after_3 = table.cache_stats().unwrap();
    assert_eq!(stats_after_3.size, 2, "Cache should still have 2 entries (LRU evicted)");

    // Access row 0 again - should be a miss (was evicted)
    table.get(&Value::Int64(0))?;

    let stats_final = table.cache_stats().unwrap();
    assert_eq!(stats_final.misses, 4, "Row 0 second access should be a miss");

    Ok(())
}

#[test]
fn test_table_without_cache() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;

    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
    ]));

    let table_dir = temp_dir.path().join("users_no_cache");
    let mut table = Table::new(
        "users".to_string(),
        schema,
        "id".to_string(),
        table_dir,
    )?;

    // Insert and get should work without cache
    let row = Row::new(vec![Value::Int64(1), Value::Text("Alice".to_string())]);
    table.insert(row.clone())?;

    let result = table.get(&Value::Int64(1))?;
    assert!(result.is_some());

    // Cache stats should be None (no cache enabled)
    assert!(table.cache_stats().is_none(), "Cache stats should be None when cache disabled");

    Ok(())
}

#[test]
fn test_enable_cache_on_existing_table() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;

    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("value", DataType::Int64, false),
    ]));

    let table_dir = temp_dir.path().join("test_enable_cache");
    let mut table = Table::new(
        "test".to_string(),
        schema,
        "id".to_string(),
        table_dir,
    )?;

    // Insert row before cache is enabled
    let row = Row::new(vec![Value::Int64(1), Value::Int64(100)]);
    table.insert(row)?;

    // No cache yet
    assert!(table.cache_stats().is_none());

    // Enable cache
    table.enable_cache(1000);

    // Now cache should work
    table.get(&Value::Int64(1))?;  // Cache miss
    table.get(&Value::Int64(1))?;  // Cache hit

    let stats = table.cache_stats().unwrap();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 1);

    Ok(())
}
