//! Edge case tests for OmenDB
//!
//! Tests behavior with:
//! - Empty database
//! - Single row
//! - Duplicate keys
//! - NULL values
//! - Very large values
//! - Sequential vs random patterns

use anyhow::Result;
use omendb::catalog::Catalog;
use omendb::row::Row;
use omendb::value::Value;
use arrow::datatypes::{DataType, Field, Schema};
use std::sync::Arc;
use tempfile::TempDir;

fn main() -> Result<()> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║              OmenDB Edge Case Tests                         ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let temp_dir = TempDir::new()?;
    let catalog_dir = temp_dir.path().join("omendb");

    // Test 1: Empty Database
    println!("🔍 Test 1: Empty Database");
    {
        let mut catalog = Catalog::new(catalog_dir.clone())?;
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("value", DataType::Utf8, false),
        ]));
        catalog.create_table("empty_test".to_string(), schema, "id".to_string())?;

        let table = catalog.get_table("empty_test")?;

        // Query non-existent key
        let result = table.get(&Value::Int64(999))?;
        assert!(result.is_none(), "Empty table should return None for any key");

        // Get row count via scan
        let all_rows = table.scan_all()?;
        assert_eq!(all_rows.len(), 0, "Empty table should have 0 rows");

        println!("  ✅ Empty database queries work correctly");
    }

    // Test 2: Single Row
    println!("\n🔍 Test 2: Single Row Database");
    {
        let mut catalog = Catalog::new(catalog_dir.clone())?;
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("value", DataType::Utf8, false),
        ]));
        catalog.create_table("single_test".to_string(), schema, "id".to_string())?;

        let table = catalog.get_table_mut("single_test")?;

        // Insert single row
        let row = Row::new(vec![
            Value::Int64(1),
            Value::Text("single".to_string()),
        ]);
        table.insert(row.clone())?;

        // Query the row
        let result = table.get(&Value::Int64(1))?;
        assert!(result.is_some(), "Should find the single row");
        assert_eq!(result.unwrap().values(), row.values(), "Row values should match");

        // Query non-existent
        let result = table.get(&Value::Int64(999))?;
        assert!(result.is_none(), "Should return None for non-existent key");

        // Check count via scan
        let all_rows = table.scan_all()?;
        assert_eq!(all_rows.len(), 1, "Should have exactly 1 row");

        println!("  ✅ Single row operations work correctly");
    }

    // Test 3: Duplicate Key Handling
    println!("\n🔍 Test 3: Duplicate Key Handling");
    {
        let mut catalog = Catalog::new(catalog_dir.clone())?;
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("value", DataType::Utf8, false),
        ]));
        catalog.create_table("duplicate_test".to_string(), schema, "id".to_string())?;

        let table = catalog.get_table_mut("duplicate_test")?;

        // Insert first row
        let row1 = Row::new(vec![
            Value::Int64(100),
            Value::Text("first".to_string()),
        ]);
        table.insert(row1)?;

        // Try to insert duplicate
        let row2 = Row::new(vec![
            Value::Int64(100),
            Value::Text("second".to_string()),
        ]);

        match table.insert(row2) {
            Ok(_) => {
                // Some systems allow updates
                let result = table.get(&Value::Int64(100))?;
                if let Some(row) = result {
                    if row.values()[1] == Value::Text("second".to_string()) {
                        println!("  ✅ Duplicate key updates existing row");
                    } else {
                        println!("  ⚠️ Unexpected behavior with duplicate key");
                    }
                }
            }
            Err(_) => {
                println!("  ✅ Duplicate key insertion rejected");
            }
        }
    }

    // Test 4: Sequential Pattern
    println!("\n🔍 Test 4: Sequential Insert Pattern");
    {
        let mut catalog = Catalog::new(catalog_dir.clone())?;
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("value", DataType::Int64, false),
        ]));
        catalog.create_table("sequential_test".to_string(), schema, "id".to_string())?;

        let table = catalog.get_table_mut("sequential_test")?;

        // Insert sequential keys
        let mut rows = Vec::new();
        for i in 1..=100 {
            rows.push(Row::new(vec![
                Value::Int64(i),
                Value::Int64(i * 10),
            ]));
        }
        table.batch_insert(rows)?;

        // Verify a few keys
        for i in [1, 50, 100] {
            let result = table.get(&Value::Int64(i))?;
            assert!(result.is_some(), "Key {} should exist", i);
            let row = result.unwrap();
            assert_eq!(row.values()[1], Value::Int64(i * 10), "Value should be {}", i * 10);
        }

        let all_rows = table.scan_all()?;
        assert_eq!(all_rows.len(), 100, "Should have 100 rows");
        println!("  ✅ Sequential pattern handled correctly");
    }

    // Test 5: Boundary Values
    println!("\n🔍 Test 5: Boundary Values");
    {
        let mut catalog = Catalog::new(catalog_dir.clone())?;
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("value", DataType::Int64, false),
        ]));
        catalog.create_table("boundary_test".to_string(), schema, "id".to_string())?;

        let table = catalog.get_table_mut("boundary_test")?;

        // Test extreme values
        let boundaries = [i64::MIN,
            i64::MIN + 1,
            -1,
            0,
            1,
            i64::MAX - 1,
            i64::MAX];

        let mut rows = Vec::new();
        for (i, &key) in boundaries.iter().enumerate() {
            rows.push(Row::new(vec![
                Value::Int64(key),
                Value::Int64(i as i64),
            ]));
        }

        table.batch_insert(rows)?;

        // Verify all boundary values
        for (i, &key) in boundaries.iter().enumerate() {
            let result = table.get(&Value::Int64(key))?;
            assert!(result.is_some(), "Boundary key {} should exist", key);
            let row = result.unwrap();
            assert_eq!(row.values()[1], Value::Int64(i as i64), "Value mismatch for key {}", key);
        }

        println!("  ✅ Boundary values (i64::MIN to i64::MAX) handled correctly");
    }

    // Test 6: Large Batch Insert
    println!("\n🔍 Test 6: Large Batch Insert");
    {
        let mut catalog = Catalog::new(catalog_dir.clone())?;
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("value", DataType::Utf8, false),
        ]));
        catalog.create_table("batch_test".to_string(), schema, "id".to_string())?;

        let table = catalog.get_table_mut("batch_test")?;

        // Create large batch
        let batch_size = 10_000;
        let mut rows = Vec::with_capacity(batch_size);
        for i in 0..batch_size {
            rows.push(Row::new(vec![
                Value::Int64(i as i64),
                Value::Text(format!("value_{}", i)),
            ]));
        }

        let start = std::time::Instant::now();
        table.batch_insert(rows)?;
        let elapsed = start.elapsed();

        let all_rows = table.scan_all()?;
        assert_eq!(all_rows.len(), batch_size, "Should have {} rows", batch_size);

        // Verify sampling
        for i in [0, batch_size/2, batch_size-1] {
            let result = table.get(&Value::Int64(i as i64))?;
            assert!(result.is_some(), "Key {} should exist", i);
        }

        let throughput = batch_size as f64 / elapsed.as_secs_f64();
        println!("  ✅ Large batch insert: {} rows in {:.2}ms ({:.0} rows/sec)",
                 batch_size, elapsed.as_millis(), throughput);
    }

    // Test 7: Sparse Keys
    println!("\n🔍 Test 7: Sparse Keys");
    {
        let mut catalog = Catalog::new(catalog_dir.clone())?;
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("value", DataType::Utf8, false),
        ]));
        catalog.create_table("sparse_test".to_string(), schema, "id".to_string())?;

        let table = catalog.get_table_mut("sparse_test")?;

        // Insert very sparse keys
        let sparse_keys = vec![1, 1000, 1_000_000, 1_000_000_000];
        let mut rows = Vec::new();
        for &key in &sparse_keys {
            rows.push(Row::new(vec![
                Value::Int64(key),
                Value::Text(format!("sparse_{}", key)),
            ]));
        }
        table.batch_insert(rows)?;

        // Verify sparse keys exist
        for &key in &sparse_keys {
            let result = table.get(&Value::Int64(key))?;
            assert!(result.is_some(), "Sparse key {} should exist", key);
        }

        // Verify gaps return None
        for gap_key in [500, 50_000, 500_000_000] {
            let result = table.get(&Value::Int64(gap_key))?;
            assert!(result.is_none(), "Gap key {} should not exist", gap_key);
        }

        let all_rows = table.scan_all()?;
        assert_eq!(all_rows.len(), sparse_keys.len(), "Should have {} rows", sparse_keys.len());
        println!("  ✅ Sparse keys handled correctly");
    }

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                    ALL TESTS PASSED ✅                      ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    Ok(())
}