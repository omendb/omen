//! Comprehensive edge case tests for OmenDB
//! Tests all the ways the system might break

use omendb::catalog::Catalog;
use omendb::sql_engine::{SqlEngine, ExecutionResult};
use omendb::value::Value;
use tempfile::TempDir;

#[test]
fn test_duplicate_primary_keys() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE test (id BIGINT PRIMARY KEY, value DOUBLE)").unwrap();
    engine.execute("INSERT INTO test VALUES (1, 1.0)").unwrap();

    // Insert duplicate key - what happens?
    let result = engine.execute("INSERT INTO test VALUES (1, 2.0)");

    // Should either error OR update the value
    // Let's verify the state
    let query_result = engine.execute("SELECT * FROM test WHERE id = 1").unwrap();
    match query_result {
        ExecutionResult::Selected { rows, data, .. } => {
            // Should have exactly 1 row (not 2)
            assert_eq!(rows, 1, "Duplicate key should not create two rows");

            // Verify we don't have two different values
            let value = data[0].get(1).unwrap();
            println!("Value after duplicate insert: {:?}", value);
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_empty_table_operations() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE empty (id BIGINT PRIMARY KEY, value DOUBLE)").unwrap();

    // Query empty table
    let result = engine.execute("SELECT * FROM empty").unwrap();
    match result {
        ExecutionResult::Selected { rows, .. } => {
            assert_eq!(rows, 0, "Empty table should return 0 rows");
        }
        _ => panic!("Expected Selected result"),
    }

    // WHERE query on empty table
    let result = engine.execute("SELECT * FROM empty WHERE id = 1").unwrap();
    match result {
        ExecutionResult::Selected { rows, .. } => {
            assert_eq!(rows, 0, "Empty table WHERE should return 0 rows");
        }
        _ => panic!("Expected Selected result"),
    }

    // Range query on empty table
    let result = engine.execute("SELECT * FROM empty WHERE id > 0 AND id < 10").unwrap();
    match result {
        ExecutionResult::Selected { rows, .. } => {
            assert_eq!(rows, 0, "Empty table range query should return 0 rows");
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_single_row_table() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE single (id BIGINT PRIMARY KEY, value DOUBLE)").unwrap();
    engine.execute("INSERT INTO single VALUES (42, 3.14)").unwrap();

    // Point query for the one row
    let result = engine.execute("SELECT * FROM single WHERE id = 42").unwrap();
    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 1);
            assert_eq!(data[0].get(0).unwrap(), &Value::Int64(42));
            assert_eq!(data[0].get(1).unwrap(), &Value::Float64(3.14));
        }
        _ => panic!("Expected Selected result"),
    }

    // Point query for non-existent key
    let result = engine.execute("SELECT * FROM single WHERE id = 999").unwrap();
    match result {
        ExecutionResult::Selected { rows, .. } => {
            assert_eq!(rows, 0, "Non-existent key should return 0 rows");
        }
        _ => panic!("Expected Selected result"),
    }

    // Range query that includes the row
    let result = engine.execute("SELECT * FROM single WHERE id > 0 AND id < 100").unwrap();
    match result {
        ExecutionResult::Selected { rows, .. } => {
            assert_eq!(rows, 1);
        }
        _ => panic!("Expected Selected result"),
    }

    // Range query that excludes the row
    let result = engine.execute("SELECT * FROM single WHERE id > 100 AND id < 200").unwrap();
    match result {
        ExecutionResult::Selected { rows, .. } => {
            assert_eq!(rows, 0);
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_boundary_values() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE bounds (id BIGINT PRIMARY KEY, value DOUBLE)").unwrap();

    // Test i64 min/max
    let result = engine.execute(&format!("INSERT INTO bounds VALUES ({}, 1.0)", i64::MIN));
    if let Err(e) = &result {
        println!("Error inserting i64::MIN: {}", e);
    }
    assert!(result.is_ok(), "Should handle i64::MIN: {:?}", result);

    let result = engine.execute(&format!("INSERT INTO bounds VALUES ({}, 2.0)", i64::MAX));
    assert!(result.is_ok(), "Should handle i64::MAX");

    // Query for min value
    let result = engine.execute(&format!("SELECT * FROM bounds WHERE id = {}", i64::MIN)).unwrap();
    match result {
        ExecutionResult::Selected { rows, .. } => {
            assert_eq!(rows, 1, "Should find i64::MIN");
        }
        _ => panic!("Expected Selected result"),
    }

    // Query for max value
    let result = engine.execute(&format!("SELECT * FROM bounds WHERE id = {}", i64::MAX)).unwrap();
    match result {
        ExecutionResult::Selected { rows, .. } => {
            assert_eq!(rows, 1, "Should find i64::MAX");
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_float_special_values() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE floats (id BIGINT PRIMARY KEY, value DOUBLE)").unwrap();

    // Test with 0.0
    engine.execute("INSERT INTO floats VALUES (1, 0.0)").unwrap();

    // Test with negative zero
    engine.execute("INSERT INTO floats VALUES (2, -0.0)").unwrap();

    // Test with very large float
    engine.execute(&format!("INSERT INTO floats VALUES (3, {})", f64::MAX)).unwrap();

    // Test with very small float
    engine.execute(&format!("INSERT INTO floats VALUES (4, {})", f64::MIN)).unwrap();

    // Query and verify
    let result = engine.execute("SELECT * FROM floats").unwrap();
    match result {
        ExecutionResult::Selected { rows, .. } => {
            assert_eq!(rows, 4, "All float inserts should succeed");
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_empty_strings() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE strings (id BIGINT PRIMARY KEY, text VARCHAR(255))").unwrap();

    // Insert empty string
    engine.execute("INSERT INTO strings VALUES (1, '')").unwrap();

    // Query it back
    let result = engine.execute("SELECT * FROM strings WHERE id = 1").unwrap();
    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 1);
            assert_eq!(data[0].get(1).unwrap(), &Value::Text("".to_string()));
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_where_correctness_vs_scan() {
    // Verify WHERE clause returns same results as filtering a full scan
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE correctness (id BIGINT PRIMARY KEY, value DOUBLE)").unwrap();

    // Insert test data
    for i in 0..100 {
        engine.execute(&format!("INSERT INTO correctness VALUES ({}, {})", i, i as f64 * 1.5)).unwrap();
    }

    // Test various WHERE conditions
    let test_cases = vec![
        ("WHERE id = 50", vec![50]),
        ("WHERE id > 90", vec![91, 92, 93, 94, 95, 96, 97, 98, 99]),
        ("WHERE id < 5", vec![0, 1, 2, 3, 4]),
        ("WHERE id >= 95", vec![95, 96, 97, 98, 99]),
        ("WHERE id <= 3", vec![0, 1, 2, 3]),
        ("WHERE id > 40 AND id < 45", vec![41, 42, 43, 44]),
    ];

    for (where_clause, expected_ids) in test_cases {
        let sql = format!("SELECT * FROM correctness {}", where_clause);
        let result = engine.execute(&sql).unwrap();

        match result {
            ExecutionResult::Selected { rows, data, .. } => {
                assert_eq!(
                    rows,
                    expected_ids.len(),
                    "WHERE clause '{}' returned wrong number of rows",
                    where_clause
                );

                // Verify IDs match
                let actual_ids: Vec<i64> = data
                    .iter()
                    .map(|row| {
                        if let Value::Int64(id) = row.get(0).unwrap() {
                            *id
                        } else {
                            panic!("Expected Int64 id");
                        }
                    })
                    .collect();

                assert_eq!(
                    actual_ids, expected_ids,
                    "WHERE clause '{}' returned wrong rows",
                    where_clause
                );
            }
            _ => panic!("Expected Selected result"),
        }
    }
}

#[test]
fn test_range_query_edge_cases() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE range_test (id BIGINT PRIMARY KEY, value DOUBLE)").unwrap();

    // Insert data with gaps
    for i in vec![1, 5, 10, 15, 20, 100, 1000] {
        engine.execute(&format!("INSERT INTO range_test VALUES ({}, {})", i, i as f64)).unwrap();
    }

    // Range with start > end (should return 0 rows)
    let result = engine.execute("SELECT * FROM range_test WHERE id > 100 AND id < 50").unwrap();
    match result {
        ExecutionResult::Selected { rows, .. } => {
            assert_eq!(rows, 0, "Range with start > end should return 0 rows");
        }
        _ => panic!("Expected Selected result"),
    }

    // Range with start == end
    let result = engine.execute("SELECT * FROM range_test WHERE id > 50 AND id < 50").unwrap();
    match result {
        ExecutionResult::Selected { rows, .. } => {
            assert_eq!(rows, 0, "Range with start == end should return 0 rows");
        }
        _ => panic!("Expected Selected result"),
    }

    // Range in gap (no data)
    let result = engine.execute("SELECT * FROM range_test WHERE id > 30 AND id < 90").unwrap();
    match result {
        ExecutionResult::Selected { rows, .. } => {
            assert_eq!(rows, 0, "Range in gap should return 0 rows");
        }
        _ => panic!("Expected Selected result"),
    }

    // Range covering one point
    let result = engine.execute("SELECT * FROM range_test WHERE id >= 10 AND id <= 10").unwrap();
    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 1, "Range covering single point should return 1 row");
            assert_eq!(data[0].get(0).unwrap(), &Value::Int64(10));
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_non_existent_table() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    // Query non-existent table
    let result = engine.execute("SELECT * FROM does_not_exist");
    assert!(result.is_err(), "Query to non-existent table should error");

    // Insert to non-existent table
    let result = engine.execute("INSERT INTO does_not_exist VALUES (1, 2)");
    assert!(result.is_err(), "Insert to non-existent table should error");
}

#[test]
fn test_type_mismatch_in_insert() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE typed (id BIGINT PRIMARY KEY, value DOUBLE)").unwrap();

    // Try to insert wrong number of values
    let result = engine.execute("INSERT INTO typed VALUES (1)");
    assert!(result.is_err(), "Insert with wrong column count should error");

    // Try to insert too many values
    let result = engine.execute("INSERT INTO typed VALUES (1, 2.0, 3)");
    assert!(result.is_err(), "Insert with too many columns should error");
}

#[test]
fn test_negative_keys() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE negative (id BIGINT PRIMARY KEY, value DOUBLE)").unwrap();

    // Insert negative keys
    for i in -50..50 {
        engine.execute(&format!("INSERT INTO negative VALUES ({}, {})", i, i as f64)).unwrap();
    }

    // Query negative key
    let result = engine.execute("SELECT * FROM negative WHERE id = -25").unwrap();
    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 1);
            assert_eq!(data[0].get(0).unwrap(), &Value::Int64(-25));
        }
        _ => panic!("Expected Selected result"),
    }

    // Range query with negative bounds
    let result = engine.execute("SELECT * FROM negative WHERE id > -10 AND id < 10").unwrap();
    match result {
        ExecutionResult::Selected { rows, .. } => {
            assert_eq!(rows, 19, "Should return -9 through 9");
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_unsorted_inserts() {
    // Insert keys in random order, verify learned index still works
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE unsorted (id BIGINT PRIMARY KEY, value DOUBLE)").unwrap();

    // Insert in reverse order
    for i in (0..100).rev() {
        engine.execute(&format!("INSERT INTO unsorted VALUES ({}, {})", i, i as f64)).unwrap();
    }

    // Verify all rows are there
    let result = engine.execute("SELECT * FROM unsorted").unwrap();
    match result {
        ExecutionResult::Selected { rows, .. } => {
            assert_eq!(rows, 100, "All 100 rows should be present");
        }
        _ => panic!("Expected Selected result"),
    }

    // Verify WHERE still works
    let result = engine.execute("SELECT * FROM unsorted WHERE id = 50").unwrap();
    match result {
        ExecutionResult::Selected { rows, data, .. } => {
            assert_eq!(rows, 1);
            assert_eq!(data[0].get(0).unwrap(), &Value::Int64(50));
        }
        _ => panic!("Expected Selected result"),
    }
}

#[test]
fn test_very_sparse_keys() {
    // Keys with huge gaps
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
    let mut engine = SqlEngine::new(catalog);

    engine.execute("CREATE TABLE sparse (id BIGINT PRIMARY KEY, value DOUBLE)").unwrap();

    // Insert keys with huge gaps
    let sparse_keys = vec![1, 1000, 1000000, 1000000000];
    for key in sparse_keys.iter() {
        engine.execute(&format!("INSERT INTO sparse VALUES ({}, {})", key, *key as f64)).unwrap();
    }

    // Query each one
    for key in sparse_keys.iter() {
        let result = engine.execute(&format!("SELECT * FROM sparse WHERE id = {}", key)).unwrap();
        match result {
            ExecutionResult::Selected { rows, .. } => {
                assert_eq!(rows, 1, "Should find sparse key {}", key);
            }
            _ => panic!("Expected Selected result"),
        }
    }

    // Range query between sparse keys
    let result = engine.execute("SELECT * FROM sparse WHERE id > 1 AND id < 1000").unwrap();
    match result {
        ExecutionResult::Selected { rows, .. } => {
            assert_eq!(rows, 0, "Range in gap should return 0 rows");
        }
        _ => panic!("Expected Selected result"),
    }
}