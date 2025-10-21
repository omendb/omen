// Simple cache test to verify it's working
use anyhow::Result;
use arrow::datatypes::{DataType, Field, Schema};
use omendb::catalog::Catalog;
use omendb::row::Row;
use omendb::value::Value;
use std::sync::Arc;
use tempfile::TempDir;

fn main() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let _catalog = Catalog::new(temp_dir.path().to_path_buf())?;

    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("value", DataType::Int64, false),
    ]));

    let table_dir = temp_dir.path().join("test_table");
    let mut table = omendb::table::Table::new_with_cache(
        "test".to_string(),
        schema,
        "id".to_string(),
        table_dir,
        100,  // Small cache
    )?;

    // Insert 10 rows
    println!("Inserting 10 rows...");
    for i in 0..10 {
        let row = Row::new(vec![
            Value::Int64(i),
            Value::Int64(i * 100),
        ]);
        table.insert(row)?;
    }

    // Query each row twice
    println!("\nQuerying each row twice:");
    for i in 0..10 {
        let key = Value::Int64(i);

        // First query - should be cache miss
        let result1 = table.get(&key)?;
        assert!(result1.is_some(), "Row {} should exist", i);

        let stats1 = table.cache_stats().unwrap();
        println!("After query {} (first): hits={}, misses={}",
            i, stats1.hits, stats1.misses);

        // Second query - should be cache hit
        let result2 = table.get(&key)?;
        assert!(result2.is_some(), "Row {} should exist", i);

        let stats2 = table.cache_stats().unwrap();
        println!("After query {} (second): hits={}, misses={}",
            i, stats2.hits, stats2.misses);
    }

    let final_stats = table.cache_stats().unwrap();
    println!("\nFinal stats:");
    println!("  Hits: {}", final_stats.hits);
    println!("  Misses: {}", final_stats.misses);
    println!("  Hit rate: {:.1}%", final_stats.hit_rate);
    println!("  Cache size: {}", final_stats.size);

    if final_stats.hits == 10 && final_stats.misses == 10 {
        println!("\n✅ Cache is working correctly!");
    } else {
        println!("\n❌ Cache not working as expected");
        println!("   Expected: 10 hits, 10 misses");
        println!("   Got: {} hits, {} misses", final_stats.hits, final_stats.misses);
    }

    Ok(())
}
